---
phase: 03-pipeline-decoupling-version
plan: "03"
status: complete
subsystem: versioning
requirements: [PKG-01]
verification: "static only: lock parses and lists 0.1.0; no build (tests skipped by owner, 2026-11-22)"
---

# Plan 03-03: Workspace version 0.1.0

## Changes

- `Cargo.toml` `[workspace.package]` gains `version = "0.1.0"`.
- The four shipping crates now inherit it (`version.workspace = true`):
  `xai-grok-pager`, `xai-grok-pager-bin`, `xai-grok-shell`, `xai-grok-version`.
  The other 82 members were already `0.1.0`; `xai-proto-build` and the vendored
  third-party crates keep their own versions.
- `Cargo.lock`: exactly four lines changed (`0.0.1-rc5` → `0.1.0`), 1246
  packages unchanged. Regenerated with `cargo metadata --offline` (resolution
  only; the offline download failure it reports is for a target-specific crate
  that does not affect the lock).
- `TESTING.md`: the `--version` expectation is `deepseek-build 0.1.0 (<commit>)`.
- `KNOWN-ISSUES.md`: the version ledger records the inherited workspace version.

## Verification

- **DEFERRED (tests skipped by owner, 2026-11-22); no build.**
- Static: `Cargo.toml` parses; `Cargo.lock` parses with
  `xai-grok-pager(-bin)`, `xai-grok-shell` and `xai-grok-version` at `0.1.0`.
