---
gsd_state_version: "1.0"
milestone: v0.1.0
current_phase: 2
current_phase_name: Loop-Item Verification
status: paused
stopped_at: Completed 02-04-PLAN.md
last_updated: "2026-11-25T21:22:43.922Z"
last_activity: "2026-11-25 21:21 UTC — handoff written; cargo test still holding /tmp/team-build.lock"
state_head: 757b1f17bbf0b857b1a47131cff9866efed3c6de
progress:
  total_phases: 5
  completed_phases: 1
  total_plans: 12
  completed_plans: 8
last_activity_desc: 02-04 complete; 02-01 remains complete; author had clocked off during evidence capture
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-11-23)

**Core value:** One DeepSeek key in, a trustworthy daily coding agent out: the right model on every turn, zero traffic to any host you did not configure, and a release you can install with confidence.
**Current focus:** Phase 02 — Loop-Item Verification

## Current Position

Phase: 2 — Loop-Item Verification
Current Plan: 3
Total Plans in Phase: 6
Plan: 02-01 complete (ec878ef0). 02-04 complete (b5a75b78, 4599b561, 30388a51). 02-02, 02-03, 02-05, 02-06 not started. The inherited key was rejected; 02-01 used the login-shell key and recorded that. Do not start a second live run.
Status: Paused — author clocked off; 02-04 finished after the handoff
Last activity: 2026-11-25 21:22 UTC — 02-04 SUMMARY written; no cargo test still running

Progress: [██░░░░░░░░] 20%

## Performance Metrics

**Velocity:**
- Total plans completed: 6
- Average duration: —
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01 | 6 | - | - |
| 02 | 6 planned | - | - |

**Recent Trend:**
- Last 5 plans: —
- Trend: —

*Updated after each plan completion*
**Per-Plan Metrics:**

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 02 P04 | 102m | 3 tasks | 14 files |

## Accumulated Context

### Decisions

Full log in PROJECT.md Key Decisions. Those shaping current work:

- Routing is cut from this milestone entirely — `route_turn`, `[models] routing`, session pins, the `Route` status-line item, `/route` and per-model cost are v2. `/model` and `/effort` are the manual switches. Do not wire, display, or price routing in any phase.
- What survives from the cost work is the honesty bug fix: `deepseek-v4-flash` priced in the table, the TUI figure labelled an estimate, headless honouring `[ui.status_line.pricing]`.
- `deepseek-flash` becomes the catalog default, with the at-ceiling caveat (63/63 both models) and the revert path (`/model deepseek-v4-pro`, `-m deepseek-v4-pro`) documented.
- Evidence per loop item = asciinema cast + `report.json` under `docs/evidence/`, absolute paths normalized. Phase 1 shipped the convention and four items (ipv6, test-gate, cost-estimate, determinism); Phase 2 is its first external consumer.
- **Determinism claims need an artifact-backed repetition:** a reported "5 runs, no rotation" was falsified by the Phase-1 verifier (2 failures in 35 runs). Fixed with a bounded 5 s/20 ms process-death poll and a committed 35/35 repetition item. Never accept a flake claim without a committed projection log — Phase 2's live/egress proofs inherit this standard.
- The sandbox fix comes before the clean-suite gate: only the product fix may zero the bwrap pair — never a skip, an `#[ignore]`, or a widened predicate.
- Publish tag is `v0.1.0` (no `deepseek-*` trigger added); the existing `deepseek-v0.1-snapshot` tag at `4b305bd0` is superseded in docs, never moved. This milestone does not publish `v0.1.0`.
- The freeze marker is non-triggering, and the author's signed acceptance note must be committed before branch `deepseek` or the marker tag exists.
- Container keeps the sandbox on, with documented run flags; no new outbound destination anywhere.
- The ~435 internal `GROK_*` env names and the `xai-grok-*` crate names are out of scope for this milestone (D2); renaming belongs to open-build.
- **Phase 2 live-proof key (user decision, 2026-11-23):** use the existing `DEEPSEEK_API_KEY` from the
  environment for PROOF-01/02/03; the author rotates/revokes it on platform.deepseek.com after the run
  (this also resolves the Phase-1 local-log exposure). The key is treated as compromised-by-policy:
  no key material in any committed artifact, and PROOF-03's revocation step is satisfied by the
  author's confirmed rotation, recorded as evidence.
- **Parallel lanes from Phase 2 (user decision, 2026-11-21):** Phase 1 finishes as one sequential lane.
  From Phase 2 onward, plans with disjoint `files_modified` may run in separate git worktrees under one
  shared `CARGO_TARGET_DIR`, one `flock` build lock, and one merge step per wave. This overrides the
  default "one implementer, one working tree" rule for this project; dependent plans (live-session
  proof, artifact builds, freeze) stay sequential. No plan is parallelized with a direct dependency.
- [Phase 02]: Permission warning is the wire title's first line, painted above the question; chrome height grows by one.
- [Phase 02]: Both headless denial lines are composed from PERMISSION_DENIAL_FLAGS naming --always-approve and --permission-mode auto.
- [Phase 02]: Pty session-grant proof uses ask mode without Bash(*) ask, because that rule skips the session-grant short-circuit.

### Pending Todos

None yet.

### Blockers/Concerns

- **Phase 2 host block — cleared on resume (2026-11-25):** the 2026-11-23 block was the remote
  Alpine box (no toolchain, 31 GB free, no key). This resume is on the author's build host:
  `rustc 1.94.0`, `scripts/disk-guard.sh` exit 0 (961 GB free). 02-01 has since executed.
  The inherited key was rejected; the login-shell key probed `models_ok` and is the one to
  rotate after Phase 2. The value was not read.
- **Acceptance gate (human, blocking):** ACC-01 needs a 20–30 minute TUI session on a real repo clone by the author. FREEZE-01/02/03 must not run until the signed note is committed — this is the milestone's only human-blocking dependency.
- **Snapshot tag is public:** `deepseek-v0.1-snapshot` already exists on origin at `4b305bd0`. It must be recorded as superseded in docs and left untouched; moving or deleting it is destructive.
- **Throwaway key for Phase 2 — RESOLVED (2026-11-23):** the author chose to use the existing env
  `DEEPSEEK_API_KEY` and rotate it after the run (see Decisions). PROOF-01/02/03 are unblocked;
  the rotation confirmation is a human action owed at the end of Phase 2.
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
| Review | IN-07 — default-mode `key-leak-check.sh` fail-open on grep rc≥2 (pre-existing; default mode frozen by plan 01-05 task 3(b); `--scan` fails closed) | Ledgered in KNOWN-ISSUES.md | 2026-11-23 | v0.1.x |

## Session Continuity

Last session: 2026-11-25T21:22:43.684Z
Stopped at: Completed 02-04-PLAN.md
Resume file: None
