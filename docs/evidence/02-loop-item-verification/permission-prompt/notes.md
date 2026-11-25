# permission-prompt

The cast is the pty step recorded in this item (`perm-01: three outcomes and the warning row on a real screen`). `PAGER_BINARY` was `/tmp/opencode/02-04/deepseek-build`, a private copy of `target/release/deepseek-build` built after the warning-row commit `b5a75b78`, not the shared cargo output. `GROK_PTY_CAST_DIR` was the item directory; the harness wrote `permission-prompt.cast` there (`wrote cast:` in the perm-01 log). No second pty run produced the committed cast.

## Re-run

To re-run the recorded steps:

```sh
export CARGO_TARGET_DIR="$PWD/target"
mkdir -p /tmp/opencode/02-04
flock -w 900 /tmp/team-build.lock cargo build --release -p xai-grok-pager-bin
cp -f target/release/deepseek-build /tmp/opencode/02-04/deepseek-build
ITEM=docs/evidence/02-loop-item-verification/permission-prompt
env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY \
  PAGER_BINARY=/tmp/opencode/02-04/deepseek-build GROK_PTY_CAST_DIR="$PWD/$ITEM" \
  flock -w 900 /tmp/team-build.lock \
  cargo test -p xai-grok-pager-pty-harness --test pty_e2e_shell_tools -- \
  --ignored --nocapture permission_prompt_outcomes_and_warning
env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY \
  flock -w 900 /tmp/team-build.lock \
  cargo test -p xai-grok-pager-bin --test headless_loop_items -- \
  --nocapture headless_permission_denial
```

Use an absolute `GROK_PTY_CAST_DIR`. A relative one is resolved from the pty-harness package directory and misses this item.

## What fired the gate

Pty: `[ui] permission_mode = "ask"` and `remember_tool_approvals = true`, with `.claude/settings.json` `{"permissions":{}}`. A blanket `Bash(*)` ask is `policy_forced_prompt` and skips the session-grant short-circuit (`ASK_FLOOR_REMEMBER` drops exact grants), so the identical `rm -rf cache` call prompted again. Ask mode alone made the first call prompt, and the saved exact grant suppressed the repeat. Commands are single argv sequences (`rm -rf build`, `rm -rf cache`, `bash -c 'echo hi'`); a chained `&&` hides the Always allow row. Row numbers were parsed from the screen, not hard-coded.

Headless denial: one `TestSandbox`, `.claude/settings.json` `{"permissions":{"ask":["Bash(*)"]}}`, and `--trust`. That combination made both the plain and JSON invocations exit 1.

## Auto mode

Observed line from the headless step log: `auto-mode probe: terminated=true json_parsed=true denial_hint=true tool_ran=false exit=Some(1)`. On this host, `--permission-mode auto` still cancelled the gated `rm -rf build` call and printed the denial hint. That outcome was printed, not asserted.

PERM-02's requirement is the denial copy on both paths; the auto-mode behaviour is recorded as observed, not claimed.

## Gate count

`scripts/test-gate.sh --refresh-expected` exited 1 and wrote nothing. The only failures were pre-existing `xai-grok-agent` lib tests (`hiding_task_drops_lifecycle_tools_when_background_shell_is_off`, `task_tool_tracks_whether_general_purpose_is_spawnable`). `scripts/test-gate.expected` was not hand-edited. See `.planning/phases/02-loop-item-verification/deferred-items.md`.
