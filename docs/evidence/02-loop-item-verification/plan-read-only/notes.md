# `plan-read-only` — SESS-02

## What this item proves

Plan mode is read-only except the session plan file, in **every** permission mode:

- `default` (seeded `[ui] permission_mode = "ask"`),
- `auto` (`/auto` after plan mode is armed),
- `always-approve` (`/always-approve` after plan mode is armed).

In each mode the shell refused a scripted `search_replace` to `<workspace>/src/main.rs`, the file
was unchanged on disk, and the refusal named the plan file. An edit to exactly that plan path was
**not** refused and actually executed (the plan file on disk contains the replacement).

The property holds by construction: `plan_mode_edit_gate` takes no permission-policy argument and
runs before permission resolution, so no mode can reach an edit it refuses. The pure unit test
calls it directly with the same inputs production builds and with no permission manager in scope.

## Execution

Build host, release binary `/tmp/opencode/02-06/deepseek-build`
(`deepseek-build 0.0.1-rc5 (4599b561193a)`). No key, no network: every pager talks to the loopback
mock provider. `GROK_AUTO_PERMISSION_MODE=1` is pinned in the spawned pager's env so `/auto` is
offered regardless of the runner's shell (a test-only env override; no product change).

Re-run commands (the exact commands `evidence.py step` executed):

```sh
# pty, three modes. GROK_PTY_CAST_DIR must be ABSOLUTE: the test binary's cwd is its crate dir.
PAGER_BINARY=/tmp/opencode/02-06/deepseek-build \
GROK_PTY_CAST_DIR="$PWD/docs/evidence/02-loop-item-verification/plan-read-only" \
flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-pager-pty-harness \
  --test pty_e2e_shell_tools -- --ignored --nocapture plan_mode_read_only_in_every_permission_mode

# the gate's own signature
flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-shell --lib plan_mode_edit_gate
```

## Chip asserted per iteration (and why)

The seeded mode is `ask` in all three iterations (`config.toml`), and the UI-SPEC's mode-indicator
table pins `Ask` to **no** permission chip. So:

| iteration | permission setting | chips required |
|---|---|---|
| `default` | seeded `ask` | `plan` only; no `auto-review`, no `always-approve` |
| `auto` | `/auto` | `plan · auto-review` (two chips) |
| `always-approve` | `/always-approve` | `plan · always-approve` (two chips) |

The `plan` chip is asserted in every iteration. The seeded mode was not changed to manufacture a
permission chip in the `default` iteration; the absence is asserted instead.

## Observed refusal, per mode

The pager finishes a failed edit block **collapsed** (`EditToolCallBlock::finished_display_mode`),
so the test expands the row with a real SGR double-click before asserting the screen. The refusal
text on screen is:

```
Rejected: file edits are not allowed in plan mode - the only editable file is the plan file (<plan path>).
```

Plan paths parsed out of that same refusal (sandbox temp paths; `evidence.py` normalizes the repo,
home and system temp prefixes):

- `default`: `/tmp/.tmp0G1FS7/home/.grok/sessions/%2Ftmp%2F.tmp0G1FS7%2Fworkspace/01a214e2-3120-7da3-a2bb-d969cc89ec74/plan.md`
- `auto`: `/tmp/.tmpopIbo4/home/.grok/sessions/%2Ftmp%2F.tmpopIbo4%2Fworkspace/01a214e2-4e48-7840-990b-938c41d51245/plan.md`
- `always-approve`: `/tmp/.tmp9h9MQk/home/.grok/sessions/%2Ftmp%2F.tmp9h9MQk%2Fworkspace/01a214e2-6c45-7ff1-9382-19b21f639820/plan.md`

Each iteration then wrote a marker into that exact file and edited it with a second scripted
`search_replace`; the file on disk afterwards contains `SESS02MARKER edited`, so "not refused" is
proved by an executed edit, not by a missing string.

## Artifacts

| File | What it is |
|------|------------|
| `plan-read-only-default.cast` | the `ask` iteration: `plan` chip, refusal, plan-file edit |
| `plan-read-only-auto.cast` | the `auto` iteration: `plan · auto-review`, refusal, plan-file edit |
| `plan-read-only-always-approve.cast` | the `always-approve` iteration: `plan · always-approve`, refusal, plan-file edit |
| `sess-02-plan-mode-refuses-out-of-plan-edits-in-every-permission-mode.log` | recorded pty step output (real exit status) |
| `sess-02-the-gate-takes-no-permission-policy.log` | recorded unit step output (11 passed, 0 failed) |

All five were normalized before they were hashed. `scripts/key-leak-check.sh --scan` over the item
exits 0 (`PASS  no key-shaped string`). Nothing here was NOT RUN.
