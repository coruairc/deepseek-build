---
phase: 02-loop-item-verification
plan: "02"
status: complete
subsystem: live-proof
requirements: [PROOF-02]
verification: "DEFERRED (tests skipped by owner, 2026-11-22)"
metrics:
  duration: "short (no runs executed)"
  completed: "2026-11-22"
---

# Plan 02-02: Runtime egress measurement (PROOF-02)

**Status:** code/docs complete; every verification step is **DEFERRED (tests
skipped by owner, 2026-11-22)**.

## Changes

- Added `docs/evidence/02-loop-item-verification/egress/` with the skipped report
  and a rerun recipe around `scripts/egress-check.sh`.
- Documented what a pass must show (provider + `web_fetch` host + AF_UNIX,
  `EGRESS_VIOLATIONS=0`) and the LD_PRELOAD coverage limits already noted in the
  script; a violation is an invariant breach, not a result to record.

## Verification

- **DEFERRED (tests skipped by owner, 2026-11-22).** No live, permission or
  resume run was executed. Evidence items are committed as `result: skipped`
  with the rerun recipe in `notes.md`.
