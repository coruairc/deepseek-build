# scripts/evals — DeepSeek tuning eval harness (Phase E)

Small, self-contained coding tasks with **automatic pass/fail**, run against a
`deepseek-build` binary, recording pass/fail, wall time, tokens, cache hit rate,
reasoning tokens, and the locally estimated cost per task.

Nothing here talks to the network itself; it only launches the binary, which
talks to the configured provider.

## Layout

```
scripts/evals/
  harness.py       # run tasks against a model/config, append JSONL records
  report.py        # aggregate JSONL records into a comparison table
  validate.py      # both-ways task validation (pristine fails, reference fix passes)
  run-baseline.sh  # E2 baseline: both models, N runs per task
  prompts/         # candidate system prompts for E3a
  tasks/*.json     # 21 task definitions (fixture files, prompt, checks)
  results/*.jsonl  # one JSON line per run (append-only, gitignored)
```

## Quick start

```sh
# The key is read from the environment only; it is never printed or recorded.
export DEEPSEEK_API_KEY=...

# Offline validation of every fixture (no model calls):
python3 scripts/evals/harness.py --dry-run

# One task, one model:
python3 scripts/evals/harness.py --tasks py-fix-add --model deepseek-flash

# The baseline (spec E2): all tasks, both models, 3 runs each:
python3 scripts/evals/harness.py --model deepseek-v4-pro --config baseline --runs 3
python3 scripts/evals/harness.py --model deepseek-flash   --config baseline --runs 3

# Comparison table (add --by-task for detail):
python3 scripts/evals/report.py
```

A run gets a fresh scratch repo (fixture files, optional `git init`) and a fresh
`$HOME`, so runs never contaminate each other or your real config. Scratch dirs
are removed unless `--keep-work` is passed.

## Experiment discipline (spec E3)

Change **one variable at a time**; keep a change only if pass rate or cost
improves. Every experiment — including its failures — is recorded in
`docs/TUNING.md` with its measured delta. Candidate variables:

- a shorter DeepSeek-specific system prompt and tool descriptions
- file edit format
- reasoning effort policy and Flash→Pro escalation
- a verification loop after edits

The prompt itself lives in the binary (`crates/codegen/xai-grok-agent/templates/prompt.md`),
so prompt experiments mean rebuilding:

```sh
# edit the template, then regenerate the embedded ciphertext
python3 scripts/encrypt_templates.py
cargo build --release -p xai-grok-pager-bin
BIN=target/release/deepseek-build python3 scripts/evals/harness.py --model deepseek-v4-pro --config short-prompt --runs 3
```

`--config-file` copies a TOML file into the scratch `$HOME/.deepseek-build/config.toml`,
which is how to vary reasoning effort, status-line pricing overrides, and so on.
`--extra-args` passes further binary flags (e.g. `--effort low`).
`--system-prompt-file` replaces the whole system prompt via the binary's
`--system-prompt-override`, which is the no-rebuild path for the E3a prompt
experiments (the tool definitions are unchanged; only the prompt text varies).

## Task format

```json
{
  "id": "py-fix-add",
  "summary": "one line",
  "prompt": "what to tell the agent",
  "files": { "calc.py": "…", "test_calc.py": "…" },
  "init_git": true,
  "timeout_s": 420,
  "max_turns": 15,
  "fails_before": true,
  "checks": [
    {"type": "command", "cmd": ["python3", "-m", "unittest", "test_calc", "-v"]},
    {"type": "file_contains", "path": "calc.py", "text": "return a - b"}
  ]
}
```

Check types: `command` (exit code, optional `expect_stdout_contains`),
`file_contains`, `file_not_contains`, `file_absent`, `file_exists`.

Add a task by dropping a file in `tasks/`; then verify it both ways with the
maintainer tool:

```sh
python3 scripts/evals/validate.py            # both-ways check, no model calls
```

It requires that the pristine fixture fails (`fails_before: true`) and that a
registered reference fix in `validate.py` makes every check pass. A new task
needs an entry in that file's `PATCHES` before it can be trusted.

## Notes

- `estimated_cost_usd` in the binary's `--output-format json` result is a local
  peak-rate estimate; the harness falls back to a provider-reported
  `total_cost_usd` when the estimate is absent.
- Runs record the binary path, extra args, and exit code, so mixed experiments
  remain distinguishable in `results/`.
- The API key is scrubbed from recorded errors; the harness never writes the
  key to disk.
