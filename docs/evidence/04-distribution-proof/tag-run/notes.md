# tag-run — the green release run, the six assets, and no `:latest`

Requirement: REL-04 (classification + assets half). This item also supplies the live
closure of Phase 3's **REL-02** (both macOS legs finish inside their budget) and the macOS
half of **REL-03** (the packaged artifact is ad-hoc signed and verifies after `tar`).

## The run

| Field | Value |
|-------|-------|
| Run | **45724842211** — https://github.com/coruairc/deepseek-build/actions/runs/45724842211 |
| Event / ref | `push` / `v0.1.0-proof1` |
| Head SHA | `b14d28eb2bc71a046b789e0a705848d651685d35` |
| Window | 2026-11-25T23:54:21Z → 2026-11-26T00:56:27Z |
| Conclusion | `success` — all seven jobs |

`run.json` is machine-read from `gh run view --json` plus the container job's log; no field
in it is transcribed by hand.

### Jobs and durations (minutes, from `startedAt`/`completedAt`)

| Job | Conclusion | Minutes |
|-----|------------|---------|
| Derive release version | success | 0.1 |
| build-linux (x86_64-unknown-linux-gnu) | success | 28.0 |
| build-linux (aarch64-unknown-linux-gnu) | success | 23.9 |
| build-macos (aarch64-apple-darwin) | success | **47.8** |
| build-macos (x86_64-apple-darwin) | success | **61.5** |
| Build and push container image | success | 3.2 |
| Publish GitHub release | success | 0.4 |

**REL-02 is closed by this run.** The two macOS legs took 47.8 and 61.5 minutes against
the 240-minute `build-macos` budget (`timeout-minutes: 240` in `release.yml`), leaving
192.2 and 178.5 minutes of margin. Both legs are also under the *old* 90-minute ceiling on
this run — record that accurately rather than claiming the ceiling would have failed here:
what cancelled `v0.0.1-rc4` (run `40111152104`) was its arm64 leg hitting 90 minutes, and
that is the observed variance the 240-minute budget was raised for (Phase 3). REL-02's
requirement is that both legs produce tarballs inside 240, and they did.

### macOS signature path — the live closure of REL-03's macOS half

Both legs ran `Ad-hoc sign the macOS binary` and `Verify the packaged macOS artifact` to
`success`, and the logs carry the action, not just the exit code.

`build-macos (aarch64-apple-darwin)`:

```
target/release-dist/deepseek-build: replacing existing signature
target/release-dist/deepseek-build: valid on disk
target/release-dist/deepseek-build: satisfies its Designated Requirement
CodeDirectory v=20400 size=932208 flags=0x2(adhoc) hashes=29125+2 location=embedded
Signature=adhoc
binary reports: deepseek-build 0.1.0-proof1 (b14d28eb2bc7)
...
/Users/runner/work/_temp/verify-packaged-artifact/deepseek-build: valid on disk
/Users/runner/work/_temp/verify-packaged-artifact/deepseek-build: satisfies its Designated Requirement
extracted binary reports: deepseek-build 0.1.0-proof1 (b14d28eb2bc7)
```

`build-macos (x86_64-apple-darwin)`:

```
CodeDirectory v=20400 size=1018576 flags=0x2(adhoc) hashes=31824+2 location=embedded
Signature=adhoc
binary reports: deepseek-build 0.1.0-proof1 (b14d28eb2bc7)
...
/Users/runner/work/_temp/verify-packaged-artifact/deepseek-build: valid on disk
/Users/runner/work/_temp/verify-packaged-artifact/deepseek-build: satisfies its Designated Requirement
extracted binary reports: deepseek-build 0.1.0-proof1 (b14d28eb2bc7)
```

The `verify-packaged-artifact` path is the **extracted tarball**, so the signature survives
`tar`; the arm64 leg additionally executes the extracted binary (the kernel refuses an
arm64 binary whose signature no longer matches), and both legs print the tag's version
stamped with the tag commit `b14d28eb2bc7`.

## Container: guard, pushed names, no `:latest`

Verbatim from the container job log:

```
guard-image-tags: resolved tag names (2):
  0.1.0-proof1
  v0.1.0-proof1
guard-image-tags: OK 2 names (prerelease)
...
amd64=sha256:7f33b41bad8811fd7550808fa8a21fd5247480b7defacb793ea385bc8c566f7b arm64=sha256:85f9de25a9863bc5c33340ccb6b0f39a4f44adc14125cf5805deda926c264410
...
Pre-release — :latest unchanged
...
Digest:    sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be
```

Names actually pushed (parsed from the log's
`pushing manifest for ghcr.io/coruairc/deepseek-build:<name>@sha256:…` lines, deduplicated):
**`0.1.0-proof1`** and **`v0.1.0-proof1`** — no `latest`. `run.json` records the same two
names in `guard_names`; the harness fails if the list is empty or contains `latest`.

The `docker/metadata-action` produced **two** names, not the four the plan predicted from
the four `tags:` patterns — the pinned action collapses the semver aliases for a
pre-release version. Both shapes are accepted by the guard (see `../tag-push/push.log`).

`image_digest` = `sha256:8f5c484ed067143aab93aa90b1441101fd643a37924f1868eeaaa103d1f199be`
— the **multi-arch OCI index** digest printed by the final `docker buildx imagetools
inspect`. Plan 04-04 consumes it. Note the per-arch `build-push-action` outputs
(`7f33b41b…` amd64, `85f9de25…` arm64) are the buildkit per-platform **indexes** (each
carries its attestation manifest); the platform manifests *inside* the assembled index are
different digests — `docker manifest inspect` reports
`sha256:1d216b60…` for linux/amd64 and `sha256:25dc0d25…` for linux/arm64. Do not confuse
the two pairs.

## The release

`https://github.com/coruairc/deepseek-build/releases/tag/v0.1.0-proof1` — `prerelease:
true`, `draft: false`, published 2026-11-26T00:56:19Z. Exactly six assets:

| Asset | Size (bytes) | GitHub `digest` |
|-------|--------------|-----------------|
| `deepseek-build-0.1.0-proof1-aarch64-apple-darwin.tar.gz` | 49,591,197 | `sha256:2b61dfab870b397b49ed810f53e073714582e01dbf5c6f9d166656367c51324b` |
| `deepseek-build-0.1.0-proof1-aarch64-unknown-linux-gnu.tar.gz` | 53,622,321 | `sha256:0cfcf04796fbdaa098c5b23c55b22d4385d3c2af564fa357482fad68837c7e9a` |
| `deepseek-build-0.1.0-proof1-x86_64-apple-darwin.tar.gz` | 53,850,000 | `sha256:ffe0f3c5f76be9ee6fa2dff1d53c34de6dce66d3553a094ee62524f58f57b3a7` |
| `deepseek-build-0.1.0-proof1-x86_64-unknown-linux-gnu.tar.gz` | 55,967,373 | `sha256:298c9f275d695fa43872bd44a0fe9da25ee66bd86a6051ac233689063ccbdaf3` |
| `install.sh` | 14,552 | `sha256:31e114d4a29dbbeb0d3e2e442a72a6ecc14ad3f4b99956fbff23f42c0cabd5e5` |
| `SHA256SUMS` | 496 | `sha256:0183ee9bccae0101945831b4b621aefce7db65a84563c8f0c3c5015bc3b2995a` |

Every tarball: local `sha256sum` == its line in the release's own `SHA256SUMS` == GitHub's
asset `digest` (all four `OK`, all six `MATCH` in `checksums.log`). Each tarball's member
list is exactly `deepseek-build`, `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES`, `README.txt`
at the top level, with no `.dSYM`/`.debug` member. The extracted linux-x86_64 binary runs on
this host and prints `deepseek-build 0.1.0-proof1 (b14d28eb2bc7)`; the two darwin binaries
are Mach-O `arm64` and `x86_64` (checked with `file`, not executed here).

No `v0.1.0`: `repos/coruairc/deepseek-build/releases/tags/v0.1.0` → **404**;
`releases/latest` → **404** (every release in this repository is a pre-release, so GitHub
has no "latest"); the release list is `v0.1.0-proof1`, `v0.0.1-rc5`, `v0.0.1-rc4` and
contains no `v0.1.0`.

## What is NOT here

- **REL-04's pull is not performed.** No container runtime exists on this host: `docker
  version` fails with `Cannot connect to the Docker daemon`. The registry-side proof (image
  digest, manifest list, tag set) belongs to plan **04-04**; this item records only the
  digest and the pushed names so 04-04 can assert against them.
- **The tarballs are not committed.** They were downloaded to
  `/tmp/opencode/04-assets.Yv4ZRD` (outside the repository) and deleted after
  `checksums.log` was written. Only text evidence is in this item; the `-size +1M` gate in
  `report.json` proves it.
- **No cleanup of the pre-release.** CONTEXT.md's "keep the throwaway pre-release"
  decision (it is the only macOS artifact) is recorded as a deviation against the
  roadmap's "clean up" line in plan 04-05's deviation file. Nothing was deleted or
  unpublished here.

## Files

| File | Meaning |
|------|---------|
| `run.json` | run/job/step conclusions, durations, container log parses, image digest |
| `release.json` | release classification, asset set, digests, tarball members, version outputs |
| `SHA256SUMS` | the release's own four-line file, unmodified |
| `checksums.log` | `sha256sum -c`, local-vs-GitHub digest table, extracted binary `--version`, `file` on the darwin binaries |
| `report.json` | the item record (`scripts/evidence.py finish --result ok`) |

## Re-run

```sh
python3 scripts/distribution-proof.py run \
  --sha b14d28eb2bc71a046b789e0a705848d651685d35 --out /tmp/run.json
tmp=$(mktemp -d)
python3 scripts/distribution-proof.py release --tag v0.1.0-proof1 --dir "$tmp" --out /tmp/release.json
rm -rf "$tmp"
```
