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
| _pending live baseline_ | deepseek-v4-pro | | | | | | | | | | |
| _pending live baseline_ | deepseek-flash | | | | | | | | | | |

Notes:

- A trivial prompt costs ~14k input tokens because of the inherited system
  prompt and tool definitions; the harness reports prompt-side tokens per task.
- Cache hit rate is high after the first run in a session; each eval run is a
  fresh session, so the numbers reflect cold-then-warm prefix behavior.

## Experiment log

Each entry: date, hypothesis, the single variable changed, command, results,
and the measured delta vs baseline. Keep failures.

### E3a — shorter DeepSeek-specific system prompt / tool descriptions

Candidate prompt: `scripts/evals/prompts/short-deepseek.md`, passed with
`harness.py --system-prompt-file`. It replaces the prompt text only, not the
tool definitions, so this experiment measures the prompt change alone.

_Not started._

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
