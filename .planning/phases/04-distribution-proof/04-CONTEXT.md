---
phase: "04"
slug: distribution-proof
created: "2026-11-22"
---

# Phase 4 — Distribution Proof: Context

## Goal

The image, the installer and the macOS artifacts proven from a real throwaway
pre-release tag, so the freeze rests on distribution evidence.

## Hard constraint from this run

The owner's standing rules are **never push, never tag**, and tests/live runs are
skipped. REL-04/05/06 are therefore **code/docs only**: no tag is created, no
workflow is dispatched, no image is pulled, no installer runs. Evidence items
record `result: skipped` with the exact procedure to run later.

## Decisions carried in

- The image is headless-only; README states that and the mounts it needs.
- The sandbox stays on: there is no disable flag, and a deny plan that cannot be
  bound makes the process refuse rather than degrade. README documents the
  minimal docker flags that let bwrap work.
- `latest` resolves GitHub's newest non-prerelease; only pre-releases exist, so
  the unpinned path is documented as having no target until a stable release.
- Cleanup of any future throwaway tag must leave `:latest` untouched.
