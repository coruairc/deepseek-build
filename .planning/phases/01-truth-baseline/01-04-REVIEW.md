---
phase: 01-truth-baseline
plan: 04
reviewed: 2026-11-21T18:57:15Z
depth: deep
files_reviewed: 9
files_reviewed_list:
  - crates/codegen/xai-grok-models/default_models.json
  - crates/codegen/xai-grok-shell/src/agent/model_catalog/resolution.rs
  - crates/codegen/xai-grok-shell/src/agent/config_tests.rs
  - crates/codegen/xai-grok-shell/src/session/acp_session_impl/context_snapshot.rs
  - README.md
  - crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md
  - crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md
  - docs/TUNING.md
  - .planning/phases/01-truth-baseline/01-04-SUMMARY.md
findings:
  critical: 0
  warning: 4
  info: 6
  total: 10
status: issues_found
---

# Phase 01 Plan 04: Code Review Report (Default model)

**Reviewed:** 2026-11-21T18:57:15Z
**Base:** `e834f647` (all change under review is uncommitted in the working tree)
**Depth:** deep
**Files Reviewed:** 9
**Status:** issues_found (no blockers)

## Summary

DEF-01 holds in the code. `default_models.json` changes exactly the `default` line
(`deepseek-v4-pro` → `deepseek-flash`), the `models` array (picker order) is untouched, and the
resolver's `first_or_fallback` closure now prefetches `crate::models::default_model()` from the
*visible* map before `visible.first()`. I traced the four resolution arms: an explicit
CLI/env/`[models] default` preference still short-circuits in the `Some(pref)` arm; when no
preference exists the catalog default wins; a hidden default is excluded by `visible_for_auth`
(`!self.hidden`), a disabled default is removed by `resolve_model_catalog`, and both fall through to
the first visible entry; `is_session_auth` is honoured through the same `visible` filter; the
"no auth-visible selectable model" and `fallback_model_id` arms are byte-for-byte unchanged. Picker
order, `validate_selectable`, allowlist handling, persisted-session restore and the tokenizer's
`default_model()` call site are all unaffected beyond the intended default move.

The three new tests are genuinely discriminating: without the resolver branch the first test resolves
to the first JSON entry (`deepseek-v4-pro`) and fails; the hidden/disabled cases additionally assert
their precondition (the entry really is hidden / gone) before asserting the fallback. Each is
`#[serial]` with `EnvGuard::unset("DEEPSEEK_BUILD_DEFAULT_MODEL")`, which restores the prior value on
drop.

The tests I could run are green (`4 passed` on the `catalog_default` filter, `1 passed` on the
tokenizer pin, `6193 passed / 0 failed / 4 ignored` for the whole `xai-grok-shell` lib suite with the
API keys unset), and `cargo fmt --all --check` / `git diff --check` are clean.

I found no correctness, security or data-loss defect in the change itself. The findings below are
docs-truth/comment-accuracy defects (one of which is a user-visible behaviour — the default reasoning
effort — that the updated docs do not state) and a pre-existing credential-test defect that this
plan's verification run exposed. Secret hygiene is clean: the diff, the tree (excluding `target/`
fixtures), `.planning/`, and the SUMMARY contain no key material; `/tmp/opencode/01-04-configdocs.log`
contains `[REDACTED]` and zero `sk-`-shaped strings.

## Warnings

### WR-01: `docs/TUNING.md` still says the shipped default is `high` reasoning effort

**File:** `docs/TUNING.md:185-187` (the paragraph edited by this plan)
**Issue:** The paragraph now reads "The shipped defaults stay: the system prompt, the grok-build
toolset, and `high` reasoning effort." After DEF-01 the shipped default model is `deepseek-flash`,
whose catalog entry carries `"reasoning_effort": "low"` (`default_models.json:53-69`); with no
`[models] default_reasoning_effort` and no `--effort`, `resolve_model_catalog` does not stamp a
higher effort and `sampling_config_for_model` sends `info.reasoning_effort` = `low`
(`agent/config.rs:4341`). So a default session no longer runs at `high` effort — the change moves
the out-of-box effort from Pro/high to Flash/low, a user-visible behaviour change that none of the
four updated doc surfaces states. The implementer edited this exact sentence (it removed
`-m deepseek-flash` from the "levers" line beneath it) but left the now-false clause.
**Fix:** qualify the measurement claim and state the new default effort, e.g.
```markdown
The shipped defaults stay: the system prompt and the grok-build toolset. The measured
cost levers are documented for the user (`--effort low`), and no automatic routing was
wired. (E3c was measured on `deepseek-v4-pro` at its `high` default effort; the shipped
default is now `deepseek-flash`, whose catalog default effort is `low`.)
```
and add one sentence to the README/`11-custom-models.md` caveat saying the Flash default runs at
its catalog `low` effort, and switching to Pro restores Pro's `high` default.

### WR-02: the tokenizer comment still names a model that does not exist and a removed vendor

**File:** `crates/codegen/xai-grok-shell/src/session/acp_session_impl/context_snapshot.rs:20-22`
**Issue:** the comment above the line this plan's test pins says "`/tokenize-text` is the xAI
tokenizer" and "Always use the baked product default (deepseek-4.6 from `default_models.json`)". No
entry in `default_models.json` is `deepseek-4.6` (the catalog is `deepseek-v4-pro`,
`deepseek-flash`, hidden `deepseek-v4-flash`), and the endpoint resolves through
`EndpointsConfig::default().xai_api_base_url` to `api.deepseek.com` (the unit test on lines 383/387
asserts that host). The comment was already stale before this plan, but the plan touches this file
to pin `deepseek-flash` and the surrounding claim is precisely the "doc says one thing, code does
another" class this phase exists to remove.
**Fix:**
```rust
// Always use the baked product default from `default_models.json`
// (`crate::models::default_model()`), not the session model: the session model may be a
// third-party id the tokenizer endpoint does not serve.
```

### WR-03: `resolve_default_model`'s doc comment omits the new catalog-default step

**File:** `crates/codegen/xai-grok-shell/src/agent/model_catalog/resolution.rs:74`
**Issue:** the rustdoc still reads "Pick the default model: CLI > env > config, falling back to the
first visible model, then [`fallback_model_id`]." After this change the no-preference path is
catalog default → first visible → fallback, which is the contract the new tests and the new inline
comment encode. The public summary of the function now describes a different algorithm than the
function body.
**Fix:**
```rust
/// Pick the default model: CLI > env > config. With no preference, the catalog default
/// (`xai_grok_models::default_model()`) when it is visible, else the first visible model,
/// then [`fallback_model_id`].
```

### WR-04: a failing credential test prints the real API key (pre-existing; surfaced by this run)

**File:** `crates/codegen/xai-grok-shell/src/agent/config_tests.rs:7099` (test at `:7068-7105`)
**Issue:** with `DEEPSEEK_API_KEY` exported, `process_key_from_model_env_key` resolves the built-in
`env_key` instead of its `TEST_MODEL_ENV_KEY`, and the `assert_eq!` at `:7099` debug-formats both
sides — including the process key — into test output. That is exactly what happened in the executor's
verbatim `--features config-docs` run: the captured log contains `left: Some("…")` with the real key
before it was redacted (`/tmp/opencode/01-04-configdocs.log:6493-6495`). The plan's recipe
(`env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY`) is the correct *gate* fix — it
is what RESEARCH 3f and `KNOWN-ISSUES.md:296-301` already prescribe, and it weakens no assertion —
but it redacts the log rather than the cause: any developer or CI job with a key exported re-leaks
it on failure. The sibling failure `has_own_credentials_guards_session_vs_external_key`
(`:1496-1503`) also encodes an invariant ("Default model must not claim own credentials") that the
shipped catalog violates whenever `DEEPSEEK_API_KEY` is set, because every built-in entry declares
that `env_key`.
**Fix:** make the assertion non-revealing and isolate the env in the test:
```rust
let got = shared_api_key_provider(mgr).current_api_key_async().await;
assert!(got.as_deref() == Some(TOKEN), "model env-key provider returned an unexpected key");
```
plus `let _key = EnvGuard::unset("DEEPSEEK_API_KEY"); let _build_key = EnvGuard::unset("DEEPSEEK_BUILD_API_KEY");`
at the top of `process_key_from_model_env_key` and `has_own_credentials_guards_session_vs_external_key`.
Track the leak-on-failure in `KNOWN-ISSUES.md` alongside the existing env-dependency note.

## Info

### IN-01: `cargo test -p xai-grok-models --lib` is vacuous, so the plan's data gate proves nothing

**File:** `crates/codegen/xai-grok-models/src/lib.rs:29-41`, `crates/codegen/xai-grok-models/Cargo.toml`
**Issue:** the crate has no `#[cfg(test)]` module and no `tests/` directory, so the command passes
with `0 passed` (confirmed). The `DEFAULTS` consistency assertion is a `LazyLock` reached only when
`default_model()` is first called, which happens inside `xai-grok-shell` tests. JSON validity is
therefore still enforced in the workspace/shell gate, but a data-only edit to this crate cannot be
validated by its own crate's tests — and the assertion checks `m.model`, while the new resolver keys
on the catalog **id** (`m.id.unwrap_or(m.model)`), so a future `id != model` entry would not trip the
models crate at all. The new shell test would catch that divergence, but only if the shell suite runs.
**Fix:** add `#[cfg(test)] mod tests { #[test] fn default_is_a_listed_model() { assert_eq!(crate::default_model(), "deepseek-flash"); } }`
(calling `default_model()` triggers the `LazyLock` assert) or make the assertion compare `id` too.

### IN-02: no test pins "explicit preference beats the catalog default"

**File:** `crates/codegen/xai-grok-shell/src/agent/config_tests.rs:2521-2585`
**Issue:** the plan's third must-have truth ("An explicit `[models] default` still wins over the
catalog default") is correct by construction — the `Some(pref)` arm returns before
`first_or_fallback` — but no test exercises it; all three new tests pass an empty `[models]` table.
A future refactor that moved the catalog-default lookup above the `match &model_pref` would leave
the suite green while silently overriding the user's config.
**Fix:** add `catalog_default_does_not_override_an_explicit_config_default` with
`[models] default = "deepseek-v4-pro"`, asserting the resolved key is `deepseek-v4-pro` and
`source == ConfigSource::Config`.

### IN-03: the repository's knowledge base still says the default is Pro

**File:** `.planning/codebase/STACK.md:75`, `.planning/codebase/INTEGRATIONS.md:15`, `AGENTS.md:125`
**Issue:** both codebase-map pages say the catalog default is `deepseek-v4-pro`. `AGENTS.md:125` is
a historical note about commit `633d80df`, but it reads as current to a resuming agent and its §0
snapshot does not mention the new default. These are not shipped docs and were outside the plan's
surfaces, but the phase is a truth baseline and the next session reads them.
**Fix:** update the two `.planning/codebase` lines and add the current default to the AGENTS.md §0
snapshot at session end.

### IN-04: the README headless example now selects the default model explicitly

**File:** `README.md:287`
**Issue:** `dsb -m deepseek-flash --effort low -p "..."` now restates the shipped default model; the
surrounding prose says Flash is already the default. It is not wrong (it demonstrates the flags),
but it no longer illustrates "choose the cheap tier".
**Fix:** either leave it as a flags example or change it to `dsb -m deepseek-v4-pro -p "..."` to
mirror the documented revert path.

### IN-05: missing blank line separates the new test block from its predecessor

**File:** `crates/codegen/xai-grok-shell/src/agent/config_tests.rs:2520-2521`
**Issue:** the closing `}` of `hidden_models_kept_in_catalog_but_not_in_acp` is immediately followed
by the `///` doc comment of `catalog_default_is_the_resolved_default`; every other test in the file
has a blank line between blocks. `rustfmt` does not reflow doc comments, so this stays.
**Fix:** insert one blank line.

### IN-06: the fallback tests hardcode `deepseek-v4-pro` as "first visible"

**File:** `crates/codegen/xai-grok-shell/src/agent/config_tests.rs:2557-2558, 2581-2582`
**Issue:** the tests are named "falls back to first visible" but assert the literal current first
entry. That is faithful to the plan and it deliberately pins picker order (CONTEXT: order unchanged),
so it is not a defect; it does mean the test reads as an implementation detail. If the intent is the
property, derive the expected key from the catalog's first `is_picker_eligible` entry and keep a
separate literal assertion for the order.
**Fix:** optional — `let first = catalog.iter().find(|(_, e)| e.info.is_picker_eligible(false)).unwrap().0;`
then assert equality, with a comment that the literal order is pinned by the JSON diff.

## Verification performed

All commands run in `/home/cein_orourke/Projects/deepseek-build` at base `e834f647`; heavy cargo
commands under `flock -w 900 /tmp/team-build.lock`. `scripts/disk-guard.sh` exited 0 (1001 GB free).
My shell has `DEEPSEEK_API_KEY` exported, so the whole-suite run used the plan's `env -u` recipe.

| # | Command | Result |
|---|---------|--------|
| 1 | `git diff --stat HEAD` | 8 files, **+107/−9**; no `Cargo.toml`/`Cargo.lock`/`deny.toml` |
| 2 | Python JSON check of `default_models.json` | `default=deepseek-flash`; ids/models = pro, flash, legacy; default is in `models[]` |
| 3 | `scripts/disk-guard.sh` | PASS — 1001 GB free |
| 4 | `cargo test -p xai-grok-models --lib` | exit 0 — **0 passed** (no test target; matches SUMMARY claim 1) |
| 5 | `cargo test -p xai-grok-shell --lib -- catalog_default` | **4 passed**, 0 failed (the 3 new + `new_session_without_meta_keeps_current_effort_over_catalog_default`); matches claim 2 |
| 6 | `cargo test -p xai-grok-shell --lib tokenize_uses_baked_product_default_model` | **1 passed**, 0 failed; matches claim 3 |
| 7 | `env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY cargo test -p xai-grok-shell --lib` | **6193 passed, 0 failed, 4 ignored** in 52.19 s; matches claim 10 |
| 8 | `git diff --check HEAD` | exit 0 |
| 9 | `cargo fmt --all --check` | exit 0 |
| 10 | Secret scan: `git diff HEAD` for `sk-[A-Za-z0-9]{20,}`; `grep -r` over the tree (excl. `target/`, `.git`), `.planning/phases/01-truth-baseline/`, `/tmp/opencode/01-04-configdocs.log` | No key material. Diff only has the pre-existing `sk-...` placeholder; log has one `[REDACTED]`, zero `sk-` strings; hits in tree are fixtures/canaries |
| 11 | `cargo clippy -p xai-grok-shell -p xai-grok-models --all-targets` | exit 0, **0 errors**; warnings only in untouched files (`agent_ops.rs:1475`, `subagent/spawn.rs:543`, `tools/notification_bridge.rs:494`, `session/unified_list/mod.rs:800`) plus dependency crates — no hit in the plan's changed files |

NOT RUN: `cargo test -p xai-grok-shell --lib --features config-docs` end-to-end. I inspected the
executor's captured run instead: `test result: FAILED. 6198 passed; 3 failed; 4 ignored` with exactly
the three tests the SUMMARY names, and the whole-suite run in row 7 (same tests, keys unset) is
green, which is consistent with the two-command claim. `cargo test --workspace` belongs to 01-05.

## Verification honesty (SUMMARY assessment)

- Claims 1, 2, 3, 10 and 13 reproduce exactly; the diff really is the single `default` line plus the
  resolver branch, the three tests, the literal pin and four doc surfaces.
- Claim 9's failing run is real: the captured log shows `6198 passed; 3 failed; 4 ignored` and the
  three failures are exactly `has_own_credentials_guards_session_vs_external_key`,
  `process_key_from_model_env_key` and `forbidden_subscription_error_no_hint_without_api_key`.
  Attribution to "the known env-dependent tests" is correct and matches `KNOWN-ISSUES.md:296-301`
  and RESEARCH 3f. The `env -u` recipe is the prescribed gate fix (it changes no assertion), so it
  is the right fix *for the gate*; it is not a root fix (see WR-04).
- Claim 1's "proves nothing on its own" is honest and I confirmed the `0 passed` result. The
  SUMMARY's deviation 4 describes the `LazyLock` behaviour accurately.
- Claim 12 reproduces (exit 0, 0 errors): every warning is in an untouched file. The SUMMARY's
  "in dependencies" is loose — all four distinct warnings are in `xai-grok-shell` itself
  (e.g. `session/unified_list/mod.rs:800`), not dependency crates — but "pre-existing" is correct
  and no warning points at a file this plan changed.
- `26-config-reference.md` is not in the diff (`git status`/`git diff --name-only`); its
  `models.default` row names no value, so the plan's "no edit needed" rationale holds.
- The unrecorded red-first run: `catalog_default_is_the_resolved_default` does fail without the
  resolver branch (first visible is `deepseek-v4-pro`), so the claim is true by inspection, but the
  report records no captured red run. Not a defect in the code; one line of evidence would have
  removed the need to reason about it.
- No fabricated numbers: 63/63 matches `scripts/evals/results/baseline-report.md:3-4`; no new cost
  figure was introduced by this plan (01-03's territory).

## Verdict

**PASS_WITH_NOTES** — DEF-01 is implemented correctly and safely: the catalog field is now the
authority on the no-preference path, hidden/disabled defaults fall back, explicit preferences still
win, picker order and `fallback_model_id` are unchanged, and the docs carry the caveat and both
revert spellings. There is no blocker. Ship after fixing WR-01 (the `high`-effort claim, which is
now false and also hides the out-of-box effort move), and preferably WR-02/WR-03 (comment accuracy
in files this plan touched). WR-04 is pre-existing, but since this plan's own verification surfaced
it and the gate recipe only redacts the log, it deserves a follow-up ticket.

---

_Reviewed: 2026-11-21T18:57:15Z_
_Reviewer: gsd-code-reviewer (subagent)_
_Depth: deep_

## Resolution (2026-11-22)

- **WR-01**: `docs/TUNING.md` records that the shipped default is Flash at its
  catalog `low` effort and that `--effort low` was measured on Pro.
- **WR-02**: the tokenizer comment names the catalog default, not a
  non-existent model.
- **WR-03**: `resolve_default_model`'s rustdoc includes the catalog-default step.
- **WR-04**: fixed 2026-11-22 — `process_key_from_model_env_key` unsets
  `DEEPSEEK_API_KEY`/`DEEPSEEK_BUILD_API_KEY` and its assertion no longer
  debug-formats the key. The sibling test was left unchanged on purpose: it is a
  non-`serial` test, so adding a process-global env mutation there would create a
  new race.
- **IN-05** verified fixed (blank line present).
