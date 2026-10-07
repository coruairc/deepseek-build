#!/bin/sh
# test-install.sh — local test harness for install.sh
#
# Builds a fake release tree (fake binary, tarballs, SHA256SUMS), serves it
# over HTTP on 127.0.0.1, and runs install.sh through its paths:
#   1. success (+ PATH hint on/off)
#   2. checksum mismatch (must abort, install nothing)
#   3. missing platform (unsupported OS; tarball 404)
#   4. uninstall (present / absent)
#   5. rerun-to-update
#   6. unwritable INSTALL_DIR (no sudo suggestion)
#
# No network beyond 127.0.0.1. Cleans up via trap.
# Run: sh scripts/test-install.sh

set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
INSTALL_SH="$ROOT/install.sh"
BASE="http://127.0.0.1:0"       # rewritten by start_server (dynamic port)
BASE_BAD="http://127.0.0.1:0"  # rewritten by start_server ... bad
FAKE_VERSION=0.0.0-test
FAKE_VERSION_2=0.0.1-test

PASS=0
FAIL=0
FAILED_NAMES=""

# ---------------------------------------------------------------------------
# Harness helpers
# ---------------------------------------------------------------------------

say() { printf '%s\n' "$*"; }

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

assert_contains() {
    # assert_contains <haystack> <needle> <case-name>
    case "$1" in
        *"$2"*) pass "$3" ;;
        *) fail "$3" "output missing: $2" ;;
    esac
}

assert_not_contains() {
    case "$1" in
        *"$2"*) fail "$3" "output unexpectedly contains: $2" ;;
        *) pass "$3" ;;
    esac
}

# Host target triple, same mapping as install.sh.
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

# ---------------------------------------------------------------------------
# Fake release tree
# ---------------------------------------------------------------------------

make_fake_binary() {
    # make_fake_binary <dir> <version-string>
    mkdir -p "$1"
    cat >"$1/deepseek-build" <<EOF
#!/bin/sh
if [ "\${1:-}" = "--version" ]; then
    printf '%s\n' "deepseek-build $2 (deadbeefdead)"
    exit 0
fi
printf '%s\n' "fake binary"
EOF
    chmod +x "$1/deepseek-build"
}

make_tarball() {
    # make_tarball <stage-dir> <version> <target> <outdir> [corrupt]
    # Writes the tarball and a SHA256SUMS containing the PRISTINE hash (computed
    # before any corruption), so a corrupt tree yields a checksum mismatch.
    _stage=$1
    _ver=$2
    _target=$3
    _out=$4
    _name="deepseek-build-$_ver-$_target.tar.gz"
    rm -rf "$_stage"
    mkdir -p "$_stage"
    make_fake_binary "$_stage" "$_ver"
    printf 'license\n' >"$_stage/LICENSE"
    printf 'notice\n' >"$_stage/NOTICE"
    printf 'third-party\n' >"$_stage/THIRD-PARTY-NOTICES"
    printf 'readme\n' >"$_stage/README.txt"
    tar -czf "$_out/$_name" -C "$_stage" deepseek-build LICENSE NOTICE THIRD-PARTY-NOTICES README.txt
    (cd "$_out" && sha256sum "$_name" >SHA256SUMS)
    if [ "${5:-}" = "corrupt" ]; then
        # Flip one byte in the middle of the gzip stream (after the sums file
        # was written, so the recorded hash no longer matches the file).
        _size=$(wc -c <"$_out/$_name")
        _off=$((_size / 2))
        printf '\x00' | dd of="$_out/$_name" bs=1 seek=$_off count=1 conv=notrunc status=none
    fi
}

build_fake_tree() {
    # build_fake_tree <tree-dir> [corrupt]
    # Mirrors the GitHub releases URL layout: <tree>/download/<tag>/<asset>.
    _tree=$1
    _corrupt=${2:-}
    _host=$(host_target)
    [ -n "$_host" ] || {
        say "SKIP: unknown host platform, cannot build fake tree"
        exit 77
    }
    mkdir -p "$_tree/download/v$FAKE_VERSION" "$_tree/download/v$FAKE_VERSION_2" "$_tree/latest/download"
    make_tarball "$_tree/stage" "$FAKE_VERSION" "$_host" "$_tree/download/v$FAKE_VERSION" "$_corrupt"
    # A second version for the rerun-to-update case.
    make_tarball "$_tree/stage2" "$FAKE_VERSION_2" "$_host" "$_tree/download/v$FAKE_VERSION_2"
    # The "latest" release points at the newest version (v2).
    cp "$_tree/download/v$FAKE_VERSION_2/deepseek-build-$FAKE_VERSION_2-$_host.tar.gz" "$_tree/latest/download/"
    cp "$_tree/download/v$FAKE_VERSION_2/SHA256SUMS" "$_tree/latest/download/SHA256SUMS"
    # install.sh as a release asset (some tests fetch it).
    cp "$INSTALL_SH" "$_tree/install.sh"
}

# ---------------------------------------------------------------------------
# HTTP server
# ---------------------------------------------------------------------------

SERVER_PID=""
BAD_PID=""
free_port() {
    # free_port — pick an unused TCP port on 127.0.0.1
    python3 - <<'PY'
import socket
s = socket.socket()
s.bind(("127.0.0.1", 0))
print(s.getsockname()[1])
s.close()
PY
}

start_server() {
    # start_server <dir> [bad] — serves <dir> on a fresh port (or the "bad" port)
    _dir=$1
    if [ "${2:-}" = "bad" ]; then
        PORT_BAD=$(free_port)
        BASE_BAD="http://127.0.0.1:$PORT_BAD"
        python3 -m http.server "$PORT_BAD" --bind 127.0.0.1 --directory "$_dir" >/dev/null 2>&1 &
        BAD_PID=$!
        _probe="$BASE_BAD/download/v$FAKE_VERSION/SHA256SUMS"
    else
        PORT=$(free_port)
        BASE="http://127.0.0.1:$PORT"
        python3 -m http.server "$PORT" --bind 127.0.0.1 --directory "$_dir" >/dev/null 2>&1 &
        SERVER_PID=$!
        _probe="$BASE/download/v$FAKE_VERSION/SHA256SUMS"
    fi
    _i=0
    while ! fetch_ok "$_probe" 2>/dev/null; do
        _i=$((_i + 1))
        if [ "$_i" -gt 50 ]; then
            say "FAIL: local HTTP server did not come up"
            exit 1
        fi
        sleep 0.2
    done
}

stop_server() {
    if [ -n "$SERVER_PID" ]; then
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
        SERVER_PID=""
    fi
    if [ -n "$BAD_PID" ]; then
        kill "$BAD_PID" 2>/dev/null || true
        wait "$BAD_PID" 2>/dev/null || true
        BAD_PID=""
    fi
}

fetch_ok() {
    # fetch_ok <url> — exit 0 if the URL is fetchable
    if command -v curl >/dev/null 2>&1; then
        curl -fsS -o /dev/null "$1"
    else
        wget -q -O /dev/null "$1"
    fi
}

# ---------------------------------------------------------------------------
# Sandbox helpers
# ---------------------------------------------------------------------------

new_sandbox() {
    # new_sandbox <name> — creates a fresh HOME + INSTALL_DIR; sets SB_HOME/SB_BIN
    SB_HOME=$(mktemp -d "${TMPDIR:-/tmp}/dsb-test-home-$1.XXXXXX")
    SB_BIN="$SB_HOME/bin"
}

cleanup() {
    # shellcheck disable=SC2317  # reached via trap
    stop_server
    # shellcheck disable=SC2317  # reached via trap
    [ -n "${SB_HOME:-}" ] && rm -rf "$SB_HOME"
    # shellcheck disable=SC2317  # reached via trap
    [ -n "${TREE:-}" ] && rm -rf "$TREE"
    # shellcheck disable=SC2317  # reached via trap
    [ -n "${TREE_BAD:-}" ] && rm -rf "$TREE_BAD"
    # shellcheck disable=SC2317  # reached via trap
    [ -n "${TREE_NOHOST:-}" ] && rm -rf "$TREE_NOHOST"
}
trap cleanup EXIT HUP INT TERM

# ---------------------------------------------------------------------------
# Setup
# ---------------------------------------------------------------------------

TREE=$(mktemp -d "${TMPDIR:-/tmp}/dsb-test-tree.XXXXXX")
TREE_BAD=$(mktemp -d "${TMPDIR:-/tmp}/dsb-test-treebad.XXXXXX")
build_fake_tree "$TREE"
build_fake_tree "$TREE_BAD" corrupt
start_server "$TREE"
start_server "$TREE_BAD" bad

say "=== install.sh test suite (host target: $(host_target)) ==="

# ---------------------------------------------------------------------------
# Case 1: success + PATH hint behavior
# ---------------------------------------------------------------------------

new_sandbox success
OUT=$(HOME="$SB_HOME" INSTALL_DIR="$SB_BIN" VERSION="$FAKE_VERSION" \
    PATH="$SB_BIN:/usr/bin:/bin" DSB_INSTALL_BASE_URL="$BASE" sh "$INSTALL_SH" 2>&1) || fail "success-install" "exit nonzero"
if [ -x "$SB_BIN/deepseek-build" ]; then
    pass "success-install: binary installed and executable"
else
    fail "success-install" "binary missing or not executable at $SB_BIN/deepseek-build"
fi
assert_contains "$OUT" "Independent project, not affiliated with DeepSeek." "success-install: non-affiliation line"
assert_contains "$OUT" "deepseek-build $FAKE_VERSION" "success-install: version printed"
assert_not_contains "$OUT" "not in your PATH" "success-install: no PATH hint when dir is on PATH"

# PATH hint when INSTALL_DIR is not on PATH
new_sandbox hint
OUT=$(HOME="$SB_HOME" INSTALL_DIR="$SB_BIN" VERSION="$FAKE_VERSION" \
    PATH="/usr/bin:/bin" DSB_INSTALL_BASE_URL="$BASE" sh "$INSTALL_SH" 2>&1) || true
assert_contains "$OUT" "not in your PATH" "path-hint: hint printed when dir not on PATH"

# ---------------------------------------------------------------------------
# Case 1b: default "latest" (no VERSION) resolves via latest/download/SHA256SUMS
# ---------------------------------------------------------------------------

new_sandbox latest
OUT=$(HOME="$SB_HOME" INSTALL_DIR="$SB_BIN" \
    PATH="$SB_BIN:/usr/bin:/bin" DSB_INSTALL_BASE_URL="$BASE" sh "$INSTALL_SH" 2>&1) || fail "latest-install" "exit nonzero"
if [ -x "$SB_BIN/deepseek-build" ]; then
    pass "latest-install: binary installed"
else
    fail "latest-install" "binary missing"
fi
assert_contains "$OUT" "Latest release: v$FAKE_VERSION_2" "latest-install: resolved newest tag"
assert_contains "$OUT" "deepseek-build $FAKE_VERSION_2" "latest-install: newest version installed"
assert_contains "$OUT" "Independent project, not affiliated with DeepSeek." "latest-install: non-affiliation line"

# ---------------------------------------------------------------------------
# Case 2: checksum mismatch — abort, install nothing
# ---------------------------------------------------------------------------

new_sandbox mismatch
OUT=$(HOME="$SB_HOME" INSTALL_DIR="$SB_BIN" VERSION="$FAKE_VERSION" \
    DSB_INSTALL_BASE_URL="$BASE_BAD" sh "$INSTALL_SH" 2>&1) && RC=0 || RC=$?
if [ "$RC" -ne 0 ]; then
    pass "checksum-mismatch: nonzero exit"
else
    fail "checksum-mismatch" "script exited 0 despite corrupt tarball"
fi
assert_contains "$OUT" "Checksum mismatch" "checksum-mismatch: mismatch message"
if [ -e "$SB_BIN" ] && [ -n "$(ls -A "$SB_BIN" 2>/dev/null)" ]; then
    fail "checksum-mismatch: nothing installed" "INSTALL_DIR not empty: $(ls -A "$SB_BIN")"
else
    pass "checksum-mismatch: nothing installed"
fi

# ---------------------------------------------------------------------------
# Case 3: missing platform
# ---------------------------------------------------------------------------

# 3a: unsupported OS via test hook
new_sandbox bados
OUT=$(HOME="$SB_HOME" INSTALL_DIR="$SB_BIN" VERSION="$FAKE_VERSION" \
    DSB_INSTALL_TEST_UNAME_S=SunOS DSB_INSTALL_BASE_URL="$BASE" sh "$INSTALL_SH" 2>&1) && RC=0 || RC=$?
if [ "$RC" -ne 0 ]; then
    pass "unsupported-os: nonzero exit"
else
    fail "unsupported-os" "exited 0 for SunOS"
fi
assert_contains "$OUT" "Unsupported platform" "unsupported-os: clear error"
assert_contains "$OUT" "SunOS" "unsupported-os: names the OS"

# 3b: tarball 404 for the host target (tree lacks the host tarball)
TREE_NOHOST=$(mktemp -d "${TMPDIR:-/tmp}/dsb-test-nohost.XXXXXX")
mkdir -p "$TREE_NOHOST/download/v$FAKE_VERSION"
cp "$TREE/download/v$FAKE_VERSION/SHA256SUMS" "$TREE_NOHOST/download/v$FAKE_VERSION/SHA256SUMS"
cp "$TREE/install.sh" "$TREE_NOHOST/install.sh"
stop_server
start_server "$TREE_NOHOST"
new_sandbox nohost
OUT=$(HOME="$SB_HOME" INSTALL_DIR="$SB_BIN" VERSION="$FAKE_VERSION" \
    DSB_INSTALL_BASE_URL="$BASE" sh "$INSTALL_SH" 2>&1) && RC=0 || RC=$?
if [ "$RC" -ne 0 ]; then
    pass "tarball-404: nonzero exit"
else
    fail "tarball-404" "exited 0 despite missing tarball"
fi
assert_contains "$OUT" "error" "tarball-404: error surfaced"
if [ -e "$SB_BIN" ] && [ -n "$(ls -A "$SB_BIN" 2>/dev/null)" ]; then
    fail "tarball-404: nothing installed" "INSTALL_DIR not empty"
else
    pass "tarball-404: nothing installed"
fi
# Restore the good tree on the main port for the remaining cases.
stop_server
start_server "$TREE"

# ---------------------------------------------------------------------------
# Case 4: uninstall
# ---------------------------------------------------------------------------

new_sandbox uninstall
HOME="$SB_HOME" INSTALL_DIR="$SB_BIN" VERSION="$FAKE_VERSION" \
    DSB_INSTALL_BASE_URL="$BASE" sh "$INSTALL_SH" >/dev/null 2>&1 || true
[ -x "$SB_BIN/deepseek-build" ] || fail "uninstall: setup" "binary missing before uninstall"
OUT=$(HOME="$SB_HOME" INSTALL_DIR="$SB_BIN" DSB_INSTALL_BASE_URL="$BASE" sh "$INSTALL_SH" --uninstall 2>&1) && RC=0 || RC=$?
if [ "$RC" -eq 0 ] && [ ! -e "$SB_BIN/deepseek-build" ]; then
    pass "uninstall: removes binary"
else
    fail "uninstall: removes binary" "rc=$RC"
fi
assert_not_contains "$OUT" "http" "uninstall: no network (no URL in output)"
OUT=$(HOME="$SB_HOME" INSTALL_DIR="$SB_BIN" sh "$INSTALL_SH" --uninstall 2>&1) && RC=0 || RC=$?
if [ "$RC" -eq 0 ]; then
    pass "uninstall: idempotent when absent"
else
    fail "uninstall: idempotent when absent" "rc=$RC"
fi

# ---------------------------------------------------------------------------
# Case 5: rerun-to-update
# ---------------------------------------------------------------------------

new_sandbox update
HOME="$SB_HOME" INSTALL_DIR="$SB_BIN" VERSION="$FAKE_VERSION" \
    DSB_INSTALL_BASE_URL="$BASE" sh "$INSTALL_SH" >/dev/null 2>&1
HOME="$SB_HOME" INSTALL_DIR="$SB_BIN" VERSION="$FAKE_VERSION_2" \
    DSB_INSTALL_BASE_URL="$BASE" sh "$INSTALL_SH" >/dev/null 2>&1
V2=$(HOME="$SB_HOME" "$SB_BIN/deepseek-build" --version)
case "$V2" in
    *"$FAKE_VERSION_2"*) pass "rerun-update: binary replaced with new version" ;;
    *) fail "rerun-update" "expected $FAKE_VERSION_2 in $V2" ;;
esac

# ---------------------------------------------------------------------------
# Case 6: unwritable INSTALL_DIR — clear failure, no sudo suggestion
# ---------------------------------------------------------------------------

new_sandbox ro
mkdir -p "$SB_BIN"
chmod 555 "$SB_BIN"
OUT=$(HOME="$SB_HOME" INSTALL_DIR="$SB_BIN" VERSION="$FAKE_VERSION" \
    DSB_INSTALL_BASE_URL="$BASE" sh "$INSTALL_SH" 2>&1) && RC=0 || RC=$?
chmod 755 "$SB_BIN"
if [ "$RC" -ne 0 ]; then
    pass "unwritable-dir: nonzero exit"
else
    fail "unwritable-dir" "exited 0 despite read-only INSTALL_DIR"
fi
assert_contains "$OUT" "not writable" "unwritable-dir: clear message"
assert_not_contains "$OUT" "sudo" "unwritable-dir: no sudo suggestion"

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

say ""
if [ "$FAIL" -eq 0 ]; then
    say "ALL PASS ($PASS assertions)"
    exit 0
else
    say "FAILED: $FAIL of $((PASS + FAIL)) assertions ($FAILED_NAMES)"
    exit 1
fi