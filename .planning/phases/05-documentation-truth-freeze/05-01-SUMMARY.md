---
phase: 05-documentation-truth-freeze
plan: "01"
status: complete
requirements: [DOCS-01, DOCS-02]
verification: "docs only; no run (tests skipped by owner, 2026-11-22)"
---

# Plan 05-01: DECISIONS and TESTING rewritten from the tree

## Changes

- `DECISIONS.md` §E (open items) rewritten: O1 legal open; O2 toolchain
  resolved; O3 DeepSeek model/usage mapping done (live-verified 2026-10-07);
  O4 crate rename deferred (D2); O5 macOS/GHCR open; O6 clean-host suite
  deferred by owner; O7 author acceptance + freeze open.
- `DECISIONS.md` §F (phase log) rewritten: phases 1-4 are complete (cleanup +
  rebrand, adapter, TUI, harden); phase 5 "quality bar / release" is **present,
  not certified** — pipeline, installer, evidence convention and the Linux-only
  `v0.0.1-rc5` exist, but no macOS artifact, no pulled image, proofs blocked and
  the suite deferred. D9's personal-tool intent is stated explicitly: none of
  the release machinery is required to use the tool.
- `TESTING.md` §2: the stale `OIDC not configured` observation is replaced with
  the API-key-only reality; no interactive key entry, no OIDC flow.
- `TESTING.md` §6: the OIDC test counts are removed ("no OIDC test counts to
  quote"), and the owner deferral is recorded.

## Verification

- **DEFERRED (tests skipped by owner, 2026-11-22).** Documentation only.
