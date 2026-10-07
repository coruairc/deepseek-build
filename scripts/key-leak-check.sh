#!/usr/bin/env bash
#
# key-leak-check.sh — verify the API key never lands in logs, sessions, or error
# output. Runs the binary headlessly with a FAKE key in an isolated HOME, then
# scans every file written under that HOME plus the captured stdout/stderr.
#
# Usage:
#   scripts/key-leak-check.sh
#   BIN=target/debug/deepseek-build scripts/key-leak-check.sh
#
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BIN:-$REPO_ROOT/target/release/deepseek-build}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
HOME_DIR="$WORK/home"; mkdir -p "$HOME_DIR"
FAKE_KEY="sk-FAKE-LEAKCANARY-0000000000000000000000000000"

if [ ! -x "$BIN" ]; then
  echo "FAIL  binary not found at $BIN"
  exit 1
fi

# Run with the fake key against the real endpoint (gets a 401) with debug logging.
env HOME="$HOME_DIR" DEEPSEEK_API_KEY="$FAKE_KEY" \
  timeout 120 "$BIN" --debug-file "$HOME_DIR/debug.log" --always-approve \
  -p "reply with the single word: hi" \
  >"$WORK/stdout.txt" 2>"$WORK/stderr.txt"
rc=$?
echo "run exit: $rc (401/other error is expected with a fake key)"

hits=0
if grep -RIl -- "$FAKE_KEY" "$HOME_DIR" "$WORK/stdout.txt" "$WORK/stderr.txt" >"$WORK/hits.txt" 2>/dev/null; then
  hits=$(wc -l <"$WORK/hits.txt")
fi

if [ "$hits" -gt 0 ]; then
  echo "FAIL  fake key found in $hits file(s):"
  sed "s#$WORK#<work>#; s#$HOME_DIR#<home>#" "$WORK/hits.txt"
  # Do not print matched lines (they contain the canary); file names only.
  exit 1
fi

files=$(find "$HOME_DIR" -type f | wc -l)
echo "PASS  fake key absent from $files saved file(s) and captured output"
exit 0
