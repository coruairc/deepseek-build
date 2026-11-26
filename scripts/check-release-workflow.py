#!/usr/bin/env python3
"""Static assertions over .github/workflows/release.yml.

Why this exists
---------------
Nothing in this repo checked the release job graph. `needs:` edges, matrix
legs, artifact names and step ordering were silently editable, and a wrong edit
only surfaced on a real tag push — where `container` had already skipped or a
release had already been created. This script parses the workflow with PyYAML
and asserts the structure Phase 3 promised (REL-01, REL-08):

  - exactly six jobs: version, build-linux, build-macos, build-windows,
    publish, container;
  - each matrix build job owns only its own OS matrix and has no `runner.os`
    conditionals left behind;
  - `build-windows` is a single native windows-latest leg that installs protoc,
    builds `--locked`, smokes the exe, and packages/uploads a `.zip`;
  - `publish` needs all three build jobs (its verify step names all four
    tarballs and the Windows zip) and `container` needs only the Linux one (a
    slow macOS/Windows leg must not erase the image);
  - the `release-<target>` artifact contract survives the split;
  - `--locked` is passed to the CI build so a stale Cargo.lock fails the job
    instead of being regenerated;
  - the image tag-list guard runs after the metadata step and before the first
    `build-push-action`, i.e. before `:latest` can move;
  - the `version` job rejects a non-semver tag before anything builds;
  - `build-macos` signs the linked binary ad-hoc after the build and before
    packaging, and verifies the *extracted tarball* (signature, version
    needles, no `__DWARF` segment, no `.dSYM`) before it uploads;
  - the macOS job carries its own 240-minute budget while the Linux and
    Windows jobs keep 90, and no `CARGO_PROFILE_*` override landed in a build
    job by accident;
  - the Windows zip is hashed into SHA256SUMS and attached to the release with
    install.ps1;
  - no `run:` body invokes `strip`/`dsymutil`, and every `uses:` stays pinned
    to a full-length commit SHA.

After the structural assertions this script also runs the macOS
extracted-artifact step's shell body in a temp directory against stubbed
`codesign`/`otool` and a fake tarball (the Linux fixture). That exercises the
step's paths, needles and DWARF gate as written, so a broken path or a changed
needle fails here instead of on the first real tag. It proves step *logic*
only: no Mach-O signature is created or verified on this host, so REL-03's
macOS execution half still needs the real tag run.

Usage
-----
    python3 scripts/check-release-workflow.py [path/to/release.yml]

Prints one PASS/FAIL line per numbered assertion, the fixture tally, and every
violation found, then exits 1 if any assertion or fixture case failed.
Defaults to the repo's release.yml relative to this script. Requires Python 3,
PyYAML, `bash` and `tar`.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any, Callable

import yaml

DEFAULT_WORKFLOW = (
    Path(__file__).resolve().parent.parent / ".github" / "workflows" / "release.yml"
)

EXPECTED_JOBS = {
    "version",
    "build-linux",
    "build-macos",
    "build-windows",
    "publish",
    "container",
}

LINUX_LEGS = {
    ("x86_64-unknown-linux-gnu", "ubuntu-22.04"),
    ("aarch64-unknown-linux-gnu", "ubuntu-22.04-arm"),
}
MACOS_LEGS = {
    ("aarch64-apple-darwin", "macos-15"),
    ("x86_64-apple-darwin", "macos-15-intel"),
}
TARBALL_TARGETS = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
]
WINDOWS_TARGET = "x86_64-pc-windows-msvc"
ALL_TARGETS = [*TARBALL_TARGETS, WINDOWS_TARGET]

# The macOS artifact contract (REL-03, REL-02) names these explicitly, and the
# fixture drives the verify step through the same variables the step declares.
SIGN_STEP_NAME = "Ad-hoc sign the macOS binary"
ARTIFACT_STEP_NAME = "Verify the packaged macOS artifact"
ARTIFACT_VERIFY_DIR_ENV = "ARTIFACT_VERIFY_DIR"
FIXTURE_TARGET = "x86_64-apple-darwin"
LINUX_TIMEOUT_MINUTES = 90
MACOS_TIMEOUT_MINUTES = 240
WINDOWS_TIMEOUT_MINUTES = 90

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
    expected = ["version", "build-linux", "build-macos", "build-windows"]
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
    for job_name in ("build-linux", "build-macos", "build-windows"):
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
    windows = jobs.get("build-windows")
    if not isinstance(windows, dict):
        problems.append("job 'build-windows' is missing")
        return problems
    uploads = [
        s for s in steps_of(windows) if uses_of(s).startswith("actions/upload-artifact@")
    ]
    if len(uploads) != 1:
        problems.append(
            f"build-windows has {len(uploads)} upload-artifact step(s), expected exactly 1"
        )
        return problems
    options = with_of(uploads[0])
    if options.get("name") != f"release-{WINDOWS_TARGET}":
        problems.append(
            f"build-windows upload-artifact with.name is {options.get('name')!r}, "
            f"expected 'release-{WINDOWS_TARGET}'"
        )
    if options.get("path") != "deepseek-build-*.zip":
        problems.append(
            f"build-windows upload-artifact with.path is {options.get('path')!r}, "
            "expected 'deepseek-build-*.zip'"
        )
    if options.get("if-no-files-found") != "error":
        problems.append(
            "build-windows upload-artifact with.if-no-files-found is "
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
    verifiers = [
        s
        for s in steps
        if all(t in run_of(s) for t in TARBALL_TARGETS)
        and WINDOWS_TARGET in run_of(s)
        and ".zip" in run_of(s)
    ]
    if not verifiers:
        problems.append(
            "publish has no verify step naming all four tarball targets and the "
            "Windows .zip"
        )
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


def a14_container_guard(doc: Workflow) -> list[str]:
    container = jobs_of(doc).get("container")
    if not isinstance(container, dict):
        return ["job 'container' is missing"]
    problems: list[str] = []
    steps = steps_of(container)
    metadata = [
        i for i, s in enumerate(steps) if uses_of(s).startswith("docker/metadata-action@")
    ]
    pushes = [
        i
        for i, s in enumerate(steps)
        if uses_of(s).startswith("docker/build-push-action@")
    ]
    guards = [
        i for i, s in enumerate(steps) if run_of(s).strip() == "scripts/guard-image-tags.sh"
    ]
    if len(metadata) != 1:
        problems.append(
            f"container has {len(metadata)} metadata-action step(s), expected exactly 1"
        )
    if len(pushes) != 2:
        problems.append(
            f"container has {len(pushes)} build-push-action step(s), expected exactly 2"
        )
    if len(guards) != 1:
        problems.append(
            f"container has {len(guards)} step(s) running scripts/guard-image-tags.sh, "
            "expected exactly 1"
        )
    if problems:
        return problems
    guard = guards[0]
    if not metadata[0] < guard:
        problems.append("the tag guard runs before the metadata step")
    if not guard < min(pushes):
        problems.append(
            "the tag guard runs after a build-push step (too late: :latest moves there)"
        )
    name = str(steps[guard].get("name") or "").strip()
    if name != "Guard the resolved image tag list":
        problems.append(
            f"guarded step name is {name!r}, expected 'Guard the resolved image tag list'"
        )
    env = env_of(steps[guard])
    for key in ("IMAGE", "TAGS", "RELEASE_VERSION", "IS_PRERELEASE"):
        if not str(env.get(key) or "").strip():
            problems.append(f"the tag guard's env is missing {key}")
    return problems


def a15_semver_gate(doc: Workflow) -> list[str]:
    version = jobs_of(doc).get("version")
    if not isinstance(version, dict):
        return ["job 'version' is missing"]
    derive = [s for s in steps_of(version) if s.get("id") == "derive"]
    if len(derive) != 1:
        return [f"version has {len(derive)} step(s) with id 'derive', expected 1"]
    body = run_of(derive[0])
    problems: list[str] = []
    if SEMVER_RE_TEXT not in body:
        problems.append(
            "the derive step no longer contains the strict semver gate regex "
            f"{SEMVER_RE_TEXT!r}"
        )
    if "::error::release tag" not in body:
        problems.append(
            "the derive step does not emit a named ::error:: for a non-semver tag"
        )
    return problems


def step_indices_by_name(job: dict[str, Any], name: str) -> list[int]:
    return [
        index
        for index, step in enumerate(steps_of(job))
        if str(step.get("name") or "").strip() == name
    ]


def a16_macos_adhoc_sign(doc: Workflow) -> list[str]:
    """REL-03/T-03-08: sign explicitly after linking and before packaging."""
    job = jobs_of(doc).get("build-macos")
    if not isinstance(job, dict):
        return ["job 'build-macos' is missing"]
    steps = steps_of(job)
    problems: list[str] = []
    builds = [i for i, s in enumerate(steps) if "cargo build" in run_of(s)]
    packages = step_indices_by_name(job, "Package tarball")
    signs = [
        i for i, s in enumerate(steps) if "codesign --force --sign -" in run_of(s)
    ]
    if len(builds) != 1:
        problems.append(
            f"build-macos has {len(builds)} 'cargo build' step(s), expected exactly 1"
        )
    if len(packages) != 1:
        problems.append(
            f"build-macos has {len(packages)} 'Package tarball' step(s), expected exactly 1"
        )
    if len(signs) != 1:
        problems.append(
            "build-macos has "
            f"{len(signs)} step(s) running 'codesign --force --sign -', expected exactly 1"
        )
    if problems:
        return problems
    if not builds[0] < signs[0] < packages[0]:
        problems.append(
            "the ad-hoc sign step is not between the 'cargo build' step and "
            "'Package tarball'"
        )
    name = str(steps[signs[0]].get("name") or "").strip()
    if name != SIGN_STEP_NAME:
        problems.append(f"signed step name is {name!r}, expected {SIGN_STEP_NAME!r}")
    return problems


def a17_macos_verify_extracted(doc: Workflow) -> list[str]:
    """REL-03: verify the packaged tarball, not the pre-tar build output."""
    job = jobs_of(doc).get("build-macos")
    if not isinstance(job, dict):
        return ["job 'build-macos' is missing"]
    steps = steps_of(job)
    problems: list[str] = []
    packages = step_indices_by_name(job, "Package tarball")
    uploads = [
        i
        for i, s in enumerate(steps)
        if uses_of(s).startswith("actions/upload-artifact@")
    ]
    verifies = step_indices_by_name(job, ARTIFACT_STEP_NAME)
    if len(packages) != 1:
        problems.append(
            f"build-macos has {len(packages)} 'Package tarball' step(s), expected exactly 1"
        )
    if len(uploads) != 1:
        problems.append(
            f"build-macos has {len(uploads)} upload-artifact step(s), expected exactly 1"
        )
    if len(verifies) != 1:
        problems.append(
            f"build-macos has {len(verifies)} step(s) named {ARTIFACT_STEP_NAME!r}, "
            "expected exactly 1"
        )
    if problems:
        return problems
    if not packages[0] < verifies[0] < uploads[0]:
        problems.append(
            f"{ARTIFACT_STEP_NAME!r} is not between 'Package tarball' and the "
            "upload-artifact step"
        )
    step = steps[verifies[0]]
    body = run_of(step)
    for needle in (
        "tar -xzf",
        "codesign --verify --strict",
        "--version",
        "__DWARF",
        "*.dSYM",
    ):
        if needle not in body:
            problems.append(
                f"{ARTIFACT_STEP_NAME!r} run body is missing {needle!r}"
            )
    env = env_of(step)
    for key in ("RELEASE_VERSION", "RT", ARTIFACT_VERIFY_DIR_ENV):
        if not str(env.get(key) or "").strip():
            problems.append(f"{ARTIFACT_STEP_NAME!r} env is missing {key}")
    return problems


def a18_build_timeouts(doc: Workflow) -> list[str]:
    """REL-02: the macOS budget is larger; the Linux budget is untouched."""
    jobs = jobs_of(doc)
    problems: list[str] = []
    for job_name, expected in (
        ("build-linux", LINUX_TIMEOUT_MINUTES),
        ("build-macos", MACOS_TIMEOUT_MINUTES),
        ("build-windows", WINDOWS_TIMEOUT_MINUTES),
    ):
        job = jobs.get(job_name)
        if not isinstance(job, dict):
            problems.append(f"job '{job_name}' is missing")
            continue
        value = job.get("timeout-minutes")
        if value != expected:
            problems.append(
                f"{job_name}.timeout-minutes is {value!r}, expected {expected}"
            )
    return problems


def a19_no_profile_overrides(doc: Workflow) -> list[str]:
    """No `CARGO_PROFILE_*` override may land in a build job by accident."""
    problems: list[str] = []
    jobs = jobs_of(doc)
    for job_name in ("build-linux", "build-macos", "build-windows"):
        job = jobs.get(job_name)
        if not isinstance(job, dict):
            continue
        job_env = job.get("env")
        for key in (job_env if isinstance(job_env, dict) else {}):
            if "CARGO_PROFILE_" in str(key):
                problems.append(f"{job_name} sets job-level env {key!r}")
        for index, step in enumerate(steps_of(job), start=1):
            if "CARGO_PROFILE_" in run_of(step):
                problems.append(
                    f"{job_name} step {index} ({step.get('name', '?')!r}) run body "
                    "mentions CARGO_PROFILE_"
                )
            for key, value in env_of(step).items():
                if "CARGO_PROFILE_" in str(key) or "CARGO_PROFILE_" in str(value):
                    problems.append(
                        f"{job_name} step {index} ({step.get('name', '?')!r}) env "
                        f"{key!r} = {value!r}"
                    )
    return problems


def a20_build_windows(doc: Workflow) -> list[str]:
    """WIN-01: a native Windows leg installs protoc, builds, and zips."""
    jobs = jobs_of(doc)
    job = jobs.get("build-windows")
    if not isinstance(job, dict):
        return ["job 'build-windows' is missing"]
    problems: list[str] = []
    if job.get("runs-on") != "windows-latest":
        problems.append(f"runs-on is {job.get('runs-on')!r}, expected 'windows-latest'")
    if needs_of(job) != ["version"]:
        problems.append(f"needs is {needs_of(job)!r}, expected ['version']")
    steps = steps_of(job)

    checkouts = [s for s in steps if uses_of(s).startswith("actions/checkout@")]
    publish = jobs.get("publish")
    publish_checkouts = (
        [s for s in steps_of(publish) if uses_of(s).startswith("actions/checkout@")]
        if isinstance(publish, dict)
        else []
    )
    if len(checkouts) != 1:
        problems.append(
            f"build-windows has {len(checkouts)} checkout step(s), expected exactly 1"
        )
    elif publish_checkouts and uses_of(checkouts[0]) != uses_of(publish_checkouts[0]):
        problems.append(
            f"build-windows checkout is {uses_of(checkouts[0])!r}, expected the "
            f"same pinned action as publish ({uses_of(publish_checkouts[0])!r})"
        )

    runs = "\n".join(run_of(s) for s in steps)
    if "1.94.0" not in runs:
        problems.append("build-windows never installs the pinned 1.94.0 toolchain")
    if "choco install protoc" not in runs or "PROTOC=" not in runs:
        problems.append(
            "build-windows does not install protoc with chocolatey and export PROTOC"
        )
    # find_protoc does PathBuf::try_exists on PROTOC, so a Git-Bash "/c/..." path is a silent
    # no-op on Windows: the override must be converted with cygpath -w before export.
    if "cygpath -w" not in runs:
        problems.append(
            "build-windows exports PROTOC without `cygpath -w`; a POSIX path is silently ignored"
        )

    caches = [s for s in steps if uses_of(s).startswith("Swatinem/rust-cache@")]
    if len(caches) != 1:
        problems.append(
            f"build-windows has {len(caches)} rust-cache step(s), expected exactly 1"
        )
    elif with_of(caches[0]).get("key") != WINDOWS_TARGET:
        problems.append(
            f"build-windows rust-cache key is {with_of(caches[0]).get('key')!r}, "
            f"expected {WINDOWS_TARGET!r}"
        )

    smokes = [
        s
        for s in steps
        if "deepseek-build.exe --version" in run_of(s)
        and env_of(s).get("RELEASE_VERSION") == "${{ needs.version.outputs.version }}"
    ]
    if not smokes:
        problems.append(
            "build-windows has no step that runs the built exe with --version and "
            "RELEASE_VERSION=needs.version.outputs.version"
        )

    for name in (
        "deepseek-build.exe",
        "LICENSE",
        "NOTICE",
        "THIRD-PARTY-NOTICES",
        "README.txt",
    ):
        if name not in runs:
            problems.append(f"build-windows never stages the top-level entry {name!r}")

    packagers = [s for s in steps if "Compress-Archive" in run_of(s)]
    if len(packagers) != 1:
        problems.append(
            f"build-windows has {len(packagers)} Compress-Archive step(s), expected exactly 1"
        )
    elif f"{WINDOWS_TARGET}.zip" not in run_of(packagers[0]):
        problems.append(
            "the Compress-Archive step does not name "
            f"deepseek-build-<version>-{WINDOWS_TARGET}.zip"
        )
    return problems


def a21_publish_zip_wiring(doc: Workflow) -> list[str]:
    """WIN-01: the zip is hashed and attached to the release."""
    publish = jobs_of(doc).get("publish")
    if not isinstance(publish, dict):
        return ["job 'publish' is missing"]
    steps = steps_of(publish)
    problems: list[str] = []
    sums = [
        s
        for s in steps
        if "sha256sum" in run_of(s) and "SHA256SUMS" in run_of(s)
    ]
    if len(sums) != 1:
        problems.append(
            f"publish has {len(sums)} SHA256SUMS step(s), expected exactly 1"
        )
    else:
        body = run_of(sums[0])
        for needle in ("deepseek-build-*.tar.gz", "deepseek-build-*.zip"):
            if needle not in body:
                problems.append(
                    f"the SHA256SUMS step does not hash {needle!r}: {body.strip()!r}"
                )
    releases = [
        s for s in steps if uses_of(s).startswith("softprops/action-gh-release@")
    ]
    if len(releases) != 1:
        problems.append(
            f"publish has {len(releases)} release step(s), expected exactly 1"
        )
        return problems
    files = str(with_of(releases[0]).get("files") or "")
    for needle in (
        "dist/SHA256SUMS",
        "dist/deepseek-build-*.tar.gz",
        "dist/deepseek-build-*.zip",
        "install.sh",
        "install.ps1",
    ):
        if needle not in files:
            problems.append(f"the release files list is missing {needle!r}")
    return problems


CODESIGN_STUB = """#!/bin/sh
# Linux fixture stand-in for macOS codesign: claim an ad-hoc signature, pass.
printf 'Executable=%s\\nSignature=adhoc\\n' "$*" >&2
exit 0
"""

OTOOL_STUB_CLEAN = """#!/bin/sh
# Linux fixture stand-in for macOS otool: no load commands, so no __DWARF.
exit 0
"""

OTOOL_STUB_DWARF = """#!/bin/sh
# Linux fixture stand-in: report a __DWARF segment the way `otool -l` would.
printf 'Load command 2\\n      cmd LC_SEGMENT_64\\n     segname __DWARF\\n'
exit 0
"""


def _write_stub(directory: Path, name: str, content: str) -> None:
    path = directory / name
    path.write_text(content, encoding="utf-8")
    path.chmod(0o755)


def run_artifact_step_fixture(doc: Workflow) -> list[str]:
    """Run the macOS verify step's shell body against stubs and a fake tarball.

    Proves the step's paths, needles and DWARF gate as written on Linux; it
    proves nothing about a real Mach-O signature, which only a macOS host can
    produce or verify. Each fixture sets RELEASE_VERSION, RT and
    ARTIFACT_VERIFY_DIR explicitly — the same names the step declares in
    `env:` — so no variable comes from this host's environment.
    """
    job = jobs_of(doc).get("build-macos")
    if not isinstance(job, dict):
        return ["fixture: job 'build-macos' is missing"]
    steps = steps_of(job)
    targets = step_indices_by_name(job, ARTIFACT_STEP_NAME)
    if len(targets) != 1:
        return [
            f"fixture: build-macos has {len(targets)} step(s) named "
            f"{ARTIFACT_STEP_NAME!r}, expected exactly 1"
        ]
    body = run_of(steps[targets[0]])
    if not body.strip():
        return [f"fixture: {ARTIFACT_STEP_NAME!r} has an empty run body"]

    # label, tarball version, version the binary prints, otool stub, .dSYM, expect success
    cases = [
        ("good 0.1.0", "0.1.0", "0.1.0", "clean", False, True),
        ("good 0.1.0-proof1", "0.1.0-proof1", "0.1.0-proof1", "clean", False, True),
        ("wrong --version fails", "0.1.0", "9.9.9", "clean", False, False),
        ("stubbed __DWARF fails", "0.1.0", "0.1.0", "dwarf", False, False),
        ("stray .dSYM fails", "0.1.0", "0.1.0", "clean", True, False),
    ]
    problems: list[str] = []
    passed = 0
    with tempfile.TemporaryDirectory(prefix="release-workflow-fixture-") as work:
        root = Path(work)
        for (
            label,
            tarball_version,
            printed_version,
            otool_mode,
            make_dsym,
            expect_ok,
        ) in cases:
            case_dir = root / re.sub(r"[^a-z0-9]+", "-", label.lower()).strip("-")
            stage = case_dir / "stage"
            stage.mkdir(parents=True)
            binary = stage / "deepseek-build"
            binary.write_text(
                "#!/bin/sh\nprintf 'deepseek-build %s (abcdef123456)\\n' '"
                + printed_version
                + "'\n",
                encoding="utf-8",
            )
            binary.chmod(0o755)
            tarball = (
                case_dir / f"deepseek-build-{tarball_version}-{FIXTURE_TARGET}.tar.gz"
            )
            try:
                subprocess.run(
                    ["tar", "-czf", str(tarball), "-C", str(stage), "deepseek-build"],
                    check=True,
                    capture_output=True,
                )
            except (OSError, subprocess.CalledProcessError) as exc:
                problems.append(f"fixture case {label!r}: could not build tarball: {exc}")
                continue
            if make_dsym:
                (case_dir / "artifact.dSYM").mkdir()
            stubs = case_dir / "stubs"
            stubs.mkdir()
            _write_stub(stubs, "codesign", CODESIGN_STUB)
            _write_stub(
                stubs,
                "otool",
                OTOOL_STUB_DWARF if otool_mode == "dwarf" else OTOOL_STUB_CLEAN,
            )
            env = {
                "PATH": f"{stubs}:{os.environ.get('PATH', '/usr/bin:/bin')}",
                "HOME": str(case_dir),
                "RELEASE_VERSION": tarball_version,
                "RT": FIXTURE_TARGET,
                ARTIFACT_VERIFY_DIR_ENV: str(case_dir / "extract"),
            }
            try:
                proc = subprocess.run(
                    ["bash", "-c", body],
                    cwd=case_dir,
                    env=env,
                    capture_output=True,
                    text=True,
                    timeout=120,
                )
            except subprocess.TimeoutExpired:
                problems.append(f"fixture case {label!r}: timed out after 120s")
                continue
            if (proc.returncode == 0) == expect_ok:
                passed += 1
            else:
                expected = "0" if expect_ok else "non-zero"
                problems.append(
                    f"fixture case {label!r}: exit {proc.returncode}, expected "
                    f"{expected}; stdout={proc.stdout!r} stderr={proc.stderr!r}"
                )
    total = len(cases)
    if problems:
        print(f"FAIL: artifact-step fixture: {passed}/{total} PASS")
        return problems
    print(f"artifact-step fixture: {passed}/{total} PASS")
    return []


ASSERTIONS: list[Assertion] = [
    (
        "A1",
        "jobs are exactly version/build-linux/build-macos/build-windows/publish/container",
        a1_job_set,
    ),
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
    (
        "A5",
        "publish needs [version, build-linux, build-macos, build-windows] and has no if",
        a5_publish_needs,
    ),
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
        "build jobs upload release-<target>; build-windows uploads the zip, if-no-files-found=error",
        a8_upload_artifact,
    ),
    (
        "A9",
        "publish merges release-* and verifies the four tarballs and the Windows zip",
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
    (
        "A14",
        "the tag-list guard sits between metadata-action and both pushes",
        a14_container_guard,
    ),
    ("A15", "the version job's derive step carries the strict semver gate", a15_semver_gate),
    (
        "A16",
        "build-macos signs the binary ad-hoc after the build and before 'Package tarball'",
        a16_macos_adhoc_sign,
    ),
    (
        "A17",
        "the packaged macOS artifact is verified after packaging and before the upload",
        a17_macos_verify_extracted,
    ),
    (
        "A18",
        "build-macos budget is 240 minutes; build-linux and build-windows keep 90",
        a18_build_timeouts,
    ),
    ("A19", "no CARGO_PROFILE_* override lands in a build job", a19_no_profile_overrides),
    (
        "A20",
        "build-windows: protoc, pinned toolchain, exe smoke, zip with the five top-level entries",
        a20_build_windows,
    ),
    (
        "A21",
        "publish hashes the zip and attaches it (plus install.ps1) to the release",
        a21_publish_zip_wiring,
    ),
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

    # The fixture runs after the structural assertions: it needs the step the
    # assertions locate, and it exercises the real shell body, not a copy.
    fixture_problems = run_artifact_step_fixture(doc)
    for problem in fixture_problems:
        print(f"      - {problem}")

    total = len(ASSERTIONS)
    if failed:
        print(f"release-workflow: {total - failed}/{total} assertions PASS, {failed} FAILED")
        return 1
    if not triggers:
        print("FAIL: the workflow has no trigger block")
        return 1
    print(f"release-workflow: {total}/{total} assertions PASS")
    return 1 if fixture_problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
