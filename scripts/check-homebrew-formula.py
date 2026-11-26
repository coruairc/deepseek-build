#!/usr/bin/env python3
"""Text-level assertions over the in-tree Homebrew formula.

This checker reads ``packaging/homebrew/deepseek-build.rb`` as **text** and
asserts the facts a reader can verify from the repository alone: the pinned
release URLs, the sha256 values, the platform blocks that pair them, the
install/test stanzas, and the absence of a source build or a placeholder hash.

It does **not** evaluate Ruby semantics. ``ruby`` is not installed on this
host, so the formula's behaviour under Homebrew (style, audit, per-arch
resolution, fetch/checksum) is asserted by the ``brew`` sessions recorded in
``docs/evidence/04-distribution-proof/homebrew-formula`` -- never by this
script. Everything here is a string assertion plus (with ``--online``) a
``HEAD`` request that the two URLs are live at the size the release reports.

Usage::

    python3 scripts/check-homebrew-formula.py \\
        --formula packaging/homebrew/deepseek-build.rb \\
        --sums docs/evidence/04-distribution-proof/tag-run/SHA256SUMS \\
        --release-json docs/evidence/04-distribution-proof/tag-run/release.json \\
        [--online] [--self-test]

``--self-test`` copies the formula three times into a temp directory -- a
mutated sha256, a deleted ``on_intel`` block and a URL on a non-github.com host
-- and requires each mutation to be rejected by a named assertion.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path
from urllib.parse import urlsplit

REPO = "coruairc/deepseek-build"
HOST = "github.com"
DOWNLOAD_PATH = f"/{REPO}/releases/download/"
TARGETS = ("aarch64-apple-darwin", "x86_64-apple-darwin")

NAME_CLASS = "class: exactly one Formula subclass named DeepseekBuild"
NAME_VERSION = "version: one literal matching the tag in the URLs"
NAME_URL_COUNT = "url: exactly two url lines"
NAME_URL_TARGETS = "url: exactly one aarch64 and one x86_64 darwin asset"
NAME_URL_SHAPE = "url: both are https on github.com under the release download path"
NAME_URL_BLOCKS = "url: on_macos wraps on_arm and on_intel, each before its asset"
NAME_SHA_COUNT = "sha256: exactly two 64-hex literals, distinct"
NAME_SHA_SUMS = "sha256: each matches the release SHA256SUMS line for its url"
NAME_SHA_RELEASE = "sha256: release.json asset digests agree with SHA256SUMS"
NAME_INSTALL = "install: bin.install, dsb symlink, licence docs and a --version test"
NAME_METADATA = "metadata: desc, homepage and Apache-2.0 licence"
NAME_SOURCE = "source build: no head, branch, patch or Xcode dependency"
NAME_HOSTS = "hosts: no URL host other than github.com"
NAME_PLACEHOLDER = "placeholder: no zeroed or TODO hash"
NAME_LIVECHECK = "livecheck: skipped or absent, so no other host is queried"


def load_sums(path: Path) -> dict[str, str]:
    """``<sha256>  <name>`` per line -> {name: sha256}."""
    sums: dict[str, str] = {}
    for line in path.read_text().splitlines():
        line = line.strip()
        if not line:
            continue
        fields = line.split()
        if len(fields) < 2:
            raise SystemExit(f"{path}: cannot parse line: {line!r}")
        digest, name = fields[0], fields[-1].lstrip("*")
        sums[name] = digest
    return sums


def load_release(path: Path) -> tuple[str, dict[str, int], dict[str, str]]:
    """-> (tag, {asset name: size}, {asset name: sha256})."""
    data = json.loads(path.read_text())
    sizes = {asset["name"]: asset["size"] for asset in data.get("assets", [])}
    digests = {
        item["name"]: item["sha256"]
        for item in data.get("sums", [])
        if isinstance(item, dict) and "name" in item and "sha256" in item
    }
    return data.get("tag", ""), sizes, digests


def parse_asset(url: str) -> tuple[str, str]:
    """-> (tag, basename) for a release download URL, or raise ValueError."""
    parts = urlsplit(url)
    if parts.scheme != "https":
        raise ValueError(f"scheme is {parts.scheme!r}, not https")
    if parts.netloc != HOST:
        raise ValueError(f"host is {parts.netloc!r}, not {HOST}")
    if not parts.path.startswith(DOWNLOAD_PATH):
        raise ValueError(f"path does not start with {DOWNLOAD_PATH}")
    rest = parts.path[len(DOWNLOAD_PATH) :].strip("/")
    if rest.count("/") != 1:
        raise ValueError(f"expected exactly <tag>/<asset>, got {rest!r}")
    tag, _, basename = rest.partition("/")
    if not tag.startswith("v") or not basename:
        raise ValueError(f"cannot split tag/asset from {parts.path!r}")
    return tag, basename


def non_comment_lines(text: str) -> list[str]:
    return [line for line in text.splitlines() if not line.lstrip().startswith("#")]


def evaluate(text: str, sums: dict[str, str], release: dict) -> list[tuple[str, bool, str]]:
    """Run every offline assertion; returns [(name, ok, detail)]."""
    release_tag, release_sizes, release_digests = release
    results: list[tuple[str, bool, str]] = []

    def record(name: str, ok: bool, detail: str = "") -> None:
        results.append((name, ok, detail))

    # 1. class
    classes = re.findall(r"^class\s+(\w+)\s*<\s*Formula\s*$", text, re.MULTILINE)
    record(
        NAME_CLASS,
        classes == ["DeepseekBuild"],
        f"found {classes!r}",
    )

    # 2. url lines (structural count first, so later checks can rely on it)
    url_matches = list(re.finditer(r'^\s*url\s+"([^"]+)"\s*$', text, re.MULTILINE))
    urls = [m.group(1) for m in url_matches]
    record(NAME_URL_COUNT, len(urls) == 2, f"found {len(urls)}: {urls!r}")

    sha_matches = list(re.finditer(r'^\s*sha256\s+"([^"]+)"\s*$', text, re.MULTILINE))
    shas = [m.group(1) for m in sha_matches]

    # 3. parse each URL, derive version, confirm the two expected assets
    parsed: list[tuple[str, str]] = []
    parse_errors: list[str] = []
    for url in urls:
        try:
            parsed.append(parse_asset(url))
        except ValueError as exc:
            parse_errors.append(str(exc))
    ok_shape = len(urls) == 2 and not parse_errors and len(parsed) == 2
    record(NAME_URL_SHAPE, ok_shape, "; ".join(parse_errors) or "both URLs are https://github.com release downloads")

    version_match = re.search(r'^\s*version\s+"([^"]+)"\s*$', text, re.MULTILINE)
    version = version_match.group(1) if version_match else None
    tags = {tag for tag, _ in parsed}
    bases = {base for _, base in parsed}
    derived_version = tags.pop()[1:] if len(tags) == 1 else None
    expected_bases = {
        f"deepseek-build-{derived_version}-{target}.tar.gz" for target in TARGETS
    }
    record(
        NAME_VERSION,
        bool(version) and version == derived_version,
        f"version={version!r} derived={derived_version!r} tags={sorted(tags)!r}",
    )
    record(
        NAME_URL_TARGETS,
        len(parsed) == 2 and bases == expected_bases and len(bases) == 2,
        f"assets={sorted(bases)!r} expected={sorted(expected_bases)!r}",
    )

    # 4. platform-block pairing: on_macos > (on_arm, on_intel), each before its asset
    def pos(pattern: str) -> int:
        found = re.search(pattern, text, re.MULTILINE)
        return found.start() if found else -1

    on_macos = pos(r"^\s*on_macos do\s*$")
    on_arm = pos(r"^\s*on_arm do\s*$")
    on_intel = pos(r"^\s*on_intel do\s*$")
    arm_url = next(
        (m.start() for m in url_matches if "aarch64-apple-darwin" in m.group(1)), -1
    )
    intel_url = next(
        (m.start() for m in url_matches if "x86_64-apple-darwin" in m.group(1)), -1
    )
    # Each URL must sit after its own arch block opens and before the other one does.
    arm_end = on_intel if on_intel > on_arm else len(text)
    intel_end = on_arm if on_arm > on_intel else len(text)
    blocks_ok = (
        -1 < on_macos < min(on_arm, on_intel)
        and on_arm < arm_url < arm_end
        and on_intel < intel_url < intel_end
    )
    record(
        NAME_URL_BLOCKS,
        blocks_ok,
        f"on_macos={on_macos} on_arm={on_arm} arm_url={arm_url} "
        f"on_intel={on_intel} intel_url={intel_url}",
    )

    # 5. sha256 count and distinctness
    hex_shas = [s for s in shas if re.fullmatch(r"[0-9a-f]{64}", s)]
    record(
        NAME_SHA_COUNT,
        len(shas) == 2 and len(hex_shas) == 2 and len(set(hex_shas)) == 2,
        f"found {len(shas)} sha256 line(s), {len(hex_shas)} valid, distinct={len(set(hex_shas)) == len(hex_shas)}",
    )

    # 6. positional pairing url -> next sha256, checked against the release sums
    pairs: list[tuple[str, str]] = []
    for url_match in url_matches:
        nxt = next((m for m in sha_matches if m.start() > url_match.start()), None)
        if nxt is not None:
            pairs.append((url_match.group(1), nxt.group(1)))
    sums_errors: list[str] = []
    for url, digest in pairs:
        try:
            _, basename = parse_asset(url)
        except ValueError as exc:
            sums_errors.append(f"{url}: {exc}")
            continue
        expected = sums.get(basename)
        if expected is None:
            sums_errors.append(f"{basename}: absent from SHA256SUMS")
        elif expected != digest:
            sums_errors.append(f"{basename}: formula {digest} != SHA256SUMS {expected}")
    record(
        NAME_SHA_SUMS,
        len(pairs) == 2 and not sums_errors,
        "; ".join(sums_errors) or "both pairs match the release SHA256SUMS",
    )

    # 7. the SHA256SUMS digest and the release asset digest are independent witnesses
    witness_errors: list[str] = []
    for basename, digest in sums.items():
        if basename not in release_digests:
            witness_errors.append(f"{basename}: absent from release.json sums")
        elif release_digests[basename] != digest:
            witness_errors.append(f"{basename}: release.json {release_digests[basename]} != SHA256SUMS {digest}")
    if release_tag and derived_version:
        expected_tag = f"v{derived_version}"
        if release_tag != expected_tag:
            witness_errors.append(f"release.json tag {release_tag!r} != {expected_tag!r}")
    record(
        NAME_SHA_RELEASE,
        not witness_errors,
        "; ".join(witness_errors) or f"release.json ({release_tag}) agrees with SHA256SUMS for all {len(sums)} assets",
    )

    # 8. install stanzas
    install_errors: list[str] = []
    if not re.search(r'^\s*bin\.install\s+"deepseek-build"\s*$', text, re.MULTILINE):
        install_errors.append('no `bin.install "deepseek-build"`')
    if not re.search(r'^\s*bin\.install_symlink\s+"deepseek-build"\s*=>\s*"dsb"\s*$', text, re.MULTILINE):
        install_errors.append('no `bin.install_symlink "deepseek-build" => "dsb"`')
    doc_install = re.search(r"^\s*doc\.install(.+)$", text, re.MULTILINE)
    doc_line = doc_install.group(1) if doc_install else ""
    for needed in ("LICENSE", "NOTICE", "THIRD-PARTY-NOTICES"):
        if f'"{needed}"' not in doc_line:
            install_errors.append(f"doc.install lacks {needed}")
    test_block = re.search(r"^\s*test do$(.*?)^\s*end$", text, re.MULTILINE | re.DOTALL)
    if not test_block:
        install_errors.append("no `test do` block")
    elif "--version" not in test_block.group(1):
        install_errors.append("test block does not run --version")
    record(NAME_INSTALL, not install_errors, "; ".join(install_errors) or "binary, dsb symlink, licence docs, --version test")

    # 9. metadata
    meta_errors: list[str] = []
    desc = re.search(r'^\s*desc\s+"([^"]*)"\s*$', text, re.MULTILINE)
    if not desc or not desc.group(1).strip():
        meta_errors.append("desc is absent or empty")
    homepage = re.search(r'^\s*homepage\s+"([^"]*)"\s*$', text, re.MULTILINE)
    if not homepage or homepage.group(1) != f"https://github.com/{REPO}":
        meta_errors.append(f'homepage is {homepage.group(1)!r}' if homepage else "homepage is absent")
    if not re.search(r'^\s*license\s+"Apache-2\.0"\s*$', text, re.MULTILINE):
        meta_errors.append('license is not "Apache-2.0"')
    record(NAME_METADATA, not meta_errors, "; ".join(meta_errors) or "desc, homepage, Apache-2.0")

    # 10. no source build
    body = "\n".join(non_comment_lines(text))
    source_hits = re.findall(
        r"^\s*(?:head\s+\"|branch\s+\"|patch\s+(?:do\b|\"|'|%w)|depends_on\s+:xcode\b|depends_on\s+\"xcode\")",
        body,
        re.MULTILINE,
    )
    record(
        NAME_SOURCE,
        not source_hits,
        f"found {source_hits!r}" if source_hits else "prebuilt tarball only",
    )

    # 11. every URL host is github.com
    all_urls = re.findall(r"https?://[^\s\"')]+", text)
    other_hosts = sorted(
        {
            urlsplit(url).netloc
            for url in all_urls
            if urlsplit(url).netloc != HOST
        }
    )
    record(
        NAME_HOSTS,
        bool(all_urls) and not other_hosts,
        f"other hosts={other_hosts!r}" if other_hosts else f"{len(all_urls)} URL(s), all on {HOST}",
    )

    # 12. no placeholder values
    placeholder_hits = [
        pattern
        for pattern in ("0" * 64, "REPLACE", "TODO", "FIXME", "placeholder", "example.com", "no_check")
        if pattern in text
    ]
    record(
        NAME_PLACEHOLDER,
        not placeholder_hits,
        f"found {placeholder_hits!r}" if placeholder_hits else "no placeholder hash or marker",
    )

    # 13. livecheck must not query another host (a `skip` block or no livecheck at all)
    livecheck = re.search(r"^\s*livecheck do$(.*?)^\s*end$", text, re.MULTILINE | re.DOTALL)
    livecheck_ok = livecheck is None or "skip" in livecheck.group(1)
    record(
        NAME_LIVECHECK,
        livecheck_ok,
        "no livecheck block"
        if livecheck is None
        else ("livecheck has a skip" if livecheck_ok else "livecheck would query the default strategy"),
    )

    return results


def head_request(url: str, timeout: int = 120) -> tuple[int | None, int | None, str]:
    """Follow redirects with a HEAD request -> (final status, content-length, raw tail)."""
    try:
        proc = subprocess.run(
            ["curl", "-sSI", "-L", "--max-time", str(timeout), url],
            capture_output=True,
            text=True,
            check=False,
        )
    except FileNotFoundError as exc:  # pragma: no cover - curl is required
        return None, None, f"curl not available: {exc}"
    status: int | None = None
    length: int | None = None
    for line in proc.stdout.splitlines():
        if line.startswith("HTTP/"):
            fields = line.split()
            status = int(fields[1]) if len(fields) > 1 and fields[1].isdigit() else None
            length = None
        elif line.lower().startswith("content-length:"):
            value = line.split(":", 1)[1].strip()
            length = int(value) if value.isdigit() else None
    tail = " | ".join(line.strip() for line in proc.stdout.splitlines()[-3:])
    if proc.returncode != 0:
        return status, length, f"curl exit {proc.returncode}: {proc.stderr.strip() or tail}"
    return status, length, tail


def online_findings(text: str, release: dict) -> list[tuple[str, bool, str]]:
    _, release_sizes, _ = release
    results: list[tuple[str, bool, str]] = []
    urls = re.findall(r'^\s*url\s+"([^"]+)"\s*$', text, re.MULTILINE)
    for url in urls:
        try:
            _, basename = parse_asset(url)
        except ValueError as exc:
            results.append((f"online: {url} is a release download", False, str(exc)))
            continue
        status, length, tail = head_request(url)
        results.append(
            (
                f"online: {basename} answers 200",
                status == 200,
                f"status={status} ({tail})",
            )
        )
        expected_size = release_sizes.get(basename)
        results.append(
            (
                f"online: {basename} Content-Length is the release size",
                expected_size is not None and length == expected_size,
                f"Content-Length={length} release.json={expected_size}",
            )
        )
    return results


def run(text: str, sums: dict[str, str], release: dict, online: bool) -> list[tuple[str, bool, str]]:
    results = evaluate(text, sums, release)
    if online:
        results.extend(online_findings(text, release))
    return results


def report(results: list[tuple[str, bool, str]]) -> bool:
    for name, ok, detail in results:
        print(f"{'PASS' if ok else 'FAIL'}  {name}" + ("" if ok else f"  -- {detail}"))
    passed = sum(1 for _, ok, _ in results if ok)
    total = len(results)
    print(f"homebrew-formula: {passed}/{total} PASS")
    return passed == total


def self_test(text: str, sums: dict[str, str], release: dict) -> bool:
    """Three mutations, each rejected by a named assertion."""
    cases: list[tuple[str, str, str]] = []
    hex_shas = re.findall(r'^\s*sha256\s+"([0-9a-f]{64})"\s*$', text, re.MULTILINE)
    if len(hex_shas) != 2:
        print("FAIL  self-test: the formula does not carry two sha256 values to mutate")
        return False

    cases.append(
        (
            "wrong-sha256",
            text.replace(hex_shas[1], hex_shas[0]),
            NAME_SHA_SUMS,
        )
    )

    without_intel = re.sub(
        r"^    on_intel do\n(?:.*\n)*?^    end\n", "", text, flags=re.MULTILINE
    )
    cases.append(("missing-on-intel", without_intel, NAME_URL_BLOCKS))

    cases.append(
        (
            "non-github-url",
            text.replace(
                "https://github.com/coruairc/deepseek-build/releases/download/",
                "https://downloads.example.com/coruairc/deepseek-build/releases/download/",
                1,
            ),
            NAME_HOSTS,
        )
    )

    passed = 0
    with tempfile.TemporaryDirectory() as tmp:
        for slug, mutated, expected_failure in cases:
            path = Path(tmp) / f"{slug}.rb"
            path.write_text(mutated)
            results = evaluate(path.read_text(), sums, release)
            failed = {name for name, ok, _ in results if not ok}
            if expected_failure in failed:
                passed += 1
                print(f"PASS  self-test: {slug} rejected by {expected_failure!r}")
            else:
                print(
                    f"FAIL  self-test: {slug} was not rejected by {expected_failure!r} "
                    f"(failures: {sorted(failed)!r})"
                )
    print(f"self-test: {passed}/{len(cases)} PASS")
    return passed == len(cases)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--formula", type=Path, required=True)
    parser.add_argument("--sums", type=Path, required=True)
    parser.add_argument("--release-json", type=Path, required=True)
    parser.add_argument("--online", action="store_true", help="HEAD both asset URLs")
    parser.add_argument("--self-test", action="store_true", help="run the three mutation cases")
    args = parser.parse_args()

    text = args.formula.read_text()
    sums = load_sums(args.sums)
    release = load_release(args.release_json)

    ok = report(run(text, sums, release, args.online))
    if args.self_test:
        ok = self_test(text, sums, release) and ok

    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
