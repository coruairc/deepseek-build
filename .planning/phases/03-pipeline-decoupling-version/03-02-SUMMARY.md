---
phase: 03-pipeline-decoupling-version
plan: "02"
status: complete
subsystem: release-profile
requirements: [REL-02, REL-03, REL-07]
verification: "DEFERRED (no CI run; job-log measurement not executed; tests skipped by owner, 2026-11-22)"
---

# Plan 03-02: macOS time budget and a DWARF-free distribution profile

**Status:** profile and workflow checks written; the job-log measurement is
deferred.

## Changes

- `Cargo.toml` `[profile.release-dist]`: `strip = "symbols"` and `debug = 0`
  (was `strip = false`, `debug = 1`), `split-debuginfo = "off"` unchanged.
  Stripping happens at codegen time, before the final link, so the macOS ad-hoc
  signature is applied to the already-stripped image; no `.debug`/`.dSYM`
  sidecar is produced.
- `release.yml` macOS leg: `codesign --verify --strict` and a `__DWARF`
  segment check on the built binary; then the tarball is extracted and the
  extracted binary runs `--version` and re-verifies its signature (REL-03).
- `release.yml` Linux leg: `readelf -S` must show no `.debug_` section and no
  `*.dwp`/`*.debug` sidecar may exist.

## Verification

- **DEFERRED (no CI run; tests skipped by owner, 2026-11-22).**
- The macOS time-budget decision (timeout vs strip vs LTO) is explicitly **not**
  made: it requires the measured job log, which this run cannot produce.
  The 90-minute job timeout is unchanged pending that measurement.
