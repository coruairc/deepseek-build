---
phase: 03-pipeline-decoupling-version
verified: 2026-11-22T06:20:00Z
status: deferred
verification_type: goal-backward
score: "5/8 met; 1 code present; 2 CI proofs deferred"
requirements_verified: [REL-01, REL-07, REL-08, PKG-01]
requirements_code_present: [PKG-02]
requirements_deferred: [REL-02, REL-03, PKG-03]
---

# Phase 3 Verification — Pipeline Decoupling & Version

**Verdict: DEFERRED (no CI run, no push/tag; tests skipped by owner,
2026-11-22).**

## Goal-backward check

**Goal:** a slow macOS leg can no longer erase the image, and the version and
tag contract is settled before any tag is pushed.

| # | Success criterion | State |
|---|---|---|
| 1 | `build-linux`/`build-macos` separate; `container` needs `build-linux`; empty tag list fails loudly | **Met in code** — `release.yml`; static parse shows the five jobs and `needs` |
| 2 | Both macOS targets produce tarballs inside the timeout on a dispatch run | **DEFERRED** — no dispatch allowed |
| 3 | macOS binary `--version` works from the extracted tarball with signature intact after strip; no DWARF, no sidecar | **Code present, run deferred** — `release-dist` strip/debug + `codesign --verify` + `otool` check + extracted-tarball step |
| 4 | Manifests report `0.1.0` via `[workspace.package]`, four crates inherit, lock regenerated; both version paths print `0.1.0` | **Met statically** — `Cargo.toml`, four manifests, `Cargo.lock`; CI asserts manifest-vs-tag and the stamped `GROK_VERSION` |
| 5 | `release.yml` verified against the `v0.1.0` tag shape; no `deepseek-*` trigger | **Verified statically** — see `03-04-SUMMARY.md` |

## Requirement status

| Req | Status |
|---|---|
| REL-01 | met (code) |
| REL-02 | DEFERRED (needs a dispatch run) |
| REL-03 | code present; run DEFERRED |
| REL-07 | met (code): strip at codegen, debug=0, no sidecar |
| REL-08 | met (code): empty-tag guard |
| PKG-01 | met statically (0.1.0 + lock) |
| PKG-02 | assertion written in both legs; run DEFERRED |
| PKG-03 | met (static verification); runtime tag run DEFERRED |

## Deferred verification commands

```sh
# CI (needs a push; out of scope for this run)
gh workflow run release.yml          # dispatch: builds, no release
gh run watch
# Local static
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"
python3 -c "import tomllib; d=tomllib.load(open('Cargo.lock','rb')); print({p['name']:p['version'] for p in d['package'] if p['name'].startswith('xai-grok-pager')})"
```

## Risks carried forward

- The macOS timeout decision is unmade without a measured job log; the 90-minute
  timeout is unchanged.
- `cargo build --release-dist` was not run, so the "no DWARF" claim rests on the
  profile settings and the CI assertions, not a local artifact.
