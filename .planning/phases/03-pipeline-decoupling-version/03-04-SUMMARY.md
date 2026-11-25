---
phase: 03-pipeline-decoupling-version
plan: 04
subsystem: release-pipeline
tags: [tag-shape, semver, prerelease, install-sh, static-gate, github-actions, shell-fixture]

requires:
  - phase: 03-pipeline-decoupling-version
    provides: "03-01's strict semver gate in the `version` job (which rejects `+` build metadata) and the split that makes `container` independent of macOS; 03-02's workspace version 0.1.0; 03-03's 19-assertion workflow checker with its 5-case macOS artifact fixture"
provides:
  - "`scripts/check-release-tag-shape.py`: 17/17 assertions — trigger is exactly push tags ['v*'] + workflow_dispatch with no 'deepseek' string; the `version` job's own derive body is extracted at test time and executed against 9 ref fixtures; release gating + prerelease wiring asserted from the parsed YAML"
  - "Derive results: v0.1.0 -> `version=0.1.0` / `is_prerelease=false`; v0.1.0-rc1 and v0.1.0-proof1 -> prerelease; branch -> `0.0.0-<sha7>`; v0.1, v1, v0.1.0.0, v0.1.0+build.5 and 1.0.0 all rejected non-zero with a named error and no `$GITHUB_OUTPUT`"
  - "`scripts/test-install.sh`: 58/58 assertions (was 39) — pinned stable `0.1.0` and `v0.1.0` installs, the Phase 4 `v0.1.0-proof1` tag shape, an invalid VERSION rejection, and the recorded `VERSION=0.1` finding; `install.sh` byte-unchanged"
  - "`scripts/test-guard-image-tags.sh`: 20/20 assertions (was 18) — the `0.1.0-proof1` two-name set accepted, the same set with `latest` rejected"
  - "`scripts/check-release-pipeline.sh`: one command for every static release gate, with all five tallies pinned so a shrunken checker fails"
affects: [04-distribution-proof]

actuals:
  tokens: 8642            # chars/4 over the realized diff (34,568 chars: 734 insertions, 6 deletions)
  tasks: 2
  commits: 2              # measured: git rev-list --count 05f13580..fa865111
  plan_head_before: 05f13580adaf8878425c78a679f81463a7d5c816
  plan_head_after: fa865111445f0023beaf8e5ee2d6bc4e0d5bb3f1

tech-stack:
  added: []
  patterns:
    - "Extract a CI shell body from the workflow YAML and execute it verbatim with a fresh `$GITHUB_OUTPUT` per fixture; a re-typed copy would test the copy, not the shipped file"
    - "Parse `$GITHUB_OUTPUT` key/values instead of matching stdout substrings, and assert a rejection wrote nothing at all"
    - "Pin each checker's tally in the one-command gate: a checker that silently stopped running assertions fails the gate instead of passing on a shorter list"
    - "Record an installer leniency as a finding with its exact output, and pin the pipelined gate that makes the lenient shape unreachable, rather than rewriting install.sh out of scope"

key-files:
  created:
    - scripts/check-release-tag-shape.py
    - scripts/check-release-pipeline.sh
  modified:
    - scripts/test-install.sh
    - scripts/test-guard-image-tags.sh
    - scripts/README.md

key-decisions:
  - "The derive fixtures are 9 rows / 17 assertions, not the plan's 7 rows: the review revisions add `v0.1.0-proof1` (the only tag Phase 4 pushes) and `v0.1.0+build.5`, which must FAIL. The shipped 03-01 gate already rejects `+`, so revision 2 needed no workflow change — only the fixture row that pins it."
  - "`v0.1.0+build.5` is asserted as a rejection with the named semver error, overriding the plan body's earlier 'accept build metadata' expectation: the pinned metadata-action sanitizes `+` to `-`, so a release created from such a tag would not match its assets."
  - "The guard fixture's `0.1.0-proof1` case is both accepted (exactly `{0.1.0-proof1, v0.1.0-proof1}`, 2 names) and rejected with `latest` added, so 'no latest' is asserted rather than implied by the accepted shape."
  - "`check-release-pipeline.sh` pins all five tallies (workflow 19/19, artifact fixture 5/5, guard 20/20, tag-shape 17/17, installer 58/58), beyond the two the review revision names: the same silent-shrinkage failure applies to every checker, and the mechanism was proven by deletion in a copied tree."
  - "`VERSION=0.1` is recorded as a finding, not fixed: install.sh's `v[0-9]*` prefix check accepts it and the fetch then 404s; release.yml's `version` job rejects `v0.1` before any build, so the shape is unreachable for a real release. No installer edit was made."
  - "The installer harness's final line changed from `ALL PASS (39 assertions)` to `ALL PASS (58/58 assertions)` to satisfy the plan's `<n>/<n>` requirement; nothing in the tree parsed the old text."

patterns-established:
  - "A tag shape is proven by running the workflow's own parse, not by re-reading it: the derive body is written to a temp file and executed once per fixture."
  - "An installer contract is proven end to end against the loopback fake tree for every version form a release can produce, including the exact prerelease tag Phase 4 pushes."

requirements-completed: [PKG-03]
requirements-open:
  - "REL-02 — both macOS legs inside the 240-minute budget is still unproven (needs the Phase 4 tag run)"
  - "REL-03 — the macOS execution half: a real Mach-O signature verified by `codesign`/`otool` on a macOS host"

coverage:
  - id: T1
    description: "The trigger is exactly `push: {tags: ['v*']}` plus `workflow_dispatch`, asserted from the parsed YAML, with no string containing `deepseek`."
    requirement: PKG-03
    verification:
      - kind: unit
        ref: "python3 scripts/check-release-tag-shape.py # A1, A2, A3"
        status: pass
    human_judgment: false
  - id: T2
    description: "The workflow's own derive body maps v0.1.0 stable, v0.1.0-rc1 / v0.1.0-proof1 prerelease, branch to 0.0.0-<sha7>, and rejects v0.1 / v1 / v0.1.0.0 / v0.1.0+build.5 / 1.0.0 with a named error."
    requirement: PKG-03
    verification:
      - kind: integration
        ref: "python3 scripts/check-release-tag-shape.py # A5-A13, derive body extracted from release.yml per run"
        status: pass
    human_judgment: false
  - id: T3
    description: "The release step stays tag-gated and wired to `needs.version.outputs.is_prerelease`; `publish` has no job-level if; `container` stays tag-gated."
    requirement: PKG-03
    verification:
      - kind: unit
        ref: "python3 scripts/check-release-tag-shape.py # A14-A17"
        status: pass
    human_judgment: false
  - id: T4
    description: "install.sh installs pinned 0.1.0 from the fake release tree via both version forms and the 0.1.0-proof1 tag, verifies the checksum before extraction, and rejects an invalid VERSION before any download."
    requirement: PKG-03
    verification:
      - kind: integration
        ref: "sh scripts/test-install.sh # stable-0.1.0, stable-v0.1.0, proof-0.1.0-proof1, invalid-version, loose-version-0.1; 58/58"
        status: pass
    human_judgment: false
  - id: T5
    description: "One command runs actionlint, shellcheck, both Python checkers, the guard fixture and the installer harness, exiting 0 with every tally pinned."
    verification:
      - kind: integration
        ref: "sh scripts/check-release-pipeline.sh # exit 0, 'release-pipeline: all gates PASS'"
        status: pass
    human_judgment: false
  - id: T6
    description: "The `VERSION=0.1` leniency is recorded with its exact output rather than fixed, and install.sh is byte-unchanged."
    verification:
      - kind: other
        ref: "git diff --exit-code -- install.sh (empty); manual loopback probe: `==> Downloading deepseek-build-0.1-x86_64-unknown-linux-gnu.tar.gz` then `curl: (22) The requested URL returned error: 404`, rc=22"
        status: pass
    human_judgment: false
---

# Phase 3 Plan 04: `v0.1.0` tag-shape verification Summary

**The `v0.1.0` tag shape is proven against the workflow's own derive body — `v0.1.0` is stable, `v0.1.0-rc1`/`v0.1.0-proof1` are prereleases, five non-semver shapes die before a build — and the installer installs pinned `0.1.0`/`0.1.0-proof1` from a fake tree; one command now runs all six static release gates at pinned tallies.**

## Performance

- **Duration:** ~14 min (plan load ≈ 2026-11-25T23:37Z → close 2026-11-25T23:51Z)
- **Started:** 2026-11-25T23:37Z (approx)
- **Completed:** 2026-11-25T23:51Z
- **Tasks:** 2/2
- **Files modified:** 5 (2 created, 3 modified)

## Accomplishments

- `scripts/check-release-tag-shape.py` — **17/17 assertions**:
  - Trigger: exactly `push: {tags: ['v*']}` + `workflow_dispatch`; `push` has no other key; no string anywhere in the trigger contains `deepseek`. Read from the parsed YAML (`on:` is the boolean `True` under PyYAML), so a text grep cannot match `deepseek-build` in the jobs.
  - Derive: the step with `id: derive` is located in `jobs.version.steps`, its `run:` body written to a temp file and executed with `bash` — a fresh `$GITHUB_OUTPUT` per fixture, `GITHUB_REF_TYPE`/`GITHUB_REF_NAME`/`GITHUB_SHA` passed through the environment only. Nine rows, all PASS (table below).
  - Gating: the `softprops/action-gh-release` step's `if` is exactly `github.ref_type == 'tag'`; its `with.prerelease` is exactly `${{ needs.version.outputs.is_prerelease }}`; `publish` has **no** job-level `if`; `container` remains `if: github.ref_type == 'tag'`.
- `scripts/test-install.sh` — **58/58 assertions** (was 39). New: pinned `VERSION=0.1.0`, `VERSION=v0.1.0`, `VERSION=v0.1.0-proof1` (tarball name, installed tag, printed version), an invalid VERSION that dies before any download, and the `VERSION=0.1` finding.
- `scripts/test-guard-image-tags.sh` — **20/20 assertions** (was 18). New: `0.1.0-proof1` two-name set accepted; the same set plus `latest` rejected.
- `scripts/check-release-pipeline.sh` — one command, six gates, all five tallies pinned, final line `release-pipeline: all gates PASS`, exit 0.
- `scripts/README.md` — entries for both new scripts; the guard and installer lines now name the `0.1.0-proof1` / stable `0.1.0` coverage.
- `install.sh` is **byte-unchanged** (`git diff --exit-code -- install.sh` empty).

## Derive fixture matrix (executed from `release.yml`)

| # | Ref (`GITHUB_REF_TYPE` / `GITHUB_REF_NAME`) | Expected | Observed (`$GITHUB_OUTPUT` unless rejected) | Result |
|---|---|---|---|---|
| A5 | tag / `v0.1.0` | exit 0, `version=0.1.0`, `is_prerelease=false` | exit 0, `{'version': '0.1.0', 'is_prerelease': 'false'}` | PASS |
| A6 | tag / `v0.1.0-rc1` | exit 0, `version=0.1.0-rc1`, `is_prerelease=true` | exit 0, `{'version': '0.1.0-rc1', 'is_prerelease': 'true'}` | PASS |
| A7 | tag / `v0.1.0-proof1` | exit 0, `version=0.1.0-proof1`, `is_prerelease=true` | exit 0, `{'version': '0.1.0-proof1', 'is_prerelease': 'true'}` | PASS |
| A8 | tag / `v0.1` | non-zero, named semver error, no output | exit 1, `::error::release tag 'v0.1' is not a semver tag (expected vX.Y.Z or vX.Y.Z-<pre>)`, `$GITHUB_OUTPUT` empty | PASS |
| A9 | tag / `v1` | non-zero, named semver error, no output | exit 1, `::error::release tag 'v1' is not a semver tag (expected vX.Y.Z or vX.Y.Z-<pre>)`, empty | PASS |
| A10 | tag / `v0.1.0.0` | non-zero, named semver error, no output | exit 1, `::error::release tag 'v0.1.0.0' is not a semver tag (expected vX.Y.Z or vX.Y.Z-<pre>)`, empty | PASS |
| A11 | tag / `v0.1.0+build.5` | non-zero, named semver error, no output | exit 1, `::error::release tag 'v0.1.0+build.5' is not a semver tag (expected vX.Y.Z or vX.Y.Z-<pre>)`, empty | PASS |
| A12 | branch / `main`, `GITHUB_SHA=abcdef1234567890` | exit 0, `version=0.0.0-abcdef1`, `is_prerelease=true` | exit 0, `{'version': '0.0.0-abcdef1', 'is_prerelease': 'true'}` | PASS |
| A13 | tag / `1.0.0` (no `v`) | non-zero, named trigger error | exit 1, `::error::release tags must match 'v*' (got '1.0.0')` | PASS |

The exact `$GITHUB_OUTPUT` for the pair the plan asks to record: **`v0.1.0` → `version=0.1.0` + `is_prerelease=false`; `v0.1.0-rc1` → `version=0.1.0-rc1` + `is_prerelease=true`; `v0.1.0-proof1` → `version=0.1.0-proof1` + `is_prerelease=true`.**

Non-vacuity was checked by mutation against temp copies of `release.yml` (the real file was never modified):

| Mutation | Checker result |
|---|---|
| derive step renamed off `id: derive` | A4 and every fixture FAIL, exit 1 |
| `deepseek-*` added to `push.tags` | A2 + A3 FAIL (`15/17`), exit 1 |
| release step's `if:` removed | A14 FAIL (`16/17`), exit 1 |
| semver regex loosened to accept `+` | A11 FAIL (`16/17`), exit 1 |

## Installer harness — new cases (19 new assertions)

| Case | Assertions | Evidence | Result |
|---|---|---|---|
| `stable-0.1.0` (`VERSION=0.1.0`) | 4 | binary installed; `Downloading deepseek-build-0.1.0-<host>.tar.gz`; `Installed v0.1.0 to`; `deepseek-build 0.1.0` | PASS ×4 |
| `stable-v0.1.0` (`VERSION=v0.1.0`) | 4 | same tarball name, same tag, same version — the two forms are equivalent | PASS ×4 |
| `proof-0.1.0-proof1` (`VERSION=v0.1.0-proof1`) | 4 | `Downloading deepseek-build-0.1.0-proof1-<host>.tar.gz`; `Installed v0.1.0-proof1 to`; `deepseek-build 0.1.0-proof1` | PASS ×4 |
| `invalid-version` (`VERSION=not-a-version`) | 3 | non-zero; `Invalid VERSION: not-a-version`; nothing installed | PASS ×3 |
| `loose-version-0.1` (`VERSION=0.1`) | 4 | non-zero; `Downloading deepseek-build-0.1-<host>.tar.gz` (past validation); no `Installed v`; nothing installed | PASS ×4 |

Existing assertions unchanged in behaviour: the shared tree's `latest` still resolves `v0.0.1-test` (`latest-install: resolved newest tag` PASS), the hyphenated `0.0.0-test`/`0.0.1-test` cases still pass, and `install.sh` was not edited. Final line: **`ALL PASS (58/58 assertions)`** (was `ALL PASS (39 assertions)`).

## The `VERSION=0.1` finding (recorded, not fixed)

`install.sh:274-281` validates the tag with `case "$VERSION_TAG" in v[0-9]*)`, so `VERSION=0.1` passes validation — the tag becomes `v0.1`, the tarball name `deepseek-build-0.1-<target>.tar.gz` — and then the fetch 404s. Verbatim from a loopback probe:

```
==> Downloading deepseek-build-0.1-x86_64-unknown-linux-gnu.tar.gz
curl: (22) The requested URL returned error: 404
```

exit 22, nothing installed. The shape is unreachable for a real release because `release.yml`'s `version` job rejects `v0.1` before any build (`is not a semver tag`, A8 above). `install.sh` is unchanged; the harness pins the failure loudly.

## One-command gate (`sh scripts/check-release-pipeline.sh`, exit 0)

```
==> 1/6 actionlint: .github/workflows/release.yml
==> 2/6 shellcheck: install.sh and the phase's shell scripts
==> 3/6 release-workflow structure checker (plus the macOS artifact fixture)
artifact-step fixture: 5/5 PASS
release-workflow: 19/19 assertions PASS
==> 4/6 image tag-list guard fixture
20/20 assertions passed, 0 failed
==> 5/6 v0.1.0 tag-shape checker
tag-shape: 17/17 assertions PASS
==> 6/6 installer harness (loopback fake release tree)
ALL PASS (58/58 assertions)
==> release-pipeline: all gates PASS
```

The tallies are pinned in the script. Proven non-vacuous by copying the tree and deleting one assertion / one fixture case: the gate exits 1 with `expected 'release-workflow: 19/19 assertions PASS'` (checker printed `18/18`) and `expected 'artifact-step fixture: 5/5 PASS'` (fixture printed `4/4`) respectively.

## Task Commits

Each task was committed atomically:

1. **Task 1: prove the `v0.1.0` tag shape against the workflow's own derive body** — `9d57b0c6` (feat)
2. **Task 2: one command for every static release gate; pin the stable installer contract** — `fa865111` (feat)

**Plan metadata:** this SUMMARY commit.

## Files Created/Modified

- `scripts/check-release-tag-shape.py` (new, 530 lines) — trigger assertions, derive extraction/execution with 9 fixtures, release gating; `tag-shape: 17/17 assertions PASS`.
- `scripts/check-release-pipeline.sh` (new, 83 lines) — six gates, five pinned tallies, no cargo, loopback-only network.
- `scripts/test-install.sh` — `FAKE_VERSION_STABLE=0.1.0`, `FAKE_VERSION_PROOF=0.1.0-proof1`, two new download directories + tarballs in `build_fake_tree`, the `Case 1c` section, and the `<n>/<n>` summary line.
- `scripts/test-guard-image-tags.sh` — `prerelease-proof1-two-names` (accepted) and `prerelease-proof1-with-latest` (rejected); header comment updated.
- `scripts/README.md` — entries for `check-release-pipeline.sh` and `check-release-tag-shape.py`; guard/installer lines updated.

## Decisions Made

- Nine derive rows rather than seven: the review revisions require `v0.1.0-proof1` (the exact Phase 4 tag) and a `v0.1.0+build.5` rejection. The shipped 03-01 gate already excludes `+`, so the second revision needed a fixture, not a workflow change.
- Five pinned tallies in the gate rather than the two the revision names: the silent-shrinkage failure mode applies to every checker, and the mechanism is one `expect` call either way. Each `expect` prints the failing checker's output and a message naming the stale count.
- The guard's `latest` prohibition for `proof1` is asserted twice (accepted set is exactly two names; the three-name set with `latest` fails), so a future change that starts publishing `:latest` from a prerelease fails locally.
- `install.sh`'s `VERSION=0.1` leniency stays: the pipeline rejects the tag, the installer only needs to fail loudly, and rewriting the installer was explicitly out of scope.

## Deviations from Plan

### Review revisions applied (mandatory — they win over the task text)

1. **`v0.1.0-proof1` everywhere.** Derive row A7 (`version=0.1.0-proof1`, `is_prerelease=true`), guard fixture (`{0.1.0-proof1, v0.1.0-proof1}`, no `latest` — two rows), installer case (`VERSION=v0.1.0-proof1` resolves `deepseek-build-0.1.0-proof1-<target>.tar.gz`). The guard fixture file was not in `files_modified`; the revision mandates the row, so the file is touched and recorded here.
2. **Build metadata rejected.** Plan Task 1(b) said `v0.1.0+build.5` should exit 0 with `version=0.1.0+build.5`; the revision requires a FAIL with the named semver error, and that is what A11 asserts. The shipped gate already behaves that way (it excludes `+`), so the workflow file was not edited.
3. **Pinned checker counts.** The gate asserts `release-workflow: 19/19 assertions PASS` and `artifact-step fixture: 5/5 PASS` exactly, and additionally pins the guard (20/20), tag-shape (17/17) and installer (58/58) tallies.

### Auto-fixed Issues

**1. [Rule 2 - Missing critical test coverage] The installer summary was not `<n>/<n>`-style and no consumer proved the new section ran**

- **Found during:** Task 2
- **Issue:** the harness ended with `ALL PASS (39 assertions)`, so a deleted stable-release case would leave a green run with an unread count.
- **Fix:** the final line is `ALL PASS (58/58 assertions)`, and the one-command gate greps for that exact line (plus the other four tallies), so a shrunken harness fails the gate.
- **Files modified:** `scripts/test-install.sh`, `scripts/check-release-pipeline.sh`
- **Verification:** gate exits 0 at 58/58; the deletion experiment exits 1 naming the stale expected tally.
- **Committed in:** `fa865111`

### Plan-vs-dispatch note

The plan's `<success_criteria>` says "No commit made; the working tree is left for the reviewer". The dispatch explicitly instructed atomic per-task commits with `(03-04):` subjects, so the two task commits and this SUMMARY commit exist **locally on `main`** (not pushed, no tag, no branch), matching 03-01…03-03.

**Total deviations:** 3 mandated review revisions + 1 auto-fixed test-coverage gap + 1 dispatch-vs-plan note.
**Impact:** no workflow, profile or `install.sh` change; scope is the four script/test files plus the README.

## Issues Encountered

- `actionlint` resolves `.github/actionlint.yaml` from the git repository root; in the disposable copied tree used for the shrinkage experiment it needed a `git init` before gate 1 passed. That is a property of the experiment only — in the real repo `actionlint` exits 0.
- PyYAML resolves the bare `on:` key to the boolean `True`; the checker uses `doc.get(True, doc.get("on"))` and a missing trigger fails rather than being skipped.

## Verification Results

| Check | Command | Result |
|-------|---------|--------|
| Tag shape | `python3 scripts/check-release-tag-shape.py` | **17/17 assertions PASS**, exit 0 |
| Tag-shape syntax | `python3 -c "import ast; ast.parse(...)"` | OK |
| Checker mutation tests | derive removed / `deepseek-*` tag / ungated release / `+` accepted | all exit 1 (see table) |
| Installer harness | `sh scripts/test-install.sh` | **58/58**, exit 0 |
| Guard fixture | `sh scripts/test-guard-image-tags.sh` | **20/20**, exit 0 |
| One-command gate | `sh scripts/check-release-pipeline.sh` | exit 0, `release-pipeline: all gates PASS` |
| Gate shrinkage proof | copy tree, drop A19 / drop one fixture case | gate exits 1 naming the stale tally |
| shellcheck | `shellcheck --shell=sh install.sh scripts/test-install.sh scripts/guard-image-tags.sh scripts/test-guard-image-tags.sh scripts/check-release-pipeline.sh` | clean |
| actionlint | `actionlint .github/workflows/release.yml` | clean (exit 0, no output) |
| install.sh unchanged | `git diff --exit-code -- install.sh` | empty |
| No side effects | `git tag --points-at HEAD` empty; branch `main`; no `gh`/dispatch/tag/push | PASS |

## NOT RUN (open by design — REL-02 and the macOS half of REL-03)

Nothing here claims a macOS artifact, a GHCR image or a published release. No CI run, dispatch, tag or push was made. The first evidence is Phase 4's throwaway pre-release tag (`v0.1.0-proof1`); its acceptance commands:

```sh
# The run exists and is the tag's:
gh run list --workflow=release.yml --limit 3

# All four artifacts arrived (expect release-<target> x4):
gh api /repos/coruairc/deepseek-build/actions/runs/<run-id>/artifacts -q '.artifacts[].name'

# The macOS legs finished inside 240 minutes:
gh run view <run-id> --json jobs -q '.jobs[] | select(.name|startswith("build-macos")) | .name, .conclusion, .startedAt, .completedAt'

# The image tag list for a prerelease tag (expect 0.1.0-proof1 + v0.1.0-proof1, no latest):
gh run view <run-id> --log | grep 'guard-image-tags:'

# On a macOS host, against the extracted release tarball:
codesign --verify --strict --verbose=2 ./deepseek-build
codesign -dv ./deepseek-build 2>&1 | grep 'Signature='
otool -l ./deepseek-build | grep -c __DWARF        # expect 0
find target -name '*.dSYM'                          # expect empty
./deepseek-build --version                          # expect 'deepseek-build 0.1.0-proof1 (<hex>)'

# install.sh against the real prerelease (no --version tag prefix):
curl -fsSL https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/install.sh | VERSION=v0.1.0-proof1 sh
```

If 240 minutes is still not enough on the macOS legs, the pre-decided next lever (apply to `build-macos` only) is `CARGO_PROFILE_RELEASE_DIST_LTO=off` + `CARGO_PROFILE_RELEASE_DIST_CODEGEN_UNITS=16`.

## Next Phase Readiness

- PKG-03 is complete: the trigger, the parse, the prerelease classification and `install.sh`'s validation are all proven against `v0.1.0` (and the Phase 4 `v0.1.0-proof1`) with no `deepseek-*` trigger.
- Phase 4 can quote a single command (`sh scripts/check-release-pipeline.sh`) for the static half of its evidence.
- Phase 3's only open items are **REL-02** (both macOS legs inside 240 minutes) and the **macOS execution half of REL-03** (real `codesign`/`otool` on a macOS host); both require the Phase 4 tag run.
- `04-01-PLAN.md` records the README baseline `not codesigned and not notarized = 1`; 03-03 already changed that sentence, so 04-01 must re-measure its count rather than assert it from plan text (unchanged from 03-03's note).

---

*Phase: 03-pipeline-decoupling-version*
*Completed: 2026-11-25*

## Self-Check: PASSED

- Key files exist on disk: `scripts/check-release-tag-shape.py`, `scripts/check-release-pipeline.sh`, `scripts/test-install.sh`, `scripts/test-guard-image-tags.sh`, `scripts/README.md`, this SUMMARY.
- Both task commits are ancestors of HEAD: `9d57b0c6`, `fa865111` (ledger-measured count: 2).
- Plan-level verification re-run at close: tag-shape `17/17`, guard `20/20`, installer `58/58`, one-command gate exit 0, `actionlint` clean, `install.sh` unchanged.
