---
phase: "02"
slug: "loop-item-verification"
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-11-23"
---

# Phase 02 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust 1.94.0, pinned in `rust-toolchain.toml`); pty e2e scenarios are `#[ignore]`d and run explicitly |
| **Config file** | `.cargo/config.toml` (no incremental), workspace `Cargo.toml` profiles, `/tmp/team-build.lock` for the shared build lock |
| **Quick run command** | `flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-pager --lib headless` |
| **Full suite command** | `scripts/test-gate.sh` (disk-guard → flock → `cargo test --workspace --no-fail-fast` → `--features config-docs`, completeness asserted against `scripts/test-gate.expected`) |
| **Estimated runtime** | quick ~90 s warm; full gate ~20–30 min on the build host |

**Hard precondition:** this plan was authored on a host with no toolchain, no binary, 31 GB free and no key. Every command above runs on the **build host** (Rust 1.94.0, shared `CARGO_TARGET_DIR`, ≥150 GB free). State that precondition in each task rather than assuming it.

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
| 02-01-01 | 01 | 1 | PROOF-01 | — | key never echoed or written | live script + report | `script -qec 'scripts/smoke-test.sh' -T <log>` then `scripts/evidence.py finish --result ok` | ❌ W0 (evidence dir) | ⬜ pending |
| 02-02-01 | 02 | 2 | PROOF-02 | T-02-02 | only provider/`web_fetch`/AF_UNIX | live script + report | `scripts/egress-check.sh` asserting `EGRESS_VIOLATIONS=0` | ❌ W0 | ⬜ pending |
| 02-03-01 | 03 | 3 | PROOF-03 | T-02-01 | no key material in tree/artifacts | scan + attestation | `scripts/key-leak-check.sh` and `scripts/key-leak-check.sh --scan docs/evidence` | ✅ | ⬜ pending |
| 02-04-01 | 04 | 1 | PERM-01 | T-02-03 | three outcomes + warning row | pty e2e | `GROK_PTY_CAST_DIR=docs/evidence/... cargo test -p xai-grok-pager-pty-harness --test pty_e2e_shell_tools -- --ignored permission_prompt_outcomes_and_warning` | ❌ W0 | ⬜ pending |
| 02-04-02 | 04 | 1 | PERM-02 | — | fail-closed with actionable hint | unit + integration | `cargo test -p xai-grok-pager --lib headless` | ✅ (extend) | ⬜ pending |
| 02-04-03 | 04 | 1 | PERM-03 | — | mode visible and stable across resume | pty e2e | new pty test asserting the chip pair before quit and after `--continue` | ❌ W0 | ⬜ pending |
| 02-04-04 | 04 | 1 | PERM-04 | — | plan mode armed headless | integration (mock) | `cargo test -p xai-grok-pager --lib headless` + `KNOWN-ISSUES.md` diff | ✅ (unit exists) | ⬜ pending |
| 02-05-01 | 05 | 2 | SESS-01 | T-02-04 | model/effort restored | integration (mock) | new headless resume test asserting the request body | ❌ W0 | ⬜ pending |
| 02-05-02 | 05 | 2 | SESS-02 | T-02-05 | writes blocked outside the plan file | unit + integration | `cargo test -p xai-grok-shell --lib plan_mode` + mode-parameterized test | ✅ (extend) | ⬜ pending |
| 02-05-03 | 05 | 2 | SESS-03 | — | drafted plan survives quit/resume | pty e2e | extend `scenarios/plan_approval_resume.rs` and run `--test plan_approval_resume` | ✅ (extend) | ⬜ pending |
| 02-05-04 | 05 | 2 | SESS-04 | — | provenance documented | doc assertion | `grep -q` over the item `notes.md` and the user-guide page | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `docs/evidence/02-loop-item-verification/{live-smoke,runtime-egress,key-leak,permission-prompt,permission-mode,resume-fidelity,plan-read-only}/` — item directories with `notes.md` and the exact re-run command, created before the first live run
- [ ] The new pty tests (`permission_prompt_outcomes_and_warning`, mode-resume, plan-read-only) — red-first against the current tree; PERM-01's warning-row fix and PERM-02's JSON message are the expected reds
- [ ] `scripts/test-gate.expected` unchanged — no new test target may be added without refreshing it explicitly

*Existing infrastructure (test-gate, evidence.py, key-leak-check --scan, the pty harness `write_cast`) covers everything else.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Key revocation after the live run | PROOF-03 | The platform has no API for it; only the account holder can confirm | Author revokes the key on platform.deepseek.com, then records a dated one-line attestation (no key material) in `docs/evidence/02-loop-item-verification/key-leak/notes.md` |
| Live provider reachability | PROOF-01/02 | Requires the author's key and outbound network to `api.deepseek.com` | Run on the build host with the key exported; the scripts fail closed on any violation |
| TUI "feel" of the permission prompt | PERM-01 | A pty cast proves the rows and copy, not pleasantness | Deferred to the Phase 5 acceptance session (ACC-01); not a Phase 2 gate |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
