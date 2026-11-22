---
phase: 02-loop-item-verification
plan: "01"
status: complete
subsystem: live-proof
requirements: [PROOF-01]
verification: "DEFERRED (tests skipped by owner, 2026-11-22)"
metrics:
  duration: "short (no runs executed)"
  completed: "2026-11-22"
---

# Plan 02-01: Live smoke (PROOF-01)

**Status:** code/docs complete; every verification step is **DEFERRED (tests
skipped by owner, 2026-11-22)**.

## Changes

- Added `docs/evidence/02-loop-item-verification/live-smoke/`: `report.json`
  (`result: skipped`) and a `notes.md` with the exact keyed rerun recipe
  (`scripts/smoke-test.sh` under `script -T`, one `evidence.py step` per loop
  item, cast + artifact, revoke the key).
- No harness code was needed: `scripts/smoke-test.sh` already covers a headless
  chat, a tool-calling chain and the adapter checks. Do not duplicate it.

## Verification

- **DEFERRED (tests skipped by owner, 2026-11-22).** No live, permission or
  resume run was executed. Evidence items are committed as `result: skipped`
  with the rerun recipe in `notes.md`.
