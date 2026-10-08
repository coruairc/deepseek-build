#!/usr/bin/env bash
#
# run-baseline.sh — Phase E2 baseline: every task, both models, 3 runs each.
#
# Requires a working DEEPSEEK_API_KEY (read from the environment; never printed).
# Results land in scripts/evals/results/ and a comparison table is printed at
# the end (also written to scripts/evals/results/baseline-report.md).
#
# Usage:
#   DEEPSEEK_API_KEY=... scripts/evals/run-baseline.sh
#   RUNS=1 scripts/evals/run-baseline.sh          # quick pass
#   BIN=target/debug/deepseek-build scripts/evals/run-baseline.sh
#
set -uo pipefail

cd "$(dirname "$0")/../.." || exit 1
BIN="${BIN:-target/release/deepseek-build}"
RUNS="${RUNS:-3}"
CONFIG="${CONFIG:-baseline}"
OUT_DIR="scripts/evals/results"

if [ -z "${DEEPSEEK_API_KEY:-${DEEPSEEK_BUILD_API_KEY:-}}" ]; then
  echo "run-baseline: DEEPSEEK_API_KEY (or DEEPSEEK_BUILD_API_KEY) is not set" >&2
  exit 1
fi
if [ ! -x "$BIN" ]; then
  echo "run-baseline: binary missing at $BIN (cargo build --release -p xai-grok-pager-bin)" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
rc=0
for model in deepseek-v4-pro deepseek-flash; do
  echo "== $model ($RUNS run(s) per task) =="
  python3 scripts/evals/harness.py \
    --bin "$BIN" \
    --model "$model" \
    --config "$CONFIG" \
    --runs "$RUNS" || rc=1
done

echo
echo "== comparison =="
python3 scripts/evals/report.py --by-task --out "$OUT_DIR/baseline-report.md"
exit "$rc"
