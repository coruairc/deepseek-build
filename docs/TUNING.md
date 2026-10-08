# TUNING — measured DeepSeek tuning log (Phase E)

Every experiment on the shipped agent configuration is recorded here, including
the ones that did not work. **No improvement is claimed without a measured
delta** against the baseline, and a change only becomes a default if it clearly
wins.

Method: `scripts/evals/harness.py` runs 21 small coding tasks with automatic
pass/fail over fresh scratch repos and a fresh `$HOME`. One JSON line per run is
written under `scripts/evals/results/`; `scripts/evals/report.py` aggregates
them. Change **one variable at a time**, 3 runs per task.

## Baseline (spec E2)

Reference point: current shipped prompt and defaults, `--no-plan`
`--no-subagents --disable-web-search`, `--max-turns 15`, default reasoning
effort (high for v4 Pro).

| configuration | model | runs | pass | pass rate | wall/task | tokens in | cache read | hit rate | out | reasoning | est. USD/task |
|---|---|---|---|---|---|---|---|---|---|---|---|
| baseline | deepseek-v4-pro | 63 | 63 | 100% | 9.9s | 1596 | 52773 | 97% | 654 | 75 | 0.00702 |
| baseline | deepseek-flash | 63 | 63 | 100% | 7.4s | 1747 | 53079 | 97% | 579 | 59 | 0.00154 |

Measured 2026-10-08 with `scripts/evals/run-baseline.sh` (21 tasks × 3 runs
each, binary `deepseek-build 1.0.45`, `dsb/integration @ 9113b219`).

Findings:

- **Both models pass every task**, so pass rate is at its ceiling. The lever
  for E3 is cost and wall time, not correctness.
- **Pro costs 4.6× Flash** (`$0.00702` vs `$0.00154` per task) for the same pass
  rate on this task set. Flash is 25% faster per task.
- The cache-hit rate is ~97% in both cases: the inherited system prompt and
  tool definitions dominate the prompt (53k cache-read tokens per task against
  ~1.7k fresh). A shorter prompt directly reduces the cache-read bucket.
- Reasoning tokens are modest (75 Pro / 59 Flash per task averaged across
  turns).

## Experiment log

Each entry: date, hypothesis, the single variable changed, command, results,
and the measured delta vs baseline. Keep failures.

### E3a — shorter DeepSeek-specific system prompt / tool descriptions

Candidate prompt: `scripts/evals/prompts/short-deepseek.md`, passed with
`harness.py --system-prompt-file`. It replaces the prompt text only, not the
tool definitions, so this experiment measures the prompt change alone.

**Result (2026-10-08): rejected — no win.** 63 runs vs the 63-run Pro baseline:

| configuration | pass | wall/task | tokens in | cache read | out | reasoning | turns | est. USD/task |
|---|---|---|---|---|---|---|---|---|
| baseline | 63/63 | 9.9s | 1596 | 52773 | 654 | 75 | 4.92 | 0.00702 |
| short-prompt | 63/63 | 10.2s | 1829 | 48488 | 724 | 78 | 5.40 | 0.00741 |

Measured delta: **+6% cost** ($+0.00040/task), +0.3s/task, 14 of 21 tasks more
expensive. The shorter prompt did shrink the cached prefix (52,773 → 48,488
cache-read tokens, −8%), but fresh input grew 15%, output grew 11%, and turns
rose 4.92 → 5.40, so the net is a loss. Reproduce:
`python3 scripts/evals/experiment.py --name short-prompt --model deepseek-v4-pro --system-prompt-file scripts/evals/prompts/short-deepseek.md`.

Secondary probe: the built-in concise toolset (`--agent grok-build-concise`,
shorter tool descriptions) was **worse** on a 1-run probe (`$0.01343` vs
`$0.00702`), driven by a much larger fresh-input bucket (8859 vs 1596) — not
promoted to a full run.

### E3b — file edit format

_Not started._

### E3c — reasoning effort policy and Flash→Pro escalation

_Not started._

### E3d — verification loop after edits

_Not started._

## Decisions

_Filled in after the first experiment batch, per the Phase E gate: adopt a
configuration as the default only if it clearly wins; otherwise leave the
shipped defaults and record the negative result here._

## Reproducing

```sh
export DEEPSEEK_API_KEY=...
scripts/evals/run-baseline.sh                       # E2 baseline (both models, 3 runs)
python3 scripts/evals/experiment.py --name <variant> --model <model> \
    [--extra-args "…" | --system-prompt-file … | --config-file …]
python3 scripts/evals/report.py --compare baseline  # measured deltas
```

Each run's record carries the binary hash, config/prompt hashes, and token and
cost metrics, so a variant measured against a different build stays
distinguishable. Every task fixture is validated both ways (fails pristine,
passes with a reference fix) by `scripts/evals/validate.py`.
