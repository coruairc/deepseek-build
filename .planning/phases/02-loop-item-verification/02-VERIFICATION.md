---
phase: 02-loop-item-verification
verified: 2026-11-22T05:55:00Z
status: deferred
verification_type: goal-backward
score: "1/13 met by documentation; 12/13 execution proofs deferred"
requirements_verified: [SESS-04]
requirements_code_and_doc_present: [PERM-02, PERM-03, PERM-04, SESS-02]
requirements_deferred: [PROOF-01, PROOF-02, PROOF-03, PERM-01, SESS-01, SESS-03]
---

# Phase 2 Verification — Loop-Item Verification

**Verdict: DEFERRED (tests skipped by owner, 2026-11-22).**

No live session, permission prompt, egress measurement or resume cycle was
executed. No throwaway key is available. This phase's writable work is the
harness recipe, the evidence scaffolding and the corrections below.

## Goal-backward check

**Goal:** the daily loop — permissions, resume, plan mode — and the runtime
egress boundary verified against a real model-driven session with a throwaway
key, so "it works" rests on committed artifacts.

| # | Success criterion | State |
|---|---|---|
| 1 | Live smoke with shell, edit and `web_fetch`, transcript committed | **DEFERRED** — item `result: skipped`, recipe committed |
| 2 | Runtime egress: provider + `web_fetch` host + AF_UNIX, zero violations | **DEFERRED** — item `result: skipped`, recipe committed |
| 3 | Key-leak green; no key material; key revoked | **DEFERRED** — item `result: skipped`, recipe committed |
| 4 | Prompt offers allow-once/allow-for-session/reject with the warning; headless denial hint on both paths; mode visible; `--permission-mode plan` verified and doc corrected | **Code/doc present, exercise DEFERRED** — the prompt exists; the JSON error path now names both flags; `KNOWN-ISSUES` and the user guide correct the plan flag |
| 5 | Resume restores model/effort/plan state; plan mode read-only everywhere; drafted plan survives; provenance documented | **Provenance documented; cycle DEFERRED** — `17-sessions.md`; contract already in `19-plan-mode.md` |

## Requirement status

| Req | Status | Where |
|---|---|---|
| PROOF-01 | DEFERRED | `docs/evidence/02-loop-item-verification/live-smoke/` |
| PROOF-02 | DEFERRED | `docs/evidence/02-loop-item-verification/egress/` |
| PROOF-03 | DEFERRED | `docs/evidence/02-loop-item-verification/key-leak/` |
| PERM-01 | DEFERRED (prompt code exists) | `views/prompt_widget/` |
| PERM-02 | code + doc present | `headless.rs` JSON/stderr messages; `22-permissions-and-safety.md` |
| PERM-03 | documented; run deferred | mode is config/launch-derived and displayed; `17-sessions.md` provenance |
| PERM-04 | doc corrected, code unchanged | `KNOWN-ISSUES.md`, `22-permissions-and-safety.md` |
| SESS-01 | documented; run deferred | `17-sessions.md` (model/effort/plan.json) |
| SESS-02 | contract documented | `19-plan-mode.md`, unchanged |
| SESS-03 | DEFERRED | drafted-plan-at-deliverable recipe in `02-05-SUMMARY.md` |
| SESS-04 | met | `17-sessions.md` provenance table |

## Deferred verification commands

```sh
# with a throwaway key, then revoke it
scripts/smoke-test.sh
scripts/egress-check.sh
scripts/key-leak-check.sh
deepseek-build -p "…" --permission-mode auto        # headless denial hint
deepseek-build -p "…"                                # JSON error path
```

## Risks carried forward

- The three evidence items are honest placeholders. A release claim needs them
  upgraded with a real keyed session.
- The headless message change is string-only; it was not compiled in this run
  (no builds allowed). `cargo fmt --check` is clean.
