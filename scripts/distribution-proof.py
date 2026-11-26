#!/usr/bin/env python3
"""distribution-proof.py — daemonless proof harness for the v0.1.0 distribution artifacts.

What it proves
--------------
Run against a real GitHub pre-release tag, this harness turns the release's own
payloads into assertions instead of prose:

  * the release run concluded `success`, its job set is exactly the five-job
    split (`version`, two `build-linux`, two `build-macos`, `publish`,
    `container`), the container job ran, the guard step approved a prerelease
    name list, and no `:latest` was created or moved;
  * both macOS legs finished inside the 240-minute budget and their
    `Ad-hoc sign the macOS binary` / `Verify the packaged macOS artifact` steps
    succeeded;
  * the release is a pre-release with exactly the four target tarballs plus
    `SHA256SUMS` and `install.sh`, every downloaded byte agrees with both the
    release's `SHA256SUMS` and GitHub's own asset `digest`, each tarball holds
    exactly the five top-level entries and no debug artifact, and the extracted
    linux-x86_64 binary reports the tag's version;
  * a `v0.1.0` release does not exist and `releases/latest` does not resolve to
    the proof tag.

No Docker daemon is ever contacted. The registry is read over plain HTTPS with
an anonymous pull token (`urllib`), or — as a fallback — through
`docker manifest inspect`, which is a registry client and never opens a daemon
socket. `gh` is shelled out to only for GitHub release/run data; everything
else is Python's standard library.

Every assertion is a pure function over a payload dict, so `fixtures` can feed
canned payloads (valid and invalid) and prove the checks actually fail on the
bad shapes. That is the offline half of the harness.

Subcommands
-----------
  fixtures                          offline: canned payloads through every assertion
  probe   --tag TAG --out FILE      thin end-to-end slice (release + digests)
  baseline --out FILE               GHCR state (tags, :latest) before/after a push
  run     --sha SHA [--timeout S] --out FILE
                                    poll the push run, summarize jobs + container log
  release --tag TAG --dir DIR --out FILE
                                    download, checksum, unpack and assert the release
  registry --tag TAG --baseline FILE --expect-digest SHA256 --out FILE
                                    registry-side proof: index digest, platform
                                    manifests, tag set, :latest vs the baseline,
                                    and the image config blob when readable

Transport summary:
  gh           — release view/download/list, run view/log, `api -i` status probes
  urllib       — ghcr.io token + manifests + tags/list (anonymous pull scope)
  docker       — `docker manifest inspect` only (no daemon)
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

REPO = "coruairc/deepseek-build"
OWNER = "coruairc"
IMAGE = f"ghcr.io/{OWNER}/deepseek-build"
REGISTRY_HOST = "ghcr.io"

TARGETS = (
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
)
LINUX_X86_64 = "x86_64-unknown-linux-gnu"

TOP_LEVEL_ENTRIES = frozenset(
    {"deepseek-build", "LICENSE", "NOTICE", "THIRD-PARTY-NOTICES", "README.txt"}
)
DEBUG_SUFFIXES = (".dSYM", ".debug")


def is_debug_member(name):
    """True when a tarball member is a .dSYM bundle anywhere or a .debug file."""
    parts = [part for part in name.split("/") if part]
    if not parts:
        return False
    if any(part.endswith(".dSYM") for part in parts):
        return True
    return parts[-1].endswith(".debug")

MACOS_BUDGET_MINUTES = 240
REQUIRED_MACOS_STEPS = ("Ad-hoc sign the macOS binary", "Verify the packaged macOS artifact")

EXPECTED_JOB_NAMES = frozenset(
    {
        "Derive release version",
        "build-linux (x86_64-unknown-linux-gnu)",
        "build-linux (aarch64-unknown-linux-gnu)",
        "build-macos (aarch64-apple-darwin)",
        "build-macos (x86_64-apple-darwin)",
        "Build and push container image",
        "Publish GitHub release",
    }
)
CONTAINER_JOB = "Build and push container image"

GUARD_LINE_RE = re.compile(r"guard-image-tags: OK (?P<count>\d+) names \((?P<kind>prerelease|stable)\)")
PUSHED_NAME_RE = re.compile(
    rf"pushing manifest for {re.escape(IMAGE)}:(?P<name>[^@\s]+)@sha256:[0-9a-f]{{64}}"
)
LATEST_LINE_RE = re.compile(r"Pre-release .*:latest unchanged")
DIGEST_RE = re.compile(r"^sha256:[0-9a-f]{64}$")
VERSION_OUTPUT_RE = re.compile(r"^deepseek-build [^ ]+ \([0-9a-f]{7,40}\)$")
PLATFORM_RE = re.compile(r"^(linux/amd64|linux/arm64)$")

# An OCI index / Docker manifest list is the only shape a multi-arch tag may
# resolve to. buildx also stores one attestation manifest per platform
# (platform `unknown/unknown`, annotated
# `vnd.docker.reference.type=attestation-manifest`); those descriptors are
# recorded separately and are never counted as platforms.
INDEX_MEDIA_TYPES = (
    "application/vnd.oci.image.index.v1+json",
    "application/vnd.docker.distribution.manifest.list.v2+json",
)
REQUIRED_PLATFORMS = ("linux/amd64", "linux/arm64")
ATTESTATION_TYPE = "attestation-manifest"


class ProofError(Exception):
    """One assertion failed; the message names the reason."""


def require(condition, reason):
    if not condition:
        raise ProofError(reason)


def now_utc():
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def version_of(tag):
    return tag[1:] if tag.startswith("v") else tag


def expected_asset_names(version):
    names = {f"deepseek-build-{version}-{target}.tar.gz" for target in TARGETS}
    names.update({"SHA256SUMS", "install.sh"})
    return names


# ---------------------------------------------------------------------------
# Pure assertions over payload dicts
# ---------------------------------------------------------------------------


def assert_release_classification(release, tag):
    require(
        release.get("tag") == tag,
        f"release tagName {release.get('tag')!r} is not {tag!r}",
    )
    require(
        release.get("prerelease") is True,
        f"release {tag} is not classified prerelease (prerelease={release.get('prerelease')!r})",
    )
    require(
        release.get("draft") is False,
        f"release {tag} is a draft (draft={release.get('draft')!r})",
    )


def assert_release_asset_set(release, version):
    expected = expected_asset_names(version)
    actual = [asset["name"] for asset in release.get("assets", [])]
    require(
        len(actual) == len(set(actual)),
        f"release asset names contain duplicates: {sorted(actual)}",
    )
    extras = sorted(set(actual) - expected)
    require(not extras, f"release carries extra asset(s): {', '.join(extras)}")
    missing = sorted(expected - set(actual))
    require(not missing, f"release is missing asset(s): {', '.join(missing)}")
    require(
        len(actual) == len(expected),
        f"release asset count is {len(actual)}, expected {len(expected)}",
    )


def parse_sums(text):
    """Parse a SHA256SUMS body into {name: sha256hex}, rejecting malformed lines."""
    sums = {}
    for line in text.splitlines():
        line = line.strip()
        if not line:
            continue
        parts = line.split()
        require(
            len(parts) == 2,
            f"SHA256SUMS line is not '<sha256>  <name>': {line!r}",
        )
        digest, name = parts
        require(
            re.fullmatch(r"[0-9a-f]{64}", digest) is not None,
            f"SHA256SUMS carries no sha256 digest for {name}: {digest!r}",
        )
        sums[name.lstrip("*")] = digest
    require(bool(sums), "SHA256SUMS is empty")
    return sums


def assert_sums_digests(release, sums, version):
    """Every SHA256SUMS line must name a real asset whose GitHub digest matches."""
    tarballs = {f"deepseek-build-{version}-{target}.tar.gz" for target in TARGETS}
    assets = {asset["name"]: asset for asset in release.get("assets", [])}
    missing = sorted(tarballs - set(sums))
    require(not missing, f"SHA256SUMS is missing tarball line(s): {', '.join(missing)}")
    unknown = sorted(set(sums) - tarballs)
    require(not unknown, f"SHA256SUMS lists unknown asset(s): {', '.join(unknown)}")
    mismatches = []
    for name, digest in sorted(sums.items()):
        asset = assets.get(name)
        require(asset is not None, f"SHA256SUMS names {name}, which is not a release asset")
        reported = asset.get("digest") or ""
        if reported != f"sha256:{digest}":
            mismatches.append(f"{name}: SHA256SUMS {digest} != GitHub digest {reported}")
    require(
        not mismatches,
        "asset digest disagrees with SHA256SUMS: " + "; ".join(mismatches),
    )


def assert_downloaded_files(release, sums, files):
    """Downloaded bytes must match both SHA256SUMS (tarballs) and GitHub's digest."""
    assets = {asset["name"]: asset for asset in release.get("assets", [])}
    mismatches = []
    for name, asset in sorted(assets.items()):
        require(name in files, f"release asset {name} was not downloaded")
        digest = files[name]
        reported = asset.get("digest") or ""
        if reported != f"sha256:{digest}":
            mismatches.append(f"{name}: downloaded {digest} != GitHub digest {reported}")
        if name in sums and sums[name] != digest:
            mismatches.append(f"{name}: downloaded {digest} != SHA256SUMS {sums[name]}")
    require(
        not mismatches,
        "downloaded file disagrees with its recorded digest: " + "; ".join(mismatches),
    )
    return mismatches


def assert_tarball_members(tarball, members):
    names = [name for name in members]
    require(
        not any(name.startswith("/") for name in names),
        f"tarball {tarball} carries an absolute member path",
    )
    require(
        not any(".." in name.split("/") for name in names),
        f"tarball {tarball} carries a parent-relative member path",
    )
    # The debug-artifact rule is checked before the entry-set rule: a stray
    # .dSYM is reported as what it is, not as an anonymous sixth entry.
    debug = sorted(name for name in names if is_debug_member(name))
    require(
        not debug,
        f"tarball {tarball} carries debug artifact(s): {', '.join(debug)}",
    )
    top_level = {name.rstrip("/") for name in names if name.rstrip("/")}
    require(
        top_level == set(TOP_LEVEL_ENTRIES),
        f"tarball {tarball} top-level entries are {sorted(top_level)}, "
        f"expected exactly the five entries {sorted(TOP_LEVEL_ENTRIES)}",
    )


def assert_version_output(output, version):
    text = (output or "").strip()
    require(
        text.startswith(f"deepseek-build {version} ("),
        f"extracted binary reports {text!r}, not the release version {version!r}",
    )
    require(
        VERSION_OUTPUT_RE.fullmatch(text) is not None,
        f"version output has an unexpected shape (name version (commit)): {text!r}",
    )


def assert_macho(tarball, output, arch):
    text = (output or "").strip()
    require("Mach-O" in text, f"{tarball} binary is not Mach-O: {text!r}")
    require(
        arch in text,
        f"{tarball} binary is not a Mach-O {arch} executable: {text!r}",
    )


def _iso_minutes(started, completed):
    if not started or not completed:
        return None
    fmt = "%Y-%m-%dT%H:%M:%SZ"
    delta = datetime.strptime(completed, fmt) - datetime.strptime(started, fmt)
    return round(delta.total_seconds() / 60.0, 1)


def assert_run(run):
    require(
        run.get("conclusion") == "success",
        f"run conclusion is {run.get('conclusion')!r}, not 'success'",
    )
    require(run.get("event") == "push", f"run event is {run.get('event')!r}, not 'push'")

    jobs = run.get("jobs", [])
    names = {job["name"] for job in jobs}
    require(
        names == set(EXPECTED_JOB_NAMES),
        "job set is "
        f"{sorted(names)}, expected exactly {sorted(EXPECTED_JOB_NAMES)}",
    )
    failed = [f"{job['name']}={job.get('conclusion')!r}" for job in jobs if job.get("conclusion") != "success"]
    require(not failed, "job(s) did not succeed: " + ", ".join(failed))

    container = run.get("container") or {}
    require(
        container.get("conclusion") == "success",
        f"container job concluded {container.get('conclusion')!r}; it must succeed "
        "(the Phase-3 split must not hide the image behind a macOS failure)",
    )
    require(not container.get("skipped"), "container job was skipped")
    require(
        container.get("guard_step_conclusion") == "success",
        f"guard step concluded {container.get('guard_step_conclusion')!r}",
    )

    macos_minutes = run.get("macos_minutes") or {}
    require(
        set(macos_minutes) == {"aarch64-apple-darwin", "x86_64-apple-darwin"},
        f"macOS legs recorded are {sorted(macos_minutes)}; expected both darwin targets",
    )
    for target, minutes in sorted(macos_minutes.items()):
        require(minutes is not None, f"macOS leg {target} has no recorded duration")
        require(
            minutes < MACOS_BUDGET_MINUTES,
            f"macOS leg {target} took {minutes} min, over the {MACOS_BUDGET_MINUTES} minute budget",
        )

    macos_steps = run.get("macos_steps_ok") or {}
    for target in ("aarch64-apple-darwin", "x86_64-apple-darwin"):
        steps = macos_steps.get(target) or {}
        for step in REQUIRED_MACOS_STEPS:
            require(
                step in steps,
                f"macOS leg {target} has no step named {step!r}",
            )
            require(
                steps[step] == "success",
                f"macOS leg {target} verify/sign step {step!r} concluded {steps[step]!r}",
            )

    guard_line = run.get("guard_line") or ""
    match = GUARD_LINE_RE.search(guard_line)
    require(
        match is not None,
        f"guard line {guard_line!r} does not match "
        "'guard-image-tags: OK <n> names (prerelease)'",
    )
    require(match.group("kind") == "prerelease", f"guard approved a {match.group('kind')} tag list")

    guard_names = run.get("guard_names") or []
    require(bool(guard_names), "no image tag name parsed from the container log")
    require(
        "latest" not in guard_names,
        "pushed tag name 'latest' is present; a prerelease must never move :latest",
    )

    latest_line = run.get("latest_line") or ""
    require(
        LATEST_LINE_RE.search(latest_line) is not None,
        "container log has no prerelease ':latest unchanged' line "
        f"(recorded {latest_line!r})",
    )

    require(
        DIGEST_RE.fullmatch(run.get("image_digest") or "") is not None,
        f"no multi-arch image digest recorded (got {run.get('image_digest')!r})",
    )


def assert_index_digest(registry, expected_digest):
    digest = registry.get("index_digest") or ""
    require(
        DIGEST_RE.fullmatch(digest) is not None,
        f"registry reports no index digest (got {digest!r})",
    )
    require(
        digest == expected_digest,
        f"registry index digest {digest} disagrees with the expected CI digest {expected_digest}",
    )


def assert_index_media_type(registry):
    media_type = registry.get("content_type") or ""
    require(
        media_type in INDEX_MEDIA_TYPES,
        f"registry index content type is {media_type!r}; expected one of {list(INDEX_MEDIA_TYPES)}",
    )


def assert_platform_set(registry):
    manifests = registry.get("manifests") or []
    platforms = [manifest.get("platform") for manifest in manifests]
    require(
        len(platforms) == len(set(platforms)),
        f"registry platform manifests carry duplicate platforms: {platforms}",
    )
    require(
        set(platforms) == set(REQUIRED_PLATFORMS),
        f"registry index platforms are {sorted(p for p in platforms if p)}; "
        f"expected exactly {' + '.join(REQUIRED_PLATFORMS)}",
    )
    require(
        len(manifests) == len(REQUIRED_PLATFORMS),
        f"registry index has {len(manifests)} platform manifests; "
        f"expected exactly {len(REQUIRED_PLATFORMS)}",
    )


def assert_platform_descriptors(registry):
    problems = []
    for manifest in registry.get("manifests") or []:
        digest = manifest.get("digest") or ""
        if DIGEST_RE.fullmatch(digest) is None:
            problems.append(f"{manifest.get('platform')!r} has no sha256 digest ({digest!r})")
        size = manifest.get("size")
        if not isinstance(size, int) or size <= 0:
            problems.append(f"{manifest.get('platform')!r} has no positive size ({size!r})")
    require(not problems, "; ".join(problems))


def assert_tag_set(registry):
    tags = registry.get("tags") or []
    require(bool(tags), "registry tag list is empty")
    require("latest" not in tags, "registry tags contain 'latest'")
    tag = registry.get("tag")
    require(
        tag in tags,
        f"registry tag list {tags} does not contain {tag!r}",
    )
    version = registry.get("version")
    if version:
        require(
            version in tags,
            f"registry tag list {tags} does not contain the version tag {version!r}",
        )


def assert_registry(registry, expected_digest):
    """The composite index assertion: digest, media type, platforms, tags."""
    assert_index_digest(registry, expected_digest)
    assert_index_media_type(registry)
    assert_platform_set(registry)
    assert_platform_descriptors(registry)
    assert_tag_set(registry)


def assert_latest_not_index(registry):
    if registry.get("latest_status") != "present":
        return
    require(
        registry.get("latest_digest") != registry.get("index_digest"),
        "latest resolves to this build's index digest; a pre-release must not move :latest",
    )


def assert_image_config(registry):
    """Assert the config blob when it was read; a recorded absence is not a pass
    for the values themselves, but it is the honest state this host can reach."""
    config_blob = registry.get("config_blob")
    require(
        config_blob in ("read", "unavailable"),
        f"config_blob is {config_blob!r}; expected 'read' or 'unavailable'",
    )
    if config_blob == "unavailable":
        return
    config = registry.get("config") or {}
    require(config.get("WorkingDir") == "/work", f"image WorkingDir is {config.get('WorkingDir')!r}, not '/work'")
    require(config.get("User") == "1000:1000", f"image User is {config.get('User')!r}, not '1000:1000'")
    require(
        config.get("Entrypoint") == ["deepseek-build"],
        f"image Entrypoint is {config.get('Entrypoint')!r}, not ['deepseek-build']",
    )
    env_names = config.get("env_names") or []
    require(
        not any("API_KEY" in name for name in env_names),
        f"image config carries an API key variable name: {sorted(n for n in env_names if 'API_KEY' in n)}",
    )


def assert_no_latest_movement(before, after):
    require(
        after.get("tags_status") == 200,
        f"tag list is unreadable after the push (status {after.get('tags_status')!r})",
    )
    before_status = before.get("latest_status")
    after_status = after.get("latest_status")
    require(
        after_status == before_status,
        "latest appeared after the tag push" if after_status == "present" else
        "latest disappeared after the tag push",
    )
    if before_status == "present":
        require(
            after.get("latest_digest") == before.get("latest_digest"),
            f"latest moved from {before.get('latest_digest')} to {after.get('latest_digest')}",
        )


def assert_no_v010(release):
    require(
        release.get("v0_1_0_status") == 404,
        f"releases/tags/v0.1.0 answered {release.get('v0_1_0_status')!r}, expected 404",
    )
    require(
        release.get("latest_tag") != "v0.1.0",
        "releases/latest resolves to v0.1.0",
    )
    require(
        "v0.1.0" not in (release.get("release_list_tags") or []),
        "release list contains a v0.1.0 release",
    )


# ---------------------------------------------------------------------------
# Offline fixtures
# ---------------------------------------------------------------------------

VALID_VERSION = "0.1.0-proof1"
VALID_TAG = f"v{VALID_VERSION}"


def _fixture_release():
    assets = []
    for target in TARGETS:
        assets.append(
            {
                "name": f"deepseek-build-{VALID_VERSION}-{target}.tar.gz",
                "size": 50_000_000,
                "digest": "sha256:" + hashlib.sha256(target.encode()).hexdigest(),
            }
        )
    assets.append({"name": "SHA256SUMS", "size": 496, "digest": "sha256:" + "1" * 64})
    assets.append({"name": "install.sh", "size": 14552, "digest": "sha256:" + "2" * 64})
    return {
        "tag": VALID_TAG,
        "prerelease": True,
        "draft": False,
        "url": f"https://github.com/{REPO}/releases/tag/{VALID_TAG}",
        "published_at": "2026-11-26T00:56:19Z",
        "asset_count": 6,
        "assets": assets,
        "v0_1_0_status": 404,
        "latest_tag": None,
        "release_list_tags": [VALID_TAG],
    }


def _fixture_sums(release):
    return {
        asset["name"]: asset["digest"].split(":", 1)[1]
        for asset in release["assets"]
        if asset["name"].endswith(".tar.gz")
    }


def _fixture_files(release):
    return {asset["name"]: asset["digest"].split(":", 1)[1] for asset in release["assets"]}


def _fixture_run():
    jobs = []
    for name in sorted(EXPECTED_JOB_NAMES):
        jobs.append({"name": name, "conclusion": "success", "status": "completed", "minutes": 10.0, "steps": []})
    return {
        "run_id": 45724842211,
        "url": f"https://github.com/{REPO}/actions/runs/45724842211",
        "event": "push",
        "head_sha": "b14d28eb2bc71a046b789e0a705848d651685d35",
        "status": "completed",
        "conclusion": "success",
        "jobs": jobs,
        "macos_minutes": {"aarch64-apple-darwin": 47.8, "x86_64-apple-darwin": 61.5},
        "macos_steps_ok": {
            "aarch64-apple-darwin": {step: "success" for step in REQUIRED_MACOS_STEPS},
            "x86_64-apple-darwin": {step: "success" for step in REQUIRED_MACOS_STEPS},
        },
        "container": {
            "name": CONTAINER_JOB,
            "conclusion": "success",
            "skipped": False,
            "guard_step_conclusion": "success",
        },
        "guard_line": "guard-image-tags: OK 2 names (prerelease)",
        "guard_names": [VALID_VERSION, VALID_TAG],
        "latest_line": "Pre-release \u2014 :latest unchanged",
        "image_digest": "sha256:" + "3" * 64,
    }


def _fixture_registry():
    return {
        "tag": VALID_TAG,
        "version": VALID_VERSION,
        "index_digest": "sha256:" + "3" * 64,
        "content_type": "application/vnd.oci.image.index.v1+json",
        "manifests": [
            {
                "platform": "linux/amd64",
                "digest": "sha256:" + "4" * 64,
                "size": 1810,
                "media_type": "application/vnd.oci.image.manifest.v1+json",
            },
            {
                "platform": "linux/arm64",
                "digest": "sha256:" + "5" * 64,
                "size": 1810,
                "media_type": "application/vnd.oci.image.manifest.v1+json",
            },
        ],
        "attestation_manifests": [
            {
                "platform": "unknown/unknown",
                "digest": "sha256:" + "a" * 64,
                "size": 838,
                "media_type": "application/vnd.oci.image.manifest.v1+json",
                "reference_digest": "sha256:" + "4" * 64,
                "reference_type": ATTESTATION_TYPE,
            }
        ],
        "tags": [VALID_VERSION, VALID_TAG],
        "tags_status": 200,
        "latest_status": "absent",
        "latest_digest": None,
        "config_blob": "read",
        "config": {
            "User": "1000:1000",
            "WorkingDir": "/work",
            "Entrypoint": ["deepseek-build"],
            "env_names": ["PATH", "DEBIAN_FRONTEND"],
        },
    }


def _fixture_baseline(latest_status="absent", latest_digest=None):
    return {
        "checked_at": "2026-11-26T00:10:00Z",
        "transport": "anonymous-http",
        "tags_status": 200,
        "tags": [VALID_VERSION, VALID_TAG],
        "latest_status": latest_status,
        "latest_digest": latest_digest,
    }


def _mutate(payload, path, value):
    """Deep-ish copy then set a dotted path; lists addressed by index."""
    clone = json.loads(json.dumps(payload))
    cursor = clone
    keys = path.split(".")
    for key in keys[:-1]:
        cursor = cursor[int(key)] if isinstance(cursor, list) else cursor[key]
    last = keys[-1]
    if isinstance(cursor, list):
        cursor[int(last)] = value
    else:
        cursor[last] = value
    return clone


def _fixture_cases():
    release = _fixture_release()
    sums = _fixture_sums(release)
    files = _fixture_files(release)
    run = _fixture_run()
    registry = _fixture_registry()
    before_absent = _fixture_baseline()
    after_absent = _fixture_baseline()
    before_present = _fixture_baseline("present", "sha256:" + "6" * 64)
    after_present = _fixture_baseline("present", "sha256:" + "6" * 64)

    cases = [
        ("release: valid asset set is accepted",
         lambda: assert_release_asset_set(release, VALID_VERSION), "pass", None),
        ("release: prerelease classification is accepted",
         lambda: assert_release_classification(release, VALID_TAG), "pass", None),
        ("release: SHA256SUMS agrees with the asset digests",
         lambda: assert_sums_digests(release, sums, VALID_VERSION), "pass", None),
        ("release: downloaded bytes agree with both digests",
         lambda: assert_downloaded_files(release, sums, files), "pass", None),
        ("release: no v0.1.0 release exists",
         lambda: assert_no_v010(release), "pass", None),
        ("tarball: exactly five top-level entries are accepted",
         lambda: assert_tarball_members("ok.tar.gz", sorted(TOP_LEVEL_ENTRIES)), "pass", None),
        ("version: the stamped version is accepted",
         lambda: assert_version_output("deepseek-build 0.1.0-proof1 (b14d28eb2bc7)", VALID_VERSION),
         "pass", None),
        ("macho: a Mach-O arm64 binary is accepted",
         lambda: assert_macho("ok.tar.gz", "Mach-O 64-bit executable arm64", "arm64"), "pass", None),
        ("run: a green run with a successful container job is accepted",
         lambda: assert_run(run), "pass", None),
        ("registry: a two-platform index is accepted",
         lambda: assert_registry(registry, "sha256:" + "3" * 64), "pass", None),
        ("registry: attestation manifests are not counted as platforms",
         lambda: assert_platform_set(registry), "pass", None),
        ("config: the image contract (uid/workdir/entrypoint) is accepted",
         lambda: assert_image_config(registry), "pass", None),
        ("config: an unavailable blob is accepted as recorded-unavailable",
         lambda: assert_image_config(_mutate(registry, "config_blob", "unavailable")), "pass", None),
        ("baseline: an absent latest that stays absent is accepted",
         lambda: assert_no_latest_movement(before_absent, after_absent), "pass", None),
        ("baseline: a present latest that does not move is accepted",
         lambda: assert_no_latest_movement(before_present, after_present), "pass", None),

        # Negatives — each must fail with the named reason.
        ("release: an extra asset is rejected",
         lambda: assert_release_asset_set(
             _mutate(release, "assets", release["assets"] + [{"name": "EXTRA", "size": 1, "digest": "sha256:" + "7" * 64}]),
             VALID_VERSION), "fail", "extra asset"),
        ("release: a missing tarball is rejected",
         lambda: assert_release_asset_set(_mutate(release, "assets", release["assets"][:-2]), VALID_VERSION),
         "fail", "missing asset"),
        ("release: prerelease false is rejected",
         lambda: assert_release_classification(_mutate(release, "prerelease", False), VALID_TAG),
         "fail", "prerelease"),
        ("release: draft true is rejected",
         lambda: assert_release_classification(_mutate(release, "draft", True), VALID_TAG),
         "fail", "draft"),
        ("release: an asset digest disagreeing with SHA256SUMS is rejected",
         lambda: assert_sums_digests(_mutate(release, "assets.0.digest", "sha256:" + "8" * 64), sums, VALID_VERSION),
         "fail", "digest disagrees"),
        ("release: a downloaded file disagreeing with SHA256SUMS is rejected",
         lambda: assert_downloaded_files(
             release, sums, {**files,
                             f"deepseek-build-{VALID_VERSION}-{TARGETS[0]}.tar.gz": "0" * 64}),
         "fail", "SHA256SUMS"),
        ("release: a v0.1.0 release is rejected",
         lambda: assert_no_v010(_mutate(release, "v0_1_0_status", 200)), "fail", "v0.1.0"),
        ("tarball: a debug member is rejected",
         lambda: assert_tarball_members("bad.tar.gz", sorted(TOP_LEVEL_ENTRIES) + ["deepseek-build.dSYM/Contents"]),
         "fail", "debug"),
        ("tarball: a sixth entry is rejected",
         lambda: assert_tarball_members("bad.tar.gz", sorted(TOP_LEVEL_ENTRIES) + ["extra.txt"]),
         "fail", "top-level entries"),
        ("version: a wrong version is rejected",
         lambda: assert_version_output("deepseek-build 0.0.1-rc5 (4b305bd0c1ab)", VALID_VERSION),
         "fail", "release version"),
        ("run: a skipped container job is rejected",
         lambda: assert_run(_mutate(_mutate(run, "container.conclusion", "skipped"), "container.skipped", True)),
         "fail", "container"),
        ("run: a macOS leg over 240 minutes is rejected",
         lambda: assert_run(_mutate(run, "macos_minutes.aarch64-apple-darwin", 241.0)),
         "fail", "240"),
        ("run: a failing macOS verify step is rejected",
         lambda: assert_run(_mutate(run, "macos_steps_ok.aarch64-apple-darwin.Verify the packaged macOS artifact", "failure")),
         "fail", "verify"),
        ("run: a missing guard step is rejected",
         lambda: assert_run(_mutate(run, "container.guard_step_conclusion", None)),
         "fail", "guard step"),
        ("run: a guard line that does not match is rejected",
         lambda: assert_run(_mutate(run, "guard_line", "guard-image-tags: nothing to say")),
         "fail", "guard line"),
        ("run: 'latest' among the pushed names is rejected",
         lambda: assert_run(_mutate(run, "guard_names", [VALID_VERSION, "latest"])),
         "fail", "latest"),
        ("run: a missing ':latest unchanged' line is rejected",
         lambda: assert_run(_mutate(run, "latest_line", "")),
         "fail", "unchanged"),
        ("run: an empty guard name list is rejected",
         lambda: assert_run(_mutate(run, "guard_names", [])),
         "fail", "no image tag name"),
        ("registry: a single-platform index is rejected",
         lambda: assert_registry(_mutate(registry, "manifests", registry["manifests"][:1]),
                                 "sha256:" + "3" * 64),
         "fail", "platform"),
        ("registry: a third real platform is rejected",
         lambda: assert_registry(
             _mutate(registry, "manifests",
                     registry["manifests"] + [{"platform": "linux/arm/v7",
                                               "digest": "sha256:" + "b" * 64,
                                               "size": 1810}]),
             "sha256:" + "3" * 64),
         "fail", "platform"),
        ("registry: a zero-size platform manifest is rejected",
         lambda: assert_platform_descriptors(_mutate(registry, "manifests.0.size", 0)),
         "fail", "size"),
        ("registry: a non-index content type is rejected",
         lambda: assert_index_media_type(
             _mutate(registry, "content_type", "application/vnd.oci.image.manifest.v1+json")),
         "fail", "content type"),
        ("registry: a 'latest' tag is rejected",
         lambda: assert_registry(_mutate(registry, "tags", registry["tags"] + ["latest"]),
                                 "sha256:" + "3" * 64),
         "fail", "latest"),
        ("registry: a digest disagreeing with the CI digest is rejected",
         lambda: assert_registry(_mutate(registry, "index_digest", "sha256:" + "9" * 64),
                                 "sha256:" + "3" * 64),
         "fail", "expected CI digest"),
        ("registry: latest equal to this build's index is rejected",
         lambda: assert_latest_not_index(
             _mutate(_mutate(registry, "latest_status", "present"),
                     "latest_digest", registry["index_digest"])),
         "fail", "latest resolves"),
        ("config: a root user is rejected",
         lambda: assert_image_config(_mutate(registry, "config.User", "0:0")),
         "fail", "User"),
        ("config: a wrong working directory is rejected",
         lambda: assert_image_config(_mutate(registry, "config.WorkingDir", "/")),
         "fail", "WorkingDir"),
        ("config: an API_KEY env name is rejected",
         lambda: assert_image_config(
             _mutate(registry, "config.env_names", ["PATH", "DEEPSEEK_API_KEY"])),
         "fail", "API key"),
        ("baseline: a moved latest is rejected",
         lambda: assert_no_latest_movement(
             before_present, _mutate(after_present, "latest_digest", "sha256:" + "a" * 64)),
         "fail", "moved"),
        ("baseline: an absent latest that appears is rejected",
         lambda: assert_no_latest_movement(before_absent, _mutate(after_absent, "latest_status", "present")),
         "fail", "appeared"),
    ]
    return cases


def cmd_fixtures(_args):
    cases = _fixture_cases()
    failures = 0
    for name, fn, want, reason in cases:
        try:
            fn()
            outcome = "pass"
            detail = ""
        except ProofError as exc:
            outcome = "fail"
            detail = str(exc)
        except Exception as exc:  # noqa: BLE001 - a crash is a failed case, never a pass
            outcome = "error"
            detail = f"{type(exc).__name__}: {exc}"
        if outcome != want:
            failures += 1
            print(f"FAIL  {name}: expected {want}, got {outcome}: {detail}")
            continue
        if want == "fail" and reason and reason.lower() not in detail.lower():
            failures += 1
            print(f"FAIL  {name}: failed for the wrong reason (wanted {reason!r}): {detail}")
            continue
        print(f"PASS  {name}" + (f"  [{detail}]" if want == "fail" else ""))
    total = len(cases)
    print(f"fixtures: {total - failures}/{total} PASS")
    return 0 if failures == 0 else 1


# ---------------------------------------------------------------------------
# Transport wrappers
# ---------------------------------------------------------------------------


def run_cmd(args, check=True, timeout=600):
    proc = subprocess.run(args, capture_output=True, text=True, timeout=timeout)
    if check and proc.returncode != 0:
        raise ProofError(f"{' '.join(args[:3])} failed (exit {proc.returncode}): {proc.stderr.strip() or proc.stdout.strip()}")
    return proc


def gh_json(args):
    proc = run_cmd(["gh"] + args + ["--repo", REPO] if "--repo" not in args else ["gh"] + args)
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        raise ProofError(f"gh {' '.join(args)} returned non-JSON: {exc}") from exc


def gh_api_status(path):
    """HTTP status of one API path, without treating a 404 as an error."""
    proc = run_cmd(["gh", "api", "-i", path], check=False)
    match = re.match(r"HTTP/\S+\s+(\d{3})", proc.stdout)
    if match is None:
        raise ProofError(f"gh api -i {path} printed no status line: {proc.stdout[:200]!r}")
    return int(match.group(1))


def gh_api_json(path):
    proc = run_cmd(["gh", "api", path])
    return json.loads(proc.stdout)


def sha256_file(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def download_release(tag, dest):
    Path(dest).mkdir(parents=True, exist_ok=True)
    run_cmd(["gh", "release", "download", tag, "--repo", REPO, "--dir", dest, "--clobber"], timeout=1800)
    return sorted(p.name for p in Path(dest).iterdir())


def release_meta(tag):
    raw = gh_json(
        ["release", "view", tag, "--json", "tagName,isPrerelease,isDraft,publishedAt,url,assets"]
    )
    return {
        "tag": raw.get("tagName") or tag,
        "prerelease": raw.get("isPrerelease"),
        "draft": raw.get("isDraft"),
        "published_at": raw.get("publishedAt"),
        "url": raw.get("url"),
        "asset_count": len(raw.get("assets") or []),
        "assets": [
            {
                "name": asset.get("name"),
                "size": asset.get("size"),
                "digest": asset.get("digest"),
            }
            for asset in (raw.get("assets") or [])
        ],
    }


# --- registry (no daemon) -------------------------------------------------


def registry_token(scope=f"repository:{OWNER}/deepseek-build:pull"):
    url = f"https://{REGISTRY_HOST}/token?service={REGISTRY_HOST}&scope={urllib.parse.quote(scope)}"
    with urllib.request.urlopen(url, timeout=30) as response:
        return json.loads(response.read())["token"]


MANIFEST_ACCEPT = ", ".join(
    (
        "application/vnd.oci.image.index.v1+json",
        "application/vnd.docker.distribution.manifest.list.v2+json",
        "application/vnd.oci.image.manifest.v1+json",
        "application/vnd.docker.distribution.manifest.v2+json",
    )
)


def registry_manifest(ref, token=None):
    """GET one manifest; returns {status, digest, body, json}. 404 is a result."""
    token = token or registry_token()
    request = urllib.request.Request(f"https://{REGISTRY_HOST}/v2/{OWNER}/deepseek-build/manifests/{ref}")
    request.add_header("Authorization", f"Bearer {token}")
    request.add_header("Accept", MANIFEST_ACCEPT)
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            body = response.read()
            digest = response.headers.get("Docker-Content-Digest") or (
                "sha256:" + hashlib.sha256(body).hexdigest()
            )
            status = response.status
    except urllib.error.HTTPError as exc:
        body = exc.read()
        return {"status": exc.code, "digest": None, "body": body, "json": None}
    except urllib.error.URLError as exc:
        return {"status": None, "digest": None, "body": b"", "json": None, "error": str(exc)}
    try:
        parsed = json.loads(body)
    except json.JSONDecodeError:
        parsed = None
    return {"status": status, "digest": digest, "body": body, "json": parsed}


def registry_tags(token=None):
    token = token or registry_token()
    request = urllib.request.Request(
        f"https://{REGISTRY_HOST}/v2/{OWNER}/deepseek-build/tags/list?n=100"
    )
    request.add_header("Authorization", f"Bearer {token}")
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return response.status, json.loads(response.read()).get("tags") or []
    except urllib.error.HTTPError as exc:
        return exc.code, []
    except urllib.error.URLError:
        return None, []


def registry_blob(blob_digest, token=None):
    """GET one blob body; None when it cannot be read."""
    if not blob_digest:
        return None
    token = token or registry_token()
    request = urllib.request.Request(f"https://{REGISTRY_HOST}/v2/{OWNER}/deepseek-build/blobs/{blob_digest}")
    request.add_header("Authorization", f"Bearer {token}")
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return response.read()
    except (urllib.error.HTTPError, urllib.error.URLError):
        return None


def split_index_descriptors(manifests):
    """Split an index's descriptors into platform manifests and attestations.

    buildx writes one `unknown/unknown` attestation manifest per platform,
    annotated `vnd.docker.reference.type=attestation-manifest`. Those are not
    runnable platforms and must never be counted as one.
    """
    platform_manifests = []
    attestations = []
    for descriptor in manifests:
        platform = descriptor.get("platform") or {}
        name = f"{platform.get('os', '')}/{platform.get('architecture', '')}".strip("/")
        annotations = descriptor.get("annotations") or {}
        entry = {
            "platform": name,
            "digest": descriptor.get("digest"),
            "size": descriptor.get("size"),
            "media_type": descriptor.get("mediaType"),
        }
        if annotations.get("vnd.docker.reference.type") == ATTESTATION_TYPE:
            entry["reference_digest"] = annotations.get("vnd.docker.reference.digest")
            entry["reference_type"] = annotations.get("vnd.docker.reference.type")
            attestations.append(entry)
        else:
            platform_manifests.append(entry)
    return platform_manifests, attestations


def read_config_blob(platform_manifests, token):
    """(config-or-None, 'read'|'unavailable') from the amd64 image manifest."""
    amd64 = next((m for m in platform_manifests if m.get("platform") == "linux/amd64"), None)
    if amd64 is None:
        return None, "unavailable"
    manifest = registry_manifest(amd64["digest"], token)
    if manifest["status"] != 200 or not manifest["json"]:
        return None, "unavailable"
    descriptor = manifest["json"].get("config") or {}
    body = registry_blob(descriptor.get("digest"), token)
    if body is None:
        return None, "unavailable"
    try:
        parsed = json.loads(body).get("config") or {}
    except json.JSONDecodeError:
        return None, "unavailable"
    return (
        {
            "digest": descriptor.get("digest"),
            "size": descriptor.get("size"),
            "User": parsed.get("User"),
            "WorkingDir": parsed.get("WorkingDir"),
            "Entrypoint": parsed.get("Entrypoint"),
            "env_names": [item.split("=", 1)[0] for item in (parsed.get("Env") or [])],
        },
        "read",
    )


def registry_snapshot():
    """(transport, tags_status, tags, latest_status, latest_digest, tried)."""
    tried = []
    try:
        token = registry_token()
        tags_status, tags = registry_tags(token)
        tried.append({"transport": "anonymous-http", "tags_status": tags_status})
        if tags_status == 200:
            latest = registry_manifest("latest", token)
            tried.append({"transport": "anonymous-http", "latest_status": latest["status"]})
            return (
                "anonymous-http",
                tags_status,
                tags,
                "present" if latest["status"] == 200 else "absent",
                latest["digest"],
                tried,
            )
    except Exception as exc:  # noqa: BLE001 - transport fallback is the point
        tried.append({"transport": "anonymous-http", "error": f"{type(exc).__name__}: {exc}"})

    docker = shutil.which("docker")
    if docker:
        proc = run_cmd([docker, "manifest", "inspect", f"{IMAGE}:latest"], check=False, timeout=120)
        status = "present" if proc.returncode == 0 else "absent"
        tried.append({"transport": "docker-manifest-inspect", "latest_status": status, "exit": proc.returncode})
        return "docker-manifest-inspect", None, [], status, None, tried

    packages = run_cmd(
        ["gh", "api", f"/users/{OWNER}/packages/container/deepseek-build/versions"], check=False
    )
    tried.append({"transport": "gh-api-packages", "exit": packages.returncode})
    return "gh-api-packages", packages.returncode, [], "unknown", None, tried


# ---------------------------------------------------------------------------
# Subcommands
# ---------------------------------------------------------------------------


def write_json(path, payload):
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    Path(path).write_text(json.dumps(payload, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def cmd_probe(args):
    tmpdir = tempfile.mkdtemp(prefix="dist-proof-probe-")
    try:
        meta = release_meta(args.tag)
        downloaded = download_release(args.tag, tmpdir)
        sums_path = Path(tmpdir) / "SHA256SUMS"
        sums = parse_sums(sums_path.read_text(encoding="utf-8")) if sums_path.exists() else {}
        files = {name: sha256_file(Path(tmpdir) / name) for name in downloaded}
        mismatches = []
        assets = {asset["name"]: asset for asset in meta["assets"]}
        for name, digest in sorted(files.items()):
            require(name in assets, f"downloaded {name}, which is not a release asset")
            reported = assets[name].get("digest") or ""
            if reported != f"sha256:{digest}":
                mismatches.append(f"{name}: downloaded {digest} != GitHub digest {reported}")
            if name in sums and sums[name] != digest:
                mismatches.append(f"{name}: downloaded {digest} != SHA256SUMS {sums[name]}")
        payload = {
            "tag": meta["tag"],
            "prerelease": meta["prerelease"],
            "draft": meta["draft"],
            "asset_count": meta["asset_count"],
            "assets": meta["assets"],
            "sums_verified": sum(1 for name in files if name in sums),
            "sums_mismatches": mismatches,
            "checked_at": now_utc(),
        }
        write_json(args.out, payload)
        require(meta["asset_count"] > 0, "release carries no assets")
        require(not mismatches, "digest mismatch: " + "; ".join(mismatches))
        require(
            all(name in files for name in assets),
            "not every listed asset was downloadable",
        )
        print(f"probe: OK {args.tag} — {payload['asset_count']} assets, "
              f"{payload['sums_verified']} verified against SHA256SUMS, 0 mismatches")
        return 0
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


def cmd_baseline(args):
    transport, tags_status, tags, latest_status, latest_digest, tried = registry_snapshot()
    payload = {
        "checked_at": now_utc(),
        "transport": transport,
        "tags_status": tags_status,
        "tags": tags,
        "latest_status": latest_status,
        "latest_digest": latest_digest,
        "transports_tried": tried,
        "image": IMAGE,
        "note": "no Docker daemon is contacted; see notes.md for what this snapshot can and cannot prove",
    }
    write_json(args.out, payload)
    print(f"baseline: transport={transport} tags_status={tags_status} "
          f"tags={tags} latest={latest_status} digest={latest_digest}")
    return 0


def wait_for_run(sha, timeout, interval):
    deadline = time.time() + timeout
    while True:
        runs = gh_json(
            [
                "run",
                "list",
                "--workflow=release.yml",
                "--event=push",
                "--limit",
                "20",
                "--json",
                "databaseId,headSha,headBranch,event,status,conclusion,createdAt,url",
            ]
        )
        matches = [run for run in runs if run.get("headSha") == sha and run.get("event") == "push"]
        require(
            len(matches) == 1,
            f"expected exactly one push run for {sha}, found {len(matches)}: "
            f"{[m.get('databaseId') for m in matches]}",
        )
        run_id = matches[0]["databaseId"]
        detail = gh_json(
            [
                "run",
                "view",
                str(run_id),
                "--json",
                "databaseId,status,conclusion,event,headSha,headBranch,url,createdAt,updatedAt,jobs",
            ]
        )
        if detail.get("status") == "completed":
            return detail
        if time.time() >= deadline:
            return detail
        time.sleep(interval)


def summarize_run(raw):
    jobs = []
    for job in raw.get("jobs") or []:
        jobs.append(
            {
                "name": job.get("name"),
                "status": job.get("status"),
                "conclusion": job.get("conclusion"),
                "started_at": job.get("startedAt"),
                "completed_at": job.get("completedAt"),
                "minutes": _iso_minutes(job.get("startedAt"), job.get("completedAt")),
                "steps": [
                    {
                        "name": step.get("name"),
                        "status": step.get("status"),
                        "conclusion": step.get("conclusion"),
                    }
                    for step in (job.get("steps") or [])
                ],
            }
        )

    macos_minutes = {}
    macos_steps_ok = {}
    for job in jobs:
        match = re.fullmatch(r"build-macos \((?P<target>[^)]+)\)", job["name"] or "")
        if match is None:
            continue
        target = match.group("target")
        macos_minutes[target] = job["minutes"]
        macos_steps_ok[target] = {
            step["name"]: step["conclusion"]
            for step in job["steps"]
            if step["name"] in REQUIRED_MACOS_STEPS
        }

    container_job = next((job for job in jobs if job["name"] == CONTAINER_JOB), None)
    container = {
        "name": CONTAINER_JOB,
        "present": container_job is not None,
        "conclusion": (container_job or {}).get("conclusion"),
        "skipped": (container_job or {}).get("conclusion") == "skipped",
        "guard_step_conclusion": None,
    }
    log = ""
    if container_job is not None:
        for step in container_job["steps"]:
            if step["name"] == "Guard the resolved image tag list":
                container["guard_step_conclusion"] = step["conclusion"]
        database_id = None
        for job in raw.get("jobs") or []:
            if job.get("name") == CONTAINER_JOB:
                database_id = job.get("databaseId")
        if database_id:
            proc = run_cmd(
                ["gh", "run", "view", str(raw["databaseId"]), "--log", "--job", str(database_id)],
                timeout=600,
            )
            log = proc.stdout

    stripped = [re.sub(r"^.*?\t\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z ", "", line) for line in log.splitlines()]
    guard_lines = [line for line in stripped if GUARD_LINE_RE.search(line)]
    latest_lines = [line for line in stripped if LATEST_LINE_RE.search(line)]
    digest_lines = [line for line in stripped if re.search(r"\bDigest:\s+sha256:[0-9a-f]{64}\s*$", line)]
    amd64_arm64 = [line for line in stripped if re.match(r"^amd64=sha256:[0-9a-f]{64} arm64=sha256:[0-9a-f]{64}$", line)]

    guard_names = []
    for match in PUSHED_NAME_RE.finditer(log):
        name = match.group("name")
        if name not in guard_names:
            guard_names.append(name)

    build_digests = {}
    if amd64_arm64:
        for part in amd64_arm64[-1].split():
            key, value = part.split("=", 1)
            build_digests[key] = value

    return {
        "run_id": raw.get("databaseId"),
        "url": raw.get("url"),
        "event": raw.get("event"),
        "ref": raw.get("headBranch"),
        "head_sha": raw.get("headSha"),
        "status": raw.get("status"),
        "conclusion": raw.get("conclusion"),
        "created_at": raw.get("createdAt"),
        "updated_at": raw.get("updatedAt"),
        "jobs": jobs,
        "macos_minutes": macos_minutes,
        "macos_steps_ok": macos_steps_ok,
        "container": container,
        "guard_line": guard_lines[-1] if guard_lines else None,
        "guard_names": guard_names,
        "latest_line": latest_lines[-1] if latest_lines else None,
        "image_digest": (re.search(r"sha256:[0-9a-f]{64}", digest_lines[-1]).group(0) if digest_lines else None),
        "build_digests": build_digests,
        "checked_at": now_utc(),
    }


def cmd_run(args):
    raw = wait_for_run(args.sha, args.timeout, args.interval)
    payload = summarize_run(raw)
    write_json(args.out, payload)
    if payload.get("status") != "completed":
        raise ProofError(
            f"run {payload.get('run_id')} did not finish before the {args.timeout}s timeout "
            f"(last status {payload.get('status')!r}) — a timeout is not a pass"
        )
    assert_run(payload)
    print(f"run: OK {payload['run_id']} conclusion={payload['conclusion']} "
          f"container={payload['container']['conclusion']} "
          f"macos={payload['macos_minutes']} digest={payload['image_digest']}")
    return 0


def tarball_members(path):
    with tarfile.open(path, "r:gz") as archive:
        return archive.getnames()


def extract_binary(tarball, dest):
    Path(dest).mkdir(parents=True, exist_ok=True)
    with tarfile.open(tarball, "r:gz") as archive:
        archive.extractall(dest)
    binary = Path(dest) / "deepseek-build"
    require(binary.exists(), f"{Path(tarball).name} has no top-level deepseek-build member")
    os.chmod(binary, 0o755)
    return binary


def cmd_release(args):
    dest = Path(args.dir)
    dest.mkdir(parents=True, exist_ok=True)
    meta = release_meta(args.tag)
    version = version_of(args.tag)
    download_release(args.tag, dest)

    sums_file = dest / "SHA256SUMS"
    require(sums_file.exists(), "the release has no SHA256SUMS asset on disk")
    sums = parse_sums(sums_file.read_text(encoding="utf-8"))
    files = {asset["name"]: sha256_file(dest / asset["name"]) for asset in meta["assets"]}

    mismatches = []
    assets = {asset["name"]: asset for asset in meta["assets"]}
    for name, digest in sorted(files.items()):
        reported = assets.get(name, {}).get("digest") or ""
        if reported != f"sha256:{digest}":
            mismatches.append(f"{name}: downloaded {digest} != GitHub digest {reported}")
        if name in sums and sums[name] != digest:
            mismatches.append(f"{name}: downloaded {digest} != SHA256SUMS {sums[name]}")

    tarballs = {}
    for name in sorted(files):
        if not name.endswith(".tar.gz"):
            continue
        members = tarball_members(dest / name)
        tarballs[name] = {
            "members": members,
            "top_level": sorted({member.rstrip("/") for member in members}),
            "debug_members": sorted(m for m in members if is_debug_member(m)),
        }

    linux_binary = extract_binary(dest / f"deepseek-build-{version}-{LINUX_X86_64}.tar.gz", dest / "extract-linux-x86_64")
    proc = run_cmd([str(linux_binary), "--version"], timeout=120)
    linux_version_output = proc.stdout.strip()

    darwin_file_output = {}
    for target in ("aarch64-apple-darwin", "x86_64-apple-darwin"):
        binary = extract_binary(dest / f"deepseek-build-{version}-{target}.tar.gz", dest / f"extract-{target}")
        file_proc = run_cmd(["file", str(binary)], timeout=60)
        darwin_file_output[target] = file_proc.stdout.strip()

    v010_status = gh_api_status(f"repos/{REPO}/releases/tags/v0.1.0")
    try:
        latest = gh_api_json(f"repos/{REPO}/releases/latest")
        latest_tag = latest.get("tag_name")
        latest_status = 200
    except ProofError:
        latest_tag = None
        latest_status = 404
    release_list = gh_json(["release", "list", "--limit", "100", "--json", "tagName"])

    payload = dict(meta)
    payload.update(
        {
            "version": version,
            "sums": [{"name": name, "sha256": digest} for name, digest in sorted(sums.items())],
            "files": [
                {
                    "name": name,
                    "sha256": digest,
                    "matches_sums": sums.get(name) == digest if name in sums else None,
                    "matches_asset_digest": (assets.get(name, {}).get("digest") or "") == f"sha256:{digest}",
                }
                for name, digest in sorted(files.items())
            ],
            "sums_verified": sum(1 for name in files if name in sums),
            "sums_mismatches": mismatches,
            "tarballs": tarballs,
            "linux_version_output": linux_version_output,
            "darwin_file_output": darwin_file_output,
            "v0_1_0_status": v010_status,
            "latest_status": latest_status,
            "latest_tag": latest_tag,
            "release_list_tags": [entry.get("tagName") for entry in release_list],
            "checked_at": now_utc(),
        }
    )
    write_json(args.out, payload)

    assert_release_classification(payload, args.tag)
    assert_release_asset_set(payload, version)
    assert_sums_digests(payload, sums, version)
    assert_downloaded_files(payload, sums, files)
    for name, info in sorted(tarballs.items()):
        assert_tarball_members(name, info["members"])
    assert_version_output(linux_version_output, version)
    assert_macho("aarch64-apple-darwin", darwin_file_output["aarch64-apple-darwin"], "arm64")
    assert_macho("x86_64-apple-darwin", darwin_file_output["x86_64-apple-darwin"], "x86_64")
    assert_no_v010(payload)

    print(f"release: OK {args.tag} — {payload['asset_count']} assets, "
          f"{payload['sums_verified']} verified, linux reports {linux_version_output!r}")
    return 0


def _payload_skeleton(tag, baseline):
    return {
        "checked_at": now_utc(),
        "tag": tag,
        "version": version_of(tag),
        "image": IMAGE,
        "transport": None,
        "visibility": "unknown",
        "index_digest": None,
        "content_type": None,
        "manifests": [],
        "attestation_manifests": [],
        "tags_status": None,
        "tags": [],
        "latest_status": "unknown",
        "latest_digest": None,
        "config_blob": "unavailable",
        "config": None,
        "baseline": {
            "latest_status": baseline.get("latest_status"),
            "latest_digest": baseline.get("latest_digest"),
        },
        "transports_tried": [],
    }


def registry_via_anonymous(tag, baseline):
    """Anonymous ghcr.io token -> manifest index, tags, :latest, config blob.

    Pure HTTPS with a pull-scoped token; no Docker daemon and no Docker CLI.
    A 200 here is what makes the package `public`.
    """
    payload = _payload_skeleton(tag, baseline)
    payload["transport"] = "anonymous-http"
    try:
        token = registry_token()
    except (urllib.error.URLError, OSError) as exc:
        payload["transports_tried"].append(
            {"transport": "anonymous-http", "error": f"token endpoint: {type(exc).__name__}"}
        )
        return None
    manifest = registry_manifest(tag, token)
    payload["transports_tried"].append(
        {"transport": "anonymous-http", "manifest_status": manifest["status"]}
    )
    if manifest["status"] != 200 or not manifest["json"]:
        return None

    body = manifest["json"]
    payload["index_digest"] = manifest["digest"]
    payload["content_type"] = body.get("mediaType")
    payload["manifests"], payload["attestation_manifests"] = split_index_descriptors(
        body.get("manifests") or []
    )

    tags_status, tags = registry_tags(token)
    payload["tags_status"] = tags_status
    payload["tags"] = tags

    latest = registry_manifest("latest", token)
    payload["latest_status"] = "present" if latest["status"] == 200 else "absent"
    payload["latest_digest"] = latest["digest"] if latest["status"] == 200 else None
    payload["transports_tried"].append(
        {"transport": "anonymous-http", "latest_status": latest["status"]}
    )

    config, config_blob = read_config_blob(payload["manifests"], token)
    payload["config"] = config
    payload["config_blob"] = config_blob
    payload["visibility"] = "public"
    return payload


def registry_via_docker_manifest(tag, baseline):
    """`docker manifest inspect` — a registry client, never a daemon socket.

    The CLI can read this only when the tag is anonymously pullable or the
    host already holds a credential, so a success here cannot by itself
    distinguish public from private: `visibility` stays `unknown` unless the
    anonymous transport also answered.
    """
    docker = shutil.which("docker")
    payload = _payload_skeleton(tag, baseline)
    payload["transport"] = "docker-manifest-inspect"
    if not docker:
        payload["transports_tried"].append(
            {"transport": "docker-manifest-inspect", "error": "docker not on PATH"}
        )
        return None
    proc = run_cmd([docker, "manifest", "inspect", f"{IMAGE}:{tag}"], check=False, timeout=120)
    payload["transports_tried"].append(
        {"transport": "docker-manifest-inspect", "exit": proc.returncode}
    )
    if proc.returncode != 0:
        return None
    try:
        body = json.loads(proc.stdout)
    except json.JSONDecodeError:
        return None
    payload["content_type"] = body.get("mediaType")
    payload["manifests"], payload["attestation_manifests"] = split_index_descriptors(
        body.get("manifests") or []
    )
    # This subcommand does not print the index digest or the tag list; the
    # digest is deliberately left None (an assertion will name it) rather than
    # guessed from a platform digest.
    latest = run_cmd([docker, "manifest", "inspect", f"{IMAGE}:latest"], check=False, timeout=120)
    payload["latest_status"] = "present" if latest.returncode == 0 else "absent"
    payload["visibility"] = "unknown"
    return payload


def registry_via_gh_packages(tag, baseline):
    """`gh api /users/<owner>/packages/container/deepseek-build/versions`.

    A credentialed read: when it answers, the package is readable with the
    caller's token. A 403 is an authorization gap, not a registry result.
    """
    payload = _payload_skeleton(tag, baseline)
    payload["transport"] = "gh-api-packages"
    proc = run_cmd(
        ["gh", "api", f"/users/{OWNER}/packages/container/deepseek-build/versions"], check=False
    )
    payload["transports_tried"].append({"transport": "gh-api-packages", "exit": proc.returncode})
    if proc.returncode != 0:
        return None
    try:
        versions = json.loads(proc.stdout)
    except json.JSONDecodeError:
        return None
    tags = []
    for version in versions:
        tags.extend(((version.get("metadata") or {}).get("container") or {}).get("tags") or [])
    payload["tags"] = sorted(set(tags))
    payload["visibility"] = "private"
    return payload


REGISTRY_ASSERTIONS = (
    ("index digest is sha256 and equals the CI digest", lambda p, d: assert_index_digest(p, d)),
    ("index content type is an OCI index or Docker manifest list", lambda p, d: assert_index_media_type(p)),
    ("index carries exactly linux/amd64 + linux/arm64", lambda p, d: assert_platform_set(p)),
    ("each platform manifest carries a digest and a positive size", lambda p, d: assert_platform_descriptors(p)),
    ("tag set names the version tag and never 'latest'", lambda p, d: assert_tag_set(p)),
    ("latest matches the pre-push baseline", lambda p, d: assert_no_latest_movement(p["baseline"], p)),
    ("latest never resolves to this build's index", lambda p, d: assert_latest_not_index(p)),
    ("image config matches the Dockerfile contract", lambda p, d: assert_image_config(p)),
)


def cmd_registry(args):
    baseline = json.loads(Path(args.baseline).read_text(encoding="utf-8"))
    payload = None
    tried = []
    for builder in (registry_via_anonymous, registry_via_docker_manifest, registry_via_gh_packages):
        attempt = builder(args.tag, baseline)
        if attempt is not None:
            payload = attempt
            break
        tried.append(builder.__name__)
    if payload is None:
        payload = _payload_skeleton(args.tag, baseline)
        payload["blocked"] = "package visibility"
        payload["transports_tried"].append({"order_exhausted": tried})
        payload["note"] = (
            "no transport could read the package; this is an authorization or visibility "
            "gap, not evidence that the image is missing"
        )
        write_json(args.out, payload)
        print(
            "registry: BLOCKED: package visibility — no transport could read "
            f"{IMAGE}:{args.tag}; see {args.out} and notes.md for the unblock",
            file=sys.stderr,
        )
        return 1

    results = []
    for name, check in REGISTRY_ASSERTIONS:
        try:
            check(payload, args.expect_digest)
        except ProofError as exc:
            results.append({"assertion": name, "ok": False, "detail": str(exc)})
        else:
            results.append({"assertion": name, "ok": True, "detail": ""})
    payload["assertions"] = results
    write_json(args.out, payload)

    failures = 0
    for result in results:
        if result["ok"]:
            print(f"PASS  {result['assertion']}")
        else:
            failures += 1
            print(f"FAIL  {result['assertion']}: {result['detail']}")
    if failures:
        print(f"registry: FAIL: {failures}/{len(results)} assertion(s) failed", file=sys.stderr)
        return 1
    print(
        f"registry: OK transport={payload['transport']} visibility={payload['visibility']} "
        f"digest={payload['index_digest']} platforms="
        f"{[m['platform'] for m in payload['manifests']]} tags={payload['tags']} "
        f"latest={payload['latest_status']} config_blob={payload['config_blob']}"
    )
    return 0


def build_parser():
    parser = argparse.ArgumentParser(
        description="Daemonless proof harness for the v0.1.0 distribution artifacts."
    )
    sub = parser.add_subparsers(dest="command", required=True)

    fixtures = sub.add_parser("fixtures", help="offline: run every assertion against canned payloads")
    fixtures.set_defaults(func=cmd_fixtures)

    probe = sub.add_parser("probe", help="thin end-to-end slice: release + digest verification")
    probe.add_argument("--tag", required=True)
    probe.add_argument("--out", required=True)
    probe.set_defaults(func=cmd_probe)

    baseline = sub.add_parser("baseline", help="record the GHCR tag/:latest state")
    baseline.add_argument("--out", required=True)
    baseline.set_defaults(func=cmd_baseline)

    run = sub.add_parser("run", help="poll the release run and summarize it")
    run.add_argument("--sha", required=True)
    run.add_argument("--timeout", type=int, default=15300)
    run.add_argument("--interval", type=int, default=60)
    run.add_argument("--out", required=True)
    run.set_defaults(func=cmd_run)

    release = sub.add_parser("release", help="download and assert the release assets")
    release.add_argument("--tag", required=True)
    release.add_argument("--dir", required=True)
    release.add_argument("--out", required=True)
    release.set_defaults(func=cmd_release)

    registry = sub.add_parser("registry", help="registry-side proof: digest, platforms, tags, :latest")
    registry.add_argument("--tag", required=True)
    registry.add_argument("--baseline", required=True, help="the pre-push snapshot JSON")
    registry.add_argument("--expect-digest", required=True, help="the CI-recorded index digest")
    registry.add_argument("--out", required=True)
    registry.set_defaults(func=cmd_registry)
    return parser


def main(argv=None):
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        return args.func(args)
    except ProofError as exc:
        print(f"{args.command}: FAIL: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
