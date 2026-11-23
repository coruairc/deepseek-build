---
phase: "02"
slug: "loop-item-verification"
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-11-23"
revised: "2026-11-23"
---

# Phase 02 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Revised with the six-plan split (02-01…02-06). The validator owns `nyquist_compliant` / `validated`; this map
> is kept accurate so those decisions rest on the real commands.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust 1.94.0, pinned in `rust-toolchain.toml`); pty e2e scenarios are `#[ignore]`d and run explicitly |
| **Config file** | `.cargo/config.toml` (no incremental), workspace `Cargo.toml` profiles, `/tmp/team-build.lock` for the shared build lock |
| **Quick run command** | `flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-pager --lib headless` |
| **Full suite command** | `scripts/test-gate.sh` (disk-guard → flock → `cargo test --workspace --no-fail-fast` → `--features config-docs` → `key-leak-check.sh --scan docs/evidence`; completeness asserted against `scripts/test-gate.expected`) |
| **Estimated runtime** | quick ~90 s warm; full gate ~20–30 min on the build host |

**Hard precondition:** this plan was authored on a host with no toolchain, no binary, 31 GB free and no key. Every
command above runs on the **build host** (Rust 1.94.0, shared `CARGO_TARGET_DIR`, ≥150 GB free). State that
precondition in each task rather than assuming it.

---

## Sampling Rate

- **After every task commit:** run the task's own `<automated>` command (quick, package-scoped)
- **After every plan wave:** run the package-scoped suites the wave touched
- **Before `/gsd-verify-work`:** `scripts/test-gate.sh` green, plus every pty/headless item's own step
- **Max feedback latency:** ~120 s per task command (warm cache)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 02-01 | 1 | PROOF-01 | T-02-01 | host armed; key never handled | human checkpoint | `scripts/disk-guard.sh 150` + `cargo --version` + author confirmation | n/a (checkpoint) | ⬜ pending |
| 02-01-02 | 02-01 | 1 | PROOF-01 | T-02-01, T-02-02 | shell + edit + real `web_fetch`; no key in artifacts | live script + report | `python3 -c` report assert; `scripts/key-leak-check.sh --scan docs/evidence/02-loop-item-verification/live-smoke` | ❌ W0 (evidence dir) | ⬜ pending |
| 02-02-01 | 02-02 | 2 | PROOF-02 | T-02-04, T-02-05 | only provider/`web_fetch`/AF_UNIX; non-vacuous counters | transcript re-read + report | `python3 docs/evidence/02-loop-item-verification/runtime-egress/destination-check.py docs/evidence/02-loop-item-verification/live-smoke/live-smoke-transcript.txt /dev/null` | ❌ W0 | ⬜ pending |
| 02-02-02 | 02-02 | 2 | PROOF-02 | T-02-04 | blind spots stated | doc assertion | `grep -q 'cannot see'` over `runtime-egress/notes.md` + `--scan` | ✅ (after 02-02-01) | ⬜ pending |
| 02-03-01 | 02-03 | 4 | PROOF-03 | T-02-07, T-02-09 | canary run + scoped `--scan`; baseline measured, not gated | script run + scan | `scripts/key-leak-check.sh` (canary) and `--scan docs/evidence`; pending-state schema assert | ✅ (script exists) | ⬜ pending |
| 02-03-02 | 02-03 | 4 | PROOF-03 | T-02-08 | revocation recorded as attestation | human checkpoint + grep | `grep -F 'Revoked: ' docs/evidence/02-loop-item-verification/key-leak/notes.md` | ❌ W0 (notes) | ⬜ pending |
| 02-04-01 | 02-04 | 1 | PERM-01, PERM-02 | T-02-11 | warning row painted from the shell's finding | unit + render | `flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-pager --lib permission_view` | ✅ (extend) | ⬜ pending |
| 02-04-02 | 02-04 | 1 | PERM-02 | T-02-13 | fail-closed with one actionable sentence on both paths | unit + integration (mock) | `cargo test -p xai-grok-pager --lib headless` + `cargo test -p xai-grok-pager-bin --test headless_loop_items -- --nocapture headless_permission_denial` | ❌ W0 (new target) | ⬜ pending |
| 02-04-03 | 02-04 | 1 | PERM-01, PERM-02 | T-02-10, T-02-13 | three outcomes + cast from the recorded run | pty e2e + report | `GROK_PTY_CAST_DIR=… cargo test -p xai-grok-pager-pty-harness --test pty_e2e_shell_tools -- --ignored --nocapture permission_prompt_outcomes_and_warning` | ❌ W0 | ⬜ pending |
| 02-04-04 | 02-04 | 1 | PERM-02 | T-02-14 | gate count refreshed only on a green run | gate | `scripts/test-gate.sh --refresh-expected` then the `results == binaries + doctests` assert | ✅ | ⬜ pending |
| 02-05-01 | 02-05 | 2 | SESS-01, SESS-03, SESS-04 | T-02-15, T-02-16 | model/effort/plan state restored on one session | integration (mock) | `cargo test -p xai-grok-pager-bin --test headless_loop_items -- --nocapture headless_resume_restores_model_effort_and_plan_state` | ❌ W0 | ⬜ pending |
| 02-05-02 | 02-05 | 2 | SESS-03 | T-02-18 | drafted plan survives quit/resume | pty e2e + report | `GROK_PTY_CAST_DIR=… cargo test -p xai-grok-pager-pty-harness --test plan_approval_resume -- --nocapture` | ✅ (extend) | ⬜ pending |
| 02-05-03 | 02-05 | 2 | SESS-04 | T-02-17 | provenance documented per setting | doc assertion | `grep -qi` over `resume-fidelity/notes.md` and the user-guide page | ❌ W0 | ⬜ pending |
| 02-06-01 | 02-06 | 3 | PERM-03 | T-02-20, T-02-21 | chip visible and matches after `-r <id>`, no flag repeated | pty e2e + report | `PAGER_BINARY=… GROK_PTY_CAST_DIR=… cargo test -p xai-grok-pager-pty-harness --test pty_e2e_shell_tools -- --ignored --nocapture permission_mode_chip_matches_after_resume` | ❌ W0 | ⬜ pending |
| 02-06-02 | 02-06 | 3 | PERM-04 | T-02-19 | plan armed headless; ledger corrected, code untouched | integration (mock) + grep | `cargo test -p xai-grok-pager-bin --test headless_loop_items -- --nocapture headless_permission_mode_plan_arms_plan_mode`; `grep -q 'permission-mode' KNOWN-ISSUES.md`; `git diff --quiet TESTING.md` | ✅ (unit exists) | ⬜ pending |
| 02-06-03 | 02-06 | 3 | SESS-02 | T-02-19, T-02-22 | writes blocked outside the plan file in all three modes | unit + pty e2e | `cargo test -p xai-grok-shell --lib plan_mode_edit_gate` + `--ignored … plan_mode_read_only_in_every_permission_mode` | ✅ (extend) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `docs/evidence/02-loop-item-verification/{live-smoke,runtime-egress,key-leak,permission-prompt,permission-mode,resume-fidelity,plan-read-only}/` — item directories with `notes.md` and the exact re-run command, created before the first live run
- [ ] The new pty tests (`permission_prompt_outcomes_and_warning`, `permission_mode_chip_matches_after_resume`, `plan_mode_read_only_in_every_permission_mode`) — red-first against the current tree; PERM-01's warning-row fix and PERM-02's final denial string are the expected reds
- [ ] `crates/codegen/xai-grok-pager-bin/tests/headless_loop_items.rs` — the one new `[[test]]` target (created in 02-04)
- [ ] `scripts/test-gate.expected` — refreshed **once**, by 02-04, with `scripts/test-gate.sh --refresh-expected` on a green run; 02-05 and 02-06 add no new target and must leave it alone

*Existing infrastructure (test-gate, evidence.py, key-leak-check --scan, the pty harness `write_cast`) covers everything else.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Key revocation after the live run | PROOF-03 | The platform has no API for it; only the account holder can confirm | Author revokes the key on platform.deepseek.com, then records a dated one-line attestation (no key material) in `docs/evidence/02-loop-item-verification/key-leak/notes.md` |
| Live provider reachability | PROOF-01/02/03 | Requires the author's key and outbound network to `api.deepseek.com`; the PROOF-03 canary also contacts the provider (fake key → 401) | Run on the build host with the key exported; the scripts fail closed on any violation. If the host could not connect, the canary item says so and does not present the PASS line as a provider-path run |
| TUI "feel" of the permission prompt | PERM-01 | A pty cast proves the rows and copy, not pleasantness | Deferred to the Phase 5 acceptance session (ACC-01); not a Phase 2 gate |
| `--permission-mode auto` on a gated headless call | PERM-02 | RESEARCH leaves the behaviour unverified; asserting it would be a false proof | Recorded as an observation by the 02-04 `auto` probe and stated as "observed, not claimed" in `permission-prompt/notes.md` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
