---
phase: 01-truth-baseline
plan: 04
status: complete
subsystem: model-catalog
tags: [default-model, def-01, resolver, docs]
requirements: [DEF-01]
dependency_graph:
  requires: ["01-03 (serialisation only; no artifact consumed)"]
  provides:
    - "authoritative catalog default (`default_models.json` default read by `first_or_fallback`)"
    - "three resolver tests: default present / hidden / absent"
  affects:
    - "session default model on every no-preference launch"
    - "README, user guide (05, 11), docs/TUNING.md"
tech_stack:
  added: []
  patterns:
    - "catalog policy lives in `model_catalog/resolution.rs`, data in `xai-grok-models`"
key_files:
  created: []
  modified:
    - crates/codegen/xai-grok-models/default_models.json
    - crates/codegen/xai-grok-shell/src/agent/model_catalog/resolution.rs
    - crates/codegen/xai-grok-shell/src/agent/config_tests.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session_impl/context_snapshot.rs
    - README.md
    - crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md
    - crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md
    - docs/TUNING.md
decisions:
  - "RESEARCH option B: the `default_models.json` default field is authoritative in `first_or_fallback`; JSON array (picker) order unchanged."
  - "A hidden or disabled default falls through to the first visible entry, so an unselectable id is never returned."
commits: 0
plan_head_before: e834f6470f0b8af60f595455d02816f8d827addf
plan_head_after: e834f6470f0b8af60f595455d02816f8d827addf
actuals:
  tokens: 3300
  tasks: 2
  commits: 0
metrics:
  duration: "~40m"
  completed: "2026-11-21"
---

# Phase 1 Plan 4: Default model flip (DEF-01) Summary

`deepseek-flash` is now the catalog default and the resolver actually honours it: `first_or_fallback` prefers `xai_grok_models::default_model()` when that entry is visible, so a no-preference session starts on Flash instead of the first JSON array entry (`deepseek-v4-pro`). Picker order is untouched; hidden/disabled defaults fall back to the first visible entry. Four documentation surfaces state the default, the 63/63 at-ceiling caveat and both revert spellings.

## Per-task changes

### Task 1 (tracer) — make the catalog default authoritative

- `crates/codegen/xai-grok-models/default_models.json`: `"default": "deepseek-v4-pro"` → `"default": "deepseek-flash"`. Single-line diff; the `models` array, `web_search`/`image_description`/`session_summary` fields and the hidden `deepseek-v4-flash` alias are untouched.
- `crates/codegen/xai-grok-shell/src/agent/model_catalog/resolution.rs` — `resolve_default_model`'s `first_or_fallback` closure now begins:
  ```rust
  if let Some((key, entry)) = visible.get_key_value(crate::models::default_model()) {
      return (key.clone(), entry.clone());
  }
  ```
  preceded by a comment recording that the catalog field is the authority for the no-preference path and that a hidden or disabled default deliberately falls through. Nothing else changed: explicit CLI/env/`[models] default` still short-circuits before this closure (`model_pref` arm), the "no auth-visible selectable model" warning arm and the final `fallback_model_id` arm are untouched, and `fallback_model_id` itself is unchanged.
- `crates/codegen/xai-grok-shell/src/agent/config_tests.rs`: three new tests, all built with `resolve_model_catalog` over `Config::new_from_toml_cfg`, then `resolve_default_model(..., is_session_auth = false)`, each asserting on the resolved key and `ConfigSource::Default`:
  - `catalog_default_is_the_resolved_default` — empty `[models]`: key == `crate::models::default_model()` == `deepseek-flash`, entry model id `deepseek-flash`. (Fails without the resolver change, so it pins the new behaviour rather than restating it.)
  - `hidden_catalog_default_falls_back_to_first_visible` — `[models] hidden_models = ["deepseek-flash"]`: asserts the entry really is hidden, then key == `deepseek-v4-pro`.
  - `absent_catalog_default_falls_back_to_first_visible` — `[models] disabled_models = ["deepseek-flash"]`: asserts the key is gone from the catalog, then resolved key == `deepseek-v4-pro`.
  - All three are `#[serial]` with `EnvGuard::unset("DEEPSEEK_BUILD_DEFAULT_MODEL")` so an env preference in the host shell cannot mask the no-preference path.
- `crates/codegen/xai-grok-shell/src/session/acp_session_impl/context_snapshot.rs`: `tokenize_uses_baked_product_default_model` literal pin `"deepseek-v4-pro"` → `"deepseek-flash"` (kept a literal, as required).

### Task 2 — shipped documentation

- `README.md`
  - phase-2 row: "DeepSeek catalog (default `deepseek-v4-pro`)" → "(default `deepseek-flash`)".
  - model list: "Models: `deepseek-flash` (default) and `deepseek-v4-pro`, plus a hidden `deepseek-v4-flash` legacy alias …".
  - new short paragraph after the model list: Flash is the default because it is cheaper and faster; both models pass 63/63 on the 21-task eval set, so that set is at its ceiling and cannot show Flash is sufficient for harder work; revert with `/model deepseek-v4-pro` (TUI) or `-m deepseek-v4-pro` (headless); links `docs/TUNING.md`. No cost figure stated (that belongs to 01-03).
  - `/model` row: "switch between `deepseek-flash` (default) and `deepseek-v4-pro`".
- `crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md`: "new sessions start with `deepseek-flash`"; one-paragraph caveat + both revert spellings; Config Default example `default = "deepseek-flash"`.
- `crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md`: `[models]` example `default = "deepseek-flash"` only; the `web_search` example line is left as-is.
- `docs/TUNING.md`: new `### Shipped default: deepseek-flash (DEF-01)` section after Findings — default is now Flash; baseline is 63/63 for both models so the pass rate is at its ceiling and the set cannot prove Flash sufficient for hard work; revert path `/model deepseek-v4-pro` and `-m deepseek-v4-pro`; provenance is the existing measurement (21 tasks × 3 runs, 2026-10-08, `scripts/evals/run-baseline.sh`), explicitly no new measurement. Also dropped the now-redundant `-m deepseek-flash` from the Decisions "cost levers" parenthetical (see Deviations).
- `26-config-reference.md` not touched.

## Verification (exact commands and results)

Run in `/home/cein_orourke/Projects/deepseek-build`, base `e834f647`.

| # | Command | Result |
|---|---------|--------|
| 0 | `scripts/disk-guard.sh` | PASS — 1004 GB free (min 60) |
| 1 | `flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-models --lib` | PASS — exit 0, `0 passed; 0 failed` (crate has no test targets; see Deviations) |
| 2 | `flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-shell --lib -- catalog_default` | PASS — `4 passed; 0 failed` (the 3 new tests + `new_session_without_meta_keeps_current_effort_over_catalog_default`) |
| 3 | `flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-shell --lib tokenize_uses_baked_product_default_model` | PASS — `1 passed; 0 failed` |
| 4 | `! grep -nE 'deepseek-v4-pro.{0,8}\(default\)\|deepseek-v4-pro.{0,16}the default' README.md` | PASS — exit 0, no matches |
| 5 | `grep -q 'deepseek-flash' .../11-custom-models.md` | PASS |
| 6 | `grep -q 'deepseek-flash' .../05-configuration.md` | PASS |
| 7 | `grep -qE '`deepseek-flash` \(default\)' README.md` | PASS — matches at lines 247 and 276 |
| 8 | `grep -n -- '-m deepseek-v4-pro' docs/TUNING.md; grep -n '/model deepseek-v4-pro' docs/TUNING.md; grep -n '63/63' docs/TUNING.md` | PASS — lines 49 (`/model deepseek-v4-pro`), 50 (`-m deepseek-v4-pro`), 42 + existing baseline rows (`63/63`) |
| 9 | `flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-shell --lib --features config-docs` | **exit 101** with `DEEPSEEK_API_KEY` exported: `6198 passed; 3 failed; 4 ignored`. Failures are exactly the three known env-dependent tests (RESEARCH 3f / `KNOWN-ISSUES.md`): `agent::config::tests::has_own_credentials_guards_session_vs_external_key`, `agent::config::tests::process_key_from_model_env_key`, `sampling::error::tests::forbidden_subscription_error_no_hint_without_api_key`. Re-run with the key unset (the plan-01-05 determinism recipe) → **PASS**: `6201 passed; 0 failed; 4 ignored` (exit 0, 51.43 s). |
| 10 | `env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-shell --lib` | PASS — `6193 passed; 0 failed; 4 ignored` (exit 0, 52.17 s) |
| 11 | `cargo fmt --all --check` | PASS — exit 0 |
| 12 | `flock -w 900 /tmp/team-build.lock cargo clippy -p xai-grok-shell -p xai-grok-models --all-targets` | PASS — exit 0, 0 errors; only pre-existing warnings in dependencies, none pointing at a file this plan touched |
| 13 | `git diff --stat` review | 8 files, +107/−9. `default_models.json` diff is the one `default` line. No `Cargo.toml`, no `Cargo.lock`. |

NOT RUN: `cargo test --workspace` (belongs to plan 01-05's gate; not in this plan's verification block).

## Deviations from plan

1. **[Environment, not a code deviation] `config-docs` step run twice.** The verbatim command fails only on the three credential tests that are known to fail when `DEEPSEEK_API_KEY` is exported. The same command with `env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY` is green (6201/6201). No test was edited, skipped or quarantined; the fix belongs to plan 01-05's gate recipe.
2. **[Minor consistency edit, Rule 2] `docs/TUNING.md` Decisions paragraph.** Removed `-m deepseek-flash` from "the measured cost levers are documented for the user (`--effort low` / `-m deepseek-flash`)" because Flash is now the default, so it is no longer a lever. No measurement wording changed.
3. **[Scope note] README/user-guide caveat paragraph.** Task 2's numbered action listed only three README spots, but `01-CONTEXT.md` ("Document the at-ceiling caveat … in the README quickstart, the user guide and `docs/TUNING.md`") and this plan's `must_haves.truths` require it, so the caveat + revert path were added to the README quickstart and `11-custom-models.md` as well as `docs/TUNING.md`. No cost figure was stated.
4. **[Empty test target] `cargo test -p xai-grok-models --lib` runs 0 tests.** The crate has no `#[test]` and no `tests/` directory; the `DEFAULTS` consistency assertion in `xai-grok-models/src/lib.rs:37-41` is a `LazyLock` evaluated on first `default_model()` call, so it fires inside the shell test runs (verification 2, 3, 9, 10) rather than in this command. The command exits 0 as the plan requires, but it proves nothing on its own.

## Risks / notes

- **The new resolver branch is keyed on the catalog *key*.** `visible.get_key_value(crate::models::default_model())` matches the built-in key, which equals the model id, exactly as specified. A user who defines `[model.some-alias] model = "deepseek-flash"` does not re-route the default; their config key is not the catalog key. This matches the plan's instruction but is worth knowing if aliasing is ever wanted.
- **`/model` picker** still lists Pro first (JSON array order untouched, per CONTEXT). A user reading the picker sees Pro at the top while the session starts on Flash; the README's `/model` row says "`deepseek-flash` (default)" in the command table only.
- **Env-secret exposure, incidental.** The `config-docs` failure output (verification 9, first run) printed the exported `DEEPSEEK_API_KEY` value inside a test assertion message (`process_key_from_model_env_key`). The value was not repeated in this report or any commit; the captured log `/tmp/opencode/01-04-configdocs.log` was rewritten with the key replaced by `[REDACTED]` (residual `sk-…`-shaped strings: 0). This is a pre-existing test hygiene issue outside this plan's `files_modified`, flagged for `KNOWN-ISSUES.md`.
- Nothing committed: the working tree holds all 8 modified files for review, as instructed.

## Files touched

```
M README.md
M crates/codegen/xai-grok-models/default_models.json
M crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md
M crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md
M crates/codegen/xai-grok-shell/src/agent/config_tests.rs
M crates/codegen/xai-grok-shell/src/agent/model_catalog/resolution.rs
M crates/codegen/xai-grok-shell/src/session/acp_session_impl/context_snapshot.rs
M docs/TUNING.md
```
