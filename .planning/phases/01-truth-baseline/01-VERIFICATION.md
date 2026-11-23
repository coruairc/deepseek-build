---
phase: 01-truth-baseline
verified: 2026-11-23T17:18:28Z
status: gaps_found
score: 23/24 must-haves verified
covered_files:
  - ".planning/phases/01-truth-baseline/01-01-PLAN.md"
  - ".planning/phases/01-truth-baseline/01-01-SUMMARY.md"
  - ".planning/phases/01-truth-baseline/01-02-PLAN.md"
  - ".planning/phases/01-truth-baseline/01-02-SUMMARY.md"
  - ".planning/phases/01-truth-baseline/01-03-PLAN.md"
  - ".planning/phases/01-truth-baseline/01-03-SUMMARY.md"
  - ".planning/phases/01-truth-baseline/01-04-PLAN.md"
  - ".planning/phases/01-truth-baseline/01-04-SUMMARY.md"
  - ".planning/phases/01-truth-baseline/01-05-PLAN.md"
  - ".planning/phases/01-truth-baseline/01-05-SUMMARY.md"
  - "KNOWN-ISSUES.md"
  - "README.md"
  - "TESTING.md"
  - "crates/codegen/xai-grok-models/default_models.json"
  - "crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md"
  - "crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md"
  - "crates/codegen/xai-grok-pager/docs/user-guide/14-headless-mode.md"
  - "crates/codegen/xai-grok-pager/docs/user-guide/25-status-line.md"
  - "crates/codegen/xai-grok-pager/src/app/status_line_tests.rs"
  - "crates/codegen/xai-grok-pager/src/headless.rs"
  - "crates/codegen/xai-grok-pager/src/headless/reducer/messages/mod.rs"
  - "crates/codegen/xai-grok-pager/src/headless/reducer/messages/tests/result_usage.rs"
  - "crates/codegen/xai-grok-pager/src/headless/reducer/messages/usage.rs"
  - "crates/codegen/xai-grok-pager/src/headless/reducer/messages/wire.rs"
  - "crates/codegen/xai-grok-pager/src/headless/reducer/mod.rs"
  - "crates/codegen/xai-grok-pager/src/headless_tests.rs"
  - "crates/codegen/xai-grok-pager/src/views/status_line/mod.rs"
  - "crates/codegen/xai-grok-pager/src/views/status_line/segments.rs"
  - "crates/codegen/xai-grok-pager/src/views/status_line/segments_tests.rs"
  - "crates/codegen/xai-grok-sandbox/src/lib.rs"
  - "crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs"
  - "crates/codegen/xai-grok-sandbox/src/runtime_sockets_tests.rs"
  - "crates/codegen/xai-grok-sandbox/tests/deny_paths_e2e.rs"
  - "crates/codegen/xai-grok-shell/src/agent/config_tests.rs"
  - "crates/codegen/xai-grok-shell/src/agent/folder_trust.rs"
  - "crates/codegen/xai-grok-shell/src/agent/model_catalog/resolution.rs"
  - "crates/codegen/xai-grok-shell/src/session/acp_session_impl/context_snapshot.rs"
  - "crates/codegen/xai-grok-shell/src/util/config/consent_tests.rs"
  - "crates/codegen/xai-grok-status-line/src/pricing.rs"
  - "crates/codegen/xai-grok-workspace/src/session/git_gate_tests.rs"
  - "docs/TUNING.md"
  - "docs/evidence/01-truth-baseline/cost-estimate/both-suites-report-test-result-ok.log"
  - "docs/evidence/01-truth-baseline/cost-estimate/cost-estimate.cast"
  - "docs/evidence/01-truth-baseline/cost-estimate/notes.md"
  - "docs/evidence/01-truth-baseline/cost-estimate/report.json"
  - "docs/evidence/01-truth-baseline/cost-estimate/script-t-capture-status-line-headless-suites.log"
  - "docs/evidence/01-truth-baseline/cost-estimate/source-pins-the-four-rendered-strings.log"
  - "docs/evidence/01-truth-baseline/cost-estimate/typescript-ran-the-pinned-cost-tests.log"
  - "docs/evidence/01-truth-baseline/ipv6-report/ipv4-refused-blocked-line-names-203-0-113-9.log"
  - "docs/evidence/01-truth-baseline/ipv6-report/ipv6-allowlist-resolves-ipv6-permits-it-and-matches-the-v4-mapped-spelling.log"
  - "docs/evidence/01-truth-baseline/ipv6-report/ipv6-loopback-1-and-ffff-127-0-0-1-permitted.log"
  - "docs/evidence/01-truth-baseline/ipv6-report/ipv6-refused-blocked-line-names-2001-db8-1.log"
  - "docs/evidence/01-truth-baseline/ipv6-report/ipv6-refused-no-non-ip-placeholder.log"
  - "docs/evidence/01-truth-baseline/ipv6-report/ipv6-report.cast"
  - "docs/evidence/01-truth-baseline/ipv6-report/notes.md"
  - "docs/evidence/01-truth-baseline/ipv6-report/report.json"
  - "docs/evidence/01-truth-baseline/ipv6-report/script-sandbox-run-sh-runs-end-to-end-bin-bin-echo.log"
  - "docs/evidence/01-truth-baseline/test-gate/expected-count-provenance-run-2-refresh-expected.log"
  - "docs/evidence/01-truth-baseline/test-gate/full-gate-confirmation-run-3.log"
  - "docs/evidence/01-truth-baseline/test-gate/notes.md"
  - "docs/evidence/01-truth-baseline/test-gate/report.json"
  - "docs/evidence/01-truth-baseline/test-gate/step2-config-docs-result-line.log"
  - "docs/evidence/01-truth-baseline/test-gate/subset-mode-mechanism-run-cast-source.log"
  - "docs/evidence/01-truth-baseline/test-gate/test-gate.cast"
  - "docs/evidence/01-truth-baseline/test-gate/test-gate.log"
  - "docs/evidence/01-truth-baseline/test-gate/toolchain-provenance-rustc-version-cargo-path.log"
  - "docs/evidence/README.md"
  - "scripts/README.md"
  - "scripts/egress-check.sh"
  - "scripts/egress_guard.c"
  - "scripts/evidence.py"
  - "scripts/key-leak-check.sh"
  - "scripts/sandbox-run.sh"
  - "scripts/test-gate.expected"
  - "scripts/test-gate.sh"
  - "scripts/test-sandbox-run.sh"
covered_digest: "v4:sha256:444eb00881b3f6b8fc4c73222c17a76e19063ac243b7b22c0e7f7c0ea0a3bbfa"
behavior_unverified: 0
overrides_applied: 0
gaps:
  - truth: "The `git_gate` / `restore_fetch` timing tests have headroom on every under-margin wait, with ordering invariants unchanged, and a recorded five-run repetition shows no rotation (TEST-03)."
    status: failed
    reason: "Independent repetition of the phase's own command falsified the claim: 35 runs of `cargo test -p xai-grok-workspace --lib -- --test-threads=4` produced 2 failures. The captured one is `restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild` at `restore_fetch_tests.rs:731` — `grandchild 706149 must not remain live`. `restore_fetch_tests.rs` was never modified by this phase (0 commits in 711524dd..HEAD; file sha256 identical to HEAD), no `#[ignore]` quarantine exists, and the five-run repetition is not committed. 'Zero failures' is still a lucky run."
    artifacts:
      - path: "crates/codegen/xai-grok-workspace/src/restore_fetch_tests.rs"
        issue: "fetch_timeout_sigkills_term_immune_grandchild checks the grandchild pid immediately after teardown; under 4 test threads the SIGKILL/reap is not yet observable roughly one run in twenty"
      - path: "KNOWN-ISSUES.md"
        issue: "States 'restore_fetch's bands were left unchanged because no rotation was observed' and marks the five-run repetition 'reported, not artifact-backed (prior session)' — the observation is not reproducible and the claim has no committed artifact"
    missing:
      - "Make `fetch_timeout_sigkills_term_immune_grandchild` deterministic — bounded poll for the pid / process group to disappear, or the plan's pure `head_timeout` fallback — or quarantine it explicitly with `#[ignore = \"<reason>; tracked in KNOWN-ISSUES.md>\"]` and a named reproducer"
      - "Commit the repetition evidence (or correct KNOWN-ISSUES.md / TESTING.md / 01-05-SUMMARY.md to state the flake remains open) so the determinism claim carries an artifact"
deferred:
  - truth: "`scripts/check-egress.sh --strict` exits 0 over the tree (zero HARD hits)"
    addressed_in: "Phase 5 (FREEZE-03)"
    evidence: "Phase 5 success criterion 4 requires the pre-freeze checklist to pass '`check-egress.sh --strict` exit 0 with a dated record'; 01-02 deferred-items.md records the three committed fixture hits (config_tests.rs:2768, xai-grok-shell-base/src/util/mod.rs:422-423) as rejected-string fixtures; TESTING.md step 11 records the known red baseline. Not a Phase 1 success criterion."
---

# Phase 1: Truth Baseline Verification Report

**Phase Goal:** As the author, I want the sandbox to stop aborting on this host, the cost figure and the default model to stop lying, and every claim to carry a committed artifact, so that a green suite actually means something.
**Verified:** 2026-11-23T17:18:28Z
**Status:** gaps_found
**Re-verification:** No — initial verification (no previous `01-VERIFICATION.md`)

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | 01-01: A runtime-socket candidate whose parent is not traversable is skipped in both shapes (ancestor EACCES; unsearchable last component) | ✓ VERIFIED | `classify_endpoint_error` + `parent_is_traversable` (`runtime_sockets.rs:116-132`) consulted at `:167-203`; `unsearchable_parent_directory_is_skipped` and `unsearchable_ancestor_is_skipped`; `cargo test -p xai-grok-sandbox --lib` → 382 passed / 0 failed |
| 2 | 01-01: Every other resolution error stays fatal (symlink, missing parent with live endpoint, LSM denial under a traversable parent, no file name) | ✓ VERIFIED | Pure table pins `PermissionDenied + traversable → Fatal` (`runtime_sockets_tests.rs:703-723`); `symlink_endpoint_is_fatal`; `materialized_socket_paths_propagate_parent_resolution_errors`; `unreadable_endpoint_under_traversable_parent_is_materialized` |
| 3 | 01-01: A candidate inside a mode-000 parent no longer aborts bwrap startup, and `deny_paths_e2e` passes unmodified — no `#[ignore]`, no new skip predicate | ✓ VERIFIED | Full integration binary run: `14 passed; 0 failed; 1 ignored` (the one ignore is the pre-existing `subprocess_entry` harness entry); diff `0bf46c84` adds no `#[ignore]` and leaves `skip_if_enforcement_unavailable` untouched |
| 4 | 01-01: `devbox_genuine_reexec_applies_enforcement` gives re-exec a real obligation on every host | ✓ VERIFIED | Diff retargets to `[profiles.genuine] extends = "devbox" deny = ["secret.pem"]`; run with `SANDBOX_E2E_REQUIRE_ENFORCEMENT=1` passes (host has no `/data`, uid 1000) |
| 5 | 01-02: `sandbox-run.sh` resolves the allow host over both families and refuses an IPv6 destination by naming it, never `non-ip` | ✓ VERIFIED | `scripts/egress_guard.c` `getaddrinfo(AF_UNSPEC)`, family-tagged allowlist, `inet_ntop` destination formatting (`:105-155`); harness assertions 1–2 PASS |
| 6 | 01-02: An allowlisted IPv6 address and IPv6 loopback are permitted | ✓ VERIFIED | `sh scripts/test-sandbox-run.sh` assertion 3 PASS (`::1` and `::ffff:127.0.0.1`), assertion 4 PASS (IPv6 allowlist + v4-mapped spelling) |
| 7 | 01-02: `::ffff:1.2.3.4` and `1.2.3.4` are one allowlist entry; IPv4 behaviour unchanged | ✓ VERIFIED | v4-mapped normalized to 4-byte form on store and match (`egress_guard.c:64-75,120-141`); assertions 4–5 PASS |
| 8 | 01-02: `sh scripts/test-sandbox-run.sh` proves the above offline and exits non-zero on any failure | ✓ VERIFIED | Run twice (direct and via gate): `ALL PASS (6 assertions)`, exit 0; script tracks failures and exits 1 |
| 9 | 01-02: `scripts/egress-check.sh` prose no longer describes the fixed `non-ip` bug | ✓ VERIFIED | `scripts/egress-check.sh:548` now states the refused address is named in its own family's notation and `non-ip` only covers a non-IP family |
| 10 | 01-03: A `deepseek-v4-flash` turn prices at Flash rates — worked example totals 0.001206 USD | ✓ VERIFIED | `FLASH_ALIAS_PRICING_MODEL` + `ModelPricing::flash()` (`pricing.rs:36,69-79,91`); `cargo test -p xai-grok-status-line --lib pricing` → 8 passed, including `flash_alias_worked_example_is_exact` |
| 11 | 01-03: The TUI cost item renders `~$0.0042 est.`, `~$0.37 est.`, `~$0.01 est.` and omits below $0.0001 | ✓ VERIFIED | `MIN_DISPLAYED_COST_USD = 0.0001` and `format_cost` (`segments.rs:18-20,141-153`); `cost_renders_the_estimate_at_fixed_width_per_branch` passes (104-test status_line run) |
| 12 | 01-03: The cost segment stays `Dim`, adds no second segment, and `Cost` stays out of `varies_mid_turn` | ✓ VERIFIED | `StatusSegment::dim`; `cost_segment_stays_dim_at_every_value` asserts exactly one segment; `StatusLineItem::Cost => false` (`config.rs:363-375`) |
| 13 | 01-03: Headless `json` and `streaming-messages-json` price from the resolved `[ui.status_line.pricing]` and agree with the TUI float | ✓ VERIFIED | `status_line_pricing()` → `HeadlessEmitter` (`headless.rs:894-912,118-160`), `reducer_for_with_pricing` → `MessagesReducer::pricing`; three tests pass in a 193-test headless run |
| 14 | 01-04: With no preference a session resolves `deepseek-flash` via `default_models.json` in `first_or_fallback` | ✓ VERIFIED | `"default": "deepseek-flash"`; `resolution.rs:92-100`; `catalog_default_is_the_resolved_default` + a 4-test `catalog_default` run pass |
| 15 | 01-04: A hidden or disabled default falls back to the first visible entry (never an unselectable id) | ✓ VERIFIED | `resolution.rs` visible lookup then `visible.first()`; `hidden_catalog_default_falls_back_to_first_visible` and `absent_catalog_default_falls_back_to_first_visible` pass |
| 16 | 01-04: An explicit `[models] default` still wins and picker order is unchanged | ✓ VERIFIED | CLI > env > config precedes `first_or_fallback`; JSON array order untouched (single-line `default` change) |
| 17 | 01-04: Shipped docs state the Flash default, the 63/63 at-ceiling caveat and both revert spellings | ✓ VERIFIED | README:247-255, `11-custom-models.md:9-11`, `05-configuration.md:51`, `docs/TUNING.md:39-50` name `deepseek-flash`, 63/63, `/model deepseek-v4-pro`, `-m deepseek-v4-pro` |
| 18 | 01-05: `test-gate.sh` refuses below 150 GB, serialises on the build lock, and runs both cargo steps with the three key vars unset | ✓ VERIFIED | `test-gate.sh:55-65` (disk-guard 150, flock re-exec), `:148-166` (`env -u` ×3); independent subset run printed both disk-guard lines and ran under the lock |
| 19 | 01-05: Completeness is asserted arithmetically — `results == binaries + doctests == expected` — and never read from the tail | ✓ VERIFIED | `test-gate.sh:175-263`; recorded run: `results=308 (raw 309) binaries=219 doctests=89`, `308 == 308`, `expected (308) PASS`; independent subset run: `9 == 7 + 2` |
| 20 | 01-05: The expected count only moves via `--refresh-expected`, which writes only after a green run with commit/date/three numbers | ✓ VERIFIED | `test-gate.sh:265-293`; `test-gate.expected` carries commit `c159a3e7`, dirty-tree marker, date, 308/219/89; review harness reproduced clean/dirty branches; red refresh refuses |
| 21 | 01-05: `cargo test -p xai-grok-shell --lib --features config-docs` is a named gate step, and the two env-global shell tests share one serial key | ✓ VERIFIED | `test-gate.sh:160-173`; independent subset run step 2 PASS (6201/0/4 recorded); bare `#[serial_test::serial]` on both tests (`folder_trust.rs`, `consent_tests.rs`) |
| 22 | 01-05: The `git_gate` / `restore_fetch` timing tests have headroom and a recorded five-run repetition shows no rotation (TEST-03) | ✗ FAILED | 35 repetitions of the phase's own command produced 2 failures; captured: `restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild` panic at `restore_fetch_tests.rs:731` (`grandchild ... must not remain live`). File untouched by the phase; no quarantine. See gaps |
| 23 | 01-05: `docs/evidence/` holds a cast + fixed-schema `report.json` per Phase-1 item with absolute paths normalized out, and the convention is documented | ✓ VERIFIED | 3 items (test-gate, ipv6-report, cost-estimate); 3 valid asciinema-v2 casts (12/8/774 events); 3 exact 8-key reports, `result: ok`, all steps ok; 4/4 recorded sha256 match; no `/home`, `/tmp` or username in report.json/casts; `docs/evidence/README.md` documents layout, keys, normalization and re-run recipes |
| 24 | 01-05: `scripts/key-leak-check.sh` can scan `docs/evidence` without a release binary and passes | ✓ VERIFIED | `--scan` mode additive and fail-closed on grep rc≥2; gate step 3 PASS in the recorded run and in both verifier runs |

**Score:** 23/24 truths verified (0 present, behavior-unverified)

### Deferred Items

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | `scripts/check-egress.sh --strict` exits 0 (currently red on three committed fixtures) | Phase 5 (FREEZE-03) | Phase 5 SC 4 requires the checklist to include "`check-egress.sh --strict` exit 0 with a dated record"; `deferred-items.md` (01-02) names the three fixture hits; `TESTING.md` step 11 records the known red baseline |

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs` | pure classifier + traversability probe | ✓ VERIFIED | `EndpointError`, `classify_endpoint_error`, `parent_is_traversable`; consumed by `materialize_runtime_socket_deny_paths_from` |
| `crates/codegen/xai-grok-sandbox/src/runtime_sockets_tests.rs` | fixture matrix | ✓ VERIFIED | 6-fixture matrix + pure table; runs in the 382-test lib suite |
| `crates/codegen/xai-grok-sandbox/tests/deny_paths_e2e.rs` | retargeted devbox genuine scenario, new unsearchable-parent scenario | ✓ VERIFIED | 14/0/1 executed; enforcement-required runs pass |
| `scripts/egress_guard.c` | checked-in shim, AF_UNSPEC family-tagged allowlist | ✓ VERIFIED | 198 lines; compiled by `sandbox-run.sh`; exercised by the harness |
| `scripts/test-sandbox-run.sh` | offline six-assertion harness | ✓ VERIFIED | exit 0, `ALL PASS (6 assertions)` |
| `crates/codegen/xai-grok-status-line/src/pricing.rs` | Flash alias constant + `ModelPricing::flash()` | ✓ VERIFIED | rates 0.006/0.30/1.20; exact worked-example tests |
| `crates/codegen/xai-grok-pager/src/views/status_line/segments.rs` | `MIN_DISPLAYED_COST_USD`, estimate formatter | ✓ VERIFIED | `format_cost`, stable per-branch width, `~$… est.` |
| `crates/codegen/xai-grok-pager/src/headless.rs` | resolved pricing table on `HeadlessEmitter` | ✓ VERIFIED | `new_with_pricing` + `status_line_pricing`; both output paths |
| `crates/codegen/xai-grok-models/default_models.json` | `"default": "deepseek-flash"` | ✓ VERIFIED | line 2 |
| `crates/codegen/xai-grok-shell/src/agent/model_catalog/resolution.rs` | catalog default preferred in `first_or_fallback` | ✓ VERIFIED | visible `default_model()` lookup before `visible.first()` |
| `crates/codegen/xai-grok-shell/src/session/acp_session_impl/context_snapshot.rs` | pins `deepseek-flash` | ✓ VERIFIED | assertion at line 376 |
| `README.md`, `05-configuration.md`, `11-custom-models.md`, `docs/TUNING.md` | default + caveat + revert path | ✓ VERIFIED | 63/63 caveat and both spellings present |
| `scripts/test-gate.sh`, `scripts/test-gate.expected` | counted gate + checked-in count | ✓ VERIFIED | 325-line script; expected 308/219/89 with provenance |
| `scripts/evidence.py`, `docs/evidence/README.md` | evidence helper + convention | ✓ VERIFIED | stdlib-only; 8-key schema, normalization, fail-closed finish |
| `docs/evidence/01-truth-baseline/{test-gate,ipv6-report,cost-estimate}/` | cast + report + notes | ✓ VERIFIED | 3/3 schema-exact, hashes match |
| `scripts/key-leak-check.sh` `--scan` | additive, release-binary-free | ✓ VERIFIED | rc 0/1/else fail-closed; gate step 3 |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `materialize_runtime_socket_deny_paths_from` | bwrap deny handoff | `append_socket_denies` → `resolve_profile_with_runtime_sockets` → `bwrap_deny_plan` | WIRED | e2e subprocess asserts the skipped `/.docker/` candidate is absent from args **and** `__GROK_BWRAP_RUNTIME_SOCKET_DENY` |
| `getaddrinfo(AF_UNSPEC)` | blocked report | family-tagged allowlist → `connect` match → `format_destination` | WIRED | harness assertions 1–5 over the real shim |
| `scripts/sandbox-run.sh` | the binary | `cc scripts/egress_guard.c` → `LD_PRELOAD` → `exec` | WIRED | harness assertion 6 runs the real script end to end with `BIN=/bin/echo` |
| `[ui.status_line.pricing]` | TUI cost row | `StatusLineConfig::pricing()` → `AppView::fill_estimated_cost` | WIRED | `config.rs:200-205`; `status_line_policy.rs:275-291`; cross-surface test |
| `[ui.status_line.pricing]` | headless estimate | `status_line_pricing(raw_config)` → `HeadlessEmitter::new_with_pricing` → `attach_cost_estimate` | WIRED | `headless.rs:894-912`; override tests pass |
| `HeadlessEmitter::new_with_pricing` | streaming-messages reducer | `reducer_for_with_pricing` → `MessagesReducer::estimated_cost_usd` | WIRED | `reducer/mod.rs`, `messages/usage.rs`; test passes |
| `default_models.json` `default` | session model id | `default_model()` → `resolve_default_model` first_or_fallback → `ModelsManager` | WIRED | 4 resolver tests pass; `include_str!` compile-time source |
| build graph executables | gate count equality | `cargo test --no-run --message-format=json` → `jq` → unique executables | WIRED | recorded 219 + 89 = 308; independent subset 7 + 2 = 9 |
| `docs/evidence` | leak assertion | `key-leak-check.sh --scan` as gate step 3 | WIRED | PASS in recorded and verifier runs |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| TUI status row | `ctx.cost.total_cost_usd` | live session usage + model id → resolved pricing table | Yes (provider-reported cost wins; else local estimate) | ✓ FLOWING |
| Headless `json` result | `estimated_cost_usd` | `_meta.usage` buckets + resolved table | Yes | ✓ FLOWING |
| `streaming-messages-json` result | `estimated_cost_usd` | reducer usage projection + injected table | Yes | ✓ FLOWING |
| Session default model | initial model id | `default_models.json` → `default_model()` → resolver | Yes | ✓ FLOWING |
| Gate verdict | `results`, `binaries`, `doctests` | real cargo log + build graph | Yes (no static fallback) | ✓ FLOWING |
| Evidence report | `steps[].ok`, `result` | `evidence.py step` real exit codes | Yes (refuses `ok` over a failed step) | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| IPv6 refusal names the destination | `sh scripts/test-sandbox-run.sh` | `ALL PASS (6 assertions)`, exit 0 | ✓ PASS |
| Sandbox e2e unmodified | `./target/debug/deps/deny_paths_e2e-613bbf…` (14 tests) | 14 passed / 0 failed / 1 ignored | ✓ PASS |
| Key e2e tests under required enforcement | same binary, `SANDBOX_E2E_REQUIRE_ENFORCEMENT=1`, `--exact` on the two new scenarios | pass (no skip panic) | ✓ PASS |
| Sandbox fixture matrix | `cargo test -p xai-grok-sandbox --lib` | 382 passed / 0 failed | ✓ PASS |
| Flash pricing | `cargo test -p xai-grok-status-line --lib pricing` | 8 passed / 0 failed | ✓ PASS |
| TUI cost rendering | `cargo test -p xai-grok-pager --lib -- status_line` | 104 passed / 0 failed incl. `cost_renders_the_estimate_at_fixed_width_per_branch` | ✓ PASS |
| Headless/TUI cost equality | `cargo test -p xai-grok-pager --lib -- headless` | 193 passed / 0 failed incl. all three override/equality tests | ✓ PASS |
| Default model resolver | `cargo test -p xai-grok-shell --lib -- catalog_default` | 4 passed / 0 failed | ✓ PASS |
| Gate mechanism | `scripts/test-gate.sh --package xai-grok-status-line --package xai-grok-sandbox` | exit 0; `9 == 7 + 2`; step 2 config-docs PASS; leak scan PASS | ✓ PASS |
| Determinism repetition (TEST-03) | 35 × `cargo test -p xai-grok-workspace --lib -- --test-threads=4` | 33 green, **2 failed**; captured `restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild` (`:731`) | ✗ FAIL |
| Evidence schema/hashes/casts | Python check over `docs/evidence/01-truth-baseline/*` | 3/3 exact 8 keys; 4/4 sha256 match; casts 12/8/774 events, v2 | ✓ PASS |
| Scripts syntax/lint | `bash -n`, `sh -n`, `shellcheck --shell=sh` (3 scripts), `ast.parse` | all exit 0 | ✓ PASS |
| Full workspace gate | recorded run #3 (`docs/evidence/…/full-gate-confirmation-run-3.log` + `test-gate.log`) | 31,913 passed / 0 failed / 386 ignored; 6,201 / 0 / 4; `GATE PASS` | ✓ PASS (recorded; not re-run per dispatch) |

### Probe Execution

No `probe-*.sh` is declared by any Phase-1 plan, and the phase is not a migration/tooling probe phase. Its script harnesses (`test-sandbox-run.sh`, `test-gate.sh`) were executed directly above; **N/A**.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| SBOX-01 | 01-01 | Non-traversable deny parents skipped; fail-closed elsewhere | ✓ MET | Truths 1–2; classifier table; 382-test lib suite |
| SBOX-02 | 01-01, 01-02 | Root-only deny parent no longer aborts bwrap; `sandbox-run.sh` reports IPv6 | ✓ MET | Truths 3–9; e2e 14/0; harness 6/6 |
| TEST-01 | 01-05 | Workspace suite exits 0 with zero failures, completeness-asserted | ✓ MET | Truth 19; recorded 308 = 219 + 89, 31,913/0; independent subset 9 = 7 + 2 |
| TEST-02 | 01-05 | `config-docs` feature tests run as a named gate step | ✓ MET | Truth 21; step 2 named and PASS (6,201/0/4) |
| TEST-03 | 01-05 | `git_gate`/`restore_fetch` flake deterministic or explicitly quarantined | ✗ NOT MET | Truth 22; 2 failures in 35 repetitions; no quarantine; five-run evidence uncommitted |
| EVID-01 | 01-05 | Committed cast + `report.json` per item, absolute artifact paths normalized | ✓ MET | Truth 23; 3 items, schema-exact, hashes match, no absolute paths |
| EVID-02 | 01-05 | Convention documented and re-runnable | ✓ MET | `docs/evidence/README.md` layout/keys/normalization/re-run recipes; each item has `notes.md`; recipes exercised |
| COST-01 | 01-03 | `deepseek-v4-flash` priced at Flash rates | ✓ MET | Truth 10; `pricing.rs`; 8 pricing tests |
| COST-02 | 01-03 | TUI cost labelled as an estimate | ✓ MET | Truths 11–12; format tests; evidence item |
| COST-03 | 01-03 | Headless honours `[ui.status_line.pricing]` | ✓ MET | Truth 13; both output paths; equality test |
| DEF-01 | 01-04 | `deepseek-flash` catalog default with caveat and revert path | ✓ MET | Truths 14–17; JSON default; resolver tests; four docs |

**Coverage:** 11/11 Phase-1 requirement IDs accounted for (5 plans declare all 11; no orphaned Phase-1 ID in REQUIREMENTS.md). 10 met, 1 not met.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `crates/codegen/xai-grok-pager/src/views/status_line/segments.rs` | 68 | `TODO(auto-routing)` | ⚠️ Warning | Pre-existing at `711524dd` (line 66), not introduced by this phase; references Phase 3, no issue/DEF id. No functional impact |
| — | — | No `TBD` / `FIXME` / `XXX` markers in phase-modified files | ℹ️ Info | The `XXXXXX` mktemp template in `test-sandbox-run.sh:88` is a false positive, not a debt marker |
| — | — | No stub returns, empty handlers, or hardcoded empty data in the phase's changes | ℹ️ Info | Changes are substantive and wired (see key links) |
| `scripts/test-gate.expected` | 2-4 | Provenance commit `c159a3e7` ≠ HEAD `1308903a` | ℹ️ Info | Header honestly says `# tree: dirty`; the three numbers are re-derivable from the committed log and the final commit changed no test code; independent subset run reproduces the mechanism |

### Human Verification Required

These are the two VALIDATION.md manual-only items plus the evidence-docs read. They do not change the status (the failed truth already forces `gaps_found`), and none is a Phase-1 blocker on its own.

1. **Live TUI estimate legibility (COST-02)**
   - **Test:** start the TUI on a throwaway repo, run one Flash turn, watch the `cost` item.
   - **Expected:** `~$… est.` fits the row and does not reflow or jitter as the number grows.
   - **Why human:** the exact strings are unit-pinned, but live legibility at the terminal width is a visual judgement.

2. **At-ceiling caveat and revert path read correctly (DEF-01)**
   - **Test:** read `README.md`, `11-custom-models.md`, `05-configuration.md`, `docs/TUNING.md`.
   - **Expected:** the caveat names 63/63 for both models and both revert spellings (`/model deepseek-v4-pro`, `-m deepseek-v4-pro`).
   - **Why human:** prose quality and clarity are a review judgement; the strings are present and accurate by grep.

3. **Evidence convention re-runnability (EVID-02)**
   - **Test:** follow `docs/evidence/README.md` "Re-running the Phase-1 items" from a clean clone, plus one item's `notes.md`.
   - **Expected:** each recipe reproduces its item without reading the cast or trusting the report.
   - **Why human:** the interpreter ran the same commands, but "documented well enough for a later reader" is a judgement on the prose.

### Carry-Forward (not a Phase 1 gap)

- **`scripts/check-egress.sh --strict` is red** on three committed fixtures (`config_tests.rs:2768`, `xai-grok-shell-base/src/util/mod.rs:422-423`). This is not a Phase-1 success criterion; Phase 5 / FREEZE-03 owns it. Recorded in `deferred-items.md`, `TESTING.md` step 11, and the `deferred` frontmatter above.
- **`.planning/codebase/CONCERNS.md`** still describes the fixed `non-ip` behaviour (from 01-02's deferred items); a codebase-map snapshot, not hand-edited mid-phase.
- **Minor SUMMARY inaccuracies (informational, not goal-level):** 01-01-SUMMARY records 383 lib tests and 13 e2e passes (observed 382 and 14); 01-05-SUMMARY's `key-files` lists `restore_fetch_tests.rs` as modified while its body correctly says untouched (the file is byte-identical to HEAD).

### Gaps Summary

Ten of the eleven Phase-1 requirements are achieved: the sandbox no longer aborts on a root-owned/non-traversable deny parent while staying fail-closed, `sandbox-run.sh` names IPv6 destinations, the Flash alias prices at Flash rates, the TUI estimate is labelled, headless honours the pricing overrides, `deepseek-flash` is the catalog default with the caveat and revert path documented, and the evidence convention is committed with three schema-exact items and a leak scan.

**TEST-03 is not achieved.** The phase's determinism claim for the `git_gate` / `restore_fetch` flake does not survive repetition. Running the phase's own command — `cargo test -p xai-grok-workspace --lib -- --test-threads=4` — 35 times on HEAD `1308903a` produced **two failing runs**; the captured one is:

```
---- restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild stdout ----
thread '…' panicked at crates/codegen/xai-grok-workspace/src/restore_fetch_tests.rs:731:5:
grandchild 706149 must not remain live
```

`restore_fetch_tests.rs` was never modified (0 commits in `711524dd..HEAD`, sha256 identical to HEAD), no `#[ignore]` quarantine exists, and the five-run repetition is marked "reported, not artifact-backed (prior session)" in `KNOWN-ISSUES.md`. This is precisely the "zero failures is a single lucky run" outcome TEST-03 and the phase goal were meant to eliminate. The recorded full-gate green (31,913/0/386) remains valid as a run, but it does not certify stability.

Everything else is ready; one focused fix (bounded wait/poll for the grandchild to disappear, or the plan's pure `head_timeout` fallback, or an explicit named quarantine) closes the phase.

---

_Verified: 2026-11-23T17:18:28Z_
_Verifier: the agent (gsd-verifier)_
