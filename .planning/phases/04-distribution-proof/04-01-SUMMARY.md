---
phase: 04-distribution-proof
plan: 01
subsystem: distribution
tags: [github-actions, ghcr, release-run, macos, codesign, evidence, rel-02, rel-03]

requires:
  - phase: 03-pipeline-decoupling-version
    provides: the five-job release split (build-linux / build-macos / publish / container), ad-hoc signing plus the extracted-tarball verify, scripts/guard-image-tags.sh, scripts/check-release-pipeline.sh, and the v0.1.0 tag-shape checker
provides:
  - "`v0.1.0-proof1` pushed once, tag-only, peeling to b14d28eb; no branch, no other tag, no v0.1.0 moved"
  - scripts/distribution-proof.py — daemonless harness (fixtures / probe / baseline / run / release, registry stub)
  - docs/evidence/04-distribution-proof/ghcr-baseline — GHCR snapshot and the rc5 probe
  - docs/evidence/04-distribution-proof/tag-push — expected SHA, remote refs, guard runs, push record
  - docs/evidence/04-distribution-proof/tag-run — run.json, release.json, the release's SHA256SUMS, checksums.log
  - "the multi-arch OCI index digest `sha256:8f5c484e…` and the two pushed image tag names, for 04-04"
affects: [04-02, 04-03, 04-04, 04-05]

actuals:
  tokens: 28973          # chars/4 over the 38 files this plan added (115,894 bytes)
  tasks: 3
  commits: 3             # measured d971abb7..48dec6fc; the metadata commit carrying this file is the 4th

tech-stack:
  added: []              # stdlib Python only — no package manifest changed, nothing installed
  patterns:
    - "Pure assertion functions over payload dicts, with an offline fixture suite that proves each negative fails with a named reason"
    - "Machine-read evidence: run/release JSON built from `gh --json` plus log parsing, never a transcribed claim"
    - "Registry reads over anonymous HTTPS (urllib) or `docker manifest inspect`; no Docker daemon is ever contacted"

key-files:
  created:
    - scripts/distribution-proof.py
    - docs/evidence/04-distribution-proof/ghcr-baseline/notes.md
    - docs/evidence/04-distribution-proof/ghcr-baseline/baseline.json
    - docs/evidence/04-distribution-proof/ghcr-baseline/probe-rc5.json
    - docs/evidence/04-distribution-proof/tag-push/expected-sha.txt
    - docs/evidence/04-distribution-proof/tag-push/main-before.txt
    - docs/evidence/04-distribution-proof/tag-push/snapshot-before.txt
    - docs/evidence/04-distribution-proof/tag-push/push.log
    - docs/evidence/04-distribution-proof/tag-run/run.json
    - docs/evidence/04-distribution-proof/tag-run/release.json
    - docs/evidence/04-distribution-proof/tag-run/SHA256SUMS
    - docs/evidence/04-distribution-proof/tag-run/checksums.log
  modified: []

key-decisions:
  - "`expected-sha.txt` records the tag's target (`git rev-parse 'v0.1.0-proof1^{commit}'` = b14d28eb…), not HEAD: main advanced one commit (d971abb7) after the run, and that commit is not part of the artifacts."
  - "The push preceded this session and was not repeated — no push, re-push, dry run or retag. The tag-push item is a read-only re-measurement and says so."
  - "baseline.json is a post-push GHCR snapshot (the pre-push moment was not captured); the `:latest` invariant is proven by the post-push 404 plus the container log's own `Pre-release — :latest unchanged` line."
  - "The image tag list actually pushed is two names, not the four the plan derived: the pinned docker/metadata-action collapses semver aliases for a pre-release. The guard was run against both shapes (2-name and 4-name) and accepted both."
  - "REL-04 is left unmarked (PARTIAL): the image is built, published and verified registry-side, but the pull is environment-blocked (no Docker daemon) and belongs to 04-04."

patterns-established:
  - "A tag-run proof is only accepted from `gh run view --json` plus the job log; a timeout writes a failure, never a pass."
  - "A release is trusted only when the downloaded bytes match two independent digests (the release's SHA256SUMS and GitHub's asset digest)."

requirements-completed: []   # deliberately empty — REL-04 is PARTIAL; see "Requirements" below

coverage:
  - id: D1
    description: "Daemonless proof harness with 35 fixture cases (12 named negatives) and a working rc5 probe"
    requirement: REL-04
    verification:
      - kind: automated
        ref: "python3 scripts/distribution-proof.py fixtures"
        status: pass
      - kind: automated
        ref: "python3 scripts/distribution-proof.py probe --tag v0.0.1-rc5"
        status: pass
    human_judgment: false
  - id: D2
    description: "One tag-only push, proven not to have moved main, the snapshot tag, or created v0.1.0"
    requirement: REL-04
    verification:
      - kind: automated
        ref: "docs/evidence/04-distribution-proof/tag-push/report.json (8 steps, result ok)"
        status: pass
      - kind: automated
        ref: "docs/evidence/04-distribution-proof/tag-push/push.log"
        status: pass
    human_judgment: false
  - id: D3
    description: "Green five-job release run with container present; both macOS legs inside 240 min with sign+verify steps green (REL-02, macOS half of REL-03)"
    verification:
      - kind: automated
        ref: "docs/evidence/04-distribution-proof/tag-run/run.json"
        status: pass
    human_judgment: false
  - id: D4
    description: "Release is a pre-release with exactly six assets; every byte matches SHA256SUMS and GitHub's digest; tarballs carry five entries, no debug; extracted linux binary reports the tag version"
    requirement: REL-04
    verification:
      - kind: automated
        ref: "docs/evidence/04-distribution-proof/tag-run/release.json"
        status: pass
      - kind: automated
        ref: "docs/evidence/04-distribution-proof/tag-run/checksums.log"
        status: pass
    human_judgment: false
  - id: D5
    description: "REL-04's pull half — pulling and running the published image as a user would"
    requirement: REL-04
    verification: []
    human_judgment: true
    rationale: "No container runtime on this host (docker CLI present, daemon absent). Registry-side facts are recorded; the pull is deferred to 04-04 and must not be claimed as done."

duration: 10min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 01: One throwaway tag, a green seven-job run, six verified assets and no `:latest`

**`v0.1.0-proof1` (tag-only) produced a fully green release run — both macOS legs 47.8 / 61.5 min against the 240 min budget, container pushed two prerelease tags and never touched `:latest` — and a pre-release with exactly six assets whose bytes match both SHA256SUMS and GitHub's own digests.**

## Performance

- **Duration:** ~10 min of recording work (the tag run it records took 62 min, 2026-11-25T23:54 → 2026-11-26T00:56)
- **Started:** 2026-11-26T05:56Z (recording session)
- **Completed:** 2026-11-26T06:07Z
- **Tasks:** 3/3
- **Files modified:** 38 committed files (1 script + 37 evidence files); no product source touched

## Accomplishments

- **The single tag did what it had to and nothing else.** `v0.1.0-proof1` is an annotated tag at `b14d28eb…`, pushed tag-only. `origin/main` is still `6e9d8d88…`, `deepseek-v0.1-snapshot` still peels to `4b305bd0…`, and no `refs/tags/v0.1.0` exists on the origin.
- **The run is green with the Phase-3 split visible in the data**: all seven jobs `success`, the `container` job present and `success` *while* the macOS legs were still the long pole (it finished at 00:25:44, 30 minutes before `publish` at 00:56). That is REL-01's decoupling demonstrated live.
- **REL-02 is closed.** macOS aarch64 47.8 min, x86_64 61.5 min, both inside the 240-minute `build-macos` timeout with 192.2 / 178.5 min of margin, each producing an uploaded tarball.
- **The macOS half of REL-03 is closed.** Both legs ad-hoc signed, then verified the signature on the **extracted tarball** and printed `deepseek-build 0.1.0-proof1 (b14d28eb2bc7)` from it.
- **The release is correctly classified and complete**: `prerelease: true`, `draft: false`, exactly six assets — four target tarballs, `SHA256SUMS`, `install.sh`. Every downloaded byte matches its `SHA256SUMS` line *and* GitHub's asset `digest`; each tarball holds exactly the five top-level entries and no `.dSYM`/`.debug`; the linux-x86_64 binary runs from the extracted tarball.
- **No `:latest` and no `v0.1.0`.** The guard approved a two-name prerelease list, the manifest loop printed its prerelease branch, and the package's tag list is exactly `0.1.0-proof1`, `v0.1.0-proof1` with `latest` 404.

## Task Commits

Each task was committed atomically:

1. **Task 1: Build the harness and probe the existing rc5 pre-release** — `f81fcc91` (feat)
2. **Task 2: Pre-flight and the single tag push — record and re-measure** — `87867fda` (docs)
3. **Task 3: Run, release set and classification** — `48dec6fc` (docs)

**Plan metadata:** `docs(04-01): complete …` commit carrying this SUMMARY, STATE/ROADMAP/REQUIREMENTS.

## The evidence, as recorded

### The tag

| Field | Value |
|-------|-------|
| Tag | `v0.1.0-proof1`, annotated, tag object `5006c5194929899f28083426acd0367100628f86` |
| Target | `b14d28eb2bc71a046b789e0a705848d651685d35` |
| Push | 2026-11-25T23:54:21Z, `git push origin v0.1.0-proof1` (single ref) |
| `origin/main` | `6e9d8d88dca11bda39afea8d3b82a8f80111d207` — unchanged |
| `deepseek-v0.1-snapshot` | `68d8590f…` → peels to `4b305bd0c1ab2fbb55468df14ab6a7e94d7c8d0b` — unchanged |
| `refs/tags/v0.1.0` | absent |

### The run

`45724842211` — https://github.com/coruairc/deepseek-build/actions/runs/45724842211 — `push` / `v0.1.0-proof1`, head `b14d28eb…`, `success`, 2026-11-25T23:54:21Z → 2026-11-26T00:56:27Z.

| Job | Conclusion | Minutes |
|-----|------------|---------|
| Derive release version | success | 0.1 |
| build-linux (x86_64-unknown-linux-gnu) | success | 28.0 |
| build-linux (aarch64-unknown-linux-gnu) | success | 23.9 |
| build-macos (aarch64-apple-darwin) | success | **47.8** |
| build-macos (x86_64-apple-darwin) | success | **61.5** |
| Build and push container image | success | 3.2 |
| Publish GitHub release | success | 0.4 |

### Container: guard line, pushed names, no `:latest`

Verbatim from the container job log (`run.json` records the same values):

```
guard-image-tags: OK 2 names (prerelease)
Pre-release — :latest unchanged
amd64=sha256:7f33b41bad8811fd7550808fa8a21fd5247480b7defacb793ea385bc8c566f7b arm64=sha256:85f9de25a9863bc5c33340ccb6b0f39a4f44adc14125cf5805deda926c264410
Digest:    sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be
```

Parsed pushed tag names (`pushing manifest for ghcr.io/coruairc/deepseek-build:<name>@…`):
**`0.1.0-proof1`**, **`v0.1.0-proof1`** — `latest` absent.

- **`image_digest` for 04-04:** `sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be` (multi-arch OCI index).
- Platform manifests inside that index (registry-side, `docker manifest inspect` without a daemon): linux/amd64 `sha256:1d216b609ed9c646cb56a5e6294bc3de5276b2823d2a90a0252b49d90b99886e`, linux/arm64 `sha256:25dc0d256765ff129b14749732f5d158b1757a540828c643410a3f142dcb6f58`. The `build_amd64`/`build_arm64` outputs (`7f33b41b…` / `85f9de25…`) are buildkit's per-platform **indexes** and are *not* the platform manifests — 04-04 must use the `1d216b60…` / `25dc0d25…` pair for a manifest-list assertion.
- The `gh api /users/coruairc/packages/container/...` path returns **403 `read:packages`** for this token. That is an authorization gap, not a missing image; use `docker manifest inspect`.

### The release

`v0.1.0-proof1` — https://github.com/coruairc/deepseek-build/releases/tag/v0.1.0-proof1 — `prerelease: true`, `draft: false`, published 2026-11-26T00:56:19Z.

| Asset | Size | `digest` |
|-------|------|----------|
| `deepseek-build-0.1.0-proof1-aarch64-apple-darwin.tar.gz` | 49,591,197 | `sha256:2b61dfab870b397b49ed810f53e073714582e01dbf5c6f9d166656367c51324b` |
| `deepseek-build-0.1.0-proof1-aarch64-unknown-linux-gnu.tar.gz` | 53,622,321 | `sha256:0cfcf04796fbdaa098c5b23c55b22d4385d3c2af564fa357482fad68837c7e9a` |
| `deepseek-build-0.1.0-proof1-x86_64-apple-darwin.tar.gz` | 53,850,000 | `sha256:ffe0f3c5f76be9ee6fa2dff1d53c34de6dce66d3553a094ee62524f58f57b3a7` |
| `deepseek-build-0.1.0-proof1-x86_64-unknown-linux-gnu.tar.gz` | 55,967,373 | `sha256:298c9f275d695fa43872bd44a0fe9da25ee66bd86a6051ac233689063ccbdaf3` |
| `install.sh` | 14,552 | `sha256:31e114d4a29dbbeb0d3e2e442a72a6ecc14ad3f4b99956fbff23f42c0cabd5e5` |
| `SHA256SUMS` | 496 | `sha256:0183ee9bccae0101945831b4b621aefce7db65a84563c8f0c3c5015bc3b2995a` |

`SHA256SUMS` (the release's own file, copied verbatim into `tag-run/SHA256SUMS`) has exactly four lines, one per tarball; `sha256sum -c` is 4/4 OK and all six files match GitHub's digest. The two Apple tarball names and digests above are the inputs 04-03's Homebrew formula needs.

No `v0.1.0`: `releases/tags/v0.1.0` → **404**, `releases/latest` → **404** (no stable release has ever been published), release list = `v0.1.0-proof1`, `v0.0.1-rc5`, `v0.0.1-rc4`.

### The harness

`scripts/distribution-proof.py` (stdlib only, executable): `fixtures` (35 cases, 12 named negatives), `probe` (release + double digest check; proven against `v0.0.1-rc5`: 4 assets, 2 SHA256SUMS lines, 0 mismatches), `baseline` (anonymous ghcr.io token → tags + `:latest`, with `docker manifest inspect` and `gh api` fallbacks), `run` (poll `gh run view`, summarize jobs/steps/durations, parse the container log for the guard line, pushed names, `:latest` line and index digest), `release` (download, checksum, tar members, Mach-O/version checks, `v0.1.0` absence). `registry` is a stub that exits 2, implemented in 04-04.

## Files Created/Modified

- `scripts/distribution-proof.py` — the daemonless proof harness above.
- `docs/evidence/04-distribution-proof/ghcr-baseline/` — `baseline.json` (post-push registry state, `:latest` 404), `probe-rc5.json`, `notes.md`, `report.json` (3 steps, ok), step logs.
- `docs/evidence/04-distribution-proof/tag-push/` — `expected-sha.txt` (`b14d28eb…`), `main-before.txt`, `snapshot-before.txt`, `push.log` (static gates, both guard shapes, all post-push ref checks), `notes.md`, `report.json` (8 steps, ok), step logs.
- `docs/evidence/04-distribution-proof/tag-run/` — `run.json`, `release.json`, `SHA256SUMS`, `checksums.log`, `notes.md`, `report.json` (9 steps, ok), step logs.

## Decisions Made

- **`expected-sha.txt` = the tag's target, not HEAD.** HEAD had advanced to `d971abb7` (an unrelated hermetic-test fix made after the run) by recording time; the plan's "record `git rev-parse HEAD`" would have recorded the wrong commit. The tag's peeled ref is the fact that matters.
- **The push was recorded, never repeated.** No push, re-push, dry run, delete or workflow dispatch was issued by this executor.
- **`baseline.json` is a post-push snapshot, labelled as such.** The pre-push moment was not captured; inventing a "before" value would be fabricated evidence. The `:latest` invariant is carried by the post-push 404 and the container log's own prerelease line.
- **Two pushed names, not four.** The plan derived four tags from the workflow's four patterns; the pinned `docker/metadata-action` emitted two for a pre-release. `push.log` records the guard accepting both shapes, and `run.json` records what was actually pushed.
- **REL-04 is PARTIAL.** Built, published, digest-verified registry-side; the pull is environment-blocked and deferred to 04-04. `requirements-completed` is deliberately empty.

## Deviations from Plan

**1. [Rule 2 - Missing critical functionality] The dry run and push transcript do not exist**
- **Found during:** Task 2
- **Issue:** The plan's Task 2(e) expects a `git push --dry-run` transcript followed by the real push output in `push.log`. The push happened before this recording session (2026-11-25T23:54:21Z), and the executor was instructed not to repeat it.
- **Fix:** `push.log` section 1 states this explicitly; the rest of the log holds live read-only re-measurements (six static gates, both guard shapes, every post-push ref). `tag-push/report.json` records 8 real steps. A fabricated dry-run/push transcript was not written.
- **Files modified:** `docs/evidence/04-distribution-proof/tag-push/*`
- **Commit:** `87867fda`

**2. [Rule 1 - Bug] `baseline.json` could not be the pre-push snapshot**
- **Found during:** Task 2
- **Issue:** The plan asks for the GHCR state *before* the push; the registry had already been written to by the time this session started.
- **Fix:** Measured the post-push state and labelled it (`checked_at`, `note`, `notes.md`). The hard invariant (`:latest` untouched) rests on the 404 plus the run's own prerelease branch line.
- **Files modified:** `docs/evidence/04-distribution-proof/ghcr-baseline/*`
- **Commit:** `f81fcc91`

**3. [Rule 1 - Bug] The plan's `expected-sha = HEAD` assumption was stale**
- **Found during:** Task 2
- **Issue:** HEAD was `d971abb7`, one commit past the tag target `b14d28eb`. Recording HEAD would have pointed the evidence at a commit that no artifact was built from.
- **Fix:** Recorded `git rev-parse 'v0.1.0-proof1^{commit}'`; `notes.md` explains the drift.
- **Commit:** `87867fda`

**4. [Rule 2 - Missing critical functionality] The plan's README baseline is stale (measured, not reverted)**
- **Found during:** pre-flight measurement
- **Issue:** The plan's context block says `not codesigned and not notarized` = 1 in README.md. It is **0** — 03-03 already replaced that sentence with the ad-hoc-signed truth ("ad-hoc signed … not notarized", `ad-hoc` = 1). The other baselines measured unchanged: `v0.1.0-proof1` = 0, `Not yet built or published` = 1, `<owner>` = 6, `sandbox devbox` = 0, `sha256:` = 0; `packaging/homebrew/` absent.
- **Fix:** Re-measured and recorded here; nothing was reverted. **04-02/04-03 must not gate on the old value of 1.**
- **Files modified:** none (measurement only)

**5. [Rule 1 - Bug] Two dispatcher facts contradicted the measured data**
- **Found during:** Task 3
- **Issue:** (a) The dispatcher stated "the old 90-minute ceiling would have failed both" macOS legs; the measured durations are 47.8 and 61.5 minutes, both **inside** 90. (b) The plan predicted a four-name image tag list; the run pushed two.
- **Fix:** Recorded the measured truth in `tag-run/notes.md` and `tag-push/notes.md` rather than repeating either claim. REL-02 still holds: the requirement is "inside the job timeout" (240 min), and the rc4 cancellation at 90 minutes is what motivated the raise.
- **Files modified:** `docs/evidence/04-distribution-proof/tag-run/notes.md`, `tag-push/notes.md`
- **Commit:** `48dec6fc`

**6. [Rule 3 - Blocking issue] Evidence items needed one re-record**
- **Found during:** Task 3
- **Issue:** `checksums.log` was recorded as an artifact before `evidence.py normalize` ran, so its digest reflected un-normalized `/tmp` paths.
- **Fix:** Normalized the log, then re-ran `begin`/`step`/`artifact`/`finish` for the item so `report.json`'s digests are over the normalized files (9/9 steps ok).
- **Commit:** `48dec6fc`

**7. [Rule 3 - Blocking issue] The plan says "no commit"; the dispatch requires commits**
- **Found during:** close-out
- **Issue:** The plan's success criteria say "No commit made; the working tree is left for the reviewer", but the executing dispatch instructed one atomic commit per task plus a SUMMARY commit.
- **Fix:** Followed the dispatch: three task commits (`f81fcc91`, `87867fda`, `48dec6fc`) and the metadata commit. The pre-existing working-tree edit (`crates/codegen/xai-grok-agent/src/builder.rs`, from the later hermetic-test commit) was not staged or touched.
- **Commit:** `f81fcc91`, `87867fda`, `48dec6fc`

## Issues Encountered

- The Docker CLI exists but **no daemon** (`Cannot connect to the Docker daemon`). Every registry read in this plan used anonymous HTTPS or `docker manifest inspect`; nothing required a daemon. REL-04's pull remains blocked and is 04-04's.
- `gh` token scopes: `repo`, `gist`, `read:org`, `admin:public_key`, `read:project` — **no `read:packages`**, so the packages REST API 403s. Not a defect; documented in the baseline notes.

## Requirements

- **REL-02 — closed by this run.** Both macOS targets produced tarballs inside the 240-minute budget (47.8 / 61.5 min).
- **REL-03 — the macOS half is closed.** The packaged macOS artifact passes its smoke check from the extracted tarball and the ad-hoc signature still validates after `tar` on both arches, with no `__DWARF` and no `.dSYM`.
- **REL-04 — PARTIAL.** *Built, published and verified registry-side* (index digest, platform manifests, tag set, `:latest` absent). *Not pulled:* no container runtime on this host. Deferred to 04-04, which must not claim a pull it cannot perform either; it records the exact commands.
- **REL-06 remains open** — the installer proof is 04-02, which consumes this release's assets and the digests above.
- **PKG-04 remains open** — the Homebrew formula is 04-03, using the two Apple digests above.
- **Keep the pre-release**: CONTEXT.md's decision to retain it (it is the only macOS artifact) is a deviation from the roadmap's "clean up" line; it is recorded in 04-05's deviation file. Nothing was deleted or unpublished here.

## Known Stubs

- `scripts/distribution-proof.py registry` — intentionally exits 2 with a named message; the registry-side proof body is 04-04's deliverable. This is a declared stub, not an unfinished claim: no evidence item depends on it in this plan.

## Threat Flags

None. No new network destination was added: the harness talks to the same `api.github.com`/`ghcr.io` endpoints the release pipeline already uses, and no secret was read or written. No package manager install ran (`npm`/`pip`/`cargo` untouched).

## Next

Ready for **04-02** (installer proof against this release) — it can consume `tag-run/SHA256SUMS`, the four asset digests and the pinned/unpinned release URLs recorded here.

## Self-Check: PASSED

- All 15 key files named in the frontmatter exist on disk (1 script, 12 evidence files and 2 item reports), as does this SUMMARY; the phase's evidence tree holds 37 files in total (ghcr-baseline 7, tag-push 15, tag-run 15).
- All three task commits are ancestors of HEAD: f81fcc91, 87867fda, 48dec6fc.
- Every artifact path recorded in the three `report.json` files resolves on disk.
- Plan-level verification re-run: fixtures 35/35, tag peels to expected-sha, origin/main unchanged, no v0.1.0 tag, snapshot unchanged, run.json/release.json assertions hold, no evidence file over 1 MiB, `deepseek-build 0.1.0-proof1 (` present in checksums.log, `gh release view` reports 6 assets.
- `scripts/key-leak-check.sh --scan docs/evidence` exits 0 (`PASS  no key-shaped string in: docs/evidence`).
