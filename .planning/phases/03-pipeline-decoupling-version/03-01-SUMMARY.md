---
phase: 03-pipeline-decoupling-version
plan: "01"
status: complete
subsystem: release-pipeline
requirements: [REL-01, REL-08]
verification: "DEFERRED (no CI run/push; tests skipped by owner, 2026-11-22)"
---

# Plan 03-01: Pipeline decoupling and the tag-list guard

**Status:** workflow written; no dispatch or push performed.

## Changes (all in `.github/workflows/release.yml`)

- The single four-leg `build` matrix is split into:
  - `build-linux` — `x86_64-unknown-linux-gnu` (ubuntu-22.04) and
    `aarch64-unknown-linux-gnu` (ubuntu-22.04-arm), 90-minute timeout.
  - `build-macos` — `aarch64-apple-darwin` (macos-15) and
    `x86_64-apple-darwin` (macos-15-intel), its own 90-minute timeout, brew
    protobuf, rustup 1.94.0, per-target cache, own tarball upload.
- `publish.needs = [version, build-linux, build-macos]` so a missing macOS
  tarball still fails the publish step's "all 4 tarballs arrived" check.
- `container.needs = [version, build-linux]` — a slow macOS leg can no longer
  skip the image.
- New `Guard — metadata produced at least one tag` step after
  `docker/metadata-action`: an empty/whitespace tag list exits 1 with
  `::error::` instead of letting the manifest loop iterate zero times and go
  green having pushed nothing.

The same file also carries the macOS artifact checks (03-02) and the
manifest-vs-tag assertion (03-04); they are committed here because all three
plans edit this one file.

## Verification

- **DEFERRED (no dispatch/push; tests skipped by owner, 2026-11-22).**
- Static: the workflow parses; `jobs` = version, build-linux, build-macos,
  publish, container, with the `needs` above.
