<!-- generated-by: gsd-doc-writer -->
# Planning

Where the milestone plan, its per-phase record and its committed proof live. This
page is a map — each row points at the file that holds the decision itself.

## Where the plan lives

| File | What it holds |
|------|---------------|
| [`PROJECT.md`](../.planning/PROJECT.md) | What the product is and why; requirements split into Validated / Active / Out of Scope; the locked Key Decisions table |
| [`ROADMAP.md`](../.planning/ROADMAP.md) | **The authoritative phase list**: goals, per-phase success criteria, requirement ids, plan checkboxes, sequencing constraints, the Progress table |
| [`REQUIREMENTS.md`](../.planning/REQUIREMENTS.md) | Every requirement id (`SBOX-01`, `REL-05`, …) with its checkbox, grouped by area, plus the deferred/v2 section and a traceability table |
| [`STATE.md`](../.planning/STATE.md) | Current position, accumulated per-phase decisions, blockers, deferred items |
| [`config.json`](../.planning/config.json) | Planning-workflow settings: model profile, whether planning docs are committed, parallelization, branch/tag strategy, research/plan-check/verifier toggles |

`ROADMAP.md` is the authority for which phases exist and what state they are in.
`STATE.md`'s frontmatter records the position last written and can lag it — at the
time of writing it still named Phase 4 as current while the roadmap was past
Phase 5.1. Quote the Progress table, not the frontmatter.

Adjacent inputs: [`.planning/codebase/`](../.planning/codebase/) (mapped stack, architecture,
conventions, integrations, concerns) and [`.planning/research/`](../.planning/research/).

## Phase artifacts

Phases live in [`.planning/phases/NN-<slug>/`](../.planning/phases/). Read a phase
in this order:

1. `NN-CONTEXT.md` — the decisions taken before planning: scope boundary, locked choices.
2. `NN-RESEARCH.md` — the codebase/ecosystem research the plan was built from, where written.
3. `NN-PATTERNS.md`, `NN-VALIDATION.md`, `NN-UI-SPEC.md` — supporting contracts, only where present.
4. `NN-0x-PLAN.md` — **the executable contract**: tasks, files touched, per-task verification.
5. `NN-0x-SUMMARY.md` — **what actually happened**, including deviations from the plan.
6. `NN-VERIFICATION.md` — the phase verdict against the roadmap's success criteria.
7. `NN-0x-REVIEW.md`, `NN-REVIEW.md`, `NN-REVIEW-DISPOSITION.md`, `NN-REVIEW-FIX.md` — review records where a review ran.

A plan says what must be done; a summary says what was done and what was not. When
they disagree, the summary and its commit are the record of the tree. Not every
phase carries the full set: only Phases 1 and 2 have a `VERIFICATION.md`, and
Phase 5.1 has only `CONTEXT` and `SUMMARY`.

## Evidence

[`docs/evidence/`](./evidence/README.md) holds the committed proof, one directory
per item: an asciinema-v2 cast (or a `script -T` capture), a `report.json` with
eight fixed keys, an optional `notes.md`, and normalized step logs. That README is
the convention — layout, the report schema, path normalization (`<REPO>`,
`<HOME>`, `<TMP>`), the `scripts/evidence.py` workflow, and a re-run recipe per phase.

- [Phase 1 — `01-truth-baseline/`](./evidence/01-truth-baseline/) — test gate, IPv6 rendering, cost estimate, determinism.
- [Phase 2 — `02-loop-item-verification/`](./evidence/02-loop-item-verification/) — live smoke, runtime egress, key leak, permissions, resume, plan mode; no separate index, the recipes are in the evidence README.
- [Phase 4 — `04-distribution-proof/`](./evidence/04-distribution-proof/README.md) — the item index, each with requirement ids, result and verbatim re-run command, plus the keep-prerelease deviation.

A later phase appends a new `NN-<phase-slug>/` tree and never rewrites an earlier
item. Phase 3 has no evidence tree: its proof is the static gate
`scripts/check-release-pipeline.sh`.

## Decisions and known gaps

- [`DECISIONS.md`](../DECISIONS.md) — the locked choices (D1…D14). It wins on conflict; if a needed decision is missing, stop and ask.
- [`KNOWN-ISSUES.md`](../KNOWN-ISSUES.md) — the honest ledger of what is broken, stale or unexercised, with the date each claim was measured.
- [`TESTING.md`](../TESTING.md) — the manual test plan and the gate: what to run, expected strings and counts, the disk budget.
- [`AGENTS.md`](../AGENTS.md) — the operational handoff: refs, what is done, what is left, the codebase map, the rules for changing the code.

## How to resume

1. [`AGENTS.md`](../AGENTS.md) §0 for the snapshot: branch, HEAD, toolchain, gates.
2. [`STATE.md`](../.planning/STATE.md) for the position, blockers and deferred items.
3. The current phase's `SUMMARY` and `VERIFICATION` under `.planning/phases/`.
4. The latest [`HANDOFF-*.md`](../HANDOFF-2026-11-26.md). The handoffs are dated and the newest wins.

## Status at a glance

Quoted from `ROADMAP.md` §Progress at the time of writing (2026-11-27):

| Phase | Plans complete | Status |
|-------|----------------|--------|
| 1. Truth Baseline | 6/6 | Complete |
| 2. Loop-Item Verification | 6/6 | Complete |
| 3. Pipeline Decoupling & Version | 4/4 | Complete (REL-02 proof deferred to Phase 4) |
| 4. Distribution Proof | 5/5 | REL-04 closed (pulled by digest + run); REL-05 turn proven, sandbox clause unmet |
| 5. Windows Support | 4/5 | WIN-02/WIN-04 closed; WIN-01/WIN-05 authored not proven; WIN-03/WIN-06 need a Windows host |
| 5.1 Project Directory Rebrand (INSERTED) | 2/2 | Complete |
| 5.2 API Key Entry (INSERTED) | 0/1 | In progress |
| 6. Documentation Truth & Freeze | 3/5 | Docs reconciled; ACC-01 (author) blocks 06-04/06-05 |

The roadmap's `Completed` date column is the dated source. "Complete" above appears
only where the roadmap says it; Phase 4 is 5/5 plans executed, not a complete phase.
