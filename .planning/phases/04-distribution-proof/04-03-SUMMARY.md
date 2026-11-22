---
phase: 04-distribution-proof
plan: "03"
status: complete
requirements: [REL-06]
verification: "docs + scaffolding only; install runs DEFERRED (tests skipped by owner, 2026-11-22)"
---

# Plan 04-03: Installer truth and proof scaffolding

## Changes

- `README.md` Install section corrected from the release facts: pre-releases
  exist, GitHub's `latest` resolves only to a **non-prerelease**, so with only
  `v0.0.1-rc*` published the unpinned path has no target; the pinned form
  (`VERSION=v0.0.1-rc5`, tag URL) is shown instead. The phase table now names
  `v0.0.1-rc5`.
- `KNOWN-ISSUES.md`: new installer entry records the `latest` semantics, the
  pinned form, and that the real-URL check for both paths is deferred.
- `docs/evidence/04-distribution-proof/installer/`: `report.json`
  (`result: skipped`) and the exact pinned/unpinned procedure, including the
  note that the fake-tree harness is not proof of the real URL.

## Verification

- **DEFERRED (tests skipped by owner, 2026-11-22).** No install was run; the
  pinned and unpinned behaviours are documented from GitHub's release-selection
  rules, not observed here.
