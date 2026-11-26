#!/usr/bin/env python3
"""check-distribution-evidence.py — validate a phase's committed `docs/evidence` items.

Phase 4's distribution proof rests on eight hand-recorded items plus the
`seal` item that indexes them. This checker re-reads every `*/report.json`
against the eight-key contract in `docs/evidence/README.md` and recomputes
every artifact hash, so a report cannot silently drift from the bytes it
claims to prove.

What it asserts per report:

    item          equals the containing directory name
    requirements  non-empty and a subset of {REL-04, REL-05, REL-06, PKG-04}
    date          YYYY-MM-DDTHH:MM:SSZ
    host          System/arch only (no hostname, no dot)
    command       non-empty
    steps         [{name, ok, detail}]; for result "ok" at least one step and
                  every step ok
    artifacts     [{path, sha256}]; the path is repository-relative with no
                  "..", the file exists, is UTF-8 with no NUL byte, is at most
                  1 MiB, and its sha256 is the recorded one
    result        "ok" | "fail" | "skipped"

REL-05 ("the image runs a real headless turn") is environment-blocked and is
recorded as `result: skipped`. To stop it drifting into a false pass, any
report that lists REL-05 must be `skipped` unless a step whose name contains
"pull" is recorded ok.

Usage:

    scripts/check-distribution-evidence.py docs/evidence/04-distribution-proof
    scripts/check-distribution-evidence.py docs/evidence/04-distribution-proof --self-test
    scripts/check-distribution-evidence.py DIR --self-test --self-test-only

`--self-test` stages a temp copy of the reports and their artifacts, then
mutates four copies — deletes an artifact, corrupts a recorded sha256, flips
the REL-05 item to `ok`, and writes a NUL byte into an artifact — requiring a
named failure for each. It prints `self-test: 4/4 PASS`. The self-test ignores
`.evidence-pending.json`, so it still runs while the `seal` item itself is
being recorded; the plain validation path refuses any pending item, because a
tree mid-write is not a sealed tree.

Exit status: 0 when every report and every self-test case passes, 1 on a
validation failure, 2 when the tree is mid-write or the arguments are unusable.
"""

import argparse
import hashlib
import json
import re
import shutil
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
KEYS = [
    "item",
    "requirements",
    "date",
    "host",
    "command",
    "steps",
    "artifacts",
    "result",
]
ALLOWED_REQUIREMENTS = {"REL-04", "REL-05", "REL-06", "PKG-04"}
ALLOWED_RESULTS = {"ok", "fail", "skipped"}
DATE_RE = re.compile(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$")
HOST_RE = re.compile(r"^[A-Za-z][A-Za-z0-9]*/[A-Za-z0-9_]+$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
MAX_ARTIFACT_BYTES = 1024 * 1024
PENDING_NAME = ".evidence-pending.json"


def validate_report(report_path, root):
    """(issues, artifact_lines) for one report."""
    issues = []
    artifact_lines = []
    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        return [f"report.json is not UTF-8 JSON: {exc}"], artifact_lines
    if not isinstance(report, dict):
        return ["report.json is not a JSON object"], artifact_lines

    if list(report.keys()) != KEYS:
        issues.append(
            "keys are not exactly " + ", ".join(KEYS) + " in order (got "
            + ", ".join(report.keys()) + ")"
        )

    if report.get("item") != report_path.parent.name:
        issues.append(
            f"item {report.get('item')!r} does not equal the directory name "
            f"{report_path.parent.name!r}"
        )

    requirements = report.get("requirements")
    if not isinstance(requirements, list) or not requirements or not all(
        isinstance(item, str) for item in requirements
    ):
        issues.append("requirements is not a non-empty list of strings")
        requirements = []
    else:
        unknown = [item for item in requirements if item not in ALLOWED_REQUIREMENTS]
        if unknown:
            issues.append("unknown requirement id(s): " + ", ".join(unknown))

    date = report.get("date")
    if not isinstance(date, str) or not DATE_RE.match(date):
        issues.append(f"date {date!r} is not YYYY-MM-DDTHH:MM:SSZ")

    host = report.get("host")
    if not isinstance(host, str) or "." in host or not HOST_RE.match(host):
        issues.append(f"host {host!r} is not System/arch only (hostname or dot present)")

    command = report.get("command")
    if not isinstance(command, str) or not command.strip():
        issues.append("command is not a non-empty string")

    steps = report.get("steps")
    if not isinstance(steps, list):
        issues.append("steps is not a list")
        steps = []
    else:
        for index, step in enumerate(steps):
            if (
                not isinstance(step, dict)
                or set(step.keys()) != {"name", "ok", "detail"}
                or not isinstance(step.get("name"), str)
                or not isinstance(step.get("ok"), bool)
                or not isinstance(step.get("detail"), str)
            ):
                issues.append(f"step {index} is not {{name, ok: bool, detail}}")

    result = report.get("result")
    if result not in ALLOWED_RESULTS:
        issues.append(f"result {result!r} is not one of ok/fail/skipped")
    else:
        if result == "ok":
            if not steps:
                issues.append("result is ok but no steps are recorded")
            failed = [
                str(step.get("name"))
                for step in steps
                if isinstance(step, dict) and step.get("ok") is not True
            ]
            if failed:
                issues.append("result is ok but step(s) failed: " + ", ".join(failed))
        if "REL-05" in requirements and result != "skipped":
            pulled = any(
                isinstance(step, dict)
                and step.get("ok") is True
                and "pull" in str(step.get("name", "")).lower()
                for step in steps
            )
            if not pulled:
                issues.append(
                    "claims REL-05 with result "
                    f"{result!r} but records no successful pull step"
                )

    artifacts = report.get("artifacts")
    if not isinstance(artifacts, list):
        issues.append("artifacts is not a list")
        artifacts = []
    for artifact in artifacts:
        if (
            not isinstance(artifact, dict)
            or set(artifact.keys()) != {"path", "sha256"}
            or not isinstance(artifact.get("path"), str)
            or not isinstance(artifact.get("sha256"), str)
        ):
            issues.append("artifact entry is not {path, sha256}")
            continue
        rel = artifact["path"]
        digest = artifact["sha256"]
        target = root / rel
        problems = []
        if not rel or Path(rel).is_absolute() or rel.startswith("~"):
            problems.append("path is not repository-relative")
        if ".." in Path(rel).parts:
            problems.append("path contains '..'")
        if not problems and not target.is_file():
            problems.append("file does not exist")
        if not problems:
            size = target.stat().st_size
            if size > MAX_ARTIFACT_BYTES:
                problems.append(f"file is {size} bytes, over the 1 MiB text limit")
            body = target.read_bytes()
            if b"\x00" in body:
                problems.append("file contains a NUL byte (not text)")
            else:
                try:
                    body.decode("utf-8")
                except UnicodeDecodeError:
                    problems.append("file is not valid UTF-8")
            if not problems:
                if not SHA256_RE.match(digest):
                    problems.append("recorded sha256 is not 64 lowercase hex characters")
                else:
                    actual = hashlib.sha256(body).hexdigest()
                    if actual != digest:
                        problems.append(f"sha256 mismatch: recorded {digest}, actual {actual}")
        if problems:
            issues.extend(f"artifact {rel}: {problem}" for problem in problems)
            artifact_lines.append(("FAIL", rel, "; ".join(problems)))
        else:
            artifact_lines.append(("PASS", rel, f"sha256 {digest}, {len(body)} B"))
    return issues, artifact_lines


def validate(phase_dir, root):
    """(ok, total, lines, reasons) for a phase evidence directory."""
    lines = []
    reasons = []
    reports = sorted(phase_dir.glob("*/report.json"))
    if not reports:
        return False, 0, ["distribution-evidence: 0/0 PASS"], [
            f"no */report.json under {phase_dir}"
        ]
    failures = 0
    for report_path in reports:
        issues, artifact_lines = validate_report(report_path, root)
        name = report_path.parent.name
        if issues:
            failures += 1
            lines.append(f"FAIL  {name}: " + "; ".join(issues))
            reasons.extend(f"{name}: {issue}" for issue in issues)
        else:
            lines.append(f"PASS  {name}")
        for status, rel, detail in artifact_lines:
            if status == "PASS":
                lines.append(f"  PASS  {rel} ({detail})")
            else:
                lines.append(f"  FAIL  {rel} ({detail})")
    total = len(reports)
    lines.append(f"distribution-evidence: {total - failures}/{total} PASS")
    lines.append(f"{total} reports validated")
    return failures == 0, total, lines, reasons


def _stage_copy(phase_dir, root, work):
    """Copy the reports and their artifacts into an isolated temp tree."""
    tmp_phase = work / "phase"
    tmp_root = work / "root"
    for report_path in sorted(phase_dir.glob("*/report.json")):
        try:
            report = json.loads(report_path.read_text(encoding="utf-8"))
        except (UnicodeDecodeError, json.JSONDecodeError):
            continue
        if not isinstance(report, dict):
            continue
        item_dir = tmp_phase / report_path.parent.name
        item_dir.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(report_path, item_dir / "report.json")
        artifacts = report.get("artifacts")
        if not isinstance(artifacts, list):
            continue
        for artifact in artifacts:
            if not isinstance(artifact, dict) or not isinstance(artifact.get("path"), str):
                continue
            rel = artifact["path"]
            if Path(rel).is_absolute() or ".." in Path(rel).parts:
                continue
            source = root / rel
            if source.is_file():
                destination = tmp_root / rel
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(source, destination)
    return tmp_phase, tmp_root


def _first_artifact_file(tmp_root):
    files = sorted(path for path in tmp_root.rglob("*") if path.is_file())
    if not files:
        raise SystemExit("self-test: staged tree has no artifact file to mutate")
    return files[0]


def _expect_failure(label, tmp_phase, tmp_root, needle):
    ok, _, _, reasons = validate(tmp_phase, tmp_root)
    if ok:
        print(f"FAIL  self-test: {label} was accepted")
        return False
    if not any(needle in reason for reason in reasons):
        print(
            f"FAIL  self-test: {label} rejected without a {needle!r} reason: "
            + "; ".join(reasons)
        )
        return False
    print(f"PASS  self-test: {label} rejected ({needle})")
    return True


def run_self_test(phase_dir, root):
    cases = [
        ("missing artifact", "does not exist"),
        ("corrupted recorded sha256", "sha256 mismatch"),
        ("REL-05 marked ok without a pull", "no successful pull step"),
        ("NUL byte in an artifact", "NUL byte"),
    ]
    with tempfile.TemporaryDirectory(prefix="distribution-evidence-selftest-") as tmp:
        tmp = Path(tmp)
        base_phase, base_root = _stage_copy(phase_dir, root, tmp / "base")
        base_ok, _, total, reasons = validate(base_phase, base_root)
        if not base_ok:
            print("FAIL  self-test: the staged copy does not validate: " + "; ".join(reasons))
            return 1

        passed = 0

        phase, staged = _stage_copy(phase_dir, root, tmp / "case1")
        _first_artifact_file(staged).unlink()
        passed += _expect_failure(cases[0][0], phase, staged, cases[0][1])

        phase, staged = _stage_copy(phase_dir, root, tmp / "case2")
        report_path = sorted(phase.glob("*/report.json"))[0]
        report = json.loads(report_path.read_text(encoding="utf-8"))
        if report.get("artifacts"):
            report["artifacts"][0]["sha256"] = "0" * 64
        report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        passed += _expect_failure(cases[1][0], phase, staged, cases[1][1])

        phase, staged = _stage_copy(phase_dir, root, tmp / "case3")
        flipped = False
        for report_path in sorted(phase.glob("*/report.json")):
            report = json.loads(report_path.read_text(encoding="utf-8"))
            if "REL-05" in (report.get("requirements") or []):
                report["result"] = "ok"
                report_path.write_text(
                    json.dumps(report, indent=2) + "\n", encoding="utf-8"
                )
                flipped = True
                break
        if not flipped:
            print("FAIL  self-test: no report claims REL-05 to flip")
        else:
            passed += _expect_failure(cases[2][0], phase, staged, cases[2][1])

        phase, staged = _stage_copy(phase_dir, root, tmp / "case4")
        with _first_artifact_file(staged).open("ab") as handle:
            handle.write(b"\x00")
        passed += _expect_failure(cases[3][0], phase, staged, cases[3][1])

        print(f"self-test: {passed}/{len(cases)} PASS")
        return 0 if passed == len(cases) else 1


def find_pending(phase_dir):
    return sorted(
        path
        for path in phase_dir.rglob(PENDING_NAME)
        if path.is_file()
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("phase_dir", help="a phase evidence directory, e.g. docs/evidence/04-distribution-proof")
    parser.add_argument(
        "--root",
        default=str(REPO_ROOT),
        help="repository root the artifact paths are relative to (default: this checkout)",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="run the four mutation cases on a temp copy before validating",
    )
    parser.add_argument(
        "--self-test-only",
        action="store_true",
        help="run only the four mutation cases (valid while this phase's seal item is pending)",
    )
    args = parser.parse_args()

    phase_dir = Path(args.phase_dir)
    root = Path(args.root)
    if not phase_dir.is_dir():
        sys.stderr.write(f"distribution-evidence: no directory at {phase_dir}\n")
        return 2

    if args.self_test or args.self_test_only:
        self_test_rc = run_self_test(phase_dir, root)
        if self_test_rc != 0 or args.self_test_only:
            return self_test_rc

    pending = find_pending(phase_dir)
    if pending:
        for path in pending:
            sys.stderr.write(f"FAIL  evidence item is mid-write: {path}\n")
        sys.stderr.write(
            "distribution-evidence: refused — finish or remove the pending item first\n"
        )
        return 2

    ok, total, lines, _ = validate(phase_dir, root)
    for line in lines:
        print(line)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
