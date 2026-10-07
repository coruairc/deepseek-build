#!/bin/sh
# install.sh — deepseek-build installer
#
# Downloads the latest (or VERSION=vX.Y.Z) release tarball and SHA256SUMS from
# GitHub Releases over HTTPS, verifies the checksum BEFORE extracting, and
# installs the binary to ${INSTALL_DIR:-$HOME/.local/bin}. No sudo.
#
# Network contract: this script contacts exactly one host — github.com (release
# download of the tarball and SHA256SUMS). Nothing else. No telemetry, no
# version-check API calls, no update checks.
#
# Usage:
#   sh install.sh                     install latest release
#   VERSION=v0.1.0 sh install.sh      install a pinned version
#   INSTALL_DIR=/opt/bin sh install.sh
#   sh install.sh --uninstall         remove the installed binary
#   sh install.sh --help
#
# Testing hooks (do not set in production):
#   DSB_INSTALL_BASE_URL     override the GitHub releases base URL (local HTTP
#                            server for tests). Changes nothing else.
#   DSB_INSTALL_TEST_UNAME_S / DSB_INSTALL_TEST_UNAME_M
#                            override uname -s / uname -m for platform tests.
#
# Rerunning with a newer VERSION (or default latest) overwrites the binary in
# place — that is the update path (the project has no auto-updater).

set -eu

REPO="coruairc/deepseek-build"
DEFAULT_INSTALL_DIR="$HOME/.local/bin"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

log() { printf '%s\n' "==> $*"; }
warn() { printf '%s\n' "!!  $*" >&2; }
die() { warn "$*"; exit 1; }

have() { command -v "$1" >/dev/null 2>&1; }

# ---------------------------------------------------------------------------
# Testing hooks (no-ops in production)
# ---------------------------------------------------------------------------

if [ "${DSB_INSTALL_TEST_UNAME_S:-}" != "" ]; then
    uname_s() { printf '%s' "$DSB_INSTALL_TEST_UNAME_S"; }
else
    uname_s() { uname -s; }
fi

if [ "${DSB_INSTALL_TEST_UNAME_M:-}" != "" ]; then
    uname_m() { printf '%s' "$DSB_INSTALL_TEST_UNAME_M"; }
else
    uname_m() { uname -m; }
fi

if [ "${DSB_INSTALL_BASE_URL:-}" != "" ]; then
    base_url() { printf '%s' "$DSB_INSTALL_BASE_URL"; }
else
    base_url() { printf 'https://github.com/%s/releases' "$REPO"; }
fi

# ---------------------------------------------------------------------------
# Platform detection
# ---------------------------------------------------------------------------

# Map (uname -s, uname -m) to a release target triple. Unsupported -> empty.
map_target() {
    _os=$(uname_s)
    _arch=$(uname_m)
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

# ---------------------------------------------------------------------------
# Download helpers
# ---------------------------------------------------------------------------

fetch() {
    # fetch <url> <dest>
    _url=$1
    _dest=$2
    if have curl; then
        curl -fSL --retry 3 --retry-delay 2 -o "$_dest" "$_url"
    elif have wget; then
        wget -q --tries=3 -O "$_dest" "$_url"
    else
        die "Neither curl nor wget is available. Install one and retry."
    fi
    [ -s "$_dest" ] || die "Downloaded file is empty: $_url"
}

# ---------------------------------------------------------------------------
# Checksum verification (before any extraction)
# ---------------------------------------------------------------------------

sha256_of() {
    if have sha256sum; then
        sha256sum "$1" | cut -d' ' -f1
    elif have shasum; then
        shasum -a 256 "$1" | cut -d' ' -f1
    else
        die "Neither sha256sum nor shasum is available to verify the checksum."
    fi
}

# Extract the expected hash for <filename> from a sha256sum-format file.
# Format: <hash>  <filename>  (two spaces). Returns nonzero if not listed.
expected_hash_for() {
    # expected_hash_for <sums-file> <filename>
    _sums=$1
    _name=$2
    # Anchor the filename at end-of-line to avoid prefix collisions
    # (e.g. "deepseek-build-0.1.0-x86_64" vs "...-x86_64-unknown-linux-gnu").
    _line=$(grep -E "^[0-9a-fA-F]{64}  ${_name}$" "$_sums" | head -n 1 || true)
    if [ -z "$_line" ]; then
        return 1
    fi
    printf '%s' "$_line" | cut -d' ' -f1
}

# ---------------------------------------------------------------------------
# Argument handling
# ---------------------------------------------------------------------------

MODE="install"
case "${1:-}" in
    "") ;;
    --uninstall) MODE="uninstall" ;;
    --help|-h)
        cat <<EOF
install.sh — deepseek-build installer

Usage:
  sh install.sh                     Install the latest release
  VERSION=v0.1.0 sh install.sh      Install a pinned version
  INSTALL_DIR=/path sh install.sh   Install elsewhere (default: \$HOME/.local/bin)
  sh install.sh --uninstall         Remove the installed binary
  sh install.sh --help              This help

Environment:
  VERSION       Release tag to install, with or without the leading "v"
                (default: latest release)
  INSTALL_DIR   Target directory (default: \$HOME/.local/bin)

Supported platforms:
  Linux x86_64 / arm64 (glibc), macOS x86_64 / arm64

The script downloads the release tarball and SHA256SUMS from github.com,
verifies the checksum before extracting, and installs without sudo. It makes
no other network calls and sends no telemetry.

Independent project, not affiliated with DeepSeek.
EOF
        exit 0
        ;;
    *) die "Unknown argument: $1 (try --help)" ;;
esac

# ---------------------------------------------------------------------------
# Uninstall mode (no network)
# ---------------------------------------------------------------------------

if [ "$MODE" = "uninstall" ]; then
    INSTALL_DIR="${INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"
    TARGET_FILE="$INSTALL_DIR/deepseek-build"
    if [ -f "$TARGET_FILE" ]; then
        rm -f "$TARGET_FILE"
        log "Removed $TARGET_FILE"
    else
        log "Nothing to remove: $TARGET_FILE is absent"
    fi
    exit 0
fi

# ---------------------------------------------------------------------------
# Install mode
# ---------------------------------------------------------------------------

INSTALL_DIR="${INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"

# Normalize VERSION: accept vX.Y.Z or X.Y.Z; empty -> latest (resolved below).
VERSION="${VERSION:-}"

# ---------------------------------------------------------------------------
# Latest-release resolution
# ---------------------------------------------------------------------------
# The release tarball name embeds the version number, so "latest" must be
# resolved to a concrete tag first. GitHub's /releases/latest/download/<asset>
# shortcut only works when the asset name is version-independent, which ours is
# not. Resolution uses the public releases list API (api.github.com, part of
# the github.com host family) and picks the newest non-draft release tag.

resolve_latest_version() {
    _api="https://api.github.com/repos/$REPO/releases/latest"
    _tag=""
    if have curl; then
        _tag=$(curl -fsSL --retry 2 "$_api" 2>/dev/null | grep -o '"tag_name": *"[^"]*"' | head -n 1 | cut -d'"' -f4)
    elif have wget; then
        _tag=$(wget -q -O - "$_api" 2>/dev/null | grep -o '"tag_name": *"[^"]*"' | head -n 1 | cut -d'"' -f4)
    fi
    [ -n "$_tag" ] || die "Could not resolve the latest release tag from api.github.com. Pin one: VERSION=vX.Y.Z sh install.sh"
    printf '%s' "$_tag"
}

if [ "$VERSION" = "" ]; then
    VERSION=$(resolve_latest_version)
fi

case "$VERSION" in
    v*) VERSION_TAG="$VERSION"; VERSION_NUM="${VERSION#v}" ;;
    *) VERSION_TAG="v$VERSION"; VERSION_NUM="$VERSION" ;;
esac
case "$VERSION_TAG" in
    v[0-9]*) ;;
    *) die "Invalid VERSION: $VERSION (expected vX.Y.Z or X.Y.Z)" ;;
esac

# ---------------------------------------------------------------------------
# Platform target
# ---------------------------------------------------------------------------

TARGET=$(map_target)
if [ -z "$TARGET" ]; then
    die "Unsupported platform: $(uname_s)/$(uname_m). Supported: Linux x86_64/arm64, macOS x86_64/arm64."
fi

TARBALL="deepseek-build-${VERSION_NUM}-${TARGET}.tar.gz"

# ---------------------------------------------------------------------------
# Download (tarball + SHA256SUMS) — the only network activity
# ---------------------------------------------------------------------------

TMPDIR_DSB=$(mktemp -d "${TMPDIR:-/tmp}/dsb-install.XXXXXX")
trap 'rm -rf "$TMPDIR_DSB"' EXIT HUP INT TERM

log "Downloading $TARBALL"
fetch "$(base_url)/download/$VERSION_TAG/$TARBALL" "$TMPDIR_DSB/$TARBALL"

log "Downloading SHA256SUMS"
fetch "$(base_url)/download/$VERSION_TAG/SHA256SUMS" "$TMPDIR_DSB/SHA256SUMS"

# ---------------------------------------------------------------------------
# Verify checksum BEFORE extracting
# ---------------------------------------------------------------------------

EXPECTED=$(expected_hash_for "$TMPDIR_DSB/SHA256SUMS" "$TARBALL") || {
    die "$TARBALL is not listed in SHA256SUMS (refusing to install an unverified tarball)"
}
ACTUAL=$(sha256_of "$TMPDIR_DSB/$TARBALL")
if [ "$ACTUAL" != "$EXPECTED" ]; then
    warn "Checksum mismatch for $TARBALL"
    warn "  expected: $EXPECTED"
    warn "  actual:   $ACTUAL"
    die "Aborting: the downloaded tarball does not match SHA256SUMS. Nothing was installed."
fi
log "Checksum OK ($ACTUAL)"

# ---------------------------------------------------------------------------
# Extract and install (no sudo; create the dir if needed)
# ---------------------------------------------------------------------------

tar -xzf "$TMPDIR_DSB/$TARBALL" -C "$TMPDIR_DSB" deepseek-build
[ -f "$TMPDIR_DSB/deepseek-build" ] || die "Tarball does not contain a deepseek-build binary"
[ -x "$TMPDIR_DSB/deepseek-build" ] || chmod +x "$TMPDIR_DSB/deepseek-build"

if ! mkdir -p "$INSTALL_DIR" 2>/dev/null; then
    die "Cannot create $INSTALL_DIR (not writable?). Set INSTALL_DIR to a writable directory."
fi
if ! mv -f "$TMPDIR_DSB/deepseek-build" "$INSTALL_DIR/deepseek-build" 2>/dev/null; then
    die "Cannot write $INSTALL_DIR/deepseek-build (not writable?). Set INSTALL_DIR to a writable directory."
fi
chmod +x "$INSTALL_DIR/deepseek-build" 2>/dev/null || true

# ---------------------------------------------------------------------------
# PATH hint
# ---------------------------------------------------------------------------

case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        warn "$INSTALL_DIR is not in your PATH."
        warn "Add it, e.g.:  export PATH=\"$INSTALL_DIR:\$PATH\""
        warn "Make it permanent in ~/.profile or your shell's rc file."
        ;;
esac

# ---------------------------------------------------------------------------
# Report
# ---------------------------------------------------------------------------

log "Installed $VERSION_TAG to $INSTALL_DIR/deepseek-build"
if "$INSTALL_DIR/deepseek-build" --version; then
    :
else
    warn "The installed binary did not report a version (install itself succeeded)."
fi

printf '%s\n' "Independent project, not affiliated with DeepSeek."