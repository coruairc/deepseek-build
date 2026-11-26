---
phase: 04-distribution-proof
plan: 03
subsystem: distribution
tags: [homebrew, formula, tap, macos, sha256, brew-audit, evidence, pkg-04]

requires:
  - phase: 04-distribution-proof
    plan: 01
    provides: the published `v0.1.0-proof1` pre-release, its `SHA256SUMS` and `release.json` (asset names, digests, sizes 49,591,197 / 53,850,000)
  - phase: 03-pipeline-decoupling-version
    provides: the ad-hoc-signed macOS artifacts the formula installs
provides:
  - packaging/homebrew/deepseek-build.rb — binary formula, on_macos/on_arm + on_intel, the proof release's two darwin URLs and sha256 values, dsb symlink, licence docs, --version test, livecheck skip
  - scripts/check-homebrew-formula.py — 15 offline + 4 online text assertions, 3-case self-test, no Ruby evaluation
  - packaging/homebrew/README.md — the author's one-time tap creation, the brew tap/install/test commands, the notarization position, the v0.1.0 repoint recipe
  - docs/evidence/04-distribution-proof/homebrew-formula — 15-step report (ok), per-step logs, resolve-branch.rb, notes.md
  - "the measured Homebrew facts: brew style 0 offenses, brew audit --strict [--online] exit 0 under HOMEBREW_SIMULATE_MACOS_ON_LINUX=1, brew fetch of 53,850,000 bytes checksum-verified, wrong-sha256 control rejected"
affects: [04-04, 04-05, 06-documentation-truth-freeze]

actuals:
  tokens: 11251          # chars/4 over 45,006 bytes in the 22 files this plan added
  tasks: 2
  commits: 2             # measured 078642a9..1b00b583; the metadata commit carrying this file is the 3rd

plan_head_before: 078642a952e09ad38925a1f96d217ce3c7d37481
plan_head_after: 1b00b58314ab1d6c88dad0a248daed080c54b9c8

tech-stack:
  added: []              # Python stdlib + Homebrew 7.0.16 already on the host; no manifest changed, nothing installed
  patterns:
    - "A package-manager formula is proven from three sides: text assertions, the release's own SHA256SUMS, and the package manager resolving/fetching the artifact"
    - "A macOS-only artifact evaluated on Linux uses the tool's own simulation switch (HOMEBREW_SIMULATE_MACOS_ON_LINUX=1), never a hand-waved 'cannot check'"
    - "Every checksum gate is paired with a wrong-value control proving the gate is engaged"
    - "An action this host cannot perform is recorded with its reason and the exact foreign-host command, never simulated"

key-files:
  created:
    - packaging/homebrew/deepseek-build.rb
    - packaging/homebrew/README.md
    - scripts/check-homebrew-formula.py
    - docs/evidence/04-distribution-proof/homebrew-formula/report.json
    - docs/evidence/04-distribution-proof/homebrew-formula/checker.log
    - docs/evidence/04-distribution-proof/homebrew-formula/resolve-branch.rb
    - docs/evidence/04-distribution-proof/homebrew-formula/notes.md
  modified: []

key-decisions:
  - "The formula is pinned to `v0.1.0-proof1` on purpose: that is the only published tag carrying macOS artifacts, and `v0.1.0` returns 404. The repoint is a documented four-line edit."
  - "`livecheck do skip` is explicit rather than omitted: Homebrew's default GitHub livecheck would query api.github.com, and the constraint is no new host. `brew audit --strict --online` still passes with the skip in place."
  - "The formula has no top-level `url`; both URLs live in `on_macos`/`on_arm`/`on_intel` as the plan requires. On Linux that means the formula cannot load (`formula requires at least a URL`); this is recorded as a Linux-host property, and Homebrew's own `HOMEBREW_SIMULATE_MACOS_ON_LINUX=1` is used to evaluate the macOS branches."
  - "`brew fetch` is the strongest available Homebrew-level proof: Homebrew downloaded the x86_64 tarball and its checksum gate accepted `ffe0f3c5…`; a wrong-hash copy is rejected with `reports different checksum`, so the gate is demonstrably active."
  - "`brew install` is NOT RUN, not faked: the tarballs are macOS Mach-O binaries and this host is Linux. The step records the reason and the exact macOS commands."
  - "`parse_asset` pairs a URL with the *next* `sha256` line, which is what a reader sees; the release SHA256SUMS and release.json `sums` are kept as two independent witnesses."

patterns-established:
  - "A formula's hashes come from the release's SHA256SUMS, cross-checked against the release metadata's own digests; a local build's hash is never an input."
  - "Proof of a package-manager artifact stops exactly where the host stops (no install of a foreign-arch binary) and says so in the report."

requirements-completed: [PKG-04]

coverage:
  - id: D1
    description: "In-tree binary formula with the proof release's two real darwin URLs and sha256 values, the dsb symlink, licence docs and a --version test"
    requirement: PKG-04
    verification:
      - kind: automated
        ref: "python3 scripts/check-homebrew-formula.py … --online — homebrew-formula: 19/19 PASS"
        status: pass
      - kind: automated
        ref: "docs/evidence/04-distribution-proof/homebrew-formula/report.json (15 steps, result ok)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Homebrew itself accepts the formula: style clean, audit --strict [--online] exit 0, both arch branches resolve, and the x86_64 tarball fetches with its checksum verified"
    requirement: PKG-04
    verification:
      - kind: integration
        ref: "brew style / brew audit --strict --online / brew ruby resolve-branch.rb {arm,intel} / brew fetch — see homebrew-formula/notes.md step table"
        status: pass
    human_judgment: false
  - id: D3
    description: "Activation documented as an author step: the tap repo does not exist, the creation commands, the consumer brew tap/install/test, the notarization position and the v0.1.0 repoint recipe"
    requirement: PKG-04
    verification:
      - kind: other
        ref: "packaging/homebrew/README.md — grep 'brew tap coruairc/deepseek-build' = 2, 'brew install deepseek-build' = 1"
        status: pass
    human_judgment: true
    rationale: "Whether the documented steps are sufficient for the author to create and maintain a public tap is a judgement call; the tap repo is explicitly the author's action and is not created here."
  - id: D4
    description: "A real `brew install coruairc/deepseek-build/deepseek-build` on macOS arm64 and x86_64"
    requirement: PKG-04
    verification: []
    human_judgment: true
    rationale: "Not runnable on this Linux host: the tarballs are macOS binaries and the tap does not exist. Recorded as NOT RUN in the evidence with the exact macOS commands."

duration: 17min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 03: A Homebrew formula pinned to the proof release, checked by Homebrew itself, activated by one documented author step

**`packaging/homebrew/deepseek-build.rb` installs the `v0.1.0-proof1` darwin tarballs on arm64 (`2b61dfab…`) and x86_64 (`ffe0f3c5…`); Homebrew 7.0.16 reports `1 file inspected, no offenses detected` and `brew audit --strict --online` exits 0, `brew fetch` downloads and checksum-verifies 53,850,000 bytes, and a wrong-hash control is rejected — with `brew install` recorded as NOT RUN because the binaries are macOS Mach-O on a Linux host.**

## Performance

- **Duration:** ~17 min
- **Started:** 2026-11-26T06:23Z
- **Completed:** 2026-11-26T06:41Z
- **Tasks:** 2/2
- **Files modified:** 22 added files (2 packaging files, 1 script, 19 evidence files); no product source touched

## Accomplishments

- **The formula carries the release's own hashes, not a build's.** Both `url`s are the exact
  `https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/…` asset paths inside
  `on_macos do on_arm … on_intel … end`; both `sha256`s are lines of
  `tag-run/SHA256SUMS` and equal GitHub's asset digest in `tag-run/release.json`. The two values differ.
- **The text is checked from three directions.** `scripts/check-homebrew-formula.py` runs 15
  offline assertions (class, version derived from the URLs, url count/shape, block pairing,
  hash count/agreement, install/test stanzas, no source build, single host, no placeholder,
  livecheck) → `homebrew-formula: 15/15 PASS`; with `--online` it adds both URLs answering 200
  with the release's `Content-Length` → `19/19 PASS`; its three mutations (wrong sha256, deleted
  `on_intel`, non-github.com URL) are each rejected by a named assertion → `self-test: 3/3 PASS`.
- **Homebrew itself accepts the formula.** On a throwaway local tap
  (`local/deepseek-test`): `brew style` → *1 file inspected, no offenses detected*;
  `brew audit --strict --formula` and `brew audit --strict --online --formula` → exit 0, no
  offenses (with `HOMEBREW_SIMULATE_MACOS_ON_LINUX=1`, Homebrew's own switch). A control formula
  with a period-terminated `desc` is caught (`Description shouldn't end with a full stop`), so the
  audit is not a no-op here.
- **Both arch branches resolve, and the real bytes fetch and verify.** `brew ruby` +
  `Homebrew::SimulateSystem` resolves `on_arm` → the aarch64 URL and `2b61dfab…`, `on_intel` → the
  x86_64 URL and `ffe0f3c5…`. `brew fetch` downloaded the x86_64 tarball (53,850,000 bytes, sha256
  `ffe0f3c5…`), and a copy with one wrong `sha256` is refused: `Formula wrongsha (0.1.0-proof1)
  reports different checksum`.
- **Activation is one documented author step and does not pretend to exist.** The tap repo
  `coruairc/homebrew-deepseek-build` does not exist; `packaging/homebrew/README.md` gives the
  creation commands, the consumer commands, the notarization/quarantine position, and the
  `v0.1.0` repoint recipe.
- **`brew install` is recorded as NOT RUN.** macOS Mach-O tarballs on a Linux host; the reason and
  the exact macOS re-run commands are in the step log rather than a fabricated success.

## Task Commits

Each task was committed atomically:

1. **Task 1: The formula, the sums cross-check, the self-test and the live URL check** — `6eaf6b09` (feat)
2. **Task 2: The activation doc and the evidence item** — `1b00b583` (feat)

**Plan metadata:** the `docs(04-03): complete …` commit carrying this SUMMARY, STATE/ROADMAP/REQUIREMENTS.

## The formula, verbatim

```ruby
class DeepseekBuild < Formula
  desc "DeepSeek-first terminal coding agent"
  homepage "https://github.com/coruairc/deepseek-build"
  version "0.1.0-proof1"
  license "Apache-2.0"

  livecheck do
    skip "Prerelease-only proof tag; repoint manually when v0.1.0 is published."
  end

  on_macos do
    on_arm do
      url "https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/deepseek-build-0.1.0-proof1-aarch64-apple-darwin.tar.gz"
      sha256 "2b61dfab870b397b49ed810f53e073714582e01dbf5c6f9d166656367c51324b"
    end

    on_intel do
      url "https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/deepseek-build-0.1.0-proof1-x86_64-apple-darwin.tar.gz"
      sha256 "ffe0f3c5f76be9ee6fa2dff1d53c34de6dce66d3553a094ee62524f58f57b3a7"
    end
  end
  # install: bin.install "deepseek-build"; bin.install_symlink "deepseek-build" => "dsb";
  #          doc.install "LICENSE", "NOTICE", "THIRD-PARTY-NOTICES", "README.txt"
  # caveats: ad-hoc signed / not notarized / no Developer ID; xattr -d com.apple.quarantine
  # test:    deepseek-build --version and dsb --version both match "deepseek-build #{version}"
end
```

| Arch | URL | sha256 | Size |
|------|-----|--------|------|
| arm64 | `…/download/v0.1.0-proof1/deepseek-build-0.1.0-proof1-aarch64-apple-darwin.tar.gz` | `2b61dfab870b397b49ed810f53e073714582e01dbf5c6f9d166656367c51324b` | 49,591,197 |
| x86_64 | `…/download/v0.1.0-proof1/deepseek-build-0.1.0-proof1-x86_64-apple-darwin.tar.gz` | `ffe0f3c5f76be9ee6fa2dff1d53c34de6dce66d3553a094ee62524f58f57b3a7` | 53,850,000 |

## The Homebrew output, verbatim

```
$ brew style local/deepseek-test/deepseek-build

1 file inspected, no offenses detected
```

```
$ HOMEBREW_SIMULATE_MACOS_ON_LINUX=1 brew audit --strict --formula local/deepseek-test/deepseek-build
brew audit --strict: 0 offenses (exit 0)
$ HOMEBREW_SIMULATE_MACOS_ON_LINUX=1 brew audit --strict --online --formula … 
brew audit --strict --online: 0 offenses (exit 0)
```

```
$ HOMEBREW_SIMULATE_MACOS_ON_LINUX=1 brew fetch --force --formula local/deepseek-test/deepseek-build
==> Fetching deepseek-build from local/deepseek-test
✔︎ Formula deepseek-build (0.1.0-proof1)
file=6a2cf2fafe044f92425037f2f8d32286b061441d90e9d7ba94fa358282b41ef3…--deepseek-build-0.1.0-proof1-x86_64-apple-darwin.tar.gz
size=53850000
sha256=ffe0f3c5f76be9ee6fa2dff1d53c34de6dce66d3553a094ee62524f58f57b3a7
```

```
# control: the same formula with one wrong sha256
==> Fetching wrongsha from local/deepseek-test
✘ Formula wrongsha (0.1.0-proof1)
Error: Formula wrongsha (0.1.0-proof1) reports different checksum: 2b61dfab870b397b49ed810f53e073714582e01dbf5c6f9d166656367c51324b
       SHA-256 checksum of downloaded file:                        ffe0f3c5f76be9ee6fa2dff1d53c34de6dce66d3553a094ee62524f58f57b3a7
```

```
# without the simulation switch, on Linux
Error: local/deepseek-test/deepseek-build: formula requires at least a URL
```

Homebrew version: **7.0.16** (Linuxbrew at `/home/linuxbrew/.linuxbrew`). No system `ruby` is
installed; `brew`'s bundled Ruby evaluated the file.

## The author activation commands (from `packaging/homebrew/README.md`)

```sh
gh repo create coruairc/homebrew-deepseek-build --public \
  --description "Homebrew tap for deepseek-build"
git clone git@github.com:coruairc/homebrew-deepseek-build.git
mkdir -p homebrew-deepseek-build/Formula
cp packaging/homebrew/deepseek-build.rb homebrew-deepseek-build/Formula/
cd homebrew-deepseek-build && git add Formula/deepseek-build.rb && git commit -m … && git push origin HEAD
```

Consumers:

```sh
brew tap coruairc/deepseek-build     # resolves github.com/coruairc/homebrew-deepseek-build
brew install deepseek-build
brew test deepseek-build
```

## Files Created/Modified

- `packaging/homebrew/deepseek-build.rb` — binary formula (51 lines): two arch URLs/hashes, `dsb`
  symlink, four doc files, caveats, `--version` test, livecheck skip.
- `packaging/homebrew/README.md` — state, author activation, what a user gets, signature and
  quarantine, the `v0.1.0` repoint recipe.
- `scripts/check-homebrew-formula.py` — 15 offline + 4 online assertions, three-case self-test,
  stdlib only, mode 100755.
- `docs/evidence/04-distribution-proof/homebrew-formula/` — `report.json` (15 steps, `result: ok`),
  `checker.log`, `resolve-branch.rb`, `notes.md`, and one log per step (13 step logs).

## Decisions Made

- **Pinned to `v0.1.0-proof1`.** It is the only tag with macOS artifacts; `v0.1.0` is 404. The
  repoint is a four-line edit plus a checker run, documented in the README.
- **`livecheck do skip`, not an omitted livecheck.** Homebrew's default strategy for a GitHub URL
  is `:github_latest`, which queries `api.github.com` — a host not otherwise in play. The explicit
  `skip` removes it; `brew audit --strict --online` still passes.
- **No top-level `url`.** The plan's shape (URLs inside `on_macos`/`on_arm`/`on_intel`) is kept
  even though it makes the formula unloadable on Linux; evaluating the macOS branches is done with
  Homebrew's own `HOMEBREW_SIMULATE_MACOS_ON_LINUX=1`.
- **`brew fetch` as the byte-level proof.** It exercises the same download+checksum path
  `brew install` uses, without installing a foreign-arch binary, and the wrong-hash control proves
  the gate is real.
- **`brew install` NOT RUN.** Stated in the evidence and the README; the tap repository is the
  author's action per CONTEXT.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] The checker's URL parser rejected every valid URL**
- **Found during:** Task 1 (first checker run)
- **Issue:** `parse_asset` raised `asset is not directly under the tag` for both real URLs because
  the `<tag>/<asset>` remainder always contains one `/`. The first run reported `11/15 PASS`.
- **Fix:** Require exactly one `/` and `partition` it; the run went to `15/15` (offline) and `19/19`
  with `--online`.
- **Files modified:** `scripts/check-homebrew-formula.py`
- **Committed in:** `6eaf6b09`

**2. [Rule 2 - Missing critical functionality] `livecheck` would have contacted `api.github.com`**
- **Found during:** Task 1
- **Issue:** The plan says "no `livecheck` that contacts another host". Omitting the block is not
  enough: `brew livecheck` then uses Homebrew's default GitHub strategy against the release URL,
  which queries `api.github.com`.
- **Fix:** Added `livecheck do skip "…" end`; `brew audit --strict --online` still exits 0. The
  checker asserts a present livecheck contains `skip`.
- **Files modified:** `packaging/homebrew/deepseek-build.rb`, `scripts/check-homebrew-formula.py`
- **Committed in:** `6eaf6b09`

**3. [Rule 3 - Blocking issue] `brew audit` cannot load a macOS-only formula on Linux**
- **Found during:** Task 1
- **Issue:** `brew audit --strict --formula` failed at load with
  `Error: … formula requires at least a URL` — on Linux no `on_macos` block applies, so the stable
  spec is empty. `--os macos --arch arm64` does not help: the load happens before simulation.
- **Fix:** Used Homebrew's own `HOMEBREW_SIMULATE_MACOS_ON_LINUX=1` (its `SimulateSystem` hook for
  exactly this case): audit and style exit 0. The un-simulated error is recorded as a step so the
  limitation is visible, not hidden.
- **Files modified:** evidence steps
- **Committed in:** `1b00b583`

**4. [Rule 3 - Blocking issue] Homebrew 7 refuses to load an untrusted tap**
- **Found during:** Task 1
- **Issue:** `brew audit` on the throwaway tap failed with
  `Refusing to load formula … from untrusted tap local/deepseek-test`.
- **Fix:** `brew trust local/deepseek-test` on the throwaway local tap (no network, no public tap).
  Recorded in `notes.md` as a re-run step.
- **Committed in:** `1b00b583`

**5. [Rule 2 - Missing critical functionality] Added a per-arch resolution helper to the evidence**
- **Found during:** Task 2
- **Issue:** The dispatch requires real `brew style`/`brew audit` output. `brew fetch` resolves only
  the host arch, so the `on_arm` branch had no Homebrew-level check.
- **Fix:** `docs/evidence/.../resolve-branch.rb` (10 lines) evaluates the formula for
  `os: :macos, arch: :arm|:intel` via `Homebrew::SimulateSystem`; both branches resolve to the exact
  URL and sha256. Added as an evidence artifact.
- **Committed in:** `1b00b583`

**6. [Rule 3 - Blocking issue] The plan says "no commit"; the dispatch requires commits**
- **Found during:** close-out
- **Issue:** The plan's success criteria say "No commit made; the working tree is left for the
  reviewer", but the executing dispatch instructs one atomic commit per task plus a SUMMARY commit.
- **Fix:** Followed the dispatch (as 04-01 and 04-02 did): two task commits plus the metadata commit.
  HEAD is `main` by project configuration (`.planning/config.json` → `git.branching_strategy: "none"`).
- **Committed in:** `6eaf6b09`, `1b00b583`

**Total deviations:** 1 auto-fixed bug (1), 2 missing-critical additions (2, 5), 3 blocking/mechanics resolutions (3, 4, 6).
**Impact on plan:** No scope change. The extra helper and the simulation switch are what make the
Homebrew-level claims real; the un-simulated failure is recorded rather than papered over.

## Issues Encountered

- **`ruby` is not on `PATH`** (`command -v ruby` empty) — the plan's assumption. Homebrew 7.0.16 ships
  its own portable Ruby, so `brew style`, `brew audit` and `brew ruby` all work; the Python checker
  remains text-only and says so in its docstring.
- **`brew audit --online` fetched Homebrew's own API metadata** (`packages.golden_gate.jws.json`
  from formulae.brew.sh). That is the tool's own bookkeeping, not a product destination; the formula
  itself adds no host beyond `github.com`.
- **The throwaway tap and the 103 MB of downloaded tarballs live outside the repo** under
  `/home/linuxbrew/.linuxbrew/Homebrew/Library/Taps/local/homebrew-deepseek-test` and
  `~/.cache/Homebrew/downloads`. Nothing was installed; nothing outside the repo was modified except
  that cache and the local tap.

## User Setup Required

**One author action, documented:** create `coruairc/homebrew-deepseek-build` (public) and copy
`Formula/deepseek-build.rb` into it. Then consumers can `brew tap coruairc/deepseek-build` /
`brew install deepseek-build`. `packaging/homebrew/README.md` has the exact commands. This is
deliberate: creating a public repository is not a build step.

## Known Stubs

- `brew install` — **NOT RUN**, not a stub in code. The formula's install path cannot execute on
  this Linux host (macOS Mach-O binaries). Recorded in the evidence as a step with the reason and
  the macOS re-run commands; coverage entry D4 carries `human_judgment: true`.
- `.planning/WINDOWS.md` was **not** appended to: `gsd_run`/`gsd-tools` are absent from `PATH`, and
  nothing here is a skipped test, an unrun `<verify>` or a code stub. The one un-runnable step
  (`brew install`) is a designed boundary of the plan, not a defect left behind.

## Threat Flags

None. The formula adds no outbound destination: both URLs are `github.com` release assets, the same
host `install.sh` already uses, and `livecheck` is explicitly skipped so `brew livecheck` cannot
query `api.github.com`. No public repository was created. No secret was read or written; no package
manager install ran.

## Next

Ready for **04-04** (GHCR image + container README). PKG-04 is complete for the formula and the
documented activation; the live `brew install` remains the author's step. 04-05's deviation file
should carry the CONTEXT decision to keep `v0.1.0-proof1` published.

## Self-Check: PASSED

- All 22 files named in the frontmatter are on disk (2 packaging, 1 script, 19 evidence files);
  `git status --short` is clean after the two task commits.
- The two task commits are ancestors of HEAD: `6eaf6b09`, `1b00b583`.
- Plan verifications re-run at close-out: checker `--self-test` → `self-test: 3/3 PASS`; url/sha256
  counts `= 2` and `= 2`; the positional-pairing Python assertion printed `urls+hashes ok`;
  `report.json` is `result: ok` with `packaging/homebrew/deepseek-build.rb` as an artifact;
  `grep -c 'brew tap coruairc/deepseek-build'` = 2 and `grep -c 'brew install deepseek-build'` = 1 in
  the README; `scripts/key-leak-check.sh --scan docs/evidence` → `PASS`.
- Artifact digests re-hashed and equal to `report.json`: formula `73d1bd43…`, README `a4696234…`,
  checker.log `c61e1534…`, resolve-branch.rb `033fa98a…`.
