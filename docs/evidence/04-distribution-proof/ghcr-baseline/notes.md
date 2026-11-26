# ghcr-baseline — what the registry looked like around the single proof-tag push

Requirement: REL-04 (provenance/classification half). Item recorded 2026-11-26 after
Phase 4's one tag push.

## Read this first: this snapshot is **post-push**, and why that is still useful

The plan asked for a GHCR snapshot taken *before* `v0.1.0-proof1` was pushed. The push
**preceded this recording session**: the tag was pushed at 2026-11-25T23:54:21Z (workflow
run `45724842211` was created then) and the run finished at 2026-11-26T00:56:27Z, before
`baseline.json` was measured at 2026-11-26T06:00:43Z. No pre-push registry measurement
exists, and inventing one would be a fabricated baseline. It is recorded as a deviation
in `04-01-SUMMARY.md`.

What the measured snapshot does prove, and what it does not:

- **Proven:** the package now exposes exactly two tags — `0.1.0-proof1` and
  `v0.1.0-proof1` — and `:latest` answers **404**. That is the Phase-4 hard invariant
  ("no `:latest` may be created or moved"): after the only tag push of the milestone,
  there is no `:latest` at all.
- **Proven on the CI side instead of by a before/after pair:** the container job's own
  log records the prerelease branch of the manifest loop —
  `Pre-release — :latest unchanged` — and the guard step approved a two-name prerelease
  list (`guard-image-tags: OK 2 names (prerelease)`). Both lines are quoted in
  `../tag-run/notes.md`.
- **Not proven by this file:** that `:latest` was absent *before* the push. The reason it
  cannot have been created by this run is structural (the workflow only creates `:latest`
  when `is_prerelease=false`) and is falsified by the post-push 404.

Context that makes the post-push 404 meaningful: every release this repository has ever
published is a pre-release (`v0.1.0-proof1`, `v0.0.1-rc5`, `v0.0.1-rc4`), so no run has
ever taken the stable branch that pushes `:latest`. The `v0.0.1-rc5` image is itself
absent (`/v2/coruairc/deepseek-build/manifests/0.0.1-rc5` → 404), which is consistent with
that run being cancelled before its container job could start.

## How it was measured

`baseline.json` comes from `python3 scripts/distribution-proof.py baseline --out …`, which:

1. requests an **anonymous** pull token
   (`https://ghcr.io/token?service=ghcr.io&scope=repository:coruairc/deepseek-build:pull`),
2. reads `GET /v2/coruairc/deepseek-build/tags/list` with that bearer token,
3. reads `GET /v2/coruairc/deepseek-build/manifests/latest` with the OCI index/manifest
   accept set and records `present` (200) or `absent` (404) plus the registry's
   `Docker-Content-Digest` when present.

Transport fallbacks, in order, are `docker manifest inspect` (a registry client — **no
Docker daemon is contacted, and none is running on this host**) and
`gh api /users/coruairc/packages/container/deepseek-build/versions`. Only the first
transport answered; `transports_tried` in the JSON records that.

`gh api /user(s)/packages/…` returns **403 `read:packages`** for this account's token. That
is an authorization gap in the token, not evidence that the image is missing — do not read
the 403 as a registry result. Plan 04-04 uses `docker manifest inspect` for the same reason.

## probe-rc5.json — the harness's offline→online smoke test

`probe-rc5.json` is the thin end-to-end slice run against the existing `v0.0.1-rc5`
pre-release before the proof tag existed: release metadata, all four assets downloaded to
a temp directory, each file hashed and compared with both the release's own `SHA256SUMS`
(2 tarball lines) and GitHub's reported asset `digest`. Result:
`probe: OK v0.0.1-rc5 — 4 assets, 2 verified against SHA256SUMS, 0 mismatches`. The
downloaded tarballs (~406 MB) were deleted; only the JSON is committed.

## Files

| File | Meaning |
|------|---------|
| `baseline.json` | measured GHCR state (post-push; tags + `:latest`) |
| `probe-rc5.json` | rc5 release metadata + per-asset digest verification |
| `fixtures.log` | the 35-case offline assertion suite (valid + 12 named negatives) |
| `probe-rc5.log` | the probe invocation's own output |
| `baseline.log` | the baseline invocation's own output |
| `report.json` | the item record (`scripts/evidence.py finish --result ok`) |

## Re-run

```sh
python3 scripts/distribution-proof.py fixtures
python3 scripts/distribution-proof.py probe --tag v0.0.1-rc5 --out /tmp/probe-rc5.json
python3 scripts/distribution-proof.py baseline --out /tmp/baseline.json
```

The probe needs ~2 GB free in `$TMPDIR` while the four rc5 assets are on disk; it deletes
them itself.
