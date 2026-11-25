---
phase: 02-loop-item-verification
plan: 05
subsystem: testing
tags: [resume, plan-mode, reasoning-effort, headless, pty, evidence, provenance]

# Dependency graph
requires:
  - phase: 02-loop-item-verification
    provides: "02-04's headless_loop_items target (mock server + borrowed sandbox pattern) and the release binary"
provides:
  - "headless_resume_restores_model_effort_and_plan_state: one session, three invocations, model/effort/plan-state proven from request bodies"
  - "plan_approval_resume scenario: model compared across the quit, SESS-03 named in every failure, cast emitted via GROK_PTY_CAST_DIR"
  - "docs/evidence/02-loop-item-verification/resume-fidelity (SESS-01, SESS-03, SESS-04)"
  - "SESS-04 provenance paragraph in the permissions user guide"
affects: [02-06-plan-read-only, phase-5-docs]

actuals:
  tokens: 20027
  tasks: 2
  commits: 2

tech-stack:
  added: []
  patterns:
    - "Resume proofs read captured request bodies, never log lines: model and effort are compared for equality with the pre-quit turn"
    - "A pty scenario writes its cast itself when GROK_PTY_CAST_DIR is set (absolute path required)"
    - "Provenance table in the evidence item plus a matching user-guide paragraph"

key-files:
  created:
    - docs/evidence/02-loop-item-verification/resume-fidelity/report.json
    - docs/evidence/02-loop-item-verification/resume-fidelity/notes.md
    - docs/evidence/02-loop-item-verification/resume-fidelity/resume-fidelity.cast
  modified:
    - crates/codegen/xai-grok-pager-bin/tests/headless_loop_items.rs
    - crates/codegen/xai-grok-pager-pty-harness/src/scenarios/plan_approval_resume.rs
    - crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md
    - crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/permission_prompt_outcomes.rs

key-decisions:
  - "The headless test declares [model.\"deepseek-v4-pro\"] in the sandbox config.toml: this tree's catalog is local (is_models_fetch_enabled() == false) and a custom models endpoint skips the built-in entries, so the mock's /v1/models is served but never consumed"
  - "The resumed plan reminder is asserted from the tail of the request: the pre-quit activation reminder stays in history and cannot satisfy a tail-only check"
  - "The pty model comparison uses summary.json plus the resumed request body; the info row renders `unknown` in this mock and cannot distinguish models"

patterns-established:
  - "One TestSandbox + run_headless_in_sandbox_borrowed for every invocation of a resume test (run_headless builds a fresh HOME per call)"
  - "Evidence step sets GROK_PTY_CAST_DIR to an absolute path so the cast lands in the committed item, not the pty-harness package dir"

requirements-completed: [SESS-01, SESS-03, SESS-04]

coverage:
  - id: D1
    description: "A headless session resumed without -m/--effort keeps the model and reasoning_effort, keeps plan state, and an explicit --effort low on a later resume wins"
    requirement: SESS-01
    verification:
      - kind: integration
        ref: "crates/codegen/xai-grok-pager-bin/tests/headless_loop_items.rs#headless_resume_restores_model_effort_and_plan_state"
        status: pass
    human_judgment: false
  - id: D2
    description: "A drafted plan survives quit and resume: plan body and approval chrome on screen after --continue, same model before and after the quit, cast emitted"
    requirement: SESS-03
    verification:
      - kind: e2e
        ref: "crates/codegen/xai-grok-pager-pty-harness/tests/plan_approval_resume.rs#plan_approval_restored_after_resume"
        status: pass
    human_judgment: false
  - id: D3
    description: "Committed resume-fidelity evidence item with both steps ok, the cast and both logs hashed"
    requirement: SESS-03
    verification:
      - kind: other
        ref: "docs/evidence/02-loop-item-verification/resume-fidelity/report.json"
        status: pass
    human_judgment: false
  - id: D4
    description: "One provenance paragraph in 22-permissions-and-safety.md: permission mode is config/CLI on every launch; plan state, plan file, model and effort come from the session; -m/--effort on resume win"
    requirement: SESS-04
    verification: []
    human_judgment: true
    rationale: "Prose accuracy: the grep gate only proves the sentence exists; a reader must judge whether each setting is attributed to the right source."

duration: 35 min
completed: 2026-11-25
status: complete
plan_head_before: 4dded936878f74bf814cf328d0559ba3c84341c7
plan_head_after: b782b5d2efaf04f5b82b87dcfc440fc9adb24ec8
---

# Phase 2 Plan 5: Resume Fidelity Summary

**Resume restores model, reasoning effort and plan state — proven from captured request bodies — a drafted plan survives quit and resume in a committed cast, and each setting is attributed to the session or to config**

## Performance

- **Duration:** 35 min
- **Started:** 2026-11-25T21:29:00Z
- **Completed:** 2026-11-25T22:05:00Z
- **Tasks:** 2
- **Files modified:** 9 (4 tracked sources/docs, 5 evidence files)

## Accomplishments

- `headless_resume_restores_model_effort_and_plan_state` drives **one** session through three
  `run_headless_in_sandbox_borrowed` invocations against one `TestSandbox` and one mock server.
  Observed facts from the recorded run (`--nocapture`):
  `started_model=deepseek-v4-pro started_effort=high resumed_model=deepseek-v4-pro
  resumed_effort=high override_model=deepseek-v4-pro override_effort=low
  plan_reminder_full=false plan_reminder_sparse=true`.
  The effort is carried by the top-level `reasoning_effort` field of the chat-completions body
  (not a `thinking` object); the mock model's `with_supports_reasoning_effort(true)` is necessary
  but not sufficient — see the catalog deviation below.
- The resumed session id equals the started one, the resumed model equals the started model, the
  resumed effort equals the started effort, the resumed turn's own message tail carries a
  plan-mode reminder, and the explicit `--effort low` on the third resume reaches the wire.
- The extended `plan_approval_resume` scenario keeps its existing choreography and assertions
  (now naming SESS-03), additionally compares the model across the quit from `summary.json`
  (`current_model_id`) and from the resumed turn's request body, and writes
  `resume-fidelity.cast` when `GROK_PTY_CAST_DIR` is set. Cast content: the seeded plan body
  `Plan GBT3703Repro`, the restored approval chrome (`request changes`, `quit plan`, `approve`),
  and the implement turn `GBT3703IMPLEMENTED` — all in the same invocation the item records.
- `docs/evidence/02-loop-item-verification/resume-fidelity` is finished with `result: ok`,
  `requirements: [SESS-01, SESS-03, SESS-04]`, two ok steps and three hashed artifacts
  (`resume-fidelity.cast` c0ac2e51…, `sess-01-…log` 0144e4c5…, `sess-03-…log` 922a01a9…).
  `scripts/key-leak-check.sh --scan` passes over the item.
- `22-permissions-and-safety.md` gains exactly one paragraph under `### How to set the mode`:
  the permission mode is re-resolved from CLI → `[ui] permission_mode` → default on every launch
  and is not carried over by resume; plan state and the plan file survive a restart; the model and
  effort come from the session and an explicit `-m`/`--effort` on resume wins.

## Task Commits

Each task was committed atomically:

1. **Task 1: Prove a resume restores model, effort and plan state** — `24db6d9c` (test)
2. **Task 2: Drafted plan survives quit and resume, provenance written down** — `b782b5d2` (docs)

**Plan metadata:** see the SUMMARY commit that follows.

## Files Created/Modified

- `crates/codegen/xai-grok-pager-bin/tests/headless_loop_items.rs` — new resume test plus body/message helpers (02-04's denial test untouched and still green).
- `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/plan_approval_resume.rs` — model comparison across the quit, SESS-03 failure labels, local `write_cast_if_requested`.
- `crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md` — one provenance paragraph.
- `crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/permission_prompt_outcomes.rs` — rustfmt-only (see deviation 3).
- `docs/evidence/02-loop-item-verification/resume-fidelity/` — `report.json`, `notes.md`, `resume-fidelity.cast`, two step logs.

## Decisions Made

- **The mock catalog is not consumed in this tree.** `ModelsManager::is_models_fetch_enabled()`
  returns `false` and `resolve_model_list` skips the built-in entries when a custom models endpoint
  is configured (which `TestSandbox::builder().mock_url()` does). The test therefore writes
  `[model."deepseek-v4-pro"] supports_reasoning_effort = true` into the sandbox's
  `$DEEPSEEK_BUILD_HOME/config.toml`. Without it, `-m deepseek-v4-pro` fails with "unknown model id"
  and `--effort` is silently dropped — verified empirically before adding the config.
- **The plan reminder on the resumed turn is the sparse variant.** `reminder_count` is 1 after the
  activation turn, and `should_use_full_reminder` is even-only, so the first resumed turn injects
  "Plan mode is still active. …". The test asserts a reminder in the *tail* of the request (the
  pre-quit full reminder is further up the history and cannot satisfy it) and accepts either
  variant, printing which one matched. Recorded in the item's `notes.md`.
- **The pty model observable is `summary.json` + the wire, not the info row.** The prompt info row
  renders `unknown` in this mock (no catalog → `effects/mod.rs` fallback), so it cannot distinguish
  models; `notes.md` states which observable was used, and no chip was invented.
- **`GROK_PTY_CAST_DIR` must be absolute.** A relative value resolves from the pty-harness package
  directory, not the repo root (the same note 02-04 recorded); the evidence step passes `$PWD/…`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] The mock catalog cannot make `-m deepseek-v4-pro` resolve**
- **Found during:** Task 1 (first test run)
- **Issue:** The plan assumed the mock's `GET /v1/models` entry is consumed. It is not:
  `is_models_fetch_enabled()` is hardcoded `false` and a custom models endpoint skips the built-in
  catalog, so the run failed with `Couldn't set model 'deepseek-v4-pro': Invalid params: "unknown
  model id"` and an empty catalog would also have dropped `--effort`.
- **Fix:** write a `[model."deepseek-v4-pro"]` table with `supports_reasoning_effort = true` into
  the sandbox's `config.toml`; the mock entry stays as the plan specified.
- **Files modified:** `crates/codegen/xai-grok-pager-bin/tests/headless_loop_items.rs`
- **Verification:** the three invocations observe `high`/`high`/`low` on the wire.
- **Committed in:** `24db6d9c`

**2. [Plan text, not a code deviation] The exact full reminder sentence is not on the resumed turn**
- **Found during:** Task 1
- **Issue:** The plan's must-have names the full sentence; the shell alternates full/sparse by
  `reminder_count`, and the resumed turn is odd, so the sparse variant is what is freshly injected.
  The full sentence does appear in the retained history.
- **Fix:** assert the reminder on the request tail, accepting either variant and printing which one
  matched; the stable clause is present in both. Documented in `notes.md` and here.
- **Verification:** `plan_reminder_full=false plan_reminder_sparse=true` in the recorded log.
- **Committed in:** `24db6d9c`

**3. [Rule 3 - Blocking] 02-04's `permission_prompt_outcomes.rs` was committed unformatted**
- **Found during:** Task 2 (`cargo fmt --all --check` for the plan's verification)
- **Issue:** `30388a51` left a rustfmt diff in that file, so the plan's own "fmt clean" gate failed
  at HEAD.
- **Fix:** `cargo fmt` on the file (line-joining only, no behavior change).
- **Files modified:** `crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/permission_prompt_outcomes.rs`
- **Verification:** `cargo fmt --all --check` exits 0.
- **Committed in:** `b782b5d2`

**4. [Plan text] The `<automated>` pty check used a relative `GROK_PTY_CAST_DIR`**
- **Found during:** Task 2
- **Issue:** relative paths resolve from the pty-harness package directory, so the cast would not
  land in the item.
- **Fix:** all runs (and the recorded step) use `$PWD/docs/evidence/...`.
- **Verification:** `wrote cast: /home/…/docs/evidence/02-loop-item-verification/resume-fidelity/resume-fidelity.cast`
- **Committed in:** `b782b5d2`

**5. [Process] Commits were made per task although the plan's `<output>` says to leave the tree uncommitted**
- **Found during:** close-out
- **Issue:** the plan inherits a C1 "reviewer commits" instruction; this dispatch explicitly asked
  for atomic per-task commits scoped `(02-05):`.
- **Fix:** two commits, one per task; the reviewer still sees them as one logical plan (same two files plus evidence).
- **Committed in:** `24db6d9c`, `b782b5d2`

---

**Total deviations:** 3 auto-fixed (2 blocking, 1 plan-text correction) + 2 process notes
**Impact on plan:** The catalog fix is required for the test to prove anything at all; the sparse
reminder is a real property of the shell, recorded rather than papered over; the fmt fix keeps the
tree's stated gate green. No scope creep.

## Issues Encountered

- `cargo clippy --all-targets` for the pty-harness crate fails on the pre-existing
  `benches/paste_latency.rs` (`Surface::as_str` no longer exists). Unmodified by this plan;
  `cargo clippy --tests` for both touched crates finishes with warnings only, 0 errors.
- No `gsd_run`/`gsd-tools` on PATH in this environment; STATE/ROADMAP/REQUIREMENTS were updated by
  hand (see the metadata commit).

## Not Touched (read-only audit)

`git diff 4dded936..HEAD --name-only` contains none of: `session_setup.rs`, `persistence.rs`,
`plan_mode.rs`, `event_loop.rs`, `settings/defs.rs`, `settings/registry.rs`, `tool_calls.rs`,
`19-plan-mode.md`, `26-config-reference.md`, `scripts/test-gate.expected`. SESS-02 is proven in the
`plan-read-only` item from 02-06, not here.

## Next Phase Readiness

- SESS-01, SESS-03 and SESS-04 are satisfied with evidence and can be marked complete.
- SESS-02 remains for 02-06, which also owns permission-mode-after-resume (PERM-03/PERM-04) and the
  headless plan-arming proof.
- 02-03 (key-leak scan over the tree and artifacts, key rotation) is still outstanding and is a
  human gate for the milestone.

## Self-Check: PASSED

- `headless_resume_restores_model_effort_and_plan_state` and 02-04's `headless_permission_denial_names_both_flags` both pass (`cargo test -p xai-grok-pager-bin --test headless_loop_items` → 2 passed).
- `plan_approval_restored_after_resume` passes with the extended assertions and writes the cast.
- `resume-fidelity/report.json` is `result: ok` with two ok steps, three 64-hex artifacts and no `.evidence-pending.json`.
- `cargo fmt --all --check` exits 0; clippy `--tests` on both touched crates: 0 errors.
- Read-only files (session_setup.rs, persistence.rs, plan_mode.rs, event_loop.rs, settings/defs.rs, settings/registry.rs, tool_calls.rs, 19-plan-mode.md, 26-config-reference.md, test-gate.expected) unchanged across `4dded936..HEAD`.
