---
gsd_state_version: "1.0"
milestone: v0.1.0
current_phase: 05
current_phase_name: Documentation Truth & Freeze
status: executing
stopped_at: Phases 1-4 plan work complete; Phase 4 proof blocked (never push/tag); Phase 5 in progress
last_updated: "2026-11-22T06:45:00.000Z"
last_activity: 2026-11-22
last_activity_desc: Phase 04 docs complete, proof blocked; Phase 05 started
state_head: 711524dd454225eeb2bcbbb3924f7d8a83f2268e
progress:
  total_phases: 5
  completed_phases: 4
  total_plans: 22
  completed_plans: 17
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-11-21)

**Core value:** One DeepSeek key in, a trustworthy daily coding agent out: the right model on every turn, zero traffic to any host you did not configure, and a release you can install with confidence.
**Current focus:** Phase 05 — Documentation Truth & Freeze

## Current Position

Phase: 05 (Documentation Truth & Freeze) — EXECUTING
Plan: 1 of 5
Status: Executing Phase 05
Last activity: 2026-11-22 — Phase 04 docs complete (proof blocked); Phase 05 started

Progress: [████████░░] 80%

## Performance Metrics

**Velocity:**
- Total plans completed: 0
- Average duration: —
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01 Truth Baseline | 5 | 5 | long-running; tests deferred |
| 02 Loop-Item Verification | 5 | 5 | scaffolding only; runs deferred |
| 03 Pipeline Decoupling & Version | 4 | 4 | code/config only; CI deferred |
| 04 Distribution Proof | 3 | 3 | docs only; proof blocked (never push/tag) |

**Recent Trend:**
- Last 5 plans: —
- Trend: —

*Updated after each plan completion*

## Accumulated Context

### Decisions

Full log in PROJECT.md Key Decisions. Those shaping current work:

- Routing is cut from this milestone entirely — `route_turn`, `[models] routing`, session pins, the `Route` status-line item, `/route` and per-model cost are v2. `/model` and `/effort` are the manual switches. Do not wire, display, or price routing in any phase.
- What survives from the cost work is the honesty bug fix: `deepseek-v4-flash` priced in the table, the TUI figure labelled an estimate, headless honouring `[ui.status_line.pricing]`.
- `deepseek-flash` becomes the catalog default, with the at-ceiling caveat (63/63 both models) and the revert path (`/model deepseek-v4-pro`, `-m deepseek-v4-pro`) documented.
- Evidence per loop item = asciinema cast + `report.json` under `docs/evidence/`, absolute paths normalized. Phase 1 defines the convention; Phase 2 is its first consumer.
- The sandbox fix comes before the clean-suite gate: only the product fix may zero the bwrap pair — never a skip, an `#[ignore]`, or a widened predicate.
- Publish tag is `v0.1.0` (no `deepseek-*` trigger added); the existing `deepseek-v0.1-snapshot` tag at `4b305bd0` is superseded in docs, never moved. This milestone does not publish `v0.1.0`.
- The freeze marker is non-triggering, and the author's signed acceptance note must be committed before branch `deepseek` or the marker tag exists.
- Container keeps the sandbox on, with documented run flags; no new outbound destination anywhere.
- The ~435 internal `GROK_*` env names and the `xai-grok-*` crate names are out of scope for this milestone (D2); renaming belongs to open-build.
- **Parallel lanes from Phase 2 (user decision, 2026-11-21):** Phase 1 finishes as one sequential lane.
  From Phase 2 onward, plans with disjoint `files_modified` may run in separate git worktrees under one
  shared `CARGO_TARGET_DIR`, one `flock` build lock, and one merge step per wave. This overrides the
  default "one implementer, one working tree" rule for this project; dependent plans (live-session
  proof, artifact builds, freeze) stay sequential. No plan is parallelized with a direct dependency.

### Pending Todos

None yet.

### Blockers/Concerns

- **Tests are deferred milestone-wide (owner decision, 2026-11-22).** No test, gate, smoke or
  egress run is performed for the remaining plans. Every verification step is recorded as
  `DEFERRED (tests skipped by owner, 2026-11-22)` in the plan SUMMARY and here; this is a directed
  deferral, not a failure. Only `cargo fmt` and at most one `cargo check -p <crate> -j 1` per plan
  are allowed. Phase 1's `test-gate` and `cost-estimate` evidence items are `result: skipped`.
- **Phase 2 live proof needs a throwaway key (owner decision):** PROOF-01/02/03 need a
  `DEEPSEEK_API_KEY` that is revoked after the run; none is available, and the live smoke/egress
  measurement is a test run, so it is deferred with the rest.
- **Phase 4 needs a real pushed throwaway tag (owner rule: never push/tag).** REL-04/05/06 cannot be
  proven in this run; only their code/docs are written.
- **Acceptance gate (human, blocking):** ACC-01 needs a 20–30 minute TUI session on a real repo clone by the author. FREEZE-01/02/03 must not run until the signed note is committed — this is the milestone's only human-blocking dependency.
- **Snapshot tag is public:** `deepseek-v0.1-snapshot` already exists on origin at `4b305bd0`. It must be recorded as superseded in docs and left untouched; moving or deleting it is destructive.
- **Throwaway key required for Phase 2:** the live smoke, egress measurement and key-leak scan need a `DEEPSEEK_API_KEY` that is revoked after the run. Without it, PROOF-01/02/03 are blocked, not retried.
- **Research base drift:** STACK/ARCHITECTURE read `main` @ `221a1a59`, PITFALLS @ `9e6780d9`. Re-verify `release.yml` edges and manifest versions before Phase 3.
- **macOS time budget (MEDIUM):** measure the macOS job log before choosing between timeout, strip and LTO override. Both arches must fit the job timeout; one cache-warming run is allowed if cold cache is the cause.
- **Dry-run side effects:** a dry run is a real push. `container` currently depends on the whole matrix and the metadata-action tag list has no empty-list guard, so Phase 3 must land before any throwaway tag, and the dispatch verification must create no release and move no `:latest`.
- **No `ci.yml`:** the gate is a green run on the author's host, wrapped in the shared build lock with `scripts/disk-guard.sh` first.
- **Egress gate is red at base (Phase 5 owner):** `scripts/check-egress.sh --strict` exits 1 on three
  committed test fixtures — `crates/codegen/xai-grok-shell-base/src/util/mod.rs:422-423`
  (`api.x.ai`, `grok.com`) and `crates/codegen/xai-grok-shell/src/agent/config_tests.rs:2768`
  (`gs://bucket`). Not caused by Phase 1; it must be green for FREEZE-03.
- **Credential test prints an exported key (hygiene):** with `DEEPSEEK_API_KEY` exported, an
  `xai-grok-shell` credential assertion prints the key in its failure message. It reached only a local
  log, which was redacted; nothing was committed. The gate now unsets the key vars; the assertion should
  be redacted too, and the key rotated if it was shared.

## Deferred Items

| Category | Item | Status | Deferred At | Milestone |
|----------|------|--------|-------------|-----------|
| Routing | ROUT-V2-01…10 — `route_turn`, `[models] routing`, pins, `Route` item, `/route`, provider-agnostic registry | Tracked in REQUIREMENTS.md §v2 | 2026-11-21 | post-freeze (open-build) |
| Cost | COST-V2-01…04 — per-model session pricing, `/usage` spend, off-peak switch, bucket disjointness | Tracked in REQUIREMENTS.md §v2 | 2026-11-21 | post-freeze |
| Pipeline | REL-V2-01/02 — `ci.yml`, debug symbol sidecars | Tracked in REQUIREMENTS.md §v2 | 2026-11-21 | v0.1.x |
| Product | PROD-V2-01/02 — undo/checkpoint product, automatic Flash-for-research subagents | Tracked in REQUIREMENTS.md §v2 | 2026-11-21 | v0.1.x |
| Naming | ~435 internal `GROK_*` env names; `xai-grok-*` crate/package names | Out of scope (D2) | 2026-11-21 | open-build |
| Release | Publishing `v0.1.0` itself | Author publishes it | 2026-11-21 | after freeze |

## Deferred Execution (owner, 2026-11-22)

| Area | State | Why |
|------|-------|-----|
| Phase 1 test gate + evidence runs | DEFERRED | tests skipped by owner |
| Phase 2 live smoke, egress, key-leak, permission/resume verification | DEFERRED | tests skipped + no throwaway key |
| Phase 3 CI runs (macOS budget, dispatch) | DEFERRED | no push/CI run allowed; workflow written only |
| Phase 4 tag dry run, GHCR pull, installer proof | BLOCKED | owner rule: never push or tag |
| Phase 5 acceptance session, marker tag | BLOCKED | owner decision (ACC-01) + never tag |

## Session Continuity

Last session: 2026-11-22
Stopped at: Phase 1 closed; Phase 2 in progress
Resume file: .planning/phases/01-truth-baseline/.continue-here.md (superseded by this file)
