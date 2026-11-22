---
phase: "03"
slug: pipeline-decoupling-version
created: "2026-11-22"
---

# Phase 3 — Pipeline Decoupling & Version: Context

## Goal

A slow macOS leg can no longer erase the image, and the version/tag contract is
settled before any tag exists.

## Decisions carried in

- Publish tag is `v0.1.0`; no `deepseek-*` trigger is added. This milestone does
  not publish it.
- `container` must depend on the Linux build only; `publish` needs both legs so
  a missing macOS tarball still fails loudly at publish.
- Distribution builds ship no DWARF and no sidecar; the macOS ad-hoc signature
  must survive.
- Four shipping crates inherit `[workspace.package] version = "0.1.0"`; the
  other 82 members were already `0.1.0`.

## Owner decision — CI runs and pushes are out of scope (2026-11-22)

No workflow is dispatched and nothing is pushed or tagged in this run. The
macOS time-budget measurement (REL-02) and the dispatch verification are
**DEFERRED (tests skipped by owner, 2026-11-22)**; the workflow and manifests are
written and statically validated only.
