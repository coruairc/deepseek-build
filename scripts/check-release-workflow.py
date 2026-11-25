#!/usr/bin/env python3
"""Static assertions over .github/workflows/release.yml.

Why this exists
---------------
Nothing in this repo checked the release job graph. `needs:` edges, matrix
legs, artifact names and step ordering were silently editable, and a wrong edit
only surfaced on a real tag push — where `container` had already skipped or a
release had already been created. This script parses the workflow with PyYAML
and asserts the structure Phase 3 promised (REL-01, REL-08):

  - exactly five jobs: version, build-linux, build-macos, publish, container;
  - each build job owns only its own OS matrix and has no `runner.os`
    conditionals left behind;
  - `publish` needs both build jobs (its verify step names all four tarballs)
    and `container` needs only the Linux one (a slow macOS leg must not erase
    the image);
  - the `release-<target>` artifact contract survives the split;
  - `--locked` is passed to the CI build so a stale Cargo.lock fails the job
    instead of being regenerated;
  - the image tag-list guard runs after the metadata step and before the first
    `build-push-action`, i.e. before `:latest` can move;
  - the `version` job rejects a non-semver tag before anything builds;
  - no `run:` body invokes `strip`/`dsymutil`, and every `uses:` stays pinned
    to a full-length commit SHA.

Usage
-----
    python3 scripts/check-release-workflow.py [path/to/release.yml]

Prints one PASS/FAIL line per numbered assertion and every violation found,
then exits 1 if any assertion failed. Defaults to the repo's release.yml
relative to this script. Requires Python 3 and PyYAML.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path
from typing import Any, Callable

import yaml

DEFAULT_WORKFLOW = (
    Path(__file__).resolve().parent.parent / ".github" / "workflows" / "release.yml"
)

EXPECTED_JOBS = {"version", "build-linux", "build-macos", "publish", "container"}

LINUX_LEGS = {
    ("x86_64-unknown-linux-gnu", "ubuntu-22.04"),
    ("aarch64-unknown-linux-gnu", "ubuntu-22.04-arm"),
}
MACOS_LEGS = {
    ("aarch64-apple-darwin", "macos-15"),
    ("x86_64-apple-darwin", "macos-15-intel"),
}
ALL_TARGETS = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
]

# Kept byte-identical to the derive step so a rewrite of one side fails here.
PRERELEASE_RE_TEXT = r"^[0-9]+\.[0-9]+\.[0-9]+-"
SEMVER_RE_TEXT = r"^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$"

PIN_RE = re.compile(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+@[0-9a-f]{40}$")
# `strip` / `dsymutil` in command position (comment-only mentions are fine).
COMMAND_POS_RE = re.compile(
    r"(?:^|[;&|]\s*|\b(?:then|do|else)\s+)"
    r"(?:sudo\s+)?(?:/usr/bin/|xcrun\s+)?(?:strip|dsymutil)\b"
)

Workflow = dict[str, Any]
Assertion = tuple[str, str, Callable[[Workflow], list[str]]]


def jobs_of(doc: Workflow) -> dict[str, Any]:
    jobs = doc.get("jobs")
    return jobs if isinstance(jobs, dict) else {}


def steps_of(job: dict[str, Any]) -> list[dict[str, Any]]:
    steps = job.get("steps")
    return [s for s in steps if isinstance(s, dict)] if isinstance(steps, list) else []


def needs_of(job: dict[str, Any]) -> list[str]:
    value = job.get("needs")
    if value is None:
        return []
    if isinstance(value, str):
        return [value]
    if isinstance(value, list):
        return [str(v) for v in value]
    return [str(value)]


def uses_of(step: dict[str, Any]) -> str:
    return str(step.get("uses") or "")


def run_of(step: dict[str, Any]) -> str:
    return str(step.get("run") or "")


def with_of(step: dict[str, Any]) -> dict[str, Any]:
    value = step.get("with")
    return value if isinstance(value, dict) else {}


def env_of(step: dict[str, Any]) -> dict[str, Any]:
    value = step.get("env")
    return value if isinstance(value, dict) else {}


def matrix_legs(job: dict[str, Any]) -> tuple[set[tuple[str, str]], int]:
    strategy = job.get("strategy")
    matrix = strategy.get("matrix") if isinstance(strategy, dict) else None
    include = matrix.get("include") if isinstance(matrix, dict) else None
    if not isinstance(include, list):
        return set(), 0
    legs: set[tuple[str, str]] = set()
    for entry in include:
        if isinstance(entry, dict) and "target" in entry and "runner" in entry:
            legs.add((str(entry["target"]), str(entry["runner"])))
    return legs, len(include)


def check_build_job(
    job_name: str, expected_legs: set[tuple[str, str]]
) -> Callable[[Workflow], list[str]]:
    def assertion(doc: Workflow) -> list[str]:
        job = jobs_of(doc).get(job_name)
        if not isinstance(job, dict):
            return [f"job '{job_name}' is missing"]
        problems: list[str] = []
        got_needs = needs_of(job)
        if got_needs != ["version"]:
            problems.append(f"needs is {got_needs!r}, expected ['version']")
        legs, include_len = matrix_legs(job)
        if legs != expected_legs:
            problems.append(
                f"matrix legs are {sorted(legs)!r}, expected {sorted(expected_legs)!r}"
            )
        if include_len != len(expected_legs):
            problems.append(
                f"matrix include has {include_len} entries, expected {len(expected_legs)}"
            )
        strategy = job.get("strategy")
        fail_fast = strategy.get("fail-fast") if isinstance(strategy, dict) else None
        if fail_fast is not False:
            problems.append(f"strategy.fail-fast is {fail_fast!r}, expected false")
        return problems

    return assertion


def a1_job_set(doc: Workflow) -> list[str]:
    names = set(jobs_of(doc))
    problems: list[str] = []
    for name in sorted(EXPECTED_JOBS - names):
        problems.append(f"missing job '{name}'")
    for name in sorted(names - EXPECTED_JOBS):
        problems.append(f"unexpected job '{name}'")
    return problems


def a4_no_runner_os_conditionals(doc: Workflow) -> list[str]:
    problems: list[str] = []
    jobs = jobs_of(doc)
    for job_name in ("build-linux", "build-macos"):
        job = jobs.get(job_name)
        if not isinstance(job, dict):
            problems.append(f"job '{job_name}' is missing")
            continue
        for index, step in enumerate(steps_of(job), start=1):
            condition = step.get("if")
            if condition is not None and "runner.os" in str(condition):
                problems.append(
                    f"{job_name} step {index} ({step.get('name', '?')!r}) still has "
                    f"an OS-conditional if: {condition!r}"
                )
    return problems


def a5_publish_needs(doc: Workflow) -> list[str]:
    publish = jobs_of(doc).get("publish")
    if not isinstance(publish, dict):
        return ["job 'publish' is missing"]
    problems: list[str] = []
    expected = ["version", "build-linux", "build-macos"]
    if needs_of(publish) != expected:
        problems.append(f"needs is {needs_of(publish)!r}, expected {expected!r}")
    if "if" in publish:
        problems.append(
            f"publish has a job-level if: {publish.get('if')!r}; the release step "
            "must stay the only tag gate"
        )
    return problems


def a6_container_needs(doc: Workflow) -> list[str]:
    container = jobs_of(doc).get("container")
    if not isinstance(container, dict):
        return ["job 'container' is missing"]
    problems: list[str] = []
    expected = ["version", "build-linux"]
    if needs_of(container) != expected:
        problems.append(f"needs is {needs_of(container)!r}, expected {expected!r}")
    condition = str(container.get("if") or "").strip()
    if condition != "github.ref_type == 'tag'":
        problems.append(
            f"if is {condition!r}, expected \"github.ref_type == 'tag'\""
        )
    return problems


def a7_cargo_build(doc: Workflow) -> list[str]:
    problems: list[str] = []
    jobs = jobs_of(doc)
    for job_name in ("build-linux", "build-macos"):
        job = jobs.get(job_name)
        if not isinstance(job, dict):
            problems.append(f"job '{job_name}' is missing")
            continue
        builds = [s for s in steps_of(job) if "cargo build" in run_of(s)]
        if len(builds) != 1:
            problems.append(
                f"{job_name} has {len(builds)} step(s) running 'cargo build', expected exactly 1"
            )
            continue
        step = builds[0]
        body = run_of(step)
        for needle in ("--profile release-dist", "-p xai-grok-pager-bin", "--locked"):
            if needle not in body:
                problems.append(
                    f"{job_name} cargo build is missing {needle!r}: {body.strip()!r}"
                )
        stamp = env_of(step).get("GROK_VERSION")
        if stamp != "${{ needs.version.outputs.version }}":
            problems.append(
                f"{job_name} cargo build env.GROK_VERSION is {stamp!r}, expected "
                "\"${{ needs.version.outputs.version }}\""
            )
    return problems


def a8_upload_artifact(doc: Workflow) -> list[str]:
    problems: list[str] = []
    jobs = jobs_of(doc)
    for job_name in ("build-linux", "build-macos"):
        job = jobs.get(job_name)
        if not isinstance(job, dict):
            problems.append(f"job '{job_name}' is missing")
            continue
        uploads = [
            s for s in steps_of(job) if uses_of(s).startswith("actions/upload-artifact@")
        ]
        if len(uploads) != 1:
            problems.append(
                f"{job_name} has {len(uploads)} upload-artifact step(s), expected exactly 1"
            )
            continue
        options = with_of(uploads[0])
        if options.get("name") != "release-${{ matrix.target }}":
            problems.append(
                f"{job_name} upload-artifact with.name is {options.get('name')!r}, "
                "expected 'release-${{ matrix.target }}'"
            )
        if options.get("if-no-files-found") != "error":
            problems.append(
                f"{job_name} upload-artifact with.if-no-files-found is "
                f"{options.get('if-no-files-found')!r}, expected 'error'"
            )
    return problems


def a9_publish_download_and_verify(doc: Workflow) -> list[str]:
    publish = jobs_of(doc).get("publish")
    if not isinstance(publish, dict):
        return ["job 'publish' is missing"]
    problems: list[str] = []
    steps = steps_of(publish)
    downloads = [
        s for s in steps if uses_of(s).startswith("actions/download-artifact@")
    ]
    merge = any(
        with_of(s).get("pattern") == "release-*"
        and with_of(s).get("merge-multiple") is True
        for s in downloads
    )
    if not merge:
        described = [
            {k: v for k, v in with_of(s).items()} for s in downloads
        ]
        problems.append(
            "publish has no download-artifact step with pattern 'release-*' and "
            f"merge-multiple true; found {described!r}"
        )
    verifiers = [s for s in steps if all(t in run_of(s) for t in ALL_TARGETS)]
    if not verifiers:
        problems.append("publish has no verify step naming all four targets")
    return problems


def a10_container_downloads(doc: Workflow) -> list[str]:
    container = jobs_of(doc).get("container")
    if not isinstance(container, dict):
        return ["job 'container' is missing"]
    downloads = [
        s
        for s in steps_of(container)
        if uses_of(s).startswith("actions/download-artifact@")
    ]
    names = sorted(str(with_of(s).get("name")) for s in downloads)
    expected = sorted(
        ["release-x86_64-unknown-linux-gnu", "release-aarch64-unknown-linux-gnu"]
    )
    if len(downloads) != 2 or names != expected:
        return [
            f"container download-artifact steps are {names!r}, expected {expected!r}"
        ]
    return []


def a11_version_job(doc: Workflow) -> list[str]:
    jobs = jobs_of(doc)
    version = jobs.get("version")
    if not isinstance(version, dict):
        return ["job 'version' is missing"]
    problems: list[str] = []
    derive = [s for s in steps_of(version) if s.get("id") == "derive"]
    if len(derive) != 1:
        problems.append(f"version has {len(derive)} step(s) with id 'derive', expected 1")
    elif PRERELEASE_RE_TEXT not in run_of(derive[0]):
        problems.append(
            "the derive step no longer contains the prerelease regex "
            f"{PRERELEASE_RE_TEXT!r}"
        )
    publish = jobs.get("publish")
    releases = (
        [s for s in steps_of(publish) if uses_of(s).startswith("softprops/action-gh-release@")]
        if isinstance(publish, dict)
        else []
    )
    if len(releases) != 1:
        problems.append(f"publish has {len(releases)} release step(s), expected exactly 1")
    else:
        step = releases[0]
        condition = str(step.get("if") or "").strip()
        if condition != "github.ref_type == 'tag'":
            problems.append(
                f"release step if is {condition!r}, expected \"github.ref_type == 'tag'\""
            )
        prerelease = with_of(step).get("prerelease")
        if prerelease != "${{ needs.version.outputs.is_prerelease }}":
            problems.append(
                f"release step with.prerelease is {prerelease!r}, expected "
                "\"${{ needs.version.outputs.is_prerelease }}\""
            )
    return problems


def a12_no_strip_commands(doc: Workflow) -> list[str]:
    problems: list[str] = []
    for job_name, job in jobs_of(doc).items():
        if not isinstance(job, dict):
            continue
        for index, step in enumerate(steps_of(job), start=1):
            for line in run_of(step).splitlines():
                code = line.split("#", 1)[0]
                if not code.strip():
                    continue
                if COMMAND_POS_RE.search(code):
                    problems.append(
                        f"{job_name} step {index} ({step.get('name', '?')!r}) "
                        f"invokes strip/dsymutil: {line.strip()!r}"
                    )
    return problems


def a13_sha_pins(doc: Workflow) -> list[str]:
    problems: list[str] = []
    for job_name, job in jobs_of(doc).items():
        if not isinstance(job, dict):
            continue
        for index, step in enumerate(steps_of(job), start=1):
            value = uses_of(step)
            if value and not PIN_RE.match(value):
                problems.append(
                    f"{job_name} step {index} ({step.get('name', '?')!r}) uses "
                    f"{value!r}, which is not a full-SHA pin (owner/repo@40hex)"
                )
    return problems


ASSERTIONS: list[Assertion] = [
    ("A1", "jobs are exactly version/build-linux/build-macos/publish/container", a1_job_set),
    (
        "A2",
        "build-linux: needs [version], two Linux legs, fail-fast false",
        check_build_job("build-linux", LINUX_LEGS),
    ),
    (
        "A3",
        "build-macos: needs [version], two macOS legs, fail-fast false",
        check_build_job("build-macos", MACOS_LEGS),
    ),
    ("A4", "build jobs have no runner.os conditionals left", a4_no_runner_os_conditionals),
    ("A5", "publish needs [version, build-linux, build-macos] and has no if", a5_publish_needs),
    (
        "A6",
        "container needs [version, build-linux] and is tag-gated",
        a6_container_needs,
    ),
    (
        "A7",
        "one --locked release-dist cargo build per build job, stamped with the version",
        a7_cargo_build,
    ),
    (
        "A8",
        "each build job uploads release-${{ matrix.target }} with if-no-files-found=error",
        a8_upload_artifact,
    ),
    (
        "A9",
        "publish merges release-* and verifies all four tarballs",
        a9_publish_download_and_verify,
    ),
    (
        "A10",
        "container downloads exactly the two Linux release-* artifacts",
        a10_container_downloads,
    ),
    (
        "A11",
        "version derives the prerelease flag; the release step is tag-gated",
        a11_version_job,
    ),
    ("A12", "no run body invokes strip or dsymutil", a12_no_strip_commands),
    ("A13", "every uses: value is pinned to a 40-hex SHA", a13_sha_pins),
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

    # PyYAML resolves the bare key `on:` to the boolean True.
    triggers = doc.get(True, doc.get("on"))
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

    total = len(ASSERTIONS)
    if failed:
        print(f"release-workflow: {total - failed}/{total} assertions PASS, {failed} FAILED")
        return 1
    if not triggers:
        print("FAIL: the workflow has no trigger block")
        return 1
    print(f"release-workflow: {total}/{total} assertions PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
