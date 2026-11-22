---
phase: 03-pipeline-decoupling-version
plan: "04"
status: complete
subsystem: versioning
requirements: [PKG-02, PKG-03]
verification: "static only; CI assertions written but not executed (tests skipped by owner, 2026-11-22)"
---

# Plan 03-04: Dual version assertion and the v0.1.0 tag shape

## Changes

- `release.yml` (both build legs): `Verify the manifest version matches the
  release tag` — on a tag run, `[workspace.package] version` must equal the
  version derived from the tag. This is the source path; the existing
  `Smoke-check the built binary` step asserts the stamped `GROK_VERSION` path.
  A mismatch fails with an actionable `::error::`.
- No `deepseek-*` trigger was added.

## Static verification of the tag shape

| Property | State |
|---|---|
| Trigger | `push: tags: ['v*']` plus `workflow_dispatch`; no `deepseek-*` |
| Version parse | `v?*` → strip leading `v`; non-`v` tag → error |
| Prerelease | hyphen suffix (`v0.1.0-rc1`) → `is_prerelease=true`; `v0.1.0` → false |
| `install.sh` | must exist at the repo root; published as a release asset |
| Release creation | tag runs only; `workflow_dispatch` creates no release |
| `:latest` | moves only when `is_prerelease=false` |

## Verification

- **DEFERRED (no tag, no dispatch, no push; tests skipped by owner,
  2026-11-22).** A dry run is a real push and is explicitly out of scope for
  this run.
