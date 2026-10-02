#!/usr/bin/env bash
#
# Egress policy guard.
#
# The shipped product may only contact:
#   1. the configured model provider (default https://api.deepseek.com), and
#   2. user-configured MCP servers.
#
# This fails if legacy xAI / telemetry / upload destinations reappear in source.
# Attribution files (LICENSE, NOTICE, THIRD-PARTY-NOTICES, docs) are excluded
# because they legitimately name the upstream project.
#
# Usage: scripts/check-egress.sh
set -euo pipefail

cd "$(dirname "$0")/.."

PATTERN='gs://|s3://|mixpanel|sentry|x\.ai|grok\.com|storage\.googleapis\.com|cli-chat-proxy'

if command -v rg >/dev/null 2>&1; then
  hits=$(rg --no-heading -n -i -e "$PATTERN" crates prod third_party \
    -g '!**/THIRD_PARTY*' -g '!**/NOTICE*' -g '!**/LICENSE*' -g '!**/*.md' \
    2>/dev/null || true)
else
  hits=$(grep -rIniE "$PATTERN" crates prod third_party 2>/dev/null \
    | grep -vE 'THIRD_PARTY|NOTICE|LICENSE|\.md:' || true)
fi

if [ -n "$hits" ]; then
  echo "EGRESS POLICY VIOLATION: forbidden destination/host strings found:" >&2
  printf '%s\n' "$hits" | awk 'NR<=200 { print }' >&2
  echo >&2
  echo "Allowed destinations: the configured model provider (api.deepseek.com)" >&2
  echo "and user-configured MCP servers only." >&2
  exit 1
fi

echo "egress policy: OK"
