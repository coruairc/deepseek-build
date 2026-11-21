#!/bin/sh
# test-sandbox-run.sh — offline assertion harness for the LD_PRELOAD connect()
# guard behind scripts/sandbox-run.sh (scripts/egress_guard.c).
#
# Compiles the tracked shim into a temp dir and drives small python3
# connectors through it, so each case is a syscall-level observation:
#   1. an IPv6 destination is refused and named in the blocked line
#   2. a refused IPv6 destination is never reported as 'non-ip'
#   3. IPv6 loopback is permitted, including its v4-mapped spelling
#   4. the allow host resolves over both families (EGRESS_ALLOW_DEBUG=1), an
#      IPv6 allowlist entry is permitted, and a v4 entry also matches its
#      v4-mapped spelling
#   5. the IPv4 refusal path still names the destination
#   6. scripts/sandbox-run.sh itself runs end to end (BIN=/bin/echo)
#
# No network beyond loopback: every allow host is a numeric literal
# (2001:db8::2, 203.0.113.9) or 'localhost', which resolves from /etc/hosts.
# A connector that prints "connect-failed: ..." has still proven its point:
# the kernel's refusal (ECONNREFUSED/ENETUNREACH) is the successful refusal,
# not a harness failure. Only the shim's BLOCKED lines are a verdict.
#
# Cleans up via trap. Run: sh scripts/test-sandbox-run.sh

set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
GUARD_C="$ROOT/scripts/egress_guard.c"
SANDBOX_SH="$ROOT/scripts/sandbox-run.sh"

PASS=0
FAIL=0
SKIP=0
FAILED_NAMES=""

# ---------------------------------------------------------------------------
# Harness helpers
# ---------------------------------------------------------------------------

say() { printf '%s\n' "$*"; }

pass() {
    PASS=$((PASS + 1))
    say "PASS: $1"
}

skip() {
    SKIP=$((SKIP + 1))
    say "SKIP: $1"
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

has() {
    case "$1" in
        *"$2"*) return 0 ;;
        *) return 1 ;;
    esac
}

flat() { printf '%s' "$1" | tr '\n' ' '; }

# ---------------------------------------------------------------------------
# Build the shim
# ---------------------------------------------------------------------------

WORK=$(mktemp -d "${TMPDIR:-/tmp}/dsb-test-sandbox.XXXXXX")
# shellcheck disable=SC2317  # reached via trap
cleanup() { rm -rf "$WORK"; }
trap cleanup EXIT HUP INT TERM

if ! cc -shared -fPIC -O2 -o "$WORK/egress_guard.so" "$GUARD_C" -ldl 2>"$WORK/cc.err"; then
    say "FAILED: could not compile $GUARD_C"
    sed 's/^/      /' "$WORK/cc.err"
    exit 1
fi
GUARD="$WORK/egress_guard.so"

cat >"$WORK/connect.py" <<'PY'
# One TCP connect attempt. The exit status is not a verdict: a refused or
# unreachable destination is the expected outcome for this harness.
import socket
import sys

family, host = sys.argv[1], sys.argv[2]
port = int(sys.argv[3]) if len(sys.argv) > 3 else 443
try:
    if family == "inet6":
        sock = socket.socket(socket.AF_INET6, socket.SOCK_STREAM)
        dest = (host, port, 0, 0)
    else:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        dest = (host, port)
except OSError as exc:
    # The family is unavailable on this kernel: the caller reports SKIP, not
    # a failure, because the shim was never exercised either way.
    print("socket-unavailable: %s" % exc, file=sys.stderr)
    sys.exit(77)
sock.settimeout(1)
try:
    sock.connect(dest)
except OSError as exc:
    print("connect-failed: %s" % exc, file=sys.stderr)
finally:
    sock.close()
PY

say "=== egress_guard.c harness ==="

# ---------------------------------------------------------------------------
# Assertion 1: an IPv6 destination is refused and named.
# ---------------------------------------------------------------------------

OUT=$(env EGRESS_ALLOW_HOST=localhost LD_PRELOAD="$GUARD" \
    python3 "$WORK/connect.py" inet6 2001:db8::1 2>&1) || true
if has "$OUT" "socket-unavailable"; then
    skip "ipv6-refused: no AF_INET6 on this kernel"
else
    assert_contains "$OUT" "sandbox-run: BLOCKED connect to 2001:db8::1" \
        "ipv6-refused: blocked line names 2001:db8::1"
fi

# ---------------------------------------------------------------------------
# Assertion 2: that refusal is not reported as the 'non-ip' placeholder.
# ---------------------------------------------------------------------------

if ! has "$OUT" "socket-unavailable"; then
    assert_not_contains "$OUT" "non-ip" "ipv6-refused: no 'non-ip' placeholder"
fi

# ---------------------------------------------------------------------------
# Assertion 3: IPv6 loopback is permitted (plain and v4-mapped spelling).
# ---------------------------------------------------------------------------

LOOP=$(env EGRESS_ALLOW_HOST=localhost LD_PRELOAD="$GUARD" \
    python3 "$WORK/connect.py" inet6 ::1 2>&1) || true
MAPPED_LOOP=$(env EGRESS_ALLOW_HOST=localhost LD_PRELOAD="$GUARD" \
    python3 "$WORK/connect.py" inet6 ::ffff:127.0.0.1 2>&1) || true
if has "$LOOP" "socket-unavailable"; then
    skip "ipv6-loopback: no AF_INET6 on this kernel"
elif has "$LOOP" "sandbox-run: BLOCKED connect" || has "$MAPPED_LOOP" "sandbox-run: BLOCKED connect"; then
    fail "ipv6-loopback: ::1 and ::ffff:127.0.0.1 permitted" \
        "$(flat "$LOOP $MAPPED_LOOP")"
else
    pass "ipv6-loopback: ::1 and ::ffff:127.0.0.1 permitted"
fi

# ---------------------------------------------------------------------------
# Assertion 4: the allow host resolves over both families, an IPv6 allowlist
# entry is permitted, and a v4 entry matches its v4-mapped spelling. A numeric
# literal resolves with no DNS, which is what keeps this harness offline.
# ---------------------------------------------------------------------------

ALLOW6=$(env EGRESS_ALLOW_HOST=2001:db8::2 EGRESS_ALLOW_DEBUG=1 LD_PRELOAD="$GUARD" \
    python3 "$WORK/connect.py" inet6 2001:db8::2 2>&1) || true
ALLOW_MAPPED=$(env EGRESS_ALLOW_HOST=203.0.113.9 LD_PRELOAD="$GUARD" \
    python3 "$WORK/connect.py" inet6 ::ffff:203.0.113.9 2>&1) || true
if has "$ALLOW6" "socket-unavailable"; then
    skip "ipv6-allowlist: no AF_INET6 on this kernel"
elif has "$ALLOW6" "sandbox-run: allow family=inet6 addr=2001:db8::2 port=443" &&
    has "$ALLOW6" "family=inet6" &&
    ! has "$ALLOW6" "sandbox-run: BLOCKED connect" &&
    ! has "$ALLOW_MAPPED" "sandbox-run: BLOCKED connect"; then
    pass "ipv6-allowlist: resolves IPv6, permits it, and matches the v4-mapped spelling"
else
    fail "ipv6-allowlist: resolves IPv6, permits it, and matches the v4-mapped spelling" \
        "$(flat "$ALLOW6") | $(flat "$ALLOW_MAPPED")"
fi

# ---------------------------------------------------------------------------
# Assertion 5: the IPv4 refusal path is unchanged and names the destination.
# ---------------------------------------------------------------------------

OUT=$(env EGRESS_ALLOW_HOST=localhost LD_PRELOAD="$GUARD" \
    python3 "$WORK/connect.py" inet 203.0.113.9 2>&1)
assert_contains "$OUT" "sandbox-run: BLOCKED connect to 203.0.113.9" \
    "ipv4-refused: blocked line names 203.0.113.9"

# ---------------------------------------------------------------------------
# Assertion 6: the real script compiles the shim and execs the binary.
# The explicit loopback host matters: the script's default is
# api.deepseek.com, and the shim's constructor resolves it, so omitting it
# would turn this offline harness into a live DNS lookup.
# ---------------------------------------------------------------------------

OUT=$(env DEEPSEEK_BUILD_EGRESS_HOST=localhost BIN=/bin/echo \
    bash "$SANDBOX_SH" --version 2>&1) && RC=0 || RC=$?
if [ "$RC" -eq 0 ] && has "$OUT" "sandbox-run: allowing only localhost:443"; then
    pass "script: sandbox-run.sh runs end to end (BIN=/bin/echo)"
else
    fail "script: sandbox-run.sh runs end to end (BIN=/bin/echo)" \
        "rc=$RC $(flat "$OUT")"
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

say ""
if [ "$FAIL" -eq 0 ]; then
    if [ "$SKIP" -gt 0 ]; then
        say "ALL PASS ($PASS assertions, $SKIP skipped)"
    else
        say "ALL PASS ($PASS assertions)"
    fi
    exit 0
else
    say "FAILED: $FAIL of $((PASS + FAIL)) assertions ($FAILED_NAMES)"
    exit 1
fi
