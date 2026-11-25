# permission-prompt

The cast is the pty step recorded in this item (`perm-01: three outcomes and the warning row on a real screen`), driven by `PAGER_BINARY=/tmp/opencode/02-04/deepseek-build` (a private copy of `target/release/deepseek-build` at `4599b561`, not the shared cargo output). `GROK_PTY_CAST_DIR` was absolute, so the harness wrote `permission-prompt.cast` directly into this directory. A first attempt used a relative cast dir and cargo's package cwd; that stray file was removed and the recorded step above was re-run.

## Re-run

```sh
export CARGO_TARGET_DIR="$PWD/target"
mkdir -p /tmp/opencode/02-04
flock -w 900 /tmp/team-build.lock cargo build --release -p xai-grok-pager-bin
cp -f target/release/deepseek-build /tmp/opencode/02-04/deepseek-build
ITEM=docs/evidence/02-loop-item-verification/permission-prompt
env PAGER_BINARY=/tmp/opencode/02-04/deepseek-build GROK_PTY_CAST_DIR="$PWD/$ITEM" \
  flock -w 900 /tmp/team-build.lock \
  cargo test -p xai-grok-pager-pty-harness --test pty_e2e_shell_tools -- \
  --ignored --nocapture permission_prompt_outcomes_and_warning
flock -w 900 /tmp/team-build.lock \
  cargo test -p xai-grok-pager-bin --test headless_loop_items -- \
  --nocapture headless_permission_denial
```

Use an absolute `GROK_PTY_CAST_DIR` so the cast lands in the item directory.

## What fired the gate

Pty: `[ui] permission_mode = "ask"` and `remember_tool_approvals = true`, with `.claude/settings.json` `{"permissions":{}}`. A blanket `Bash(*)` ask is `policy_forced_prompt` and skips the session-grant short-circuit, so the identical `rm -rf cache` call prompted again. Ask mode alone made the first call prompt, and the saved exact grant suppressed the repeat. Commands are single argv sequences (`rm -rf build`, `rm -rf cache`); a chained `&&` hides the Always allow row.

Headless denial: the same sandbox, `.claude/settings.json` `{"permissions":{"ask":["Bash(*)"]}}`, and `--trust`. That combination made both the plain and JSON invocations exit 1.

## Auto mode

Observed line from the headless step log: `auto-mode probe: terminated=true json_parsed=true denial_hint=true tool_ran=false exit=Some(1)`. On this host, `--permission-mode auto` still cancelled the gated `rm -rf build` call and printed the denial hint. That outcome was printed, not asserted.

PERM-02's requirement is the denial copy on both paths; the auto-mode behaviour is recorded as observed, not claimed.
