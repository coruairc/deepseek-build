---
phase: "05"
slug: documentation-truth-freeze
created: "2026-11-22"
---

# Phase 5 — Documentation Truth & Freeze: Context

## Goal

Docs describe the tree that exists, the author's acceptance session is signed
off, and a freeze marker moves no published tag.

## Owner decision and standing rules (2026-11-22)

- Tests, gates, smoke/egress runs and release builds are **deferred**: recorded
  as `DEFERRED (tests skipped by owner, 2026-11-22)`.
- **Never push, never force, never tag.** No `deepseek` branch is created either,
  because the acceptance note (ACC-01) must be committed first and it is an owner
  decision this run cannot make.
- The pre-existing `deepseek-v0.1-snapshot` tag is superseded and must not be
  moved or deleted; `v0.1.0` stays unpublished.

## Decisions carried in

- D9's personal-tool intent (no release obligation) stands while the docs record
  that the release pipeline exists as an author-run path.
- Documentation describes the tree, not an intended future one.
