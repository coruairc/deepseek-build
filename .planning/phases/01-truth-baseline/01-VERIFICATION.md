---
phase: 01-truth-baseline
verified: 2026-11-23T20:22:00Z
status: passed
score: 24/24 must-haves verified
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
  - ".planning/phases/01-truth-baseline/01-06-PLAN.md"
  - ".planning/phases/01-truth-baseline/01-06-SUMMARY.md"
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
  - "crates/codegen/xai-grok-workspace/src/restore_fetch.rs"
  - "crates/codegen/xai-grok-workspace/src/restore_fetch_tests.rs"
  - "crates/codegen/xai-grok-workspace/src/session/git_gate_tests.rs"
  - "docs/TUNING.md"
  - "docs/evidence/01-truth-baseline/cost-estimate/both-suites-report-test-result-ok.log"
  - "docs/evidence/01-truth-baseline/cost-estimate/cost-estimate.cast"
  - "docs/evidence/01-truth-baseline/cost-estimate/notes.md"
  - "docs/evidence/01-truth-baseline/cost-estimate/report.json"
  - "docs/evidence/01-truth-baseline/cost-estimate/script-t-capture-status-line-headless-suites.log"
  - "docs/evidence/01-truth-baseline/cost-estimate/source-pins-the-four-rendered-strings.log"
  - "docs/evidence/01-truth-baseline/cost-estimate/typescript-ran-the-pinned-cost-tests.log"
  - "docs/evidence/01-truth-baseline/determinism/all-35-runs-exited-zero.log"
  - "docs/evidence/01-truth-baseline/determinism/all-runs-report-one-passed-count.log"
  - "docs/evidence/01-truth-baseline/determinism/determinism.cast"
  - "docs/evidence/01-truth-baseline/determinism/every-run-exercised-the-named-test.log"
  - "docs/evidence/01-truth-baseline/determinism/every-run-s-summary-is-ok-with-zero-failed.log"
  - "docs/evidence/01-truth-baseline/determinism/leak-scan-over-docs-evidence.log"
  - "docs/evidence/01-truth-baseline/determinism/notes.md"
  - "docs/evidence/01-truth-baseline/determinism/post-fix-repetition-35-full-crate-runs.log"
  - "docs/evidence/01-truth-baseline/determinism/repetition-35-runs.log"
  - "docs/evidence/01-truth-baseline/determinism/report.json"
  - "docs/evidence/01-truth-baseline/determinism/targeted-repetition-20-named-test-runs.log"
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
covered_digest: "v4:sha256:d24b4ffc15b8b3ae5f4eb3b87caa5f5bb34103262bbc3172a31d7ff2b287a082"
behavior_unverified: 0
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: 23/24
  gaps_closed:
    - "The `git_gate` / `restore_fetch` timing tests have headroom on every under-margin wait, with ordering invariants unchanged, and a recorded five-run repetition shows no rotation (TEST-03)."
  gaps_remaining: []
  regressions: []
deferred:
  - truth: "`scripts/check-egress.sh --strict` exits 0 over the tree (zero HARD hits)"
    addressed_in: "Phase 5 (FREEZE-03)"
    evidence: "Phase 5 success criterion 4 requires the pre-freeze checklist to pass '`check-egress.sh --strict` exit 0 with a dated record'; 01-02 deferred-items.md records the three committed fixture hits (config_tests.rs:2768, xai-grok-shell-base/src/util/mod.rs:422-423) as rejected-string fixtures; TESTING.md step 11 records the known red baseline. Not a Phase 1 success criterion."
---

# Phase 1: Truth Baseline Verification Report

**Phase Goal:** As the author, I want the sandbox to stop aborting on this host, the cost figure and the default model to stop lying, and every claim to carry a committed artifact, so that a green suite actually means something.
**Verified:** 2026-11-23T20:22:00Z
**Status:** passed
**Re-verification:** Yes — after the TEST-03 gap closure (01-06, commits `ad4d5e8a`/`fd6fdd3f`) and the review wording fixes (`5b4c30c4`, bookkeeping `c574a9f2`)

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | 01-01: A runtime-socket candidate whose parent is not traversable is skipped in both shapes (ancestor EACCES; unsearchable last component) | ✓ VERIFIED | `classify_endpoint_error` + `parent_is_traversable` (`runtime_sockets.rs:116-136`) consulted at `:167-203`; `unsearchable_parent_directory_is_skipped` (`runtime_sockets_tests.rs:521`) and `unsearchable_ancestor_is_skipped` (`:552`); `cargo test -p xai-grok-sandbox --lib` → 382 passed / 0 failed (previous verification; those files unchanged) |
| 2 | 01-01: Every other resolution error stays fatal (symlink, missing parent with live endpoint, LSM denial under a traversable parent, no file name) | ✓ VERIFIED | Pure table pins `PermissionDenied + traversable → Fatal` (`runtime_sockets_tests.rs:703-723`); `symlink_endpoint_is_fatal`; `materialized_socket_paths_propagate_parent_resolution_errors`; `unreadable_endpoint_under_traversable_parent_is_materialized` (previous verification; unchanged) |
| 3 | 01-01: A candidate inside a mode-000 parent no longer aborts bwrap startup, and `deny_paths_e2e` passes unmodified — no `#[ignore]`, no new skip predicate | ✓ VERIFIED | Full integration binary run: `14 passed; 0 failed; 1 ignored` (the one ignore is the pre-existing `subprocess_entry` harness entry); diff `0bf46c84` adds no `#[ignore]` and leaves `skip_if_enforcement_unavailable` untouched (previous verification; unchanged) |
| 4 | 01-01: `devbox_genuine_reexec_applies_enforcement` gives re-exec a real obligation on every host | ✓ VERIFIED | Diff retargets to `[profiles.genuine] extends = "devbox" deny = ["secret.pem"]`; run with `SANDBOX_E2E_REQUIRE_ENFORCEMENT=1` passes (host has no `/data`, uid 1000) (previous verification; unchanged) |
| 5 | 01-02: `sandbox-run.sh` resolves the allow host over both families and refuses an IPv6 destination by naming it, never `non-ip` | ✓ VERIFIED | `scripts/egress_guard.c` `getaddrinfo(AF_UNSPEC)` (`:108`), family-tagged allowlist, `inet_ntop` destination formatting (`:87-93`); harness assertions 1–2 PASS (previous verification; unchanged) |
| 6 | 01-02: An allowlisted IPv6 address and IPv6 loopback are permitted | ✓ VERIFIED | `sh scripts/test-sandbox-run.sh` assertion 3 PASS (`::1` and `::ffff:127.0.0.1`), assertion 4 PASS (IPv6 allowlist + v4-mapped spelling) (previous verification; unchanged) |
| 7 | 01-02: `::ffff:1.2.3.4` and `1.2.3.4` are one allowlist entry; IPv4 behaviour unchanged | ✓ VERIFIED | v4-mapped normalized to 4-byte form on store and match (`egress_guard.c:64-75,120-141`); assertions 4–5 PASS (previous verification; unchanged) |
| 8 | 01-02: `sh scripts/test-sandbox-run.sh` proves the above offline and exits non-zero on any failure | ✓ VERIFIED | Run twice (direct and via gate): `ALL PASS (6 assertions)`, exit 0; script tracks failures and exits 1 (previous verification; unchanged) |
| 9 | 01-02: `scripts/egress-check.sh` prose no longer describes the fixed `non-ip` bug | ✓ VERIFIED | `scripts/egress-check.sh:548` now states the refused address is named in its own family's notation and `non-ip` only covers a non-IP family (previous verification; unchanged) |
| 10 | 01-03: A `deepseek-v4-flash` turn prices at Flash rates — worked example totals 0.001206 USD | ✓ VERIFIED | `FLASH_ALIAS_PRICING_MODEL` + `ModelPricing::flash()` (`pricing.rs:36,69-79,91`); `cargo test -p xai-grok-status-line --lib pricing` → 8 passed, including `flash_alias_worked_example_is_exact` (previous verification; unchanged) |
| 11 | 01-03: The TUI cost item renders `~$0.0042 est.`, `~$0.37 est.`, `~$0.01 est.` and omits below $0.0001 | ✓ VERIFIED | `MIN_DISPLAYED_COST_USD = 0.0001` and `format_cost` (`segments.rs:20,141-155`); `cost_renders_the_estimate_at_fixed_width_per_branch` passes (104-test status_line run) (previous verification; unchanged) |
| 12 | 01-03: The cost segment stays `Dim`, adds no second segment, and `Cost` stays out of `varies_mid_turn` | ✓ VERIFIED | `StatusSegment::dim`; `cost_segment_stays_dim_at_every_value` asserts exactly one segment; `StatusLineItem::Cost => false` (`config.rs:363-375`) (previous verification; unchanged) |
| 13 | 01-03: Headless `json` and `streaming-messages-json` price from the resolved `[ui.status_line.pricing]` and agree with the TUI float | ✓ VERIFIED | `status_line_pricing()` → `HeadlessEmitter` (`headless.rs:903-938`), `reducer_for_with_pricing` → `MessagesReducer::pricing`; three tests pass in a 193-test headless run (previous verification; unchanged) |
| 14 | 01-04: With no preference a session resolves `deepseek-flash` via `default_models.json` in `first_or_fallback` | ✓ VERIFIED | `"default": "deepseek-flash"` (line 2); `resolution.rs:92-100`; `catalog_default_is_the_resolved_default` + a 4-test `catalog_default` run pass (previous verification; unchanged) |
| 15 | 01-04: A hidden or disabled default falls back to the first visible entry (never an unselectable id) | ✓ VERIFIED | `resolution.rs` visible lookup then `visible.first()`; `hidden_catalog_default_falls_back_to_first_visible` and `absent_catalog_default_falls_back_to_first_visible` pass (previous verification; unchanged) |
| 16 | 01-04: An explicit `[models] default` still wins and picker order is unchanged | ✓ VERIFIED | CLI > env > config precedes `first_or_fallback`; JSON array order untouched (single-line `default` change) (previous verification; unchanged) |
| 17 | 01-04: Shipped docs state the Flash default, the 63/63 at-ceiling caveat and both revert spellings | ✓ VERIFIED | README:247-255, `11-custom-models.md:9-11`, `05-configuration.md:51`, `docs/TUNING.md:39-50` name `deepseek-flash`, 63/63, `/model deepseek-v4-pro`, `-m deepseek-v4-pro` (previous verification; unchanged) |
| 18 | 01-05: `test-gate.sh` refuses below 150 GB, serialises on the build lock, and runs both cargo steps with the three key vars unset | ✓ VERIFIED | `test-gate.sh:55-65` (disk-guard 150, flock re-exec), `:148-166` (`env -u` ×3); independent subset run printed both disk-guard lines and ran under the lock (previous verification; unchanged) |
| 19 | 01-05: Completeness is asserted arithmetically — `results == binaries + doctests == expected` — and never read from the tail | ✓ VERIFIED | `test-gate.sh:175-263`; recorded run: `results=308 (raw 309) binaries=219 doctests=89`, `308 == 308`, `expected (308) PASS`; independent subset run: `9 == 7 + 2` (previous verification; unchanged) |
| 20 | 01-05: The expected count only moves via `--refresh-expected`, which writes only after a green run with commit/date/three numbers | ✓ VERIFIED | `test-gate.sh:265-293`; `test-gate.expected` carries commit `c159a3e7`, dirty-tree marker, date, 308/219/89; review harness reproduced clean/dirty branches; red refresh refuses (previous verification; unchanged) |
| 21 | 01-05: `cargo test -p xai-grok-shell --lib --features config-docs` is a named gate step, and the two env-global shell tests share one serial key | ✓ VERIFIED | `test-gate.sh:160-173`; independent subset run step 2 PASS (6201/0/4 recorded); bare `#[serial_test::serial]` on both tests (`folder_trust.rs`, `consent_tests.rs`) (previous verification; unchanged) |
| 22 | 01-05/01-06: The `git_gate` / `restore_fetch` timing flake is made deterministic — every process-death assertion observes death through a deadline-bounded poll, the poll's bound is unit-pinned, and a committed 35/35 post-fix repetition shows zero rotation (TEST-03) | ✓ VERIFIED | `wait_until_dead` (`restore_fetch_tests.rs:623-643`, `PROCESS_DEATH_DEADLINE = 5 s`, `PROCESS_DEATH_POLL_INTERVAL = 20 ms`) at all five sites (`:589`, `:593`, `:756`, `:793`, `:802`); original messages verbatim; `<2 s`/`<5 s` prompt-return bounds still evaluated before any poll; `wait_until_dead_is_bounded_and_deadline_respecting` (`:654`) passes; no `#[ignore]` in the file; `restore_fetch.rs` sha256 `7bc753de…` unchanged. Evidence: `docs/evidence/01-truth-baseline/determinism/report.json` 8 keys, `requirements: [TEST-03]`, 7/7 steps ok, `result: ok`, 4/4 artifact sha256 match; `repetition-35-runs.log` 35/35 `rc=0 named=ok` with `test result: ok. 1695 passed; 0 failed`, one distinct passed count; valid v2 cast (3 events) carries all 35 lines; all 35 raw logs present and matching. Independent verifier repetition: 10/10 exact runs + 3/3 full-crate runs green |
| 23 | 01-05/01-06: `docs/evidence/` holds a cast + fixed-schema `report.json` per Phase-1 item with absolute paths normalized out, and the convention is documented | ✓ VERIFIED | 4 items (test-gate, ipv6-report, cost-estimate, determinism); 4 valid asciinema-v2 casts (12/8/774/3 events); 4 exact 8-key reports, `result: ok`, all steps ok; all recorded artifact sha256 match (4/4 determinism, 4/4 prior); no `/home`, `/tmp` or username in `report.json`/casts; `docs/evidence/README.md` documents layout, keys, normalization and re-run recipes including determinism |
| 24 | 01-05: `scripts/key-leak-check.sh` can scan `docs/evidence` without a release binary and passes | ✓ VERIFIED | `--scan` mode additive and fail-closed on grep rc≥2; gate step 3 PASS in the recorded run; re-run this verification over the 4-item tree: `PASS  no key-shaped string in: docs/evidence`, exit 0 |

**Score:** 24/24 truths verified (0 present, behavior-unverified)

### Deferred Items

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | `scripts/check-egress.sh --strict` exits 0 (currently red on three committed fixtures) | Phase 5 (FREEZE-03) | Phase 5 SC 4 requires the checklist to include "`check-egress.sh --strict` exit 0 with a dated record"; `deferred-items.md` (01-02) names the three fixture hits; `TESTING.md` step 11 records the known red baseline |

### Advisory (New Scope, Unevidenced)

None. The only findings from this re-verification are the carried-forward and informational items listed under Carry-Forward; none is a new-scope blocker without evidence.

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
| `scripts/test-gate.sh`, `scripts/test-gate.expected` | counted gate + checked-in count | ✓ VERIFIED | 325-line script; expected 308/219/89 with provenance; unchanged by 01-06 (target count, not test count) |
| `scripts/evidence.py`, `docs/evidence/README.md` | evidence helper + convention | ✓ VERIFIED | stdlib-only; 8-key schema, normalization, fail-closed finish; determinism re-run recipe added |
| `docs/evidence/01-truth-baseline/{test-gate,ipv6-report,cost-estimate,determinism}/` | cast + report + notes | ✓ VERIFIED | 4/4 schema-exact, hashes match, casts v2 |
| `crates/codegen/xai-grok-workspace/src/restore_fetch_tests.rs` | bounded death poll, five sites, bounds test | ✓ VERIFIED | `wait_until_dead` + constants + `wait_until_dead_is_bounded_and_deadline_respecting`; +75/−5 lines; no `#[ignore]` |
| `crates/codegen/xai-grok-workspace/src/restore_fetch.rs` | production teardown untouched | ✓ VERIFIED | sha256 `7bc753de81e646fbe76e7c1052113998b9a0551c49d1d0f660b1f7e7acd3c8aa` identical to HEAD |
| `docs/evidence/01-truth-baseline/determinism/` | cast + 8-key report + projection log + notes | ✓ VERIFIED | 11 committed files; 35/35 lines; 7/7 steps ok; 4 sha256 match; no absolute home paths |
| `KNOWN-ISSUES.md`, `TESTING.md`, `.planning/phases/01-truth-baseline/01-05-SUMMARY.md` | 2-in-35 falsification + fix + evidence path | ✓ VERIFIED | all three cite the determinism item; five-run claim superseded; second-failure identity correctly recorded as not captured; no artifact-free determinism claim remains |
| `scripts/key-leak-check.sh` `--scan` | additive, release-binary-free | ✓ VERIFIED | rc 0/1/else fail-closed; gate step 3; re-run PASS this verification |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `materialize_runtime_socket_deny_paths_from` | bwrap deny handoff | `append_socket_denies` → `resolve_profile_with_runtime_sockets` → `bwrap_deny_plan` | WIRED | e2e subprocess asserts the skipped `/.docker/` candidate is absent from args **and** `__GROK_BWRAP_RUNTIME_SOCKET_DENY` |
| `getaddrinfo(AF_UNSPEC)` | blocked report | family-tagged allowlist → `connect` match → `format_destination` | WIRED | harness assertions 1–5 over the real shim |
| `scripts/sandbox-run.sh` | the binary | `cc scripts/egress_guard.c` → `LD_PRELOAD` → `exec` | WIRED | harness assertion 6 runs the real script end to end with `BIN=/bin/echo` |
| `[ui.status_line.pricing]` | TUI cost row | `StatusLineConfig::pricing()` → `AppView::fill_estimated_cost` | WIRED | `config.rs:200-205`; `status_line_policy.rs:275-291`; cross-surface test |
| `[ui.status_line.pricing]` | headless estimate | `status_line_pricing(raw_config)` → `HeadlessEmitter::new_with_pricing` → `attach_cost_estimate` | WIRED | `headless.rs:903-938`; override tests pass |
| `HeadlessEmitter::new_with_pricing` | streaming-messages reducer | `reducer_for_with_pricing` → `MessagesReducer::estimated_cost_usd` | WIRED | `reducer/mod.rs`, `messages/usage.rs`; test passes |
| `default_models.json` `default` | session model id | `default_model()` → `resolve_default_model` first_or_fallback → `ModelsManager` | WIRED | 4 resolver tests pass; `include_str!` compile-time source |
| build graph executables | gate count equality | `cargo test --no-run --message-format=json` → `jq` → unique executables | WIRED | recorded 219 + 89 = 308; independent subset 7 + 2 = 9 |
| `docs/evidence` | leak assertion | `key-leak-check.sh --scan` as gate step 3 | WIRED | PASS in recorded, previous-verifier and this re-verifier run |
| `wait_until_dead` | `pid_alive` | loop: sleep 20 ms while alive, `false` at the 5 s deadline | WIRED | All five process-death sites call it (`restore_fetch_tests.rs:589,593,756,793,802`); `pid_alive` unchanged |
| repetition driver | committed projection log | per-run raw cargo log → `run NN/35: rc= named=ok <test result>` → `repetition-35-runs.log` | WIRED | 35/35 lines; all match the raw logs' `test result:` lines exactly |
| projection log | `report.json` | four fail-closed `awk` count steps + artifact sha256 | WIRED | 7/7 steps ok; 4 artifact hashes match; `result: ok` |
| `script -T` typescript | `determinism.cast` | `scripts/evidence.py cast` | WIRED | valid v2 header; 3 events; all 35 lines present verbatim |
| corrected documents | determinism item | `KNOWN-ISSUES.md`, `TESTING.md`, `01-05-SUMMARY.md` cite the path | WIRED | grep confirms all three; numbers match the committed log |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| TUI status row | `ctx.cost.total_cost_usd` | live session usage + model id → resolved pricing table | Yes (provider-reported cost wins; else local estimate) | ✓ FLOWING |
| Headless `json` result | `estimated_cost_usd` | `_meta.usage` buckets + resolved table | Yes | ✓ FLOWING |
| `streaming-messages-json` result | `estimated_cost_usd` | reducer usage projection + injected table | Yes | ✓ FLOWING |
| Session default model | initial model id | `default_models.json` → `default_model()` → resolver | Yes | ✓ FLOWING |
| Gate verdict | `results`, `binaries`, `doctests` | real cargo log + build graph | Yes (no static fallback) | ✓ FLOWING |
| Evidence report | `steps[].ok`, `result` | `evidence.py step` real exit codes | Yes (refuses `ok` over a failed step) | ✓ FLOWING |
| Determinism projection | `rc`, `named`, `test result:` per line | each run's real cargo log via the driver | Yes (35 distinct raw logs cross-checked; no fabricated projection) | ✓ FLOWING |
| Determinism report steps | four counts + leak scan | fail-closed `awk` over the committed log; real scan exit | Yes (a short or red log fails the counts) | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Exact previously-flaky test (independent repetition) | 10 × `cargo test -p xai-grok-workspace --lib -- --exact restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild` (keys unset, flock) | 10/10 rc=0, each `1 passed; 0 failed` (2.40 s) | ✓ PASS |
| Full workspace-crate suite (independent repetition) | 3 × `cargo test -p xai-grok-workspace --lib -- --test-threads=4` (keys unset, flock) | 3/3 rc=0, each `1695 passed; 0 failed; 0 ignored` (6.04–6.09 s harness) | ✓ PASS |
| Committed 35-run evidence vs raw logs | Python cross-check over `repetition-35-runs.log` and `/tmp/opencode/det-01-06/determinism-raw/run-*.log` | 35 raw logs, 35 distinct sha256, 0 mismatched `test result:` lines, named test ok in every raw log, mtime span 1777 s | ✓ PASS |
| Evidence schema/hashes/casts | Python check over `docs/evidence/01-truth-baseline/*` | 4/4 exact 8 keys; 4/4 reports `result: ok`; 8/8 artifact sha256 match; determinism cast v2 (3 events) carrying all 35 lines; no `/home` path | ✓ PASS |
| Leak scan over evidence | `scripts/key-leak-check.sh --scan docs/evidence` | `PASS  no key-shaped string in: docs/evidence`, exit 0 | ✓ PASS |
| Formatting | `cargo fmt --all --check` | exit 0 | ✓ PASS |
| IPv6 refusal names the destination | `sh scripts/test-sandbox-run.sh` | `ALL PASS (6 assertions)`, exit 0 (previous verification; unchanged) | ✓ PASS |
| Sandbox e2e unmodified | `deny_paths_e2e` binary (14 tests) | 14 passed / 0 failed / 1 ignored (previous verification; unchanged) | ✓ PASS |
| Key e2e tests under required enforcement | same binary, `SANDBOX_E2E_REQUIRE_ENFORCEMENT=1`, `--exact` on the two new scenarios | pass (no skip panic) (previous verification; unchanged) | ✓ PASS |
| Sandbox fixture matrix | `cargo test -p xai-grok-sandbox --lib` | 382 passed / 0 failed (previous verification; unchanged) | ✓ PASS |
| Flash pricing | `cargo test -p xai-grok-status-line --lib pricing` | 8 passed / 0 failed (previous verification; unchanged) | ✓ PASS |
| TUI cost rendering | `cargo test -p xai-grok-pager --lib -- status_line` | 104 passed / 0 failed incl. `cost_renders_the_estimate_at_fixed_width_per_branch` (previous verification; unchanged) | ✓ PASS |
| Headless/TUI cost equality | `cargo test -p xai-grok-pager --lib -- headless` | 193 passed / 0 failed incl. all three override/equality tests (previous verification; unchanged) | ✓ PASS |
| Default model resolver | `cargo test -p xai-grok-shell --lib -- catalog_default` | 4 passed / 0 failed (previous verification; unchanged) | ✓ PASS |
| Gate mechanism | `scripts/test-gate.sh --package xai-grok-status-line --package xai-grok-sandbox` | exit 0; `9 == 7 + 2`; step 2 config-docs PASS; leak scan PASS (previous verification; unchanged) | ✓ PASS |
| Evidence scripts syntax/lint | `bash -n`, `sh -n`, `shellcheck --shell=sh` (3 scripts), `ast.parse` | all exit 0 (previous verification; unchanged) | ✓ PASS |
| Full workspace gate | recorded run #3 (`docs/evidence/…/full-gate-confirmation-run-3.log` + `test-gate.log`) | 31,913 passed / 0 failed / 386 ignored; 6,201 / 0 / 4; `GATE PASS` (recorded; not re-run per dispatch; 01-06 changes one test file, not the target count) | ✓ PASS (recorded) |

### Probe Execution

No `probe-*.sh` is declared by any Phase-1 plan and none exists under `scripts/*/tests/`; the phase is not a migration/tooling probe phase. Its script harnesses (`test-sandbox-run.sh`, `test-gate.sh`) were executed directly; **N/A**.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| SBOX-01 | 01-01 | Non-traversable deny parents skipped; fail-closed elsewhere | ✓ MET | Truths 1–2; classifier table; 382-test lib suite |
| SBOX-02 | 01-01, 01-02 | Root-only deny parent no longer aborts bwrap; `sandbox-run.sh` reports IPv6 | ✓ MET | Truths 3–9; e2e 14/0; harness 6/6 |
| TEST-01 | 01-05 | Workspace suite exits 0 with zero failures, completeness-asserted | ✓ MET | Truth 19; recorded 308 = 219 + 89, 31,913/0; independent subset 9 = 7 + 2 |
| TEST-02 | 01-05 | `config-docs` feature tests run as a named gate step | ✓ MET | Truth 21; step 2 named and PASS (6,201/0/4) |
| TEST-03 | 01-05, 01-06 | `git_gate`/`restore_fetch` flake deterministic or explicitly quarantined | ✓ MET | Truth 22; bounded poll at all five sites + unit-pinned bound; committed 35/35 repetition; 10/10 exact + 3/3 full independent re-runs; no `#[ignore]` |
| EVID-01 | 01-05, 01-06 | Committed cast + `report.json` per item, absolute artifact paths normalized | ✓ MET | Truth 23; 4 items, schema-exact, hashes match, no absolute paths |
| EVID-02 | 01-05 | Convention documented and re-runnable | ✓ MET | `docs/evidence/README.md` layout/keys/normalization/re-run recipes incl. determinism; each item has `notes.md`; recipes exercised |
| COST-01 | 01-03 | `deepseek-v4-flash` priced at Flash rates | ✓ MET | Truth 10; `pricing.rs`; 8 pricing tests |
| COST-02 | 01-03 | TUI cost labelled as an estimate | ✓ MET | Truths 11–12; format tests; evidence item |
| COST-03 | 01-03 | Headless honours `[ui.status_line.pricing]` | ✓ MET | Truth 13; both output paths; equality test |
| DEF-01 | 01-04 | `deepseek-flash` catalog default with caveat and revert path | ✓ MET | Truths 14–17; JSON default; resolver tests; four docs |

**Coverage:** 11/11 Phase-1 requirement IDs accounted for (the six plans declare all 11; no orphaned Phase-1 ID in REQUIREMENTS.md). 11 met, 0 not met.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `crates/codegen/xai-grok-pager/src/views/status_line/segments.rs` | 68 | `TODO(auto-routing)` | ⚠️ Warning | Pre-existing at `711524dd` (line 66), not introduced by this phase; references Phase 3, no issue/DEF id. No functional impact |
| — | — | No `TBD` / `FIXME` / `XXX` markers in files changed by 01-06 (`restore_fetch_tests.rs`, `KNOWN-ISSUES.md`, `TESTING.md`, `docs/evidence/README.md`, determinism item) or in the phase's earlier changes | ℹ️ Info | Debt-marker gate clean; the `XXXXXX` mktemp template in `test-sandbox-run.sh:88` is a false positive |
| — | — | No stub returns, empty handlers, hardcoded empty data, or `#[ignore]` in `restore_fetch_tests.rs` | ℹ️ Info | Changes are substantive and wired; the only test quirk discussed is resolved by polling, not quarantine |
| `scripts/test-gate.expected` | 2-4 | Provenance commit `c159a3e7` ≠ HEAD | ℹ️ Info | Header honestly says `# tree: dirty`; 01-06 adds one test function (target passed count 1694 → 1695), never the target count, so 308/219/89 correctly stays; independent subset run reproduces the mechanism |
| `docs/evidence/01-truth-baseline/determinism/` | — | Raw per-run logs are not committed; the cast buffers output into 3 events; the targeted log drops per-test `… ok` lines | ℹ️ Info | Disclosed in `notes.md` and review IN-01/IN-04; the 35 projection lines + `named=ok` markers remain the durable count proof, and this verifier cross-checked them against the still-present raw logs (35/35 exact match) plus 13 fresh runs |

### Human Verification Required

None is status-gating. Three judgement checks remain from VALIDATION.md; they do not affect goal achievement (the strings/recipes are unit-pinned or committed), so the status stays `passed` per the re-verification dispatch.

1. **Live TUI estimate legibility (COST-02)** — non-blocking
   - **Test:** start the TUI on a throwaway repo, run one Flash turn, watch the `cost` item.
   - **Expected:** `~$… est.` fits the row and does not reflow or jitter as the number grows.
   - **Why human:** the exact strings are unit-pinned, but live legibility at the terminal width is a visual judgement.

2. **At-ceiling caveat and revert path read correctly (DEF-01)** — non-blocking
   - **Test:** read `README.md`, `11-custom-models.md`, `05-configuration.md`, `docs/TUNING.md`.
   - **Expected:** the caveat names 63/63 for both models and both revert spellings (`/model deepseek-v4-pro`, `-m deepseek-v4-pro`).
   - **Why human:** prose quality and clarity are a review judgement; the strings are present and accurate by grep.

3. **Evidence convention re-runnability (EVID-02)** — non-blocking
   - **Test:** follow `docs/evidence/README.md` "Re-running the Phase-1 items" from a clean clone, plus the determinism `notes.md` recipe.
   - **Expected:** each recipe reproduces its item without reading the cast or trusting the report.
   - **Why human:** the interpreter ran the same commands, but "documented well enough for a later reader" is a judgement on the prose.

### Carry-Forward (not a Phase 1 gap)

- **`scripts/check-egress.sh --strict` is red** on three committed fixtures (`config_tests.rs:2768`, `xai-grok-shell-base/src/util/mod.rs:422-423`). This is not a Phase-1 success criterion; Phase 5 / FREEZE-03 owns it. Recorded in `deferred-items.md`, `TESTING.md` step 11, and the `deferred` frontmatter above.
- **`.planning/codebase/CONCERNS.md`** still describes the fixed `non-ip` behaviour (from 01-02's deferred items); a codebase-map snapshot, not hand-edited mid-phase.
- **Minor SUMMARY inaccuracies (informational, not goal-level):** 01-01-SUMMARY records 383 lib tests and 13 e2e passes (observed 382 and 14). 01-05-SUMMARY's key-files no longer lists `restore_fetch_tests.rs` as modified; its body correctly records the file as untouched during 01-05.
- **Review advisories carried for any future flake claim (not TEST-03 blockers):** R1 (the async-SIGKILL mechanism is asserted, not captured from a failing `/proc` snapshot) and R2 (sample power) — the committed 35/35 plus this verifier's 13 fresh runs (10 exact + 3 full) give 48 post-fix green runs, and R4/IN-01 (raw logs uncommitted) is disclosed in `notes.md`; IN-04 applies to the targeted log only.
- **Bookkeeping:** `c574a9f2` removed the stale `HANDOFF.json`, `.continue-here.md` and `01-REVIEW-06.md` (content folded into `01-06-REVIEW.md`); the working tree is clean at that commit.

### Gap Closure Summary

The single previously failed truth — TEST-03 — is closed. The previous verification's own 35-run repetition of `cargo test -p xai-grok-workspace --lib -- --test-threads=4` caught two failures, the captured one being `restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild` panicking at `restore_fetch_tests.rs:731` (`grandchild <pid> must not remain live`). Plan 01-06 fixed the race at its observation point: SIGKILL delivery and reaping are asynchronous, so the test now asserts the eventual property through `wait_until_dead` (5 s deadline, 20 ms interval) at all five process-death sites, while every prompt-return bound is still evaluated before any polling and every original message is preserved. The helper's bound is pinned by a Linux-only regression test, no `#[ignore]` exists, and production `restore_fetch.rs` is byte-identical.

The claim now carries a committed artifact: `docs/evidence/01-truth-baseline/determinism/` holds a valid asciinema-v2 cast, an exact 8-key `report.json` (`requirements: [TEST-03]`, 7/7 steps ok, `result: ok`, 4/4 artifact sha256), a 35-line projection log in which every run is `rc=0 named=ok` with `test result: ok. 1695 passed; 0 failed` and exactly one passed count, and a `notes.md` re-run recipe. This verifier independently confirmed the 35 projection lines against the still-present raw per-run cargo logs (35 distinct files, 0 mismatches, named test ok in every run), and then ran the suite itself: 10/10 exact runs of the previously failing test and 3/3 full-crate runs green (`1695 passed; 0 failed` each). `KNOWN-ISSUES.md`, `TESTING.md` and `01-05-SUMMARY.md` now record the 2-in-35 falsification, the fix and the evidence path; the old five-run claim is marked superseded, and the post-review commit `5b4c30c4` records that only the captured failure's identity is known. No regression was found in the other ten requirements, whose implementation files are unchanged since the previous report.

---

_Verified: 2026-11-23T20:22:00Z_
_Verifier: the agent (gsd-verifier)_
