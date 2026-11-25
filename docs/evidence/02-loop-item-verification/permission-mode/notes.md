# `permission-mode` — PERM-03 and PERM-04

## What this item proves

- **PERM-03:** the permission chip is visible and matches after a resume. A session launched with
  `[ui] permission_mode = "always-approve"` in the sandbox's own `config.toml` and **no**
  `--permission-mode` flag shows the `always-approve` chip; resuming that exact session with
  `-r <sessionId>` and still no flag shows the same chip again.
- **PERM-04:** `--permission-mode plan` really arms plan mode in a headless run. The captured
  inference request body carries the shell's stable reminder sentence
  `Plan mode is active. Do not make any edits or writes to the system.` and the session's own
  `plan_mode.json` reads `state: "Active"`.

## Execution

Produced on the build host from the release binary at `/tmp/opencode/02-06/deepseek-build`
(`deepseek-build 0.0.1-rc5 (4599b561193a)`, built from the tree that includes 02-04/02-05).
No live model key and no network: the pty scenario and the headless run both talk to the
loopback mock provider.

Re-run commands (both were the commands `evidence.py step` executed, in this order):

```sh
# perm-03 (pty). GROK_PTY_CAST_DIR must be ABSOLUTE: the test binary's cwd is
# crates/codegen/xai-grok-pager-pty-harness, so a repo-relative path lands beside the crate.
PAGER_BINARY=/tmp/opencode/02-06/deepseek-build \
GROK_PTY_CAST_DIR="$PWD/docs/evidence/02-loop-item-verification/permission-mode" \
flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-pager-pty-harness \
  --test pty_e2e_shell_tools -- --ignored --nocapture permission_mode_chip_matches_after_resume

# perm-04 (headless, loopback mock)
flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-pager-bin \
  --test headless_loop_items -- --nocapture headless_permission_mode_plan_arms_plan_mode
```

## PERM-03 — how the resume target was resolved

Both launches pass `--trust --no-leader` and **neither** passes `--permission-mode`, so the
chip can only come from `config.toml`; a chip that reappears because a flag was repeated is
the false proof research Pitfall 4 names.

The test quits the first pager with the same two Ctrl-Q presses the plan-approval scenario
uses, then lists `<grok home>/sessions/*/*`. A session directory is named by its session id
(`session/storage/relocation/mod.rs` keys candidates by the directory name), so the single
directory name **is** the id. More than one candidate fails the test with the listing instead
of guessing; `--continue` is deliberately not used, because it would pick the most recent
session rather than the one the test started.

Observed run (from the step log):

```
PERM-03 observation: plan banner=false plan+permission pair=true
PERM-03: resuming session by id 01a214df-a338-7fe0-b015-83dac1e86b75 with no permission flag
PERM-03: chip before quit = 'always-approve', after -r 01a214df-a338-7fe0-b015-83dac1e86b75 = 'always-approve';
         transcript_restored=true; plan chip survived=true
```

- Chip before quit: `always-approve` (welcome frame, agent frame, and the last pre-quit frame).
- Chip after `-r <sessionId>`: `always-approve`, on the frame that also replays the first turn's
  transcript (`transcript_restored=true` proves the resumed process is that session).
- `/plan` was sent before the quit purely as an observation. The `Switched to mode: Plan` banner
  was not observed (`plan banner=false`), but the `plan · always-approve` pair did render
  (`plan+permission pair=true`), and it **did** survive the resume (`plan chip survived=true`) —
  the shell persists plan state in `plan_mode.json` and the resumed pager re-arms the chip from
  it. That is recorded here as an observation only; PERM-03's claim is the permission chip,
  which is config-resolved on both launches.

## PERM-04 — what was matched

The headless run was
`deepseek-build -p "plan the change" --permission-mode plan --trust --output-format json`
against the loopback mock. Exit status 0, `stopReason: "end_turn"`. In the last foreground
`/chat/completions` body the matched reminder line is:

```
Plan mode is active. Do not make any edits or writes to the system.

## Plan File:
No plan written yet. Write your plan to <session>/plan.md using the search_replace tool.
...
```

`<grok home>/sessions/<cwd>/<session id>/plan_mode.json` existed for the run and read
`state: "Active"`, so the plan edit gate was armed in the same session — not only the reminder.
No product code was changed for this: `apply_headless_permission_mode` already sent the single
`session/set_mode` request before the prompt, and no second request was added. `TESTING.md` is
untouched; the stale `KNOWN-ISSUES.md` bullet was corrected instead of the code.

Nothing in this item was NOT RUN. `scripts/key-leak-check.sh --scan` over the item exits 0
(`PASS  no key-shaped string`).

## Artifacts

| File | What it is |
|------|------------|
| `permission-mode.cast` | the **resumed** pager: the matching `always-approve` chip on the restored frame |
| `permission-mode-before-quit.cast` | the **pre-quit** pager, same chip, so both sides of the comparison are in the item |
| `perm-03-permission-chip-matches-after-resume-by-session-id.log` | recorded step output (real exit status) |
| `perm-04-headless-permission-mode-plan-arms-plan-mode.log` | recorded step output (real exit status) |

Both casts and both logs were normalized (`scripts/evidence.py normalize --path`, once per file)
before they were hashed. `evidence.py` substitutes the repository root, the user home and the
system temp directory; the pager's own header abbreviates the sandbox workspace it displays
(`/t/.tmpXXXX/workspace`), which is a display string, not a real path — no real absolute sandbox
path survives in the casts and no username appears in any artifact.
