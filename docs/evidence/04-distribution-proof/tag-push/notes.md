# tag-push — the one tag this milestone pushed, and what it moved

Requirement: REL-04 (provenance half).

## The tag

| Field | Value |
|-------|-------|
| Tag | `v0.1.0-proof1` (annotated — `refs/tags/v0.1.0-proof1` = `5006c519…`) |
| Target commit | `b14d28eb2bc71a046b789e0a705848d651685d35` (`docs(03-04): record the summary's status…`) |
| Pushed | 2026-11-25T23:54:21Z — the instant release run `45724842211` was created |
| Push command | `git push origin v0.1.0-proof1` — a **single ref** |
| Not run | `git push --tags`, `git push origin main`, `gh workflow run`, any retag |

The tag is the only network mutation of Phase 4. `expected-sha.txt` therefore records the
tag's target (`git rev-parse 'v0.1.0-proof1^{commit}'`), not "whatever HEAD was when this
evidence was written": by recording time `main` had advanced one commit past the tag
(`d971abb7 fix(xai-grok-agent): make the task-hiding tests hermetic`, committed
2026-11-26 05:53Z, after the run finished). The release was built from `b14d28eb` and its
binaries stamp `b14d28eb2bc7`; the later commit is unrelated to the artifacts and is not
part of them.

## What did NOT happen in this recording session

**The push preceded this session and was not repeated.** No dry run, no re-push, no
delete, no re-run of the workflow. `push.log` section 1 states this; there is no
`git push --dry-run` transcript because running one now would describe a second push that
never happened. This is the one place the plan's Task 2 wording ("dry-run … then push")
cannot be satisfied literally, and it is recorded as a deviation in
`../../../../.planning/phases/04-distribution-proof/04-01-SUMMARY.md`.

Everything else in `push.log` is a live, read-only re-measurement: the static release
gates, the image tag-list guard against two name lists, and the post-push refs.

## The image tag list: 2 names, not the predicted 4

The plan derived four names from the workflow's four `tags:` patterns
(`{{version}}`, `{{major}}.{{minor}}`, `{{major}}`, `ref,event=tag`):
`0.1.0-proof1`, `0.1`, `0`, `v0.1.0-proof1`. The pinned `docker/metadata-action`
(v5.10.0) **collapsed them to two** for a pre-release version — the run log's
`DOCKER_METADATA_OUTPUT_TAGS` carries only `0.1.0-proof1` and `v0.1.0-proof1`, and the
guard step printed `guard-image-tags: OK 2 names (prerelease)`.

Both shapes are accepted by the Phase-3 guard, and `push.log` records both runs:

- actual pushed list (2 names) → `guard-image-tags: OK 2 names (prerelease)`, exit 0
- the plan's predicted list (4 names) → `guard-image-tags: OK 4 names (prerelease)`, exit 0

So the guard did not reject anything, and no `latest` appears in either list. The names
that were actually pushed are taken from the container log's
`pushing manifest for ghcr.io/coruairc/deepseek-build:<name>@sha256:…` lines, not guessed
(see `../tag-run/notes.md`).

## Remote refs, before and after

| Ref | Value | Meaning |
|-----|-------|---------|
| `refs/heads/main` | `6e9d8d88dca11bda39afea8d3b82a8f80111d207` | unchanged: the tag push moved no branch (`main-before.txt`, re-measured identical) |
| `refs/tags/v0.1.0-proof1^{}` | `b14d28eb2bc71a046b789e0a705848d651685d35` | peels to the recorded commit |
| `refs/tags/v0.1.0` | *(absent)* | no stable tag exists — `git ls-remote --tags origin refs/tags/v0.1.0` is empty |
| `refs/tags/deepseek-v0.1-snapshot^{}` | `4b305bd0c1ab2fbb55468df14ab6a7e94d7c8d0b` | unchanged (matches the roadmap's `4b305bd0`) |

`main-before.txt` and `snapshot-before.txt` hold the re-measured values, byte-identical to
the pre-push values recorded in the Phase-4 dispatch facts. The single-ref push could not
have moved either one; the equality checks in `report.json` are the executable form of
that claim.

## Run discovery

`gh run list --workflow=release.yml --event=push` returns exactly one entry with
`headSha = b14d28eb…` and `event = push`: run **45724842211**
(https://github.com/coruairc/deepseek-build/actions/runs/45724842211), conclusion
`success`. Its record is the `tag-run` item.

## Files

| File | Meaning |
|------|---------|
| `expected-sha.txt` | the tag's target commit |
| `main-before.txt` | `git ls-remote origin refs/heads/main` |
| `snapshot-before.txt` | the pre-existing snapshot tag, tag object + peeled commit |
| `push.log` | the guard runs, the static gates, and every post-push ref re-measurement |
| `report.json` | the item record (`scripts/evidence.py finish --result ok`) |
