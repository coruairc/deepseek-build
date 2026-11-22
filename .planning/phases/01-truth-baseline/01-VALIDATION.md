---
phase: "01"
slug: "truth-baseline"
# status lifecycle: draft (seeded by plan-phase) → validated (set by validate-phase §6)
status: draft
nyquist_compliant: true
wave_0_complete: false
created: "2026-11-21"
---

# Phase 01 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `libtest` (`cargo test`), `tokio::test` for async, `serial_test` where a test mutates process-global env |
| **Config file** | none — workspace `Cargo.toml`; `.cargo/config.toml` sets `incremental = false` and `RUST_MIN_STACK` |
| **Quick run command** | `flock -w 900 /tmp/team-build.lock cargo test -p <touched-crate> --lib` |
| **Full suite command** | `scripts/test-gate.sh` |
| **Estimated runtime** | build + suite ~30 min on the author's host (last recorded full run 2026-10-09: 31,939 tests across 307 targets; the target count is re-derived, not carried) |

---

## Sampling Rate

- **After every task:** the task's own `<automated>` command (crate-local `cargo test`, `bash -n`/`shellcheck`, or the script harness).
- **After every plan wave:** `flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-sandbox --lib && cargo test -p xai-grok-status-line --lib && cargo test -p xai-grok-pager --lib`.
- **Before `/gsd-verify-work`:** `scripts/test-gate.sh` must be green, with the recorded evidence committed.
- **Max feedback latency:** ~120 s for a crate-local suite; the full gate is a phase-boundary check, not a per-task loop.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-01-T1 | 01 | 3 | SBOX-01 | T-01-01, T-01-02 | A non-traversable parent is skipped, a maskable endpoint is still bound | unit | `flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-sandbox --lib runtime_sockets` | ✅ extend `runtime_sockets_tests.rs` | ⬜ pending |
| 01-01-T2 | 01 | 3 | SBOX-01 | T-01-02 | The fail-closed row stays fatal under a traversable parent | unit | same + `… --lib read_deny_verify` | ✅ extend | ⬜ pending |
| 01-01-T3 | 01 | 3 | SBOX-02 | T-01-03 | Re-exec survives a skipped candidate; devbox scenario keeps its assertions | unit + integration | `… cargo test -p xai-grok-sandbox --test deny_paths_e2e` and `… --lib a_non_traversable_socket_parent_does_not_abort_reexec` | ✅ extend `deny_paths_e2e.rs`, `lib.rs` tests | ⬜ pending |
| 01-02-T1 | 01 | 4 | SBOX-02 | T-01-07 | A refused IPv6 destination is named, never `non-ip` | script | `bash -n scripts/sandbox-run.sh && sh -n scripts/test-sandbox-run.sh && shellcheck --shell=sh …` + `sh scripts/test-sandbox-run.sh` | ❌ W0 create | ⬜ pending |
| 01-02-T2 | 01 | 4 | SBOX-02 | T-01-07 | Offline harness resolves both families without leaving the host | script | `sh scripts/test-sandbox-run.sh` + `! grep -n 'print as' scripts/egress-check.sh` | ❌ W0 create | ⬜ pending |
| 01-03-T1 | 01 | 1 | COST-01 | T-01-SC | The hidden alias prices at Flash rates, not Pro | unit | `… cargo test -p xai-grok-status-line --lib pricing` + `… -p xai-grok-pager --lib headless` | ✅ extend | ⬜ pending |
| 01-03-T2 | 01 | 1 | COST-02 | — | The estimate is labelled and fixed-width per branch, no tone change | unit (tdd) | `… cargo test -p xai-grok-pager --lib status_line` | ✅ extend `segments_tests.rs` | ⬜ pending |
| 01-03-T3 | 01 | 1 | COST-03 | — | Headless and TUI report the same number for the same usage + override | unit | `… -p xai-grok-pager --lib headless` + `… --lib status_line` | ✅ extend | ⬜ pending |
| 01-04-T1 | 01 | 2 | DEF-01 | T-01-SC | Catalog default is authoritative only below CLI/env/config | unit | `… cargo test -p xai-grok-models --lib` + `… -p xai-grok-shell --lib -- catalog_default` | ❌ W0 add test | ⬜ pending |
| 01-04-T2 | 01 | 2 | DEF-01 | — | Docs state Flash, the caveat and both revert spellings | docs gates | per-file `grep -q 'deepseek-flash'` + TUNING gates + `… -p xai-grok-shell --lib --features config-docs` | ✅ extend | ⬜ pending |
| 01-05-T1 | 01 | 5 | TEST-01, TEST-02 | T-01-13 | Completeness is counted, never read from a tail | script | `bash -n scripts/test-gate.sh && shellcheck --shell=sh scripts/test-gate.sh` + `scripts/test-gate.sh --package … --package …` | ❌ W0 create | ⬜ pending |
| 01-05-T2 | 01 | 5 | TEST-03 | T-01-15 | Flake resolved deterministically or quarantined by name | repeated run | `… -p xai-grok-shell --lib --features config-docs` + 5× `… -p xai-grok-workspace --lib` | ✅ existing | ⬜ pending |
| 01-05-T3 | 01 | 5 | EVID-01, EVID-02 | T-01-16 | Committed artifacts carry no absolute paths and no key material | script + schema | `scripts/key-leak-check.sh --scan docs/evidence` + `report.json` key assertion + `grep '<HOME>'` + `scripts/test-gate.sh` | ❌ W0 create | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/codegen/xai-grok-sandbox/src/runtime_sockets_tests.rs` — fixture matrix and pure-predicate table.
- [ ] `crates/codegen/xai-grok-sandbox/src/lib.rs` tests — upper-layer re-exec survival test.
- [ ] `crates/codegen/xai-grok-status-line/src/pricing.rs` — alias entry and worked example.
- [ ] `crates/codegen/xai-grok-pager/src/views/status_line/segments_tests.rs` — label format rows.
- [ ] `crates/codegen/xai-grok-pager/src/headless_tests.rs` + `reducer/messages/tests/result_usage.rs` — override and cross-surface equality cases.
- [ ] `crates/codegen/xai-grok-shell` model-catalog test for the resolved default with no preference.
- [ ] `scripts/test-gate.sh`, `scripts/test-gate.expected`, `scripts/test-sandbox-run.sh`, `scripts/evidence.py`, `docs/evidence/README.md`.

*Framework install: none — the workspace already carries the test infrastructure.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| At-ceiling caveat and revert path read correctly | DEF-01 | Prose quality is a review judgement, not a pattern | Read `README.md`, `11-custom-models.md`, `05-configuration.md`, `docs/TUNING.md`; confirm the caveat names 63/63 and both revert spellings |
| The status-line estimate reads well in a live TUI | COST-02 | The segment is pinned by unit tests; live legibility is a human call | Start the TUI on a throwaway repo, run one Flash turn, confirm `~$… est.` fits and does not reflow the row |

*All other phase behaviors have automated verification.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 120 s for crate-local runs
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** DEFERRED (tests skipped by owner, 2026-11-22) — the gate and per-task runs are not executed; the code and evidence convention are committed, and `scripts/test-gate.sh` must be run before a release claim.
