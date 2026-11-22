---
phase: 04-distribution-proof
verified: 2026-11-22T06:40:00Z
status: blocked
verification_type: goal-backward
score: "0/4 execution proofs; 2/3 plans' docs written"
requirements_verified: []
requirements_code_and_doc_present: [REL-05]
requirements_blocked: [REL-04, REL-06]
---

# Phase 4 Verification — Distribution Proof

**Verdict: BLOCKED.** The owner's standing rule for this run is *never push,
never tag*, and tests/live runs are skipped (2026-11-22). No throwaway tag, no
GHCR pull, no installer run.

## Goal-backward check

**Goal:** the image, the installer and the macOS artifacts proven from a real
throwaway pre-release tag.

| # | Success criterion | State |
|---|---|---|
| 1 | Throwaway tag produces the GHCR image; pulled from the registry | **BLOCKED** — no tag/push allowed; item `result: skipped` |
| 2 | Pulled image runs a headless turn with a shell command under the sandbox-on flags; README states headless-only + mounts | **Docs met, run BLOCKED** — README Container section rewritten |
| 3 | `install.sh` pinned install prints the version; unpinned `latest` behaviour established from the real URL | **BLOCKED** — documented from GitHub's release-selection rule; real-URL check deferred |
| 4 | Throwaway tag/release/image cleaned up, `:latest` not moved, evidence committed | **BLOCKED** — nothing was created; cleanup procedure in `dry-run/notes.md` |

## Requirement status

| Req | Status |
|---|---|
| REL-04 | BLOCKED (never push/tag) |
| REL-05 | docs complete; in-image run BLOCKED |
| REL-06 | docs complete; install run BLOCKED |

## What is committed

- `README.md` — headless-only image, mount table, sandbox flags, pinned installer
  form, `latest` limitation.
- `KNOWN-ISSUES.md` — container decoupling, unexercised container docs, installer
  `latest` semantics.
- `docs/evidence/04-distribution-proof/{dry-run,installer}/` — skipped reports
  with exact procedures.

## Risks carried forward

- The container and installer claims are documentation, not observation. The
  freeze checklist (FREEZE-03) includes the release dry-run evidence, so the
  freeze cannot pass until this phase's proof is actually run.
