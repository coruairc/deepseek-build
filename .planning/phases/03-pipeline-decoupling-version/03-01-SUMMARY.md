---
phase: 03-pipeline-decoupling-version
plan: 01
subsystem: infra
tags: [github-actions, release-pipeline, docker-metadata-action, ghcr, semver, shellcheck]

requires:
  - phase: 02-loop-item-verification
    provides: a verified product tree and a committed handoff (no artifact is built from a moving tree)

provides:
  - "release.yml with two build jobs (build-linux, build-macos) and no `build` job"
  - "publish needs [version, build-linux, build-macos]; container needs [version, build-linux]"
  - "--locked on both release-dist cargo builds (stale Cargo.lock is a red job)"
  - "scripts/check-release-workflow.py — 15 executable assertions over the parsed workflow"
  - "scripts/guard-image-tags.sh — pre-push image tag-list guard with resolved-name diagnostics"
  - "scripts/test-guard-image-tags.sh — 18-fixture suite for the guard"
  - "strict semver gate in the version job (rejects v0.1, v1, v0.1.0.0 and any +build metadata)"

affects: [04-distribution-proof, 05-windows-support, 06-documentation-truth-freeze]

actuals:
  tokens: 11368          # chars/4 over the realized diff of the two task commits (.github + scripts)
  tasks: 2
  commits: 2             # this plan's commits: bd3193ec, ba3b4912
plan_head_before: 49ff61c12b7365ae5d46a8aa2ee75756df916254
plan_head_after: ba3b49125154f4c8c5d3fb8879180898d40ddc82

tech-stack:
  added: []
  patterns:
    - "Parsed-YAML assertions over the release workflow instead of text greps"
    - "POSIX sh guard with fixture suite, reason substrings asserted (not just exit codes)"

key-files:
  created:
    - scripts/check-release-workflow.py
    - scripts/guard-image-tags.sh
    - scripts/test-guard-image-tags.sh
  modified:
    - .github/workflows/release.yml
    - scripts/README.md

key-decisions:
  - "The macOS job keeps the 90-minute budget in this plan; REL-02 (both legs inside the timeout) is re-verified by Phase 4's throwaway pre-release tag run and is NOT claimed closed here."
  - "The `{{major}}` alias resolving to `0` for v0.1.0 is ACCEPTED and published as ghcr.io/...:0, not suppressed; no `enable=` conditional was added. Stated in the guard header and the workflow comment."
  - "The prerelease tag list accepts BOTH shapes: the 2-name collapsed set and the 4-name expanded set (version, v-version, major.minor, major). `latest` and any name outside that set are rejected; major.minor/major are optional but at most once."
  - "The guard prints every resolved tag name before its verdict so the Phase-4 evidence can quote exactly which refs were pushed."
  - "`--locked` and 03-02's regenerated Cargo.lock must land together before the Phase 4 tag run; no workflow runs during Phase 3."

patterns-established:
  - "Workflow structure is executable: scripts/check-release-workflow.py is the gate for job graph, needs, matrices, artifact names and step ordering."
  - "A CI shell step that decides what gets published is a script under scripts/ with a fixture suite, not an inline `run:`."

requirements-completed: [REL-01, REL-08]

coverage:
  - id: D1
    description: "release.yml declares exactly version/build-linux/build-macos/publish/container; the build job is gone; publish needs both build jobs and container needs only build-linux."
    requirement: REL-01
    verification:
      - kind: unit
        ref: "python3 scripts/check-release-workflow.py (A1, A5, A6) -> 15/15 PASS"
        status: pass
    human_judgment: false
  - id: D2
    description: "Each build job owns only its own OS matrix and steps, passes --locked, and uploads release-<target> with if-no-files-found=error."
    requirement: REL-01
    verification:
      - kind: unit
        ref: "python3 scripts/check-release-workflow.py (A2, A3, A4, A7, A8) -> PASS"
        status: pass
      - kind: unit
        ref: "/home/cein_orourke/.local/bin/actionlint .github/workflows/release.yml -> exit 0"
        status: pass
    human_judgment: false
  - id: D3
    description: "The image tag list is guarded after metadata-action and before both build-push steps, and the version job rejects a non-semver tag first."
    requirement: REL-08
    verification:
      - kind: unit
        ref: "python3 scripts/check-release-workflow.py (A14, A15) -> PASS"
        status: pass
    human_judgment: false
  - id: D4
    description: "Every guard failure mode is fixture-proven: empty list, prerelease latest, name outside the set, each missing stable alias, extra alias, ref outside IMAGE, duplicate."
    requirement: REL-08
    verification:
      - kind: unit
        ref: "sh scripts/test-guard-image-tags.sh -> 18/18 assertions passed, 0 failed"
        status: pass
      - kind: unit
        ref: "shellcheck --shell=sh scripts/guard-image-tags.sh scripts/test-guard-image-tags.sh -> exit 0"
        status: pass
    human_judgment: false
  - id: D5
    description: "REL-02: both macOS targets produce tarballs inside the job timeout on a real run, creating no release and moving no :latest."
    requirement: REL-02
    verification: []
    human_judgment: true
    rationale: "Cannot be proven locally or statically. The first evidence is Phase 4's throwaway pre-release tag run; this plan explicitly leaves REL-02 open."
  - id: D6
    description: "The edited workflow parses and runs on GitHub Actions (no syntax/expression error at dispatch time)."
    verification: []
    human_judgment: true
    rationale: "actionlint is a static linter; only a real run proves the runner accepts the file. Deferred to Phase 4 by decision (no CI dispatch in Phase 3)."

duration: 5 min
completed: 2026-11-25
status: complete
---

# Phase 3 Plan 01: Split build-linux/build-macos and guard the image tag list Summary

**Release graph split into two OS-scoped build jobs (container no longer waits for macOS), with a 15-assertion workflow checker, a POSIX tag-list guard wired before the first push, and a strict semver gate in the version job.**

## Performance

- **Duration:** 5 min (2026-11-25T23:21:21Z → 2026-11-25T23:26:07Z)
- **Started:** 2026-11-25T23:21:21Z
- **Completed:** 2026-11-25T23:26:07Z
- **Tasks:** 2
- **Files modified:** 5 (3 created, 2 modified)

## Accomplishments

- `release.yml` now has five jobs — `version`, `build-linux`, `build-macos`, `publish`, `container` — and no `build`. `build-linux` carries the two gnu legs only, `build-macos` the two apple legs only; every `if: runner.os` conditional is deleted rather than left dead.
- The `needs` edges as written: both build jobs `needs: version`; `publish.needs: [version, build-linux, build-macos]`; `container.needs: [version, build-linux]` with `container.if: github.ref_type == 'tag'` unchanged and `publish` still without a job-level `if`.
- Artifact contract unchanged: both build jobs upload `release-${{ matrix.target }}` with `if-no-files-found: error`; `publish` merges `release-*` and its verify step still names all four targets; `container` downloads exactly `release-x86_64-unknown-linux-gnu` and `release-aarch64-unknown-linux-gnu`.
- Both build jobs now pass `--locked`, so a stale `Cargo.lock` fails the job instead of being silently regenerated.
- `scripts/guard-image-tags.sh` runs between `Extract metadata (tags + labels)` and both `build-push-action` steps, asserts the exact tag list, prints the resolved names, and fails every bad list with an `::error::`.
- The `version` job rejects a non-semver tag (`v0.1`, `v1`, `v0.1.0.0`, and any `+build` metadata) before any build or push starts.
- `scripts/check-release-workflow.py` makes the whole graph executable: 15/15 assertions PASS on the final tree.

## Task Commits

Each task was committed atomically:

1. **Task 1: Split the build matrix and prove the job graph** — `bd3193ec` (feat)
2. **Task 2: Guard the tag list and reject non-semver tags** — `ba3b4912` (feat)

**Plan metadata:** `docs(03-01): complete the pipeline decoupling plan` (see git log; created after this file)

## Checker assertion list and result

Final run: `release-workflow: 15/15 assertions PASS`, exit 0.

| ID | Assertion | Final |
|----|-----------|-------|
| A1 | jobs are exactly {version, build-linux, build-macos, publish, container} | PASS |
| A2 | `build-linux`: `needs [version]`, two Linux legs, `fail-fast: false` | PASS |
| A3 | `build-macos`: `needs [version]`, two macOS legs, `fail-fast: false` | PASS |
| A4 | no `runner.os` conditional left in either build job | PASS |
| A5 | `publish.needs == [version, build-linux, build-macos]`, no job `if` | PASS |
| A6 | `container.needs == [version, build-linux]`, `if == github.ref_type == 'tag'` | PASS |
| A7 | one `cargo build` per build job with `--profile release-dist`, `-p xai-grok-pager-bin`, `--locked`, stamped `GROK_VERSION` | PASS |
| A8 | one `upload-artifact` per build job: `release-${{ matrix.target }}`, `if-no-files-found: error` | PASS |
| A9 | `publish` merges `release-*` (`merge-multiple: true`) and verifies all four tarballs | PASS |
| A10 | `container` downloads exactly the two Linux artifacts | PASS |
| A11 | derive step carries the prerelease regex; release step is tag-gated and passes `prerelease` | PASS |
| A12 | no `run:` body invokes `strip` or `dsymutil` | PASS |
| A13 | every `uses:` is pinned to a 40-hex SHA | PASS |
| A14 | tag guard sits between metadata-action and both pushes, run is `scripts/guard-image-tags.sh`, env carries IMAGE/TAGS/RELEASE_VERSION/IS_PRERELEASE | PASS |
| A15 | derive step carries the strict semver gate and a named `::error::` | PASS |

### Red baseline (checker against the unmodified workflow, before Task 1 edits)

```
FAIL: A1 (missing build-linux, missing build-macos, unexpected build)
FAIL: A2 (job 'build-linux' is missing)
FAIL: A3 (job 'build-macos' is missing)
FAIL: A4 (jobs missing)
FAIL: A5 (needs ['version','build'], expected ['version','build-linux','build-macos'])
FAIL: A6 (needs ['version','build'], expected ['version','build-linux'])
FAIL: A7 (jobs missing)
FAIL: A8 (jobs missing)
PASS: A9 A10 A11 A12 A13
release-workflow: 5/13 assertions PASS, 8 FAILED   (exit 1)
```

Note: A4/A7/A8 initially passed vacuously while their job was absent (their loops simply skipped). They were changed to report the missing job as a violation before the split was made, so the checker cannot pass on an absent job. That is the only checker change beyond the plan's assertion list.

## Guard fixture suite — 18 cases, all PASS

`sh scripts/test-guard-image-tags.sh` → `18/18 assertions passed, 0 failed`, exit 0.

| # | Fixture | Mode | Expectation | Result |
|---|---------|------|-------------|--------|
| 1 | `stable-exact` | stable | exit 0, `OK 5 names (stable)`, names printed | PASS |
| 2 | `stable-shuffled` | stable | exit 0 (order independent) | PASS |
| 3 | `stable-with-blank-lines` | stable | exit 0 (blank lines dropped) | PASS |
| 4 | `prerelease-two-names` | prerelease | exit 0, `OK 2 names (prerelease)` | PASS |
| 5 | `prerelease-four-names` | prerelease | exit 0, `OK 4 names (prerelease)` | PASS |
| 6 | `prerelease-rc-two-names` (`0.1.0-rc1`) | prerelease | exit 0 | PASS |
| 7 | `prerelease-rc-four-names` (`0.1.0-rc1`, `0.1`, `0`) | prerelease | exit 0 | PASS |
| 8 | `empty-list` | stable | non-zero, `tag list is empty` | PASS |
| 9 | `prerelease-with-latest` | prerelease (4 names + latest) | non-zero, `'latest' must not be present` | PASS |
| 10 | `prerelease-outside-set` (`0.3`) | prerelease | non-zero, `'0.3' is not allowed for a prerelease` | PASS |
| 11 | `stable-missing-version` | stable | non-zero, `'0.1.0' must appear exactly once` | PASS |
| 12 | `stable-missing-vversion` | stable | non-zero, `'v0.1.0' must appear exactly once` | PASS |
| 13 | `stable-missing-minor` | stable | non-zero, `'0.1' must appear exactly once` | PASS |
| 14 | `stable-missing-major` | stable | non-zero, `'0' must appear exactly once` | PASS |
| 15 | `stable-missing-latest` | stable | non-zero, `'latest' must appear exactly once` | PASS |
| 16 | `stable-extra-alias` (`0.2`) | stable | non-zero, `'0.2' is not in the allowed set` | PASS |
| 17 | `ref-outside-image` | stable | non-zero, `is not under '…:'` | PASS |
| 18 | `duplicate-version` | stable | non-zero, `duplicate tag name '0.1.0'` | PASS |

Guard diagnostics as executed (the lines Phase 4 can quote):

```
stable (0.1.0):
  guard-image-tags: resolved tag names (5): 0.1.0, v0.1.0, 0.1, 0, latest
  guard-image-tags: OK 5 names (stable)
prerelease 2-name (0.1.0-rc1): 0.1.0-rc1, v0.1.0-rc1 -> OK 2 names (prerelease)
prerelease 4-name (0.1.0-rc1): 0.1.0-rc1, v0.1.0-rc1, 0.1, 0 -> OK 4 names (prerelease)
```

(The actual script prints one name per line; the above is compacted for this table.)

## Other verification run

- `actionlint .github/workflows/release.yml` → exit 0 (1.7.7, with `.github/actionlint.yaml` declaring `macos-15-intel`).
- `shellcheck --shell=sh scripts/guard-image-tags.sh scripts/test-guard-image-tags.sh` → exit 0 (0.10.0).
- `git diff` reviewed: the set of `uses:` values is identical to HEAD's (`diff` of sorted pin lists is empty) — every "added" pin line is a duplicated step, no pin changed, no new action, no `deepseek-*` trigger, `publish` has no `if:`.
- The `README.txt` heredoc inside `Package tarball` is byte-identical to HEAD's (diff empty).
- `test -x scripts/guard-image-tags.sh -a -x scripts/test-guard-image-tags.sh` → both executable.
- Workflow trigger block unchanged: `push: tags: ['v*']`, `workflow_dispatch: {}`.
- **No branch, tag, push, `gh` command or workflow dispatch was created.** `git tag --points-at HEAD` is empty; the branch is `main` (project config `git.branching_strategy: none`).

## Semver gate — local regex evidence (not a workflow run)

The exact regex from the derive step was exercised on the candidate tags:

| Tag | Verdict |
|-----|---------|
| `v0.1.0` | ACCEPT |
| `v0.1.0-rc1` | ACCEPT |
| `v0.1.0+build.5` | REJECT (build metadata) |
| `v0.1` | REJECT |
| `v1` | REJECT |
| `v0.1.0.0` | REJECT |
| `0.0.0-abc1234` (dispatch fallback) | ACCEPT |

This proves the regex, not the job: the gate is only truly exercised by a tag push in Phase 4.

## Decisions Made

- **REL-02 stays open.** The macOS job keeps `timeout-minutes: 90` (the plan's scope; the budget/strip work is 03-03 and the live proof is Phase 4). Both macOS legs inside the budget is NOT claimed here.
- **Major alias `:0` is intentional.** `{{major}}` resolves to `0` for `v0.1.0`; it is published, not suppressed. No `enable=` conditional was added. Recorded in the guard header and the workflow comment (review revision 4).
- **Prerelease accepts 2-name and 4-name lists.** `version` and `v<version>` are required exactly once; `major.minor` and `major` are optional but at most once each; `latest` and anything outside the set are forbidden. This supersedes the plan's revision item 2 (see Deviations).
- **The guard prints the resolved names before its verdict** so the Phase-4 release run can record exactly which refs were pushed.

## Deviations from Plan

### Contract deviation (dispatched mid-execution)

**1. Prerelease tag-list contract corrected: tolerant of 2-name and 4-name shapes**
- **Found during:** Task 2, while writing the guard.
- **Issue:** The plan's "Review revisions (mandatory)" item 2 required a prerelease list of exactly two names (`<version>`, `v<version>`) and asserted the other aliases absent. A dispatching-session contract correction states the pinned `docker/metadata-action` may either collapse `{{major}}.{{minor}}`/`{{major}}` into the full prerelease string (2 names) or emit them expanded (`0.1.0-proof1`, `0.1`, `0`, `v0.1.0-proof1` = 4 names), so a guard hardcoded either way would fail a real tag run.
- **Fix:** Prerelease now requires `<version>` and `v<version>` exactly once; `latest` is forbidden; `<major>.<minor>` and `<major>` are optional (at most once each, enforced by the duplicate check); anything outside `{version, v-version, major.minor, major}` fails. The guard also prints every resolved name before the verdict. Fixtures cover both accepted shapes (plus both rc shapes) and the 4-name-plus-`latest` rejection.
- **Files modified:** `scripts/guard-image-tags.sh`, `scripts/test-guard-image-tags.sh`.
- **Verification:** 18/18 fixtures PASS; shellcheck clean.
- **Committed in:** `ba3b4912`

**2. Stable "all five aliases" requirement kept as specified**
- No change: `<version>`, `v<version>`, `<major>.<minor>`, `<major>`, `latest` are each required exactly once, with `stable-missing-minor` and `stable-missing-major` fixtures failing. This is the correction's "Stable — unchanged" clause.

### Process deviations

**3. Commits were made, although the plan's `<output>` says "leave the working tree uncommitted"**
- **Found during:** Task 1.
- **Reason:** The dispatching session's brief explicitly instructs "Commit each task atomically with `(03-01):` subjects, then `03-01-SUMMARY.md`, then commit it." The brief is the execution contract for this run, so per-task commits were made. If a reviewer expects an uncommitted tree, `git reset --soft 49ff61c1` restores it (the two commits are `bd3193ec` and `ba3b4912`).
- **Impact:** None on the tree's correctness; only on the plan's literal output instruction.

**4. Plan-phase artifacts were untracked at execution start**
- `?? .planning/phases/03-pipeline-decoupling-version/` (PLAN/CONTEXT/RESEARCH/DISCUSSION-LOG and 03-02…03-04 plans) was not in git when this plan began. The whole `03-pipeline-decoupling-version/` directory is included in the metadata commit so the SUMMARY lands with its PLAN. (Phases 04–06 were committed concurrently by another session in `e311a3c4`; they were not touched.)

**5. `--locked` and 03-02's lock regeneration must land together**
- **Found during:** Task 1 (checker flag).
- **Issue:** Both build jobs now pass `--locked`. Until 03-02 regenerates `Cargo.lock` for the `0.1.0` workspace version, a build would fail on the stale lock. No workflow runs during Phase 3, so nothing is broken now.
- **Requirement:** `--locked` and 03-02's regenerated `Cargo.lock` must both be on `main` before the Phase 4 tag run. Phase 4 must not create a tag from a tree where 03-02 is incomplete.

**6. Concurrent commit inside the ledger range**
- **Found during:** SUMMARY metrics.
- **Issue:** `git rev-list --count 49ff61c1..HEAD` returns 3 because `e311a3c4` (phase 4/5/6 planning docs) was committed by another session between Task 1 and Task 2.
- **Handling:** `actuals.commits` records this plan's 2 commits (`bd3193ec`, `ba3b4912`); the range count is 3. No files from `e311a3c4` were staged or altered.

**7. Small hardening inside the guard (no behavioral change to the accepted lists)**
- `IS_PRERELEASE` is validated to be exactly `true`/`false`; `IMAGE` is rejected if it carries a tag; `RELEASE_VERSION` must be `X.Y.Z[-pre]` with a numeric major. These are input-validation guards for a workflow step that decides what is published (Rule 2, missing critical validation).

---

**Total deviations:** 5 recorded (1 contract correction, 4 process/hardening; no behavioural scope creep beyond the corrected prerelease contract).
**Impact on plan:** The split, the `needs` edges, the `--locked` flag, the semver gate and the pre-push guard all landed as specified. REL-02 remains explicitly open.

## Issues Encountered

- Two rc fixtures initially failed because the harness pins `RELEASE_VERSION=0.1.0`; a `check_v` wrapper now runs a fixture with `RELEASE_VERSION=0.1.0-rc1`. Resolved before the suite was committed.
- No other issues.

## Deferred / NOT RUN (explicitly)

**REL-02 is NOT proven by this plan.** The acceptance commands for Phase 4, documented but **NOT run**:

```sh
gh run list --workflow=release.yml --limit 3
gh api /repos/<owner>/<repo>/actions/runs/<id>/artifacts     # expect 4 release-* artifacts
docker buildx imagetools inspect ghcr.io/<owner>/deepseek-build:latest
```

Also NOT RUN, by decision: any `workflow_dispatch`, branch push, tag creation, or `cargo` build. The workflow edits are shipped untested on a runner; the first live evidence is Phase 4's throwaway pre-release tag run. If a macOS leg exceeds the budget there, that run is the finding and Phase 3 re-opens (REL-02).

Deferred to Phase 4/5 as planned: the GHCR image pull and in-image run, the README GHCR block, `install.sh` against a real release, and docs reconciliation.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Ready for 03-02 (`[workspace.package] version = "0.1.0"` + four inheriting crates + regenerated `Cargo.lock`). Its lock regeneration is a hard prerequisite for the Phase 4 tag because of `--locked`.
- 03-03 owns the macOS budget/strip/sign work; REL-02 cannot close before 03-03 and a Phase-4 run.
- The checker and fixture suite are cheap gates to re-run after 03-03 edits: `python3 scripts/check-release-workflow.py && sh scripts/test-guard-image-tags.sh && actionlint .github/workflows/release.yml`. Note A12 will reject a `run:` body that invokes an external `strip`, which 03-03 is explicitly forbidden from doing anyway.

---

*Phase: 03-pipeline-decoupling-version*
*Completed: 2026-11-25*

## Self-Check: PASSED

- Created files exist: `scripts/check-release-workflow.py`, `scripts/guard-image-tags.sh`, `scripts/test-guard-image-tags.sh`, `.planning/phases/03-pipeline-decoupling-version/03-01-SUMMARY.md`.
- Commits exist on this branch: `bd3193ec` (Task 1), `ba3b4912` (Task 2).
- Final gates re-run at SUMMARY write: checker 15/15 PASS; fixture suite 18/18 PASS; actionlint exit 0; shellcheck exit 0; both scripts executable.
