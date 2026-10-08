#!/usr/bin/env bash
#
# Egress policy guard for deepseek-build.
#
# Allowed network destinations (final allowlist):
#   1. the configured model provider (default https://api.deepseek.com)
#   2. user-configured MCP servers
#   3. loopback
#
# HARD gate: legacy xAI / telemetry / upload destinations. Must be zero
#   everywhere except attribution text (LICENSE / NOTICE / THIRD-PARTY-*).
# SOFT report: the ACP `x.ai/*` namespace and `grok` branding. Cleared by the
#   rebrand slice; pass `--strict` to make it fatal (used after rebrand).
#
# Usage: scripts/check-egress.sh [--strict]
#
# This is a bash script (see the shebang); the pipefail option, arrays, and
# `local`/array references below are intentional. The repo also lints scripts
# with `shellcheck --shell=sh` for consistency, so declare them here.
# shellcheck disable=SC3040,SC3030,SC3043,SC3054
set -euo pipefail

cd "$(dirname "$0")/.."
strict="${1:-}"

# Hosts/DSNs that must never appear.
HARD='api\.mixpanel\.com|sentry\.io|sentry\.dev|SENTRY_DSN|sentry::|auth\.x\.ai|accounts\.x\.ai|x\.ai/cli|code\.grok\.com|cli-chat-proxy|api\.x\.ai|grok\.com|gs://|s3://|/v1/storage'
# Branding / ACP namespace.
SOFT='x\.ai/|Grok Build|SpaceXAI|SuperGrok|grok-[0-9]'

# Attribution and docs are allowed to name the upstream project.
EXCLUDES=(-g '!**/THIRD_PARTY*' -g '!**/THIRD-PARTY*' -g '!**/NOTICE*' -g '!**/LICENSE*' -g '!**/*.md')

scan() {
  local pattern="$1"
  if command -v rg >/dev/null 2>&1; then
    rg --no-heading -n -i -e "$pattern" crates prod third_party "${EXCLUDES[@]}" 2>/dev/null || true
  else
    grep -rIniE "$pattern" crates prod third_party 2>/dev/null \
      | grep -vE 'THIRD_PARTY|THIRD-PARTY|NOTICE|LICENSE|\.md:' || true
  fi
}

hard_hits="$(scan "$HARD")"
if [ -n "$hard_hits" ]; then
  echo "HARD EGRESS VIOLATION: forbidden destination/host strings found:" >&2
  printf '%s\n' "$hard_hits" | awk 'NR<=200 { print }' >&2
  echo >&2
  total=$(printf '%s\n' "$hard_hits" | wc -l)
  echo "($total hard hits total; allowed: api.deepseek.com + user MCP + loopback)" >&2
  exit 1
fi
echo "HARD egress gate: OK"

soft_hits="$(scan "$SOFT")"
if [ -n "$soft_hits" ]; then
  total=$(printf '%s\n' "$soft_hits" | wc -l)
  echo "SOFT report: $total branding/ACP-namespace hits remain (must be 0 after rebrand)." >&2
  if [ "$strict" = "--strict" ]; then
    printf '%s\n' "$soft_hits" | awk 'NR<=100 { print }' >&2
    exit 1
  fi
fi
