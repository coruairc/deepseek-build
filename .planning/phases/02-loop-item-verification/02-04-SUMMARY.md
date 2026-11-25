---
phase: 02-loop-item-verification
plan: 04
subsystem: permissions
tags: [permission-prompt, headless, pty, security-warning]

requires:
  - phase: 02-loop-item-verification
    provides: UI-SPEC copy for the warning row and both denial lines
provides:
  - PermissionViewState.warning painted from the wire title's first line
  - Shared PERMISSION_DENIAL_FLAGS sentence for both headless denial lines
  - permission-prompt evidence item (PERM-01, PERM-02)
affects: [02-05, 02-06]

actuals:
  tokens: 22236
  tasks: 3
  commits: 5

tech-stack:
  added: []
  patterns:
    - "Security warning is carried by prompt state, not re-derived from the command"
    - "Both headless denial lines are composed from one flags constant"

key-files:
  created:
    - crates/codegen/xai-grok-pager-bin/tests/headless_loop_items.rs
    - crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/permission_prompt_outcomes.rs
    - docs/evidence/02-loop-item-verification/permission-prompt/report.json
  modified:
    - crates/codegen/xai-grok-pager/src/app/acp_handler/permissions.rs
    - crates/codegen/xai-grok-pager/src/views/permission_view.rs
    - crates/codegen/xai-grok-pager/src/headless.rs

key-decisions:
  - "The warning row is the wire title's first line when it starts with the shell prefix; later lines are ignored."
  - "A blanket Bash(*) ask skips the session-grant short-circuit, so the pty grant proof uses ask mode without that rule."
  - "scripts/test-gate.expected was not hand-edited after --refresh-expected refused on unrelated red tests."

patterns-established:
  - "security_warning_line is the only seam from the shell warning to the view"

requirements-completed: [PERM-01, PERM-02]

coverage:
  - id: D1
    description: "High-signal security warning paints above the permission question and chrome height grows by one"
    requirement: PERM-01
    verification:
      - kind: unit
        ref: "xai-grok-pager permission_view::tests::security_warning_paints_above_the_question_and_grows_chrome_by_one"
        status: pass
      - kind: e2e
        ref: "pty permission_prompt_outcomes_and_warning"
        status: pass
    human_judgment: false
  - id: D2
    description: "Both headless denial lines name --always-approve and --permission-mode auto, exit 1, and keep stopReason cancelled"
    requirement: PERM-02
    verification:
      - kind: unit
        ref: "xai-grok-pager headless::tests::permission_cancel_hint_only_fires_for_unapproved_gated_cancel"
        status: pass
      - kind: integration
        ref: "xai-grok-pager-bin headless_loop_items::headless_permission_denial_names_both_flags"
        status: pass
    human_judgment: false

duration: 1h 42m
completed: 2026-11-25
status: complete
plan_head_before: 6e9d8d88dca11bda39afea8d3b82a8f80111d207
plan_head_after: 30388a5170daf3518ff17e8155628940c71f31eb
---

# Phase 02 Plan 04: Permission prompt warning and headless denial Summary

The permission prompt now paints the shell's high-signal warning as its own row, and both headless denial lines name `--always-approve` and `--permission-mode auto` from one constant.

## Performance

- **Duration:** 1h 42m
- **Started:** 2026-11-25T19:39:19Z
- **Completed:** 2026-11-25T21:21:34Z
- **Tasks:** 3
- **Files changed:** pager permission seam, headless denial copy, pty scenario, evidence item

## Accomplishments

- `security_warning_line` (`permissions.rs:195`) keeps only a first line that starts with `⚠ Security warning: `. `enqueue_permission` stores that string on `PermissionViewState.warning` (`permission_view.rs:171`) before `build_permission_display` rebuilds the execute title.
- The view paints the warning in `theme.warning` at normal weight above the bold question, and `permission_chrome_height` grows by exactly one when it is present.
- `PERMISSION_DENIAL_FLAGS` (`headless.rs:478`) composes both denial strings. Hint: `Turn ended: a permission-gated tool was cancelled because headless mode cannot prompt for approval. Re-run with --always-approve, or choose a non-interactive permission mode such as --permission-mode auto.` Error: `Error: permission-gated tool cancelled; re-run with --always-approve, or choose a non-interactive permission mode such as --permission-mode auto.`
- Pty scenario `permission_prompt_outcomes_and_warning` drove allow-once (`rm -rf build`), always-allow plus a no-reprompt repeat (`rm -rf cache`), and reject (`bash -c 'echo hi'`) on one session. Labels asserted: `Yes, proceed`, `Always allow:`, `No, reject (type to add feedback)`, and `Security warning:`.
- Evidence item `permission-prompt` is finished (`result: ok`, requirements PERM-01 and PERM-02).

## Task Commits

1. **Task 1: warning row** — `b5a75b78` (feat)
2. **Task 2: denial copy** — `4599b561` (fix)
3. **Task 3: pty proof and evidence** — `30388a51` (test)

`git rev-list --count 6e9d8d88dca11bda39afea8d3b82a8f80111d207..HEAD` is 5 because `ec878ef0` and `1d13f0c3` (plan 02-01) landed on `main` between this plan's commits. Those two are not this plan's work.

## Files Created/Modified

- `crates/codegen/xai-grok-pager/src/app/acp_handler/permissions.rs` — splitter and enqueue wiring
- `crates/codegen/xai-grok-pager/src/views/permission_view.rs` — warning field, row, chrome height
- `crates/codegen/xai-grok-pager/src/headless.rs` — shared flags sentence
- `crates/codegen/xai-grok-pager-bin/tests/headless_loop_items.rs` — both output paths plus the auto probe
- `crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/permission_prompt_outcomes.rs` — three outcomes
- `docs/evidence/02-loop-item-verification/permission-prompt/` — finished item

## Decisions Made

- First-line-only warning extraction. A prefix on a later line does not invent a row.
- Pty persistent-grant proof does not use `Bash(*)` ask. That rule is `policy_forced_prompt` and skips the session-grant short-circuit, so the identical call prompted again. Ask mode plus `remember_tool_approvals = true` and `{"permissions":{}}` made the first call prompt and the saved exact grant suppress the repeat. Headless denial still uses `{"permissions":{"ask":["Bash(*)"]}}` plus `--trust`.
- Commands are single argv sequences. A chained `&&` hides Always allow because that grant cannot cover the whole script.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Mock conversation numbers**
- **Found during:** Task 2
- **Issue:** One `Conversation::nth(1)` script is consumed by the first headless process. The JSON invocation then got the unscripted echo and exited 0.
- **Fix:** Script conversations 1, 2, and 3. Call `finish_scripts()` and accept `Unfinished { replies_served: 0 }` because cancellation returns before the scripted reply.
- **Files modified:** `crates/codegen/xai-grok-pager-bin/tests/headless_loop_items.rs`
- **Commit:** `4599b561`

**2. [Rule 3 - Blocking] Always-allow row and session grant**
- **Found during:** Task 3
- **Issue:** `rm -rf build && printf ...` hid Always allow. `Bash(*)` ask then forced a prompt on the identical repeat.
- **Fix:** Bare `rm -rf build` / `rm -rf cache`, sentinel `exit: 0` plus the directory disappearing. Ask mode without a blanket ask rule for the pty grant proof.
- **Files modified:** `crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/permission_prompt_outcomes.rs`
- **Commit:** `30388a51`

### Gate refresh refused

`scripts/test-gate.sh --refresh-expected` exited 1 and wrote nothing. Step 2 (`config-docs`) passed. Step 1 failed only in pre-existing `xai-grok-agent` tests this plan does not touch:

- `builder::tests::hiding_task_drops_lifecycle_tools_when_background_shell_is_off`
- `builder::tests::task_tool_tracks_whether_general_purpose_is_spawnable`

The gate still derived `results=309`, `binaries=220`, `doctests=89` (`results == binaries + doctests`). `scripts/test-gate.expected` remains `results=308`, `binaries=219`, `doctests=89`. It was not hand-edited.

`cargo clippy -p xai-grok-pager --all-targets -- -D clippy::indexing_slicing` also fails on pre-existing example and dependency indexing. `cargo clippy -p xai-grok-pager --lib --tests --no-deps` reported 0 errors on the lib (pre-existing warnings only). `cargo fmt --all --check` was clean.

## Auto-mode probe

Printed, not asserted: `auto-mode probe: terminated=true json_parsed=true denial_hint=true tool_ran=false exit=Some(1)`. On this host `--permission-mode auto` still cancelled the gated `rm -rf build` call. PERM-02's requirement is the denial copy on both paths; the auto-mode behaviour is recorded as observed, not claimed.

## Evidence

| Step | ok | Artifact sha256 |
|------|----|-----------------|
| perm-01: three outcomes and the warning row on a real screen | true | cast `00e30d48518d5ccee44c17f11aaeebd623c160a700c5fd53cce71e8cfaa1a1fa`; log `73147e2e386c3a47c6ea1ad8f7929edd730d5aeba65dd32e9df0768c6074f0c1` |
| perm-02: both denial paths name both flags | true | `40992b73e0f5d955dbc7522cd12f489325f08e162caaef687e8dd6af685e060a` |
| perm-02: auto-mode probe line recorded | true | `ee975175c4b02cc034e5fccc016841d8592f652bb9fe04f387bdd14b153faa5a` |

`PAGER_BINARY` was `/tmp/opencode/02-04/deepseek-build`. `TESTING.md`, `permission_cursor.rs`, `prompter.rs`, `26-config-reference.md`, and `KNOWN-ISSUES.md` were not touched. 02-01 was not marked complete.

## Authentication Gates

None. `DEEPSEEK_API_KEY` and `DEEPSEEK_BUILD_API_KEY` were unset for the tests. No call to `api.deepseek.com`.

## Known Stubs

None.

## Self-Check: PASSED

- FOUND: `crates/codegen/xai-grok-pager/src/app/acp_handler/permissions.rs`
- FOUND: `crates/codegen/xai-grok-pager/src/views/permission_view.rs`
- FOUND: `crates/codegen/xai-grok-pager/src/headless.rs`
- FOUND: `crates/codegen/xai-grok-pager-bin/tests/headless_loop_items.rs`
- FOUND: `crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/permission_prompt_outcomes.rs`
- FOUND: `docs/evidence/02-loop-item-verification/permission-prompt/report.json`
- FOUND: `b5a75b78`
- FOUND: `4599b561`
- FOUND: `30388a51`
