#!/usr/bin/env bash
#
# sandbox-run.sh — run deepseek-build with outbound network restricted to the
# configured model provider (default api.deepseek.com) plus loopback.
#
# Preferred: a network namespace (unshare -rn) that drops all external routes.
#   NOTE: that also makes the provider unreachable, which is useful for
#   "does it try anything else?" runs but not for real prompts.
# Fallback (used here): an LD_PRELOAD shim (scripts/egress_guard.c) that permits
#   connect() only to loopback, AF_UNIX, and the resolved addresses of
#   $DEEPSEEK_BUILD_EGRESS_HOST (default api.deepseek.com); everything else is
#   refused with ECONNREFUSED and named on stderr. The host is resolved over
#   both families (A and AAAA), so an IPv6 provider address is allowlisted too.
#
# Usage:
#   scripts/sandbox-run.sh [deepseek-build args...]
#   DEEPSEEK_BUILD_EGRESS_HOST=my.proxy.example scripts/sandbox-run.sh -p "hi"
#
# EGRESS_ALLOW_DEBUG=1 prints the resolved allowlist (one line per address).
#
# This is a bash script (see the shebang); pipefail is intentional. The repo
# also lints scripts with `shellcheck --shell=sh` for consistency, so declare
# that here.
# shellcheck disable=SC3040
set -euo pipefail

cd "$(dirname "$0")/.."
BIN="${BIN:-target/release/deepseek-build}"
HOST="${DEEPSEEK_BUILD_EGRESS_HOST:-api.deepseek.com}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

cc -shared -fPIC -O2 -o "$WORK/egress_guard.so" scripts/egress_guard.c -ldl

echo "sandbox-run: allowing only ${HOST}:443 + loopback; running: $BIN $*" >&2
exec env EGRESS_ALLOW_HOST="$HOST" LD_PRELOAD="$WORK/egress_guard.so" "$BIN" "$@"
