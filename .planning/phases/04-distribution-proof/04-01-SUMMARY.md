---
phase: 04-distribution-proof
plan: "01"
status: blocked
requirements: [REL-04, REL-05]
verification: "BLOCKED — never push/tag; DEFERRED (tests skipped by owner, 2026-11-22)"
---

# Plan 04-01: Throwaway tag dry run

**Status: BLOCKED.** The owner's standing rule for this run is *never push,
never tag*. A dry run is a real tag push, so it cannot happen here.

## Changes

- `docs/evidence/04-distribution-proof/dry-run/` — `report.json`
  (`result: skipped`) and `notes.md` with the full procedure: throwaway tag,
  watch the split workflow, `docker pull` from GHCR, a sandbox-on headless turn,
  then delete the tag, release and image tag and confirm `:latest` did not move.

## Verification

- **BLOCKED / DEFERRED (tests skipped by owner, 2026-11-22).**
