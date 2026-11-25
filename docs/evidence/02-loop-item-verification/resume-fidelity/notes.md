# resume-fidelity — what comes back when a session resumes, and from where

Proves SESS-01 (resume restores the session's model and reasoning effort), SESS-03 (a drafted
plan survives quit and resume) and SESS-04 (the provenance of each resumed setting is written
down). SESS-02 (plan mode is read-only in every permission mode) is proven in the
`plan-read-only` item from 02-06, not here.

## Runs recorded in this item

- `sess-01: resumed model, effort and plan state` —
  `cargo test -p xai-grok-pager-bin --test headless_loop_items -- --nocapture headless_resume_restores_model_effort_and_plan_state`
  (log: `sess-01-resumed-model-effort-and-plan-state.log`). One `TestSandbox`, three
  `run_headless_in_sandbox_borrowed` invocations of one session: start with
  `-m deepseek-v4-pro --effort high --permission-mode plan`, resume with `-r <sessionId>`, resume
  again with `--effort low`. The test prints the observed facts:
  `started_model=deepseek-v4-pro started_effort=high resumed_model=deepseek-v4-pro
  resumed_effort=high override_model=deepseek-v4-pro override_effort=low
  plan_reminder_full=false plan_reminder_sparse=true`.
- `sess-03: drafted plan survives quit and resume` —
  `cargo test -p xai-grok-pager-pty-harness --test plan_approval_resume -- --nocapture`
  (log: `sess-03-drafted-plan-survives-quit-and-resume.log`), run with
  `PAGER_BINARY=/tmp/opencode/02-05/deepseek-build` (a private copy of the release binary that
  already contains 02-04; 02-05 changes no product code) and `GROK_PTY_CAST_DIR` set to this
  item, so `resume-fidelity.cast` came from the same invocation whose exit status is recorded.
  `GROK_PTY_CAST_DIR` must be absolute: a relative one resolves from the pty-harness package
  directory, not the repository root (same note as `permission-prompt/notes.md`).

`resume-fidelity.cast` (asciinema v2) shows the resumed plan viewer with the seeded plan body
(`Plan GBT3703Repro`), the restored approval chrome (`a approve | s request changes | c comment |
y copy plan | q quit plan`), and the implement turn (`GBT3703IMPLEMENTED`) after the approval.

## Provenance of each resumed setting

| Setting | Where the resumed value comes from | Code path | Proven by |
|---|---|---|---|
| Model | the session record: `summary.json` → `current_model_id`; an explicit `-m` on resume wins | `restore_persisted_model` (`session_setup.rs:1675`) resolves `summary.current_model_id` and applies it through `model_switch::apply`; the headless client sends `-m` after session open (`headless.rs` `apply_headless_model_and_effort`) | `sess-01`: the resumed request body's `model` equals the started request's `model` (`deepseek-v4-pro`); `sess-03`: the resumed turn asked the provider for the same model as before the quit |
| Reasoning effort | the session record: `summary.json` → `reasoning_effort`; an explicit `--effort` on resume wins | `restore_effort = initial_reasoning_effort.or(summary.reasoning_effort)` (`session_setup.rs:1697`) → `model_switch::apply(..., SwitchEffort::Set(..))` | `sess-01`: the resumed body's `reasoning_effort` equals the started body's (`high`), and the third invocation's explicit `--effort low` is what reaches the wire |
| Plan state | the session directory: `plan_mode.json` (`state`, `reminder_count`, `awaiting_plan_approval`), restored by `PlanModeTracker::from_snapshot` (`plan_mode.rs:84`); the shell also seeds `current_prompt_mode` from it (`spawn.rs:653`), which is how a prompt with no `mode` meta resolves to Plan | shell session restore (read-only for this plan) | `sess-01`: the resumed turn carries a plan-mode reminder in its own messages (the sparse variant — see below); `sess-03`: the parked `exit_plan_mode` is re-issued and the approval chrome returns |
| Plan file | the session directory: `plan.md` (`storage/mod.rs` `PLAN_MODE_FILE` neighbourhood; `PlanModeTracker::plan_file_path`) | shell session restore; the pager renders it through its normal plan viewer | `sess-03`: the plan body is on screen after resume |
| Permission mode | **config/CLI**, not the session: the CLI flag, then `[ui] permission_mode`, then the default, resolved at every launch | `event_loop.rs:1098-1103` (`cli_owns_mode` / `toml_owns_mode`) and `:1504` (`display_mode`) | **not in the session record**; the resume tests pass no permission flag in `sess-03` and the chip is the configured one, which is a config fact |
| Pager `plan_mode` toggle | per-session only, not persisted (`settings/registry.rs:1269`: “plan_mode: per-session, not persisted”) | pager settings registry | a plan chip on resume comes from the shell's `plan_mode.json`, not from this setting |

Two consequences worth stating explicitly:

- The permission chip matching after resume is a **config fact, not evidence that the session
  remembered it**. Nothing in `summary.json` or `plan_mode.json` carries the permission mode.
- The pager's own `plan_mode` toggle is per-session; the plan state that survives a restart is the
  shell's `plan_mode.json`.

## Observables used, and one finding

- The model name after the resume was proven from the shell's session summary
  (`summary.json` → `current_model_id`) and from the resumed turn's `chat/completions` request
  body. The pager's prompt info row rendered `unknown` on both frames because this pty mock has
  no model catalog (`effects/mod.rs` falls back to `unknown`), so the info row cannot distinguish
  models here; no chip was invented.
- The resumed turn's own plan reminder is the **sparse** variant
  (`Plan mode is still active. Do not make any edits or writes to the system except for the plan
  file.`), not the full activation variant. The shell alternates them by `reminder_count`
  (`PlanModeTracker::should_use_full_reminder`, even = full): the activation turn recorded one
  reminder, so the first resumed turn is odd. The stable clause
  `Do not make any edits or writes to the system` is present either way, and the test asserts the
  reminder is in the tail of the resumed request (the activation reminder from the pre-quit turn
  is further up the history and cannot satisfy that assertion).
- In this tree the model catalog is local (`ModelsManager::is_models_fetch_enabled()` returns
  `false`) and a custom models endpoint skips the built-in entries, so the headless test declares
  `[model."deepseek-v4-pro"]` in the sandbox's `config.toml`; the mock's `GET /v1/models` entry is
  served but not consumed. Without the `[model.*]` table, `-m deepseek-v4-pro` fails with
  “unknown model id”, and `--effort` is silently dropped because no catalog entry advertises it.
