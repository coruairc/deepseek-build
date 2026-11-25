#!/usr/bin/env python3
"""Static `v0.1.0` tag-shape verification for .github/workflows/release.yml.

Why this exists
---------------
A dry run is a real push, so every fact about the `v0.1.0` tag shape has to be
proven before a tag exists. This script proves three things from the shipped
workflow, with no network and no push:

  1. the trigger is exactly `push: {tags: ['v*']}` plus `workflow_dispatch` —
     no second tag pattern, no `deepseek` pattern (a `deepseek-*` trigger could
     classify a snapshot as a stable release);
  2. the `version` job's derive step behaves correctly on the tag shapes that
     matter: `v0.1.0` is stable, `v0.1.0-rc1` and `v0.1.0-proof1` are
     prereleases, a branch ref falls back to `0.0.0-<sha7>`, and the
     non-semver shapes (`v0.1`, `v1`, `v0.1.0.0`, `v0.1.0+build.5`, `1.0.0`)
     fail before anything is built or pushed;
  3. the release step stays tag-gated (`if: github.ref_type == 'tag'`) and its
     `prerelease:` input is wired to the derived flag, and `publish` carries no
     job-level `if` — so a branch dispatch stays release-inert while `v0.1.0`
     and `v0.1.0-rc1` classify without a second code path.

The derive step's `run:` body is extracted from the workflow file at test time
and executed with `bash`, with a fresh `$GITHUB_OUTPUT` per fixture. It is
never re-typed: a copy would test the copy, not the shipped workflow. The
checker fails loudly if the step is missing rather than falling back to one.

Build metadata is rejected on purpose (03-01's semver gate excludes `+`): the
pinned metadata-action sanitizes `+` to `-`, so `v0.1.0+build.5` would leave
`publish`'s release asset names unmatched. The fixture records that rule.

Usage
-----
    python3 scripts/check-release-tag-shape.py [path/to/release.yml]

Prints one PASS/FAIL line per assertion (with the observed `$GITHUB_OUTPUT`
for each derive fixture), every violation, and `tag-shape: N/N assertions
PASS` on success. Exits 1 on any failure. Requires Python 3, PyYAML and `bash`.
"""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any, Callable

import yaml

DEFAULT_WORKFLOW = (
    Path(__file__).resolve().parent.parent / ".github" / "workflows" / "release.yml"
)

# The derive body's own error text. A rejection that does not name the reason
# would leave a red run without a diagnosable cause, so each is asserted.
SEMVER_ERROR_NEEDLE = "is not a semver tag"
TRIGGER_ERROR_NEEDLE = "release tags must match 'v*'"

BRANCH_SHA = "abcdef1234567890"

Workflow = dict[str, Any]
Assertion = tuple[str, str, Callable[[Workflow], list[str]]]

# Observed evidence per assertion id, printed as an indented line under PASS.
OBSERVED: dict[str, str] = {}


def jobs_of(doc: Workflow) -> dict[str, Any]:
    jobs = doc.get("jobs")
    return jobs if isinstance(jobs, dict) else {}


def steps_of(job: dict[str, Any]) -> list[dict[str, Any]]:
    steps = job.get("steps")
    return [s for s in steps if isinstance(s, dict)] if isinstance(steps, list) else []


def uses_of(step: dict[str, Any]) -> str:
    return str(step.get("uses") or "")


def run_of(step: dict[str, Any]) -> str:
    return str(step.get("run") or "")


def with_of(step: dict[str, Any]) -> dict[str, Any]:
    value = step.get("with")
    return value if isinstance(value, dict) else {}


def trigger_of(doc: Workflow) -> Any:
    # PyYAML resolves the bare key `on:` to the boolean True.
    if True in doc:
        return doc[True]
    return doc.get("on")


def strings_in(value: Any) -> list[str]:
    """Every string anywhere in a nested YAML value."""
    if isinstance(value, str):
        return [value]
    if isinstance(value, dict):
        found: list[str] = []
        for key, item in value.items():
            found.extend(strings_in(key))
            found.extend(strings_in(item))
        return found
    if isinstance(value, list):
        found = []
        for item in value:
            found.extend(strings_in(item))
        return found
    return []


# ---------------------------------------------------------------------------
# (a) Trigger assertions
# ---------------------------------------------------------------------------


def a1_trigger_keys(doc: Workflow) -> list[str]:
    trigger = trigger_of(doc)
    if trigger is None:
        return ["the workflow has no trigger block (expected push tags ['v*'] + workflow_dispatch)"]
    if not isinstance(trigger, dict):
        return [f"the trigger is {trigger!r}, expected a mapping with push/workflow_dispatch"]
    keys = set(trigger)
    expected = {"push", "workflow_dispatch"}
    problems: list[str] = []
    for missing in sorted(expected - keys):
        problems.append(f"the trigger is missing '{missing}'")
    for extra in sorted(str(k) for k in keys - expected):
        problems.append(f"the trigger has an unexpected key '{extra}'")
    return problems


def a2_push_tags(doc: Workflow) -> list[str]:
    trigger = trigger_of(doc)
    if not isinstance(trigger, dict):
        return ["the workflow has no trigger mapping, so push.tags cannot be checked"]
    push = trigger.get("push")
    if not isinstance(push, dict):
        return [f"push is {push!r}, expected a mapping with tags: ['v*']"]
    problems: list[str] = []
    extra = sorted(str(k) for k in set(push) - {"tags"})
    if extra:
        problems.append(f"push has unexpected keys {extra!r}; the trigger is exactly push.tags")
    tags = push.get("tags")
    if tags != ["v*"]:
        problems.append(f"push.tags is {tags!r}, expected ['v*'] (exactly one pattern)")
    return problems


def a3_no_deepseek_pattern(doc: Workflow) -> list[str]:
    trigger = trigger_of(doc)
    if trigger is None:
        return ["the workflow has no trigger block to scan for a 'deepseek' pattern"]
    offenders = [s for s in strings_in(trigger) if "deepseek" in s.lower()]
    return [
        f"the trigger contains the string {value!r}; a deepseek-* trigger must never exist"
        for value in offenders
    ]


# ---------------------------------------------------------------------------
# (b) Derive-body extraction and execution
# ---------------------------------------------------------------------------


def derive_body(doc: Workflow) -> tuple[str, list[str]]:
    version = jobs_of(doc).get("version")
    if not isinstance(version, dict):
        return "", ["job 'version' is missing, so there is no derive step to execute"]
    derives = [s for s in steps_of(version) if s.get("id") == "derive"]
    if len(derives) != 1:
        return "", [
            f"job 'version' has {len(derives)} step(s) with id 'derive', expected exactly 1 "
            "(the checker never falls back to a copied body)"
        ]
    body = run_of(derives[0])
    if not body.strip():
        return "", ["the derive step has an empty run body"]
    return body, []


def parse_github_output(path: Path) -> tuple[dict[str, str], list[str]]:
    """Parse a `key=value`-line $GITHUB_OUTPUT file; return (outputs, problems)."""
    outputs: dict[str, str] = {}
    problems: list[str] = []
    if not path.exists():
        return outputs, []
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        key, sep, value = line.partition("=")
        if not sep or not key:
            problems.append(f"$GITHUB_OUTPUT has a malformed line: {line!r}")
            continue
        if key in outputs:
            problems.append(f"$GITHUB_OUTPUT writes {key!r} twice")
        outputs[key] = value
    return outputs, problems


def run_derive(
    body: str, ref_type: str, ref_name: str, sha: str = BRANCH_SHA
) -> tuple[int, dict[str, str], str, list[str]]:
    """Execute the extracted derive body once; return (rc, outputs, combined, problems)."""
    problems: list[str] = []
    with tempfile.TemporaryDirectory(prefix="tag-shape-fixture-") as work:
        root = Path(work)
        script = root / "derive.sh"
        script.write_text(body, encoding="utf-8")
        output_file = root / "github_output"
        output_file.write_text("", encoding="utf-8")
        env = {
            "PATH": os.environ.get("PATH", "/usr/bin:/bin"),
            "HOME": str(root),
            "GITHUB_OUTPUT": str(output_file),
            "GITHUB_REF_TYPE": ref_type,
            "GITHUB_REF_NAME": ref_name,
            "GITHUB_SHA": sha,
        }
        try:
            proc = subprocess.run(
                ["bash", str(script)],
                cwd=root,
                env=env,
                capture_output=True,
                text=True,
                timeout=60,
            )
        except (OSError, subprocess.TimeoutExpired) as exc:
            return 1, {}, "", [f"could not execute the derive body: {exc}"]
        outputs, problems = parse_github_output(output_file)
        combined = (proc.stdout or "") + (proc.stderr or "")
        return proc.returncode, outputs, combined, problems


def fixture_assertion(
    assertion_id: str,
    label: str,
    ref_type: str,
    ref_name: str,
    *,
    expect_ok: bool,
    expected_outputs: dict[str, str] | None = None,
    expected_messages: tuple[str, ...] = (),
) -> Callable[[Workflow], list[str]]:
    def assertion(doc: Workflow) -> list[str]:
        body, problems = derive_body(doc)
        if problems:
            return [f"fixture {label!r} not run: {problem}" for problem in problems]
        rc, outputs, combined, parse_problems = run_derive(body, ref_type, ref_name)
        problems = list(parse_problems)
        if expect_ok:
            if rc != 0:
                problems.append(
                    f"derive exited {rc}, expected 0; output={combined!r}"
                )
            if expected_outputs is not None and outputs != expected_outputs:
                problems.append(
                    f"$GITHUB_OUTPUT is {outputs!r}, expected {expected_outputs!r}"
                )
            OBSERVED[assertion_id] = (
                f"exit {rc}, $GITHUB_OUTPUT={outputs!r}, stdout={combined.strip()!r}"
            )
        else:
            if rc == 0:
                problems.append(
                    f"derive exited 0 for {ref_name!r}, expected a non-zero rejection; "
                    f"output={combined!r}"
                )
            for needle in expected_messages:
                if needle not in combined:
                    problems.append(
                        f"the rejection of {ref_name!r} does not contain {needle!r}; "
                        f"output={combined!r}"
                    )
            if outputs:
                problems.append(
                    f"the rejection of {ref_name!r} still wrote $GITHUB_OUTPUT={outputs!r}"
                )
            OBSERVED[assertion_id] = f"exit {rc}, stdout={combined.strip()!r}"
        return problems

    return assertion


# ---------------------------------------------------------------------------
# (c) Release-gating assertions
# ---------------------------------------------------------------------------


def release_steps(doc: Workflow) -> list[dict[str, Any]]:
    publish = jobs_of(doc).get("publish")
    if not isinstance(publish, dict):
        return []
    return [
        s
        for s in steps_of(publish)
        if uses_of(s).startswith("softprops/action-gh-release@")
    ]


def a14_release_tag_gate(doc: Workflow) -> list[str]:
    steps = release_steps(doc)
    if len(steps) != 1:
        return [f"publish has {len(steps)} action-gh-release step(s), expected exactly 1"]
    condition = str(steps[0].get("if") or "").strip()
    if condition != "github.ref_type == 'tag'":
        return [
            f"the release step's if is {condition!r}, expected "
            "\"github.ref_type == 'tag'\" (a branch dispatch must create no release)"
        ]
    return []


def a15_prerelease_wiring(doc: Workflow) -> list[str]:
    steps = release_steps(doc)
    if len(steps) != 1:
        return [f"publish has {len(steps)} action-gh-release step(s), expected exactly 1"]
    prerelease = with_of(steps[0]).get("prerelease")
    if prerelease != "${{ needs.version.outputs.is_prerelease }}":
        return [
            f"the release step's with.prerelease is {prerelease!r}, expected "
            "\"${{ needs.version.outputs.is_prerelease }}\""
        ]
    return []


def a16_publish_has_no_job_if(doc: Workflow) -> list[str]:
    publish = jobs_of(doc).get("publish")
    if not isinstance(publish, dict):
        return ["job 'publish' is missing"]
    if "if" in publish:
        return [
            f"publish has a job-level if: {publish.get('if')!r}; only the release step "
            "may be tag-gated, so a branch dispatch still builds and verifies artifacts"
        ]
    return []


def a17_container_tag_gate(doc: Workflow) -> list[str]:
    container = jobs_of(doc).get("container")
    if not isinstance(container, dict):
        return ["job 'container' is missing"]
    condition = str(container.get("if") or "").strip()
    if condition != "github.ref_type == 'tag'":
        return [
            f"container's if is {condition!r}, expected \"github.ref_type == 'tag'\""
        ]
    return []


# ---------------------------------------------------------------------------
# Assertion list
# ---------------------------------------------------------------------------

ASSERTIONS: list[Assertion] = [
    (
        "A1",
        "the trigger keys are exactly {push, workflow_dispatch}",
        a1_trigger_keys,
    ),
    ("A2", "push.tags is exactly ['v*'] with no second tag pattern", a2_push_tags),
    ("A3", "no string in the trigger contains 'deepseek'", a3_no_deepseek_pattern),
    ("A4", "the derive step (id: derive) is present and has a run body", lambda doc: derive_body(doc)[1]),
    (
        "A5",
        "v0.1.0 -> version=0.1.0, is_prerelease=false (the stable release)",
        fixture_assertion(
            "A5",
            "v0.1.0",
            "tag",
            "v0.1.0",
            expect_ok=True,
            expected_outputs={"version": "0.1.0", "is_prerelease": "false"},
        ),
    ),
    (
        "A6",
        "v0.1.0-rc1 -> version=0.1.0-rc1, is_prerelease=true",
        fixture_assertion(
            "A6",
            "v0.1.0-rc1",
            "tag",
            "v0.1.0-rc1",
            expect_ok=True,
            expected_outputs={"version": "0.1.0-rc1", "is_prerelease": "true"},
        ),
    ),
    (
        "A7",
        "v0.1.0-proof1 -> version=0.1.0-proof1, is_prerelease=true (the Phase 4 tag)",
        fixture_assertion(
            "A7",
            "v0.1.0-proof1",
            "tag",
            "v0.1.0-proof1",
            expect_ok=True,
            expected_outputs={"version": "0.1.0-proof1", "is_prerelease": "true"},
        ),
    ),
    (
        "A8",
        "v0.1 is rejected with the named semver error",
        fixture_assertion(
            "A8",
            "v0.1",
            "tag",
            "v0.1",
            expect_ok=False,
            expected_messages=("::error::", "'v0.1'", SEMVER_ERROR_NEEDLE),
        ),
    ),
    (
        "A9",
        "v1 is rejected with the named semver error",
        fixture_assertion(
            "A9",
            "v1",
            "tag",
            "v1",
            expect_ok=False,
            expected_messages=("::error::", "'v1'", SEMVER_ERROR_NEEDLE),
        ),
    ),
    (
        "A10",
        "v0.1.0.0 is rejected with the named semver error",
        fixture_assertion(
            "A10",
            "v0.1.0.0",
            "tag",
            "v0.1.0.0",
            expect_ok=False,
            expected_messages=("::error::", "'v0.1.0.0'", SEMVER_ERROR_NEEDLE),
        ),
    ),
    (
        "A11",
        "v0.1.0+build.5 is rejected: build metadata would mismatch the release assets",
        fixture_assertion(
            "A11",
            "v0.1.0+build.5",
            "tag",
            "v0.1.0+build.5",
            expect_ok=False,
            expected_messages=("::error::", "'v0.1.0+build.5'", SEMVER_ERROR_NEEDLE),
        ),
    ),
    (
        "A12",
        "branch ref (main, sha abcdef1234) -> version=0.0.0-abcdef1, is_prerelease=true",
        fixture_assertion(
            "A12",
            "branch main",
            "branch",
            "main",
            expect_ok=True,
            expected_outputs={"version": "0.0.0-abcdef1", "is_prerelease": "true"},
        ),
    ),
    (
        "A13",
        "1.0.0 (no leading v) is rejected by the v* case",
        fixture_assertion(
            "A13",
            "1.0.0",
            "tag",
            "1.0.0",
            expect_ok=False,
            expected_messages=("::error::", "'1.0.0'", TRIGGER_ERROR_NEEDLE),
        ),
    ),
    (
        "A14",
        "the release step is tag-gated (if: github.ref_type == 'tag')",
        a14_release_tag_gate,
    ),
    (
        "A15",
        "the release step passes prerelease from needs.version.outputs.is_prerelease",
        a15_prerelease_wiring,
    ),
    ("A16", "publish has no job-level if", a16_publish_has_no_job_if),
    ("A17", "container is tag-gated", a17_container_tag_gate),
]


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_WORKFLOW
    try:
        doc = yaml.safe_load(path.read_text(encoding="utf-8"))
    except OSError as exc:
        print(f"FAIL: cannot read {path}: {exc}")
        return 1
    except yaml.YAMLError as exc:
        print(f"FAIL: {path} is not valid YAML: {exc}")
        return 1
    if not isinstance(doc, dict):
        print(f"FAIL: {path} did not parse to a mapping")
        return 1

    failed = 0
    for assertion_id, description, assertion in ASSERTIONS:
        problems = assertion(doc)
        if problems:
            failed += 1
            print(f"FAIL: {assertion_id} {description}")
            for problem in problems:
                print(f"      - {problem}")
        else:
            print(f"PASS: {assertion_id} {description}")
            if assertion_id in OBSERVED:
                print(f"      observed: {OBSERVED[assertion_id]}")

    total = len(ASSERTIONS)
    if failed:
        print(f"tag-shape: {total - failed}/{total} assertions PASS, {failed} FAILED")
        return 1
    print(f"tag-shape: {total}/{total} assertions PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
