# Phase 2: Loop-Item Verification - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-11-23
**Phase:** 2-loop-item-verification
**Areas discussed:** Live proof and key handling, Permission surface, Resume fidelity, Defect policy, Evidence, Plan sequencing

---

## Run mode note

This run was non-interactive: the harness could not present questions, so no answer below is a user's
answer. Hard-to-reverse questions were decided with the recommended option and the rationale is recorded
here and in CONTEXT.md; reversible technical questions were decided autonomously. If any of these needs
to change, edit the corresponding bullet in `02-CONTEXT.md` before planning.

## Live proof and key handling

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse `egress-check.sh` + `smoke-test.sh` | Existing real-session scripts already exercise shell, edit, `web_fetch`, MCP, JSON, resume and compaction | ✓ |
| Write a new live-session harness | More control, but new untested scope and a second measurement path | |
| Mock-only proof | No key needed, but cannot prove the real egress boundary | |

**User's choice:** (not asked — non-interactive) Recommended option taken: reuse the existing scripts.
**Notes:** The key is compromised-by-policy once used. Environment-only sourcing, no file, no echo, scan
before commit, and the author's rotation recorded as a dated attestation.

## Permission surface

| Option | Description | Selected |
|--------|-------------|----------|
| pty scenario against the real binary | Harness emits asciinema v2 casts; matches the Phase 1 evidence convention | ✓ |
| Unit tests only | Cheaper, but does not exercise the prompt end to end | |
| Manual author walkthrough now | Strongest evidence, but the author's acceptance session is the Phase 5 gate | |

**User's choice:** (not asked — non-interactive) Recommended option taken: pty scenario, with the
headless hint asserted on both the stderr and JSON paths.
**Notes:** The stderr hint already names both flags; the JSON-path error string names only
`--always-approve`. Aligning it is the expected minimal in-scope fix.

## Resume fidelity

| Option | Description | Selected |
|--------|-------------|----------|
| Promote the existing `plan_approval_resume` scenario | Already drafts a plan, quits and resumes; extend to assert model/effort/plan state | ✓ |
| New standalone resume scenario | Duplicates coverage that exists | |
| Headless-only assertions | Misses the interactive plan-approval path the scenario already covers | |

**User's choice:** (not asked — non-interactive) Recommended option taken.
**Notes:** SESS-02 spans `default`, `always-approve` and `auto`; SESS-04 is a provenance table plus the
user-guide sentence.

## Defect policy

| Option | Description | Selected |
|--------|-------------|----------|
| Fix minimal, in scope, with a regression test | Keeps the loop item working; SUMMARY records it | ✓ |
| Document only | Pure verification, but ships known-broken loop items | |
| Stop and ask each time | Not available in this run | |

**User's choice:** (not asked — non-interactive) Recommended option taken, with PERM-04 following the
roadmap's doc-correction rule.
**Notes:** Anything outside the loop items goes to `KNOWN-ISSUES.md`.

## Evidence

| Option | Description | Selected |
|--------|-------------|----------|
| One item directory per claim, seven slugs | Matches the Phase 1 convention and the requirement grouping | ✓ |
| One directory per plan | Fewer directories, but a reviewer cannot map a requirement to an artifact | |

**User's choice:** (not asked — non-interactive) Recommended option taken.

## Plan sequencing

| Option | Description | Selected |
|--------|-------------|----------|
| 02-01/02/03 sequential, 02-04/05 parallel if disjoint | Live proof and its scan are dependent; practice/UI work is not | ✓ |
| All five sequential | Simpler, but leaves the parallel-lane decision unused | |

**User's choice:** (not asked — non-interactive) Recommended option taken.

---

## the agent's Discretion

- Item slugs beyond the seven named, pty scenario file names, the security-warning assertion wording,
  whether SESS-02 uses three pty runs or one parameterized run, and the provenance table's shape.

## Deferred Ideas

- Routing, per-model cost, `/route` — v2 / open-build.
- `ci.yml`, debug-symbol sidecars — REL-V2-01/02.
- `IN-07` (`key-leak-check.sh` default-mode fail-open) — v0.1.x; `--scan` (the mode used here) fails closed.
- New evidence tooling (asciinema CLI, cast renderers) — not needed.

## Decided autonomously

[auto] [Evidence] — Q: "New evidence tree or extend Phase 1?" → Selected: "New `docs/evidence/02-loop-item-verification/` tree; earlier items are never rewritten" (reversible technical — appending a directory is local and cheap to undo)
[auto] [Live proof] — Q: "Write a new live-session harness?" → Selected: "Reuse `scripts/egress-check.sh` and `scripts/smoke-test.sh`" (reversible technical — a new script would be untested scope)
[auto] [Cast source] — Q: "Use the `asciinema` CLI?" → Selected: "Use the pty harness `write_cast` (asciinema v2)" (reversible technical — already in-tree, no new dependency)
[auto] [Scanning] — Q: "New leak scanner?" → Selected: "`scripts/key-leak-check.sh` default mode plus `--scan` over artifacts" (reversible technical — the script already exists and fails closed in `--scan` mode)
[auto] [Key storage] — Q: "Read the key from a file?" → Selected: "Environment only, never a file" (reversible technical — a file is a second leak surface, removable at will)
[auto] [PERM-04] — Q: "Fix the plan-mode flag in code?" → Selected: "Verify and correct `KNOWN-ISSUES.md`" (reversible technical — the roadmap and CONCERNS.md both say the tree already sends the call)
[auto] [Order] — Q: "Run the loop items in any order?" → Selected: "PROOF-01 → PROOF-02 → PROOF-03, then PERM/SESS" (reversible technical — ordering is a run-time choice, re-runnable)
