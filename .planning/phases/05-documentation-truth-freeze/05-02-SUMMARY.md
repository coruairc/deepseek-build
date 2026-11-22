---
phase: 05-documentation-truth-freeze
plan: "02"
status: complete
requirements: [DOCS-03]
verification: "docs only (tests skipped by owner, 2026-11-22)"
---

# Plan 05-02: KNOWN-ISSUES reconciled

## Changes

- New **Tags and release identity** section:
  - `deepseek-v0.1-snapshot` at `4b305bd0` is **superseded**, not the freeze
    marker; it must never be moved or deleted (a remote ref change is
    destructive).
  - `v0.1.0` is not published by this milestone; no `deepseek-*` trigger exists
    or should be added.
- The corrected plan-mode entry (from 02-04) stands: `--permission-mode plan`
  parses to `ask`; plan mode is a TUI session mode.
- The sandbox findings stand: the root-owned deny parent no longer aborts
  bwrap; `deny_paths_e2e` is green with the single pre-existing harness ignore.
- The suite/gate entry records the owner deferral and that the 2026-11-21 figures
  are not re-confirmed.

## Verification

- **DEFERRED (tests skipped by owner, 2026-11-22).**
