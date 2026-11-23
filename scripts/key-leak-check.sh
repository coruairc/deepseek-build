#!/usr/bin/env bash
#
# key-leak-check.sh — verify the API key never lands in logs, sessions, or error
# output. Runs the binary headlessly with a FAKE key in an isolated HOME, then
# scans every file written under that HOME plus the captured stdout/stderr.
#
# Usage:
#   scripts/key-leak-check.sh
#   BIN=target/debug/deepseek-build scripts/key-leak-check.sh
#   scripts/key-leak-check.sh --scan docs/evidence [path...]
#
# `--scan <path>...` is an additive mode for committed evidence: it skips the
# release-binary precheck and the live run, scans the given files/directories
# for the canary and for any `sk-` key shape, prints file names only (never
# matched lines), and exits 1 on a hit. With no arguments the script behaves
# exactly as before.
#
# This is a bash script (see the shebang); pipefail is intentional. The repo
# also lints scripts with `shellcheck --shell=sh` for consistency, so declare
# that here.
# shellcheck disable=SC3040
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BIN:-$REPO_ROOT/target/release/deepseek-build}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
HOME_DIR="$WORK/home"; mkdir -p "$HOME_DIR"
FAKE_KEY="sk-FAKE-LEAKCANARY-0000000000000000000000000000"

scan=0
while [ $# -gt 0 ]; do
  case "$1" in
    --scan)
      scan=1
      shift
      break
      ;;
    -h|--help)
      echo "Usage: scripts/key-leak-check.sh [--scan <path>...]"
      exit 0
      ;;
    *)
      echo "FAIL  unknown argument: $1"
      exit 2
      ;;
  esac
done

if [ "$scan" -eq 1 ]; then
  if [ $# -eq 0 ]; then
    echo "FAIL  --scan requires at least one path"
    exit 2
  fi
  for path in "$@"; do
    if [ ! -e "$path" ]; then
      echo "FAIL  path not found: $path"
      exit 2
    fi
  done
  # File names only: grep -l never prints the matched line, and -I skips
  # binaries (casts are text, but a future artifact may not be — see the
  # text-only convention in docs/evidence/README.md). grep exits 0 on a
  # match, 1 on no match, and 2 on error (unreadable file, I/O error,
  # recursive symlink loop); an error must fail closed, never pass as
  # "no match". The script runs without `set -e`, so capturing $? is safe.
  grep -RIlE -- "sk-FAKE-LEAKCANARY-[A-Za-z0-9-]+|sk-[A-Za-z0-9]{20,}" "$@" \
    >"$WORK/hits.txt" 2>"$WORK/scan-err.txt"
  grep_rc=$?
  if [ "$grep_rc" -eq 0 ]; then
    hits=$(wc -l <"$WORK/hits.txt")
    echo "FAIL  key-shaped string found in $hits file(s):"
    cat "$WORK/hits.txt"
    exit 1
  elif [ "$grep_rc" -ne 1 ]; then
    echo "FAIL  leak scan could not complete (grep rc=$grep_rc):"
    cat "$WORK/scan-err.txt"   # grep diagnostics only — never matched lines
    exit 1
  fi
  echo "PASS  no key-shaped string in: $*"
  exit 0
fi

if [ $# -gt 0 ]; then
  echo "FAIL  unexpected argument: $1"
  exit 2
fi

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
