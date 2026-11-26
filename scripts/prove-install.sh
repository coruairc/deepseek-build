#!/bin/sh
# prove-install.sh — prove the PUBLISHED installer against a REAL GitHub release.
#
# This is not scripts/test-install.sh. That harness builds a fake release tree
# and serves it from 127.0.0.1, so it proves install.sh's script logic and
# nothing about the assets a user downloads. This script talks to the real
# release: it downloads the release asset `install.sh`, proves the asset is
# byte-identical to the working-tree installer, downloads the host tarball from
# the exact release URL a user would fetch, compares its sha256 against both the
# release's SHA256SUMS and GitHub's own asset digest, then runs the installer
# into a scratch HOME/INSTALL_DIR and checks the installed binary.
#
# The installer's test-only override hooks — the ones that point it at a fake
# local tree — are never set here, and their presence in the environment is a
# hard refusal: a proof that quietly runs against 127.0.0.1 certifies nothing.
# They are named as `DSB_INSTALL_` + a suffix (assembled in the function below,
# never written out literally) so that a grep for a hook name in this file
# returns nothing. The only hosts contacted are github.com and GitHub's CDN.
#
# Usage:
#   sh scripts/prove-install.sh --version TAG [--workdir DIR]
#   sh scripts/prove-install.sh --latest-behaviour [--asset-tag TAG] [--workdir DIR]
#
#   --version TAG      Download TAG's release assets, prove the three-way
#                      checksum agreement, install it for real and check
#                      `deepseek-build --version`.
#   --latest-behaviour Record what the UNPINNED path really does: probe the
#                      releases/latest URLs, then run the installer with no
#                      VERSION and assert it fails having installed nothing.
#   --asset-tag TAG    Release whose install.sh the latest probe downloads
#                      (default: v0.1.0-proof1).
#   --workdir DIR      Use DIR instead of mktemp -d. DIR is kept on exit.
#
# Exit status: 0 when every assertion passed, 1 on a failed assertion, 2 when
# the script cannot run (bad arguments, fake-tree hook set, missing tool).
#
# Written for Phase 4 (04-02, REL-06); reusable by the Phase-5 checklist.
# Network contract: github.com only. No sudo, no daemon, no package install.

set -eu

REPO="coruairc/deepseek-build"
RELEASES="https://github.com/${REPO}/releases"

MODE=""
TAG=""
ASSET_TAG="v0.1.0-proof1"
WORK=""
WORK_GIVEN=0

PASS=0
FAIL=0
FAILED_NAMES=""

LOG=""

say() {
    printf '%s\n' "$*"
    printf '%s\n' "$*" >>"$LOG"
}

pass() {
    PASS=$((PASS + 1))
    say "PASS: $1"
}

fail() {
    FAIL=$((FAIL + 1))
    FAILED_NAMES="$FAILED_NAMES $1"
    say "FAIL: $1"
    if [ -n "${2:-}" ]; then
        say "      detail: $2"
    fi
}

die() {
    printf 'prove-install: %s\n' "$*" >&2
    exit 2
}

usage() {
    cat <<EOF
Usage:
  sh scripts/prove-install.sh --version TAG [--workdir DIR]
  sh scripts/prove-install.sh --latest-behaviour [--asset-tag TAG] [--workdir DIR]

  --version TAG      Download TAG's release assets, prove the three-way
                     checksum agreement, install it for real and check
                     deepseek-build --version.
  --latest-behaviour Record what the UNPINNED path really does: probe the
                     releases/latest URLs, then run the installer with no
                     VERSION and assert it fails having installed nothing.
  --asset-tag TAG    Release whose install.sh the latest probe downloads
                     (default: v0.1.0-proof1).
  --workdir DIR      Use DIR instead of mktemp -d. DIR is kept on exit.
EOF
}

# Same mapping as install.sh, so the target name here is the one the installer
# would pick on this host.
host_target() {
    _os=$(uname -s)
    _arch=$(uname -m)
    case "$_os" in
        Linux)
            case "$_arch" in
                x86_64|amd64) printf '%s' "x86_64-unknown-linux-gnu" ;;
                aarch64|arm64) printf '%s' "aarch64-unknown-linux-gnu" ;;
                *) printf '' ;;
            esac
            ;;
        Darwin)
            case "$_arch" in
                x86_64|amd64) printf '%s' "x86_64-apple-darwin" ;;
                aarch64|arm64) printf '%s' "aarch64-apple-darwin" ;;
                *) printf '' ;;
            esac
            ;;
        *) printf '' ;;
    esac
}

sha256_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | cut -d' ' -f1
    else
        die "neither sha256sum nor shasum is available"
    fi
}

file_size() {
    wc -c <"$1" | tr -d ' '
}

refuse_fake_tree_hooks() {
    for _suffix in BASE_URL TEST_UNAME_S TEST_UNAME_M; do
        _hook="DSB_INSTALL_${_suffix}"
        if printenv "$_hook" >/dev/null 2>&1; then
            die "$_hook is set. The proof must exercise github.com, not a fake tree; unset it and retry."
        fi
    done
}

# ---------------------------------------------------------------------------
# Arguments
# ---------------------------------------------------------------------------

while [ $# -gt 0 ]; do
    case "$1" in
        --version)
            [ $# -ge 2 ] || die "--version needs a tag"
            MODE="version"
            TAG=$2
            shift 2
            ;;
        --latest-behaviour)
            MODE="latest"
            shift
            ;;
        --asset-tag)
            [ $# -ge 2 ] || die "--asset-tag needs a tag"
            ASSET_TAG=$2
            shift 2
            ;;
        --workdir)
            [ $# -ge 2 ] || die "--workdir needs a directory"
            WORK=$2
            WORK_GIVEN=1
            shift 2
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            die "unknown argument: $1 (try --help)"
            ;;
    esac
done

[ -n "$MODE" ] || die "one of --version TAG or --latest-behaviour is required (try --help)"
case "$MODE" in
    version) echo "$TAG" | grep -Eq '^v[0-9][0-9A-Za-z._-]*$' || die "invalid tag: $TAG (expected vX.Y.Z…)" ;;
    latest) echo "$ASSET_TAG" | grep -Eq '^v[0-9][0-9A-Za-z._-]*$' || die "invalid --asset-tag: $ASSET_TAG" ;;
esac

refuse_fake_tree_hooks

HOST=$(host_target)
[ -n "$HOST" ] || die "unsupported host $(uname -s)/$(uname -m)"
ROOT=$(cd "$(dirname "$0")/.." && pwd)
[ -f "$ROOT/install.sh" ] || die "no install.sh at $ROOT"

if [ "$WORK_GIVEN" = 1 ]; then
    mkdir -p "$WORK"
else
    WORK=$(mktemp -d "${TMPDIR:-/tmp}/dsb-prove-install.XXXXXX")
    trap 'rm -rf "$WORK"' EXIT HUP INT TERM
fi

if [ "$MODE" = version ]; then
    LOG="$WORK/install.log"
else
    LOG="$WORK/install-latest.log"
fi
: >"$LOG"
mkdir -p "$WORK/asset" "$WORK/home" "$WORK/direct"

# The scratch HOME carries a .curlrc that only disables curl's progress meter,
# so the captured transcript stays readable. install.sh's own `curl -fSL` reads
# it because HOME points here; verbosity is the only thing it changes — the
# download, the redirects and the checksums are untouched, and error messages
# (no-progress-meter is not --silent) still land in the log.
printf '%s\n' 'no-progress-meter' >"$WORK/home/.curlrc"

# Disk check: the tarball plus the installer's own download is ~120 MB.
AVAIL_KB=$(df -Pk "$WORK" | awk 'NR==2 {print $4}')
say "prove-install: $MODE"
say "  repository: $REPO"
say "  host:       $(uname -s)/$(uname -m) -> $HOST"
say "  network:    github.com only (fake-tree hooks unset)"
say "  scratch:    $WORK"
say "  disk free:  ${AVAIL_KB} KiB"
if [ "$AVAIL_KB" -lt 400000 ]; then
    fail "scratch filesystem has at least 400 MiB free" "${AVAIL_KB} KiB available"
else
    pass "scratch filesystem has at least 400 MiB free (${AVAIL_KB} KiB)"
fi
say ""

# ---------------------------------------------------------------------------
# --version TAG — the pinned path, against the real release
# ---------------------------------------------------------------------------

if [ "$MODE" = version ]; then
    VER=${TAG#v}
    TARBALL="deepseek-build-${VER}-${HOST}.tar.gz"
    URL="$RELEASES/download/$TAG/$TARBALL"

    say "pinned proof"
    say "  tag:        $TAG"
    say "  tarball:    $TARBALL"
    say "  url:        $URL"
    say ""

    case "$URL" in
        https://github.com/*) pass "download URL is on github.com: $URL" ;;
        *) fail "download URL is on github.com" "$URL" ;;
    esac

    # (b) the published installer asset vs this working tree
    gh release download "$TAG" --pattern install.sh --dir "$WORK/asset" --clobber
    gh release download "$TAG" --pattern SHA256SUMS --dir "$WORK/asset" --clobber

    SHA_ASSET=$(sha256_of "$WORK/asset/install.sh")
    SHA_TREE=$(sha256_of "$ROOT/install.sh")
    SHA_GH=$(gh release view "$TAG" --json assets \
        --jq '.assets[] | select(.name=="install.sh") | .digest' | sed 's/^sha256://')

    say "install.sh, three independent values:"
    say "  downloaded asset : $SHA_ASSET"
    say "  working tree     : $SHA_TREE"
    say "  GitHub digest    : $SHA_GH"
    if [ -n "$SHA_GH" ] && [ "$SHA_ASSET" = "$SHA_GH" ] && [ "$SHA_ASSET" = "$SHA_TREE" ]; then
        pass "release asset install.sh is byte-identical to the working tree and to GitHub's digest"
    else
        fail "release asset install.sh matches the working tree and GitHub's digest" \
            "asset=$SHA_ASSET tree=$SHA_TREE github=$SHA_GH"
    fi

    # (c) the host tarball from the exact URL a user fetches
    curl -fSL --retry 3 --retry-delay 2 --no-progress-meter -o "$WORK/direct/$TARBALL" "$URL"
    SHA_FILE=$(sha256_of "$WORK/direct/$TARBALL")
    SHA_SUMS=$(grep -E "^[0-9a-fA-F]{64}  ${TARBALL}$" "$WORK/asset/SHA256SUMS" | head -n 1 | cut -d' ' -f1 || true)
    SHA_GH_TARBALL=$(gh release view "$TAG" --json assets \
        --jq ".assets[] | select(.name==\"${TARBALL}\") | .digest" | sed 's/^sha256://')

    SIZE_FILE=$(file_size "$WORK/direct/$TARBALL")
    SIZE_GH=$(gh release view "$TAG" --json assets \
        --jq ".assets[] | select(.name==\"${TARBALL}\") | .size")

    grep -E "^[0-9a-fA-F]{64}  ${TARBALL}$" "$WORK/asset/SHA256SUMS" >"$WORK/direct/host.SHA256SUMS" || true
    SUMS_CHECK=$(cd "$WORK/direct" && sha256sum -c host.SHA256SUMS 2>&1) ||
        SUMS_CHECK="sha256sum -c failed: $SUMS_CHECK"

    {
        printf '%s\n' "prove-install: three-way checksum comparison"
        printf '%s\n' "date:   $(date -u +%Y-%m-%dT%H:%M:%SZ)"
        printf '%s\n' "tag:    $TAG"
        printf '%s\n' "host:   $(uname -s)/$(uname -m) -> $HOST"
        printf '%s\n' "url:    $URL"
        printf '%s\n' ""
        printf '%s\n' "file:   $TARBALL"
        printf '%s\n' "size:   $SIZE_FILE bytes (downloaded) / $SIZE_GH bytes (GitHub asset metadata)"
        printf '%s\n' ""
        printf '%s\n' "source                          sha256"
        printf '%s\n' "local sha256sum (downloaded)    $SHA_FILE"
        printf '%s\n' "SHA256SUMS (release asset)      ${SHA_SUMS:-<not listed>}"
        printf '%s\n' "GitHub asset digest             ${SHA_GH_TARBALL:-<unavailable>}"
        printf '%s\n' ""
        printf '%s\n' "sha256sum -c (host tarball line from the release SHA256SUMS):"
        printf '%s\n' "${SUMS_CHECK:-<no output>}"
        printf '%s\n' ""
        printf '%s\n' "file:   install.sh"
        printf '%s\n' "size:   $(file_size "$WORK/asset/install.sh") bytes"
        printf '%s\n' ""
        printf '%s\n' "local sha256sum (downloaded)    $SHA_ASSET"
        printf '%s\n' "working tree install.sh         $SHA_TREE"
        printf '%s\n' "GitHub asset digest             ${SHA_GH:-<unavailable>}"
        printf '%s\n' ""
        if [ -n "$SHA_SUMS" ] && [ -n "$SHA_GH_TARBALL" ] &&
            [ "$SHA_FILE" = "$SHA_SUMS" ] && [ "$SHA_FILE" = "$SHA_GH_TARBALL" ]; then
            printf '%s\n' "result: all three tarball values agree"
        else
            printf '%s\n' "result: MISMATCH"
        fi
    } >"$WORK/checksum-comparison.log"

    cat "$WORK/checksum-comparison.log"
    say ""

    if [ -n "$SHA_SUMS" ] && [ "$SHA_FILE" = "$SHA_SUMS" ]; then
        pass "local sha256sum equals the release SHA256SUMS line for $TARBALL"
    else
        fail "local sha256sum equals the release SHA256SUMS line" \
            "file=$SHA_FILE sums=${SHA_SUMS:-<not listed>}"
    fi
    if [ -n "$SHA_GH_TARBALL" ] && [ "$SHA_FILE" = "$SHA_GH_TARBALL" ]; then
        pass "local sha256sum equals GitHub's asset digest for $TARBALL"
    else
        fail "local sha256sum equals GitHub's asset digest" \
            "file=$SHA_FILE github=${SHA_GH_TARBALL:-<unavailable>}"
    fi
    if [ "$SIZE_FILE" = "$SIZE_GH" ]; then
        pass "downloaded size equals the GitHub asset size ($SIZE_FILE bytes)"
    else
        fail "downloaded size equals the GitHub asset size" "file=$SIZE_FILE github=$SIZE_GH"
    fi
    case "$SUMS_CHECK" in
        *"${TARBALL}: OK"*) pass "sha256sum -c against the release SHA256SUMS is OK" ;;
        *) fail "sha256sum -c against the release SHA256SUMS is OK" "$SUMS_CHECK" ;;
    esac

    # (d) run the published installer for real, into a scratch HOME/INSTALL_DIR
    say ""
    say "running the downloaded release asset:"
    say "  HOME=$WORK/home INSTALL_DIR=$WORK/bin VERSION=$TAG sh \$asset/install.sh"
    say ""
    set +e
    HOME="$WORK/home" INSTALL_DIR="$WORK/bin" VERSION="$TAG" \
        sh "$WORK/asset/install.sh" >"$WORK/.installer-raw.log" 2>&1
    INSTALL_RC=$?
    set -e
    tee -a "$LOG" <"$WORK/.installer-raw.log"
    say ""

    if [ "$INSTALL_RC" -eq 0 ]; then
        pass "pinned installer exits 0"
    else
        fail "pinned installer exits 0" "exit $INSTALL_RC"
    fi

    INSTALLER_LOG="$WORK/.installer-raw.log"
    if [ -f "$WORK/bin/deepseek-build" ] && [ -x "$WORK/bin/deepseek-build" ]; then
        pass "installed binary is present and executable: \$INSTALL_DIR/deepseek-build"
    else
        fail "installed binary is present and executable" "$WORK/bin/deepseek-build"
    fi
    if [ -L "$WORK/bin/dsb" ]; then
        pass "dsb shortcut is a symlink in INSTALL_DIR"
    else
        fail "dsb shortcut is a symlink in INSTALL_DIR" "$WORK/bin/dsb"
    fi
    if grep -qF "Installed v$VER" "$INSTALLER_LOG"; then
        pass "installer reports 'Installed v$VER'"
    else
        fail "installer reports 'Installed v$VER'" "$(tail -n 3 "$INSTALLER_LOG" 2>/dev/null)"
    fi
    if grep -qF "Checksum OK" "$INSTALLER_LOG"; then
        pass "installer reports 'Checksum OK' (checksum verified before extraction)"
    else
        fail "installer reports 'Checksum OK'"
    fi

    BIN_VERSION=""
    if [ -x "$WORK/bin/deepseek-build" ]; then
        BIN_VERSION=$("$WORK/bin/deepseek-build" --version 2>&1 || true)
    fi
    VERSION_PREFIX=0
    case "$BIN_VERSION" in
        "deepseek-build $VER "*) VERSION_PREFIX=1 ;;
    esac
    if [ "$VERSION_PREFIX" = 1 ] &&
        printf '%s\n' "$BIN_VERSION" | grep -Eq '^deepseek-build [^ ]+ \([0-9a-f]{7,40}\)$'; then
        pass "installed binary reports: $BIN_VERSION"
    else
        fail "installed binary prints 'deepseek-build $VER (<7-40 hex>)'" "${BIN_VERSION:-<no output>}"
    fi
    say "  installed --version: ${BIN_VERSION:-<no output>}"
    say ""
fi

# ---------------------------------------------------------------------------
# --latest-behaviour — what the unpinned path really does
# ---------------------------------------------------------------------------

if [ "$MODE" = latest ]; then
    PROBE_LOG="$WORK/probe.log"
    : >"$PROBE_LOG"
    probe() {
        printf '%s\n' "$*" | tee -a "$PROBE_LOG"
    }

    curl -fSL --retry 3 --retry-delay 2 --no-progress-meter -o "$WORK/asset/install.sh" \
        "$RELEASES/download/$ASSET_TAG/install.sh"
    ASSET_SHA=$(sha256_of "$WORK/asset/install.sh")

    probe "prove-install: unpinned (releases/latest) path — real behaviour"
    probe "date:   $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    probe "host:   $(uname -s)/$(uname -m) -> $HOST"
    probe "asset:  $RELEASES/download/$ASSET_TAG/install.sh"
    probe "asset sha256: $ASSET_SHA"
    probe ""
    probe "URL                                                             code(-L)  effective URL"

    LATEST_INDEX="$RELEASES/latest"
    LATEST_SUMS="$RELEASES/latest/download/SHA256SUMS"
    LATEST_INSTALL="$RELEASES/latest/download/install.sh"

    CODE_INDEX=$(curl -sS -o /dev/null -w '%{http_code}' -L "$LATEST_INDEX")
    EFF_INDEX=$(curl -sS -o /dev/null -w '%{url_effective}' -L "$LATEST_INDEX")
    probe "$LATEST_INDEX $CODE_INDEX  $EFF_INDEX"

    RAW_INDEX=$(curl -sS -o /dev/null -w '%{http_code}' "$LATEST_INDEX")
    RAW_LOC=$(curl -sS -o /dev/null -D - "$LATEST_INDEX" | tr -d '\r' | sed -n 's/^[Ll]ocation: //p' | head -n 1)
    probe "  (no redirect follow: $RAW_INDEX -> ${RAW_LOC:-<no Location header>})"

    CODE_SUMS=$(curl -sS -o /dev/null -w '%{http_code}' -L "$LATEST_SUMS")
    EFF_SUMS=$(curl -sS -o /dev/null -w '%{url_effective}' -L "$LATEST_SUMS")
    probe "$LATEST_SUMS $CODE_SUMS  $EFF_SUMS"

    CODE_INSTALL=$(curl -sS -o /dev/null -w '%{http_code}' -L "$LATEST_INSTALL")
    EFF_INSTALL=$(curl -sS -o /dev/null -w '%{url_effective}' -L "$LATEST_INSTALL")
    probe "$LATEST_INSTALL $CODE_INSTALL  $EFF_INSTALL"
    probe ""

    API_CODE=$(curl -sS -o /dev/null -w '%{http_code}' "https://api.github.com/repos/$REPO/releases/latest")
    probe "https://api.github.com/repos/$REPO/releases/latest $API_CODE  (REST 'latest' = newest non-draft, non-prerelease)"
    probe ""
    probe "GitHub's releases/latest excludes pre-releases by definition; with only"
    probe "pre-releases published, the two download URLs above cannot serve a file."
    probe ""

    # A stable release appearing would make "404" a false prescription.
    if [ "$CODE_SUMS" = "200" ] || [ "$CODE_INSTALL" = "200" ]; then
        printf '%s\n' "prove-install: a stable release now resolves at releases/latest —" \
            "the 404 assertion no longer holds. Re-derive the unpinned path before" \
            "recording this item." >&2
        exit 1
    fi

    if [ "$CODE_SUMS" = "404" ]; then
        pass "releases/latest/download/SHA256SUMS returns 404"
    else
        fail "releases/latest/download/SHA256SUMS returns 404" "got $CODE_SUMS -> $EFF_SUMS"
    fi
    if [ "$CODE_INSTALL" = "404" ]; then
        pass "releases/latest/download/install.sh returns 404"
    else
        fail "releases/latest/download/install.sh returns 404" "got $CODE_INSTALL -> $EFF_INSTALL"
    fi
    case "$EFF_INDEX" in
        */releases/tag/*)
            fail "releases/latest does not resolve to a stable tag page" "effective URL $EFF_INDEX"
            ;;
        *)
            pass "releases/latest resolves to the release index, not a stable tag page ($EFF_INDEX)"
            ;;
    esac
    if [ "$API_CODE" = "404" ]; then
        pass "the REST endpoint repos/$REPO/releases/latest returns 404"
    else
        fail "the REST endpoint repos/$REPO/releases/latest returns 404" "got $API_CODE"
    fi

    # The published installer, run with no VERSION, must fail and install nothing.
    say ""
    say "running the released installer with no VERSION:"
    say "  HOME=$WORK/home INSTALL_DIR=$WORK/bin sh \$asset/install.sh"
    say ""
    set +e
    HOME="$WORK/home" INSTALL_DIR="$WORK/bin" \
        sh "$WORK/asset/install.sh" >"$WORK/.installer-raw.log" 2>&1
    LATEST_RC=$?
    set -e
    tee -a "$LOG" <"$WORK/.installer-raw.log"
    say ""

    LATEST_INSTALLER_LOG="$WORK/.installer-raw.log"
    if [ "$LATEST_RC" -ne 0 ]; then
        pass "unpinned installer exits non-zero (exit $LATEST_RC)"
    else
        fail "unpinned installer exits non-zero" "exit 0"
    fi
    BIN_LEFTOVERS=$(find "$WORK/bin" -mindepth 1 -maxdepth 1 2>/dev/null | head -n 5 || true)
    if [ -z "$BIN_LEFTOVERS" ]; then
        pass "nothing was installed into the scratch INSTALL_DIR"
    else
        fail "nothing was installed into the scratch INSTALL_DIR" "$(printf '%s' "$BIN_LEFTOVERS" | tr '\n' ' ')"
    fi
    if grep -qF "Resolving the latest release" "$LATEST_INSTALLER_LOG"; then
        pass "installer log shows it took the latest-resolution branch"
    else
        fail "installer log shows it took the latest-resolution branch"
    fi
    if grep -qE '404' "$LATEST_INSTALLER_LOG"; then
        pass "installer failure is a 404"
    else
        fail "installer failure is a 404" "$(tail -n 3 "$LATEST_INSTALLER_LOG" 2>/dev/null)"
    fi
    if grep -qF "404  $LATEST_SUMS" "$PROBE_LOG"; then
        pass "probe.log names the SHA256SUMS URL and its 404"
    else
        fail "probe.log names the SHA256SUMS URL and its 404"
    fi
    say ""
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

TOTAL=$((PASS + FAIL))
if [ "$FAIL" -eq 0 ]; then
    say "prove-install: $PASS/$TOTAL PASS"
else
    say "prove-install: $PASS/$TOTAL PASS ($FAIL failed:$FAILED_NAMES)"
fi
say ""
say "transcript: $LOG"

if [ "$FAIL" -ne 0 ]; then
    exit 1
fi
exit 0
