---
phase: 03-pipeline-decoupling-version
plan: 03
subsystem: release-pipeline
tags: [cargo-profile, dwarf, strip, macos, codesign, ad-hoc-signing, github-actions, release-timeout, shell-fixture]

requires:
  - phase: 03-pipeline-decoupling-version
    provides: "03-01's build-linux/build-macos split, so the macOS budget and the new sign/verify steps are job-local; 03-02's workspace version 0.1.0, which the extracted-artifact version needles now assert"
provides:
  - "`[profile.release-dist]` = `debug = false` + `strip = \"debuginfo\"`: a real build has 0 `.debug` sections and 1 `.symtab`"
  - "`build-macos` ad-hoc signs (`codesign --force --sign -`) after the build and before packaging, then verifies the extracted tarball (signature, both `--version` needles, no `__DWARF`, no `*.dSYM`) before upload"
  - "`build-macos.timeout-minutes` = 240 with `build-linux` still 90; no `CARGO_PROFILE_*` override"
  - "`scripts/check-release-workflow.py` = 19/19 assertions (A16–A19 new) plus a 5-case Linux fixture that executes the verify step's real shell body against stub `codesign`/`otool`"
  - "README: macOS binaries are ad-hoc signed, not notarized, and carry no Developer ID"
affects: [04-distribution-proof, 06-documentation-truth-freeze]

actuals:
  tokens: 5894            # chars/4 over the realized diff (23,575 chars: 419 insertions, 11 deletions)
  tasks: 2
  commits: 2              # measured: git rev-list --count <ledger>..HEAD
  plan_head_before: 2f1285767791e7ab11ed2632bc3299a7a6a4115a
  plan_head_after: d448f7dee11568b37cf4c1d5a147236e8bd67dc6

tech-stack:
  added: []
  patterns:
    - "Strip only at link time (`-C strip=debuginfo`, via the Cargo profile key): an external strip after signing rewrites the Mach-O and invalidates the signature, and on arm64 the kernel then refuses to exec"
    - "Verify the shipped container (the tarball), not the pre-tar build output, and verify it before the upload so a bad artifact never reaches `publish`"
    - "Exercise a CI shell body on Linux against stubs and a fake artifact, and state in the script docstring exactly what the fixture cannot prove (no Mach-O signature here)"

key-files:
  created: []
  modified:
    - Cargo.toml
    - .github/workflows/release.yml
    - scripts/check-release-workflow.py
    - README.md

key-decisions:
  - "`strip = \"debuginfo\"`, not `\"symbols\"`: DWARF goes but the symbol table stays, so `xai-crash-handler` backtraces still name frames (RESEARCH R5). Proven: 0 `.debug` / 1 `.symtab`."
  - "The macOS verify step takes its working directory from a declared `ARTIFACT_VERIFY_DIR` env var instead of `mktemp -d` inside the body (review revision 1). This keeps the body `set -u`-clean and lets the Linux fixture hand it a directory under the same name; `RT` is declared too because the tarball name embeds `matrix.target`."
  - "Ad-hoc signing runs on both arches, not just Intel: rustc signs arm64 output by itself, but an unsigned Intel binary would fail `codesign --verify` for a non-defect, making the check meaningless on that leg."
  - "The pre-tar smoke step stays where it was — it fails faster on a bad version stamp; the extracted-tarball step is the artifact proof."
  - "`build-macos` gets 240 minutes and `build-linux` keeps 90. No `CARGO_PROFILE_*` override was added; the exact levers for Phase 4 (`CARGO_PROFILE_RELEASE_DIST_LTO=off`, `CARGO_PROFILE_RELEASE_DIST_CODEGEN_UNITS=16`, macOS job only) are recorded below and in the workflow comment."
  - "The fixture is 5 cases, not 3: the plan's good/wrong-version/`__DWARF` trio plus the addendum's `0.1.0-proof1` success case plus a stray-`.dSYM` failure case (see Deviations)."
  - "Only REL-07 is marked complete. REL-02 (both macOS legs inside the budget) and the macOS execution half of REL-03 have no evidence until the Phase 4 tag run, so marking them complete would contradict this plan's own must-have 9."

patterns-established:
  - "A profile claim is proven on a real `release-dist` artifact with `readelf`, never by reading the profile block; the `ptyctl-cli` bin keeps that probe cheap (2m06s, no pager crates)."
  - "A check that runs only on another OS is fixture-tested on this host: the step's body is extracted from the YAML and executed verbatim against stubs, so a broken path or a changed needle fails locally instead of at the first tag."
  - "Every artifact assertion names the container it validates: the extracted file after `tar`, because `tar` preserves bytes and the signature must survive packaging."

requirements-completed: [REL-07]
requirements-open:
  - "REL-02 — no CI run; both macOS legs inside 240 minutes is unproven (Phase 4 tag run)"
  - "REL-03 — extracted-tarball step and signature ordering landed and are fixture-tested, but no macOS binary was signed or verified on this host"

coverage:
  - id: D1
    description: "`[profile.release-dist]` ships no DWARF: `debug = false`, `strip = \"debuginfo\"`, `split-debuginfo = \"off\"` kept, with a comment that describes the tree that exists."
    requirement: REL-07
    verification:
      - kind: integration
        ref: "cargo build --locked --profile release-dist -p ptyctl-cli -j 1 && readelf -S target/release-dist/ptyctl | grep -c '\\.debug' (0) ; grep -c '\\.symtab' (1)"
        status: pass
    human_judgment: false
  - id: D2
    description: "`build-macos` signs the linked binary ad-hoc after the build and before `Package tarball`."
    requirement: REL-03
    verification:
      - kind: automated_ui
        ref: "python3 scripts/check-release-workflow.py # A16 (run body contains `codesign --force --sign -`, ordered build < sign < package)"
        status: pass
    human_judgment: true
    rationale: "A16 proves the command and its position, not that codesign accepted the binary; only a macOS runner can produce and verify a Mach-O signature."
  - id: D3
    description: "The packaged tarball is verified before upload: `codesign --verify --strict`, a `Signature=` line, both version needles on the extracted binary, no `__DWARF`, no `*.dSYM`."
    requirement: REL-03
    verification:
      - kind: automated_ui
        ref: "python3 scripts/check-release-workflow.py # A17 + 'artifact-step fixture: 5/5 PASS'"
        status: pass
    human_judgment: true
    rationale: "The fixture proves the step's paths, needles and gates against stubs; it proves nothing about a real Mach-O signature or a real `otool` output."
  - id: D4
    description: "`build-macos` carries a 240-minute budget; `build-linux` stays at 90; no `CARGO_PROFILE_*` override landed."
    requirement: REL-02
    verification:
      - kind: unit
        ref: "python3 scripts/check-release-workflow.py # A18, A19 + PyYAML timeout assertion"
        status: pass
    human_judgment: true
    rationale: "The budget is a configured number; whether it is enough for a cold thin-LTO build on a 3 vCPU runner is only observable in CI (Phase 4)."
  - id: D5
    description: "README states the signature truth: ad-hoc signed, not notarized, no Developer ID."
    verification:
      - kind: other
        ref: "grep -n 'ad-hoc signed' README.md"
        status: pass
    human_judgment: true
    rationale: "The sentence was matched to the workflow step by reading both; no automated check ties prose to behaviour, and the real Gatekeeper experience is a macOS observation."
  - id: D6
    description: "REL-02 (both macOS legs inside the budget) and the macOS half of REL-03 (real signature, real timeout) are open, with the Phase-4 acceptance commands recorded."
    requirement: REL-02
    verification: []
    human_judgment: true
    rationale: "Requires a real macOS runner; this plan's host has no codesign/otool and no CI dispatch was made."
---

# Phase 3 Plan 03: No DWARF in `release-dist`, and a macOS artifact signed then verified from the tarball Summary

**`release-dist` now emits zero `.debug` sections (one `.symtab` retained), and `build-macos` ad-hoc signs the binary after linking and verifies the extracted tarball — signature, both version needles, no `__DWARF`, no `.dSYM` — inside a 240-minute budget, with the step's body fixture-tested 5/5 on Linux.**

## Performance

- **Duration:** ~5 min (2026-11-25T23:40:50Z → 2026-11-25T23:45Z); the `release-dist` probe build was **2m 06s** wall-clock (126 s), rebuild after cache 0.23 s
- **Started:** 2026-11-25T23:40:50Z
- **Completed:** 2026-11-25T23:45:18Z
- **Tasks:** 2/2
- **Files modified:** 4

## Accomplishments

- `[profile.release-dist]` is honest: `debug = false` + `strip = "debuginfo"` replace `debug = 1` / `strip = false`, so DWARF is not emitted and the strip happens **during linking**; `lto = "thin"`, `codegen-units = 1` and `split-debuginfo = "off"` are untouched, so no `.dSYM` appears and Phase 4 still has its speed levers. Proven on a real build: `readelf -S target/release-dist/ptyctl` → **0** `.debug` sections, **1** `.symtab`.
- `build-macos` signs explicitly: `codesign --force --sign - target/release-dist/deepseek-build`, then `codesign --verify --strict --verbose=2`, then `codesign -dv` (log records `Signature=adhoc`). Position: step 6 of 10, immediately after the build and before the smoke and packaging steps.
- New `Verify the packaged macOS artifact` step (step 9 of 10, after `Package tarball`, before the upload) extracts `deepseek-build-<version>-<target>.tar.gz` into `${ARTIFACT_VERIFY_DIR}`, verifies the signature on the **extracted** file, requires a `Signature=(adhoc|Authority=)` line, runs the extracted `--version` through both existing needles, and fails on a `__DWARF` segment or any `*.dSYM` under the tree.
- Budget: `build-macos.timeout-minutes` **240**, `build-linux` **90**; the comment above the jobs names the rc4 evidence (`v0.0.1-rc4`, run `40111152104`, the 90-minute timeout on `aarch64-apple-darwin`), the 3 vCPU/7 GB arm64 runner, and the next lever without applying it.
- `scripts/check-release-workflow.py` grew from 15 to **19/19 assertions** and a **5/5** fixture. New assertions:
  - **A16** — `build-macos` has exactly one step whose run body contains `codesign --force --sign -`, ordered after the `cargo build` step and before `Package tarball`, named `Ad-hoc sign the macOS binary`.
  - **A17** — exactly one step named `Verify the packaged macOS artifact`, ordered after `Package tarball` and before the upload; its body contains `tar -xzf`, `codesign --verify --strict`, `--version`, `__DWARF`, `*.dSYM`; its `env:` declares `RELEASE_VERSION`, `RT`, `ARTIFACT_VERIFY_DIR`.
  - **A18** — `build-macos.timeout-minutes == 240` and `build-linux.timeout-minutes == 90`.
  - **A19** — no `run:` body, step `env:` or job-level `env:` in either build job contains a `CARGO_PROFILE_*` variable.
- README's signature claim now matches the pipeline (see the before/after below).

## Task Commits

Each task was committed atomically:

1. **Task 1: `release-dist` emits no DWARF, proven on a real built binary** — `7a7f56b8` (feat)
2. **Task 2: Sign ad-hoc, verify the extracted tarball, raise the macOS budget, fixture the step on Linux** — `d448f7de` (feat)

**Plan metadata:** this SUMMARY commit.

## Files Created/Modified

- `Cargo.toml` — `[profile.release-dist]`: `strip = "debuginfo"`, `debug = false`, rewritten comment (10 insertions, 4 deletions; `[profile.release]` and every other profile untouched).
- `.github/workflows/release.yml` — budget comment, `build-macos.timeout-minutes: 240`, `Ad-hoc sign the macOS binary`, `Verify the packaged macOS artifact`. `publish`, `container`, `version` and `build-linux` are byte-identical jobs (verified by YAML comparison against `2f128576`); no `uses:` pin changed; no `if:` added; no `CARGO_PROFILE_*` env.
- `scripts/check-release-workflow.py` — A16–A19, the fixture helpers/stubs, `artifact-step fixture` reporting; docstring updated to say what the fixture does and does not prove.
- `README.md` — the macOS signature sentence (one-line edit; broader docs reconciliation stays in Phase 6).

### The profile block as written

```toml
[profile.release-dist]
inherits = "release"
lto = "thin"
codegen-units = 1
strip = "debuginfo"
debug = false
split-debuginfo = "off"
```

Preceded by a comment stating: no debug info ships; `strip = "debuginfo"` keeps the symbol table so crash-handler backtraces still name frames; there is no `.debug` sidecar and no `.dSYM` because no sidecar extraction exists (REL-V2-02 deferred); the strip is rustc's link-time `-C strip=debuginfo` and no external strip may ever be added after signing.

### Workflow step order in `build-macos` (after)

1. Checkout tag · 2. Install system dependencies · 3. Install Rust toolchain · 4. Restore cargo cache · 5. Build release-dist binary · **6. Ad-hoc sign the macOS binary** · 7. Smoke-check the built binary · 8. Package tarball · **9. Verify the packaged macOS artifact** · 10. Upload tarball as a job artifact

### Fixture cases (all run by default, 5/5 PASS)

| Case | Artifact | Stub | Expected | Result |
|------|----------|------|----------|--------|
| good `0.1.0` | prints `deepseek-build 0.1.0 (abcdef123456)` | `Signature=adhoc`, clean `otool` | exit 0 | PASS |
| good `0.1.0-proof1` | prints `deepseek-build 0.1.0-proof1 (abcdef123456)` — the exact Phase 4 tag shape, hyphen present | same | exit 0 | PASS |
| wrong `--version` | tarball `0.1.0`, binary prints `9.9.9` | same | non-zero | PASS (failed as required) |
| stubbed `__DWARF` | correct version | `otool` prints a `__DWARF` segment | non-zero | PASS (failed as required) |
| stray `.dSYM` | correct version | clean `otool`, `artifact.dSYM/` in cwd | non-zero | PASS (failed as required) |

Non-vacuity was checked by mutation: with the `__DWARF` gate replaced by `if false;` (needle kept in a comment so A17 still passes), the fixture reports `4/5 PASS` and the checker exits 1.

### README sentence

- **Before:** `The macOS binaries are **not codesigned and not notarized**, so first launch may need a right-click **Open**, or `xattr -d com.apple.quarantine <path-to-binary>`.`
- **After:** `The macOS binaries are **ad-hoc signed** (`codesign --sign -`) so Apple silicon will execute them, **not notarized**, and carry **no Developer ID**, so first launch may need a right-click **Open**, or `xattr -d com.apple.quarantine <path-to-binary>`.`

## Decisions Made

- `strip = "debuginfo"` over `"symbols"`: the tree ships `xai-crash-handler`, and `"symbols"` would degrade its backtraces (RESEARCH R5). The probe proves the choice costs nothing in section terms (0 `.debug`, 1 `.symtab`).
- The verify step's temp directory is a declared `env:` value (`ARTIFACT_VERIFY_DIR: ${{ runner.temp }}/verify-packaged-artifact`) rather than a `mktemp -d` inside the body. That is the review revision's requirement, and it makes the Linux fixture drive the body through the same names.
- Signing happens before the pre-tar smoke, so the smoke executes the signed binary on macOS too. The smoke's job (fail fast on a bad version stamp) is unchanged.
- The fixture is 5 cases rather than the plan's 3 or the addendum's 4 — see Deviations.
- REL-07 is the only requirement marked complete; REL-02 and the CI half of REL-03 stay open with the Phase-4 commands below.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical test coverage] Added a fifth fixture case: a stray `.dSYM` fails the step**

- **Found during:** Task 2 (fixture implementation)
- **Issue:** the verify step fails on `find . -name '*.dSYM' -print -quit`, and that is half of the T-03-09 mitigation (no DWARF, no sidecar). With only the plan's three cases plus the addendum's `0.1.0-proof1` success case, no fixture ever exercised the `.dSYM` gate, so a broken `find` (wrong flag, wrong directory) would have passed unnoticed until a real tag.
- **Fix:** a fifth case creates `artifact.dSYM/` next to the tarball and expects a non-zero exit. The fixture now prints `artifact-step fixture: 5/5 PASS`.
- **Files modified:** `scripts/check-release-workflow.py`
- **Verification:** the case passes (step exits non-zero); the `__DWARF` mutation test still drops the tally to 4/5 and exits 1, so the fixture is not vacuous.
- **Committed in:** `d448f7de`

### Contract extensions (not defects)

- A16 also asserts the sign step's name, and A17 also asserts the verify step's three `env:` keys — both are required by the review revision or the plan's step naming, and both fail loudly if a later edit renames or drops them.
- The verify step declares `RT` in `env:` in addition to the two variables the revision names, because the tarball filename embeds `matrix.target`; leaving it undeclared would break `set -u`.

**Total deviations:** 1 auto-fixed (1 missing critical test coverage), plus two documented contract extensions.
**Impact:** no scope creep — the extra case only exercises a gate the plan already mandates; the extensions make the mandated names machine-checked.

## Issues Encountered

- `python3 -m py_compile` (a syntax check) left `scripts/__pycache__/`; it was removed before committing. `.gitignore` only covers `/scripts/evals/__pycache__/`; adding a general rule was out of scope for a bounded change.
- **Phase 4 baseline note:** `04-01-PLAN.md` records the README baseline `not codesigned and not notarized = 1`. This plan's mandated README edit makes it 0, while `04-02-PLAN.md` already requires the new "ad-hoc signed but not notarized" wording. When 04-01 executes, its count baseline must be re-measured rather than asserted from the plan text.

## Verification Results

| Check | Command | Result |
|-------|---------|--------|
| Probe build | `scripts/disk-guard.sh 60 && flock -w 900 /tmp/team-build.lock cargo build --locked --profile release-dist -p ptyctl-cli -j 1` | exit 0, **2m 06s** (948 GB free at start) |
| No DWARF | `readelf -S target/release-dist/ptyctl \| grep -c '\.debug'` | **0** |
| Symbol table kept | `readelf -S target/release-dist/ptyctl \| grep -c '\.symtab'` | **1** |
| Profile assertions | block grep: no `strip = false` / `debug = 1`; has `debug = false` / `strip = "debuginfo"` | PASS |
| Checker | `python3 scripts/check-release-workflow.py` | **19/19 assertions PASS**, fixture **5/5 PASS**, exit 0 |
| Workflow lint | `actionlint .github/workflows/release.yml` | exit 0, no output |
| Timeouts | PyYAML: `build-macos == 240`, `build-linux == 90` | PASS |
| Job isolation | YAML compare vs `2f128576`: `version`, `build-linux`, `publish`, `container` identical | PASS |
| No strip/dsymutil commands | A12 | PASS |
| No `.rs` change | `git diff --name-only 2f128576..HEAD \| grep '\.rs$'` | none (so `cargo fmt --all --check` is unaffected) |
| No side effects | `git tag --points-at HEAD` empty; branch `main`; no `gh`/dispatch/tag created | PASS |

## NOT RUN (open by design — REL-02 and the macOS half of REL-03)

No macOS build, signature check or timeout was claimed. The first evidence is Phase 4's throwaway pre-release tag run. Its acceptance commands, recorded here verbatim:

```sh
# The run exists and is the tag's:
gh run list --workflow=release.yml --limit 3

# All four artifacts arrived (expect release-<target> x4):
gh api /repos/coruairc/deepseek-build/actions/runs/<run-id>/artifacts -q '.artifacts[].name'

# The macOS legs finished inside 240 minutes (read the job durations in the run):
gh run view <run-id> --json jobs -q '.jobs[] | select(.name|startswith("build-macos")) | .name, .conclusion, .startedAt, .completedAt'

# On a macOS host, against the extracted release tarball:
codesign --verify --strict --verbose=2 ./deepseek-build
codesign -dv ./deepseek-build 2>&1 | grep 'Signature='
otool -l ./deepseek-build | grep -c __DWARF        # expect 0
find target -name '*.dSYM'                          # expect empty
./deepseek-build --version                          # expect 'deepseek-build 0.1.0 (<hex>)'
```

If 240 minutes is still not enough, the next lever (apply to `build-macos` only, never to `build-linux`) is:

```yaml
env:
  CARGO_PROFILE_RELEASE_DIST_LTO: "off"
  CARGO_PROFILE_RELEASE_DIST_CODEGEN_UNITS: "16"
```

## Next Phase Readiness

- 03-04 can assert the checker at **19/19** (A16 sign ordering, A17 extracted-artifact verify, A18 timeouts, A19 no profile override) and the fixture at 5/5.
- The `v0.1.0` tag shape the fixture's second success case now covers is `0.1.0-proof1` — the exact prerelease shape Phase 4 pushes.
- REL-02 stays a Phase 4 finding: if a macOS leg still times out at 240, that run re-opens Phase 3, and the `CARGO_PROFILE_*` levers above are the pre-decided next step.
- No branch, tag, `gh` invocation or workflow dispatch was created; nothing was pushed.

---
*Phase: 03-pipeline-decoupling-version*
*Completed: 2026-11-25*

## Self-Check: PASSED

- Key files exist on disk: `Cargo.toml`, `.github/workflows/release.yml`, `scripts/check-release-workflow.py`, `README.md`, this SUMMARY.
- Both task commits are ancestors of HEAD: `7a7f56b8`, `d448f7de` (ledger-measured count: 2).
- Plan-level verification re-run at close: checker `19/19` + fixture `5/5`, `actionlint` exit 0, `readelf` `0` `.debug` / `1` `.symtab`.
