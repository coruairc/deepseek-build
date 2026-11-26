# Roadmap: deepseek-build

## Overview

**Milestone:** `v0.1.0` — the freeze. A subsequent milestone on the already-shipping rc5 lineage:
the adapter, rebrand, deletions, eval harness, TUI, tools, permissions, MCP, compaction and a
Linux-only release pipeline all exist and are not re-planned here. The milestone does not add a
product surface. It makes the finished thing honest, proves it on a real session, proves its
artifacts, and freezes it.

Six phases, three workstreams, sequential. **Truth:** fix the sandbox deny materialization, make the
cost figure and the default model honest, then assert a zero-failure suite and stand up the evidence
convention. **Proof:** exercise the daily loop and measure the runtime egress boundary on a real
model-driven session with a throwaway key, each with a committed artifact. **Distribution:** decouple
the pipeline and settle the version/tag shape before any tag exists, prove the image and installer
from a throwaway pre-release tag, make Windows a supported target at a named tier, then reconcile the
docs, take the author's signed acceptance, and cut the freeze marker last.

Routing and per-model cost are **not** in this milestone. `/model` and `/effort` are the manual
switches; automatic routing is designed in open-build after the freeze, where it generalizes across
providers.

Research base: `main` @ `221a1a59` (STACK/ARCHITECTURE read `221a1a59`; PITFALLS read `9e6780d9`).
Re-verify `release.yml` edges and manifest versions before Phase 3. Design decisions are locked in
`PROJECT.md` §Key Decisions.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Truth Baseline** - Sandbox stops aborting, cost and default model stop lying, and the suite is provably green (completed 2026-11-23)
- [x] **Phase 2: Loop-Item Verification** - Permissions, resume and plan mode proven against a real session, with the egress boundary measured (completed 2026-11-25)
- [x] **Phase 3: Pipeline Decoupling & Version** - A slow macOS leg can no longer erase the image and the tag shape is settled (completed 2026-11-25; REL-02 closed by Phase 4's tag run 45724842211 on 2026-11-26: macOS legs 47.8 / 61.5 min against the 240 min budget)
- [ ] **Phase 4: Distribution Proof** - Image, installer and macOS artifacts proven from a throwaway tag (5/5 plans executed; REL-04 closed after the image was pulled by digest and run; REL-05's turn is proven but its sandbox clause is unmet because the image ships no bwrap; the pre-release is kept by recorded deviation)
- [ ] **Phase 5: Windows Support** - Builds, installs and runs on Windows at a named tier, with the sandbox refusing honestly instead of no-opping
- [x] **Phase 5.1: Project Directory Rebrand (INSERTED)** - Project-local config is `.deepseek/`, `.grok` stays as a compatibility fallback, and one constant flips to `.ob` for open-build (completed 2026-11-26)
- [ ] **Phase 5.2: API Key Entry (INSERTED)** - A masked popup stores a key in a 0600 file when none is set, with the env var still winning
- [ ] **Phase 6: Documentation Truth & Freeze** - Docs match the tree, the author signs off, and the freeze marker moves no published tag

## Phase Details

### Phase 1: Truth Baseline
**Goal:** As the author, I want the sandbox to stop aborting on this host, the cost figure and the default model to stop lying, and every claim to carry a committed artifact, so that a green suite actually means something.
**Mode:** mvp
**Depends on**: Nothing (first phase)
**Requirements**: SBOX-01, SBOX-02, TEST-01, TEST-02, TEST-03, EVID-01, EVID-02, COST-01, COST-02, COST-03, DEF-01
**Success Criteria** (what must be TRUE):
  1. `xai-grok-sandbox --test deny_paths_e2e` passes unmodified — no `#[ignore]`, no widened skip predicate — a root-owned deny-path parent no longer aborts bwrap startup, a traversable-but-unreadable endpoint still errors, and `scripts/sandbox-run.sh` reports IPv6 destinations instead of `non-ip`.
  2. `cargo test --workspace` exits 0 with zero failures on the author's host, and the gate proves completeness by counting `test result:` lines against the target count (307 today) instead of reading the tail; `cargo test -p xai-grok-shell --lib --features config-docs` runs as a named gate step; the `git_gate`/`restore_fetch` flake is deterministic under repetition or explicitly quarantined with a recorded reason.
  3. `docs/evidence/` holds a committed `.cast` plus `report.json` with absolute paths normalized out, and the convention is documented so later phases can append to it and a later reader can re-run it.
  4. A Flash turn prices at Flash rates instead of Pro rates, the TUI cost figure is labelled an estimate, and headless honours `[ui.status_line.pricing]` so the TUI and the JSON output report the same number.
  5. `deepseek-flash` is the catalog default, and the documentation that ships with it states the at-ceiling caveat (63/63 for both models, so the eval set cannot prove Flash sufficient for hard work) and the revert path (`/model deepseek-v4-pro`, `-m deepseek-v4-pro`).
**Plans**: 6/6 plans complete

Plans:
- [x] 01-01: Sandbox deny materialization — skip candidates whose parent is not traversable, stay fail-closed everywhere else, fixture matrix, `deny_paths_e2e` green unmodified
- [x] 01-02: `scripts/sandbox-run.sh` resolves and reports IPv6 destinations instead of `non-ip`
- [x] 01-03: Cost honesty — `deepseek-v4-flash` in the pricing table, estimate label on the TUI figure, headless honours `[ui.status_line.pricing]`
- [x] 01-04: Default model flip to `deepseek-flash` with the at-ceiling caveat and the revert path documented
- [x] 01-05: Test gate — completeness assertion against the target count, the `--features config-docs` step, `git_gate`/`restore_fetch` determinism or quarantine, evidence convention under `docs/evidence/`, full workspace run recorded

### Phase 2: Loop-Item Verification
**Goal:** As the author, I want the daily loop — permissions, resume, plan mode — and the runtime egress boundary verified against a real model-driven session with a throwaway key, so that "it works" rests on committed artifacts rather than on a pty scenario I wrote myself.
**Mode:** mvp
**Depends on**: Phase 1
**Requirements**: PROOF-01, PROOF-02, PROOF-03, PERM-01, PERM-02, PERM-03, PERM-04, SESS-01, SESS-02, SESS-03, SESS-04
**Success Criteria** (what must be TRUE):
  1. A live smoke run on a real model-driven session with a throwaway key passes — shell command, file edit and `web_fetch` — and the transcript is committed under `docs/evidence/` per the Phase 1 convention.
  2. Runtime egress measured on that same session shows only the configured provider, the `web_fetch` host and AF_UNIX sockets, with zero violations, and the measurement is committed as evidence.
  3. `scripts/key-leak-check.sh` is green over the tree and over the committed artifacts, no key material appears in logs, saved sessions or evidence files, and the throwaway key is revoked after the run.
  4. The interactive permission prompt offers allow-once, allow-for-session and reject with the security warning on high-signal findings; headless denial fails closed with an actionable hint naming `--always-approve` and `--permission-mode auto` on both the stderr and JSON error paths; permission mode is visible and matches after resume; `--permission-mode plan` is verified to arm plan mode headless, with `KNOWN-ISSUES.md` corrected rather than the code changed.
  5. Resume restores the session's model, reasoning effort and plan state; plan mode is read-only except the plan file in every permission mode including `always-approve` and `auto`; a drafted plan survives quit and resume as committed evidence; and the provenance of each resumed setting (session versus config) is documented.
**Plans**: 6/6 plans complete

Plans:
- [x] 02-01: Live smoke on a real model-driven session with a throwaway key; transcript committed as evidence
- [x] 02-02: Runtime egress measurement on that session — provider, `web_fetch` host, AF_UNIX only; measurement committed
- [x] 02-03: Key-leak canary plus scoped `--scan` over the committed artifacts, measured whole-tree baseline, dated revocation attestation
- [x] 02-04: Permission prompt — the high-signal warning row, the three outcomes on a real screen, and one denial sentence naming both flags on both headless paths
- [x] 02-05: Resume fidelity — model, effort and plan state restored, a drafted plan surviving quit and resume, provenance documented
- [x] 02-06: Permission mode across resume, headless plan arming, and plan mode read-only in every permission mode
**UI hint**: yes

### Phase 3: Pipeline Decoupling & Version
**Goal:** As the author, I want the pipeline decoupled so a slow macOS leg can no longer erase the image, and the version and tag contract settled before any tag is pushed, so that a dry run can only prove the release rather than break it.
**Mode:** mvp
**Depends on**: Phase 2 (sequential execution; the product tree must be final before an artifact is built from it)
**Requirements**: REL-01, REL-02, REL-03, REL-07, REL-08, PKG-01, PKG-02, PKG-03
**Preconditions**: the push path is the SSH remote `git@github.com:coruairc/deepseek-build.git`, so workflow-file edits need no `workflow`-scoped token; re-verify the remote before the first workflow push.
**Success Criteria** (what must be TRUE):
  1. `build-linux` and `build-macos` are separate jobs and `container` depends on `build-linux` only, and the `docker/metadata-action` tag list is guarded so an empty list fails loudly instead of producing a silent no-op manifest.
  2. Both macOS targets (aarch64 and x86_64) produce tarballs inside the job timeout on a `workflow_dispatch` run — one cache-warming run allowed if cold cache is the cause — and that run creates no release and moves no `:latest`.
  3. The macOS binary passes `--version` when extracted from the final tarball with its signature intact after any strip or debug-profile change, and `release-dist` ships no DWARF with no `.debug`/`.dSYM` sidecar assumed.
  4. Manifests report `0.1.0` through `[workspace.package] version` with the four remaining crates inheriting it and `Cargo.lock` regenerated; both the CI `GROK_VERSION` stamp and a source build's `CARGO_PKG_VERSION` print `0.1.0`.
  5. `release.yml` is verified against the `v0.1.0` tag shape — trigger, version parse, prerelease classification, `install.sh` validation — with no `deepseek-*` trigger added.
**Plans**: 4/4 plans complete

Plans:
- [x] 03-01: Split `build-linux`/`build-macos`; `publish` needs both, `container` needs linux only; strict semver gate in `version`; tag-list guard before the first push
- [x] 03-02: `[workspace.package] version = "0.1.0"`, four inheriting crates, `Cargo.lock` regenerated, both version paths asserted
- [x] 03-03: `release-dist` ships no DWARF (`debug = false`, link-time strip), explicit ad-hoc sign, extracted-tarball `--version` and `codesign --verify`, macOS budget
- [x] 03-04: `v0.1.0` tag-shape verification — trigger, the workflow's own derive body, prerelease classification, `install.sh` validation; one command for every static gate

### Phase 4: Distribution Proof
**Goal:** As the author, I want the image, the installer and the macOS artifacts proven from a real throwaway pre-release tag, so that the freeze rests on distribution evidence rather than on a fake-tree harness.
**Mode:** mvp
**Depends on**: Phase 3
**Requirements**: REL-04, REL-05, REL-06
**Success Criteria** (what must be TRUE):
  1. A throwaway pre-release tag produces the GHCR image, and the image is pulled from the registry — not just built locally — as the artifact a user would get.
  2. The pulled image runs a real headless turn that executes a shell command, with the documented run flags that keep the sandbox on, and the README states the image is headless-only with the mounts it needs.
  3. `install.sh` installs a pinned `VERSION=` from the real release and prints that version, and the unpinned `latest` path's actual behaviour is established by checking the real URL rather than the fake-tree harness.
  4. The throwaway tag, its release and the image tag it created are cleaned up, with no `:latest` left pointing at a throwaway build and the dry-run evidence committed.
**Plans**: 5/5 plans executed — 2 requirements partially/blocked (REL-04 Partial, REL-05 blocked); not a fully complete phase

Plans:
- [x] 04-01: Throwaway tag `v0.1.0-proof1` — pushed once (tag-only, target `b14d28eb`); run 45724842211 green (7/7 jobs, macOS 47.8 / 61.5 min, container success, two prerelease image tags, no `:latest`); release is a pre-release with exactly six assets, every byte matching `SHA256SUMS` and GitHub's digest. REL-02 and the macOS half of REL-03 closed. REL-04 stays PARTIAL (pull environment-blocked, 04-04)
- [x] 04-02: Installer proof against the real release — pinned proof against `v0.1.0-proof1`: the release asset `install.sh` is byte-identical to the tree (`31e114d4…`), the host tarball's sha256 equals both its `SHA256SUMS` line and GitHub's asset digest (`298c9f27…`), and the installed binary prints `deepseek-build 0.1.0-proof1 (b14d28eb2bc7)`. Unpinned `latest` behaviour observed: both `latest/download` URLs 404 and the installer exits 22 having installed nothing. README install truth corrected and gated by `scripts/check-readme-distribution.py` (install stage 12/12). **REL-06 closed.**
- [x] 04-03: Homebrew formula with real URLs and SHA-256 values, plus the tap activation doc — `packaging/homebrew/deepseek-build.rb` pins the two `v0.1.0-proof1` darwin assets (`2b61dfab…` arm64 / `ffe0f3c5…` x86_64) and installs the tarball with the `dsb` symlink; `scripts/check-homebrew-formula.py` is 19/19 + `self-test: 3/3`; Homebrew 7.0.16 reports `no offenses detected` on `brew style` and exit 0 on `brew audit --strict [--online]` (under `HOMEBREW_SIMULATE_MACOS_ON_LINUX=1`), both arch branches resolve, and `brew fetch` verified 53,850,000 bytes with the wrong-hash control rejected. `brew install` is NOT RUN (macOS binaries on a Linux host); the tap repo is the author's step, documented in `packaging/homebrew/README.md`. **PKG-04 closed.**
- [x] 04-04: Registry-side image proof and the environment-blocked in-image run — anonymous HTTPS read of `ghcr.io/coruairc/deepseek-build:v0.1.0-proof1`: OCI index `sha256:8f5c484e…` (equal to the CI digest), `linux/amd64` `sha256:1d216b60…` + `linux/arm64` `sha256:25dc0d25…` (both 1810 B, plus two attestation manifests recorded separately), exactly two tags and **no `:latest`** (absent after == absent baseline), package **public**, config blob read (`WorkingDir=/work`, `User=1000:1000`, `Entrypoint=["deepseek-build"]`, no API-key var). `image-registry` is 8/8 `result: ok`; `image-run-blocked` is **`result: skipped`** with the digest-pinned pull, `--version` and the sandbox-on headless turn (`--sandbox devbox`) because this host's Docker daemon is absent — **the run did not happen**. README container section rewritten (real digests, headless-only contract, two mounts, not-executed statement; checker 7/7). **REL-04 stays Partial (pull deferred), REL-05 stays open; ROADMAP criterion 1 "pulled from the registry" is recorded as unmet.**
- [x] 04-05: Evidence sealed, the keep-prerelease deviation recorded, leak scan green — `scripts/check-distribution-evidence.py` validates every report against the eight-key schema, the relative text-artifact rules and recomputed sha256 (self-test 4/4; 9/9 PASS, no file over 1 MiB); the nine-item index carries each item's requirements, result and re-run command; `DEVIATION-keep-prerelease.md` names **ROADMAP Phase 4 Success Criteria item 4** and `04-CONTEXT.md` §Cleanup and records the retained `v0.1.0-proof1` tag, its six-asset pre-release and its GHCR tags (the milestone's only macOS artifact) with `v0.1.0` unpublished and `:latest` absent; `scripts/key-leak-check.sh --scan` over `docs/evidence` and the phase directory exits 0; the retention invariants are re-asserted read-only (`:latest` absent, no `v0.1.0` release/tag, `deepseek` branch and `deepseek-v0.1-snapshot` still `4b305bd0`). **No cleanup was performed. REL-04 stays Partial (pull not executed), REL-05 stays blocked — Phase 4 is 5/5 plans executed, 2 requirements partially/blocked, not fully complete.**

### Phase 5: Windows Support
**Goal:** As the author, I want deepseek-build to build, ship and install on Windows x86_64 at a named tier — builds and runs, sandbox refuses honestly — so that a supported platform means something on all three and Windows never ships a silent sandbox no-op.
**Mode:** mvp
**Depends on**: Phase 4
**Requirements**: WIN-01, WIN-02, WIN-03, WIN-04, WIN-05, WIN-06
**Success Criteria** (what must be TRUE):
  1. A `build-windows` job (`windows-latest`) builds `x86_64-pc-windows-msvc` and produces a `.zip` containing the binary and the license files, and `publish` carries it as `release-x86_64-pc-windows-msvc` alongside the four existing tarballs.
  2. The compile spike (`cargo check --workspace --all-targets --target x86_64-pc-windows-msvc`) runs on a Windows runner and its error list is triaged into real gaps, missing `cfg`s and test-only failures; the phase reports the resulting file count rather than assuming the upstream port compiles unchanged.
  3. The TUI runs under ConPTY: a shell command and a file edit complete through the pager on a real Windows terminal.
  4. The sandbox is honest on Windows: only `off` is accepted, any other `--sandbox` value is a hard error naming the unsupported platform, and the README plus the sandbox user-guide page state the tier. No artifact claims "sandboxed".
  5. `install.ps1` downloads the zip, verifies SHA-256 before extracting, installs to `%LOCALAPPDATA%`, prints a PATH hint and supports `-Uninstall`; `scripts/test-install.ps1` exercises it, and the real proof runs against a throwaway tag.
  6. A Windows acceptance record reports the test-target count and the pass/fail/unix-gated totals (never a tail line), `--version` from the extracted zip, one headless turn with a sentinel, the two `*_windows.rs` PTY scenarios with `--ignored`, and an explicit list of what did not run.
**Plans**: 4/5 executed; 05-05 is blocked on a Windows host

Plans:
- [x] 05-01: Compile spike on `windows-latest` — `cargo check --all-targets` for the MSVC target, triage every error, produce the definitive file list (no product change)
- [x] 05-02: Sandbox honesty — reject non-`off` `--sandbox` on Windows, fix the misleading fallback message, platform note in README and the sandbox guide
- [x] 05-03: Long-tail correctness from 05-01's list (temp paths, file-lock slot, search fallback, gated modules)
- [x] 05-04: Distribution — `build-windows` job, `.zip`, `install.ps1` + `test-install.ps1`, publish wiring and verify loop
- [ ] 05-05: Windows acceptance record and docs, honestly listing what did not run

### Phase 5.1: Project Directory Rebrand (INSERTED)
**Goal:** As the author, I want the project-local config directory to match the product — `.deepseek/` here, one constant away from `.ob/` for open-build — with `.grok/` still read as a compatibility fallback, so that a project's config, agents, skills, workflows, hooks and settings live under the product's own name.
**Mode:** mvp
**Depends on**: Phase 5 (the Windows surface is settled before the directory name changes again)
**Requirements**: BRAND-01, BRAND-02, BRAND-03
**Success Criteria** (what must be TRUE):
  1. One constant in `xai-grok-brand` names the project directory (`.deepseek`) and one list names its legacy fallbacks (`.grok`); agents, plugins, skills, workflows, project config, hooks, LSP and `workspaced.toml` all resolve through it, with no remaining hardcoded project-scoped `.grok`.
  2. Precedence is deterministic and tested: `.deepseek` wins over `.grok`, `.claude`/`.cursor`/`.agents` keep their relative order, and a repo with both directories resolves the `.deepseek` file.
  3. The home directory is unchanged (`~/.deepseek-build`), and the user guide plus the config reference name `.deepseek/` for project scope and state the fallback, with the `config_docs` test green.
  4. The open-build variant requires exactly one edit: `PROJECT_DIR_NAME` → `.ob`.
**Plans**: 2/2 plans complete

Plans:
- [x] 05.1-01: Brand constant, agent/plugin/skill/workflow/project-config/hook/LSP/workspaced resolution with `.grok` fallback, precedence tests
- [x] 05.1-02: User guide and config reference rewrite to `.deepseek/`, fallback documented, `config_docs` gate

### Phase 5.2: API Key Entry (INSERTED)
**Goal:** As the author, I want to be asked for the DeepSeek key, masked, the first time I run without one — and have it stored privately — so that the binary is usable without editing shell profiles.
**Mode:** mvp
**Depends on**: Phase 5.1
**Requirements**: AUTH-01, AUTH-02, AUTH-03
**Success Criteria** (what must be TRUE):
  1. With no key in the environment and none saved, the interactive TUI opens a masked field; Enter saves and continues, Esc exits with the existing message, and headless never prompts.
  2. The key is written to `~/.deepseek-build/credentials.json` at mode 0600, atomically, and is read only after both environment variables; a group/other-readable file is refused with a `chmod 600` hint.
  3. The key never appears in a log, session, evidence file, panic message or commit; `Debug` redacts it and the leak scan is green.
**Plans**: 1 plan

Plans:
- [ ] 05.2-01: Credentials store, auth precedence, masked startup modal, headless guard, tests and docs

### Phase 6: Documentation Truth & Freeze
**Goal:** As the author, I want the docs to describe the tree that exists, my own acceptance session signed off, and the DeepSeek entity cut onto a `deepseek-build` branch with a non-triggering marker tag that moves no published ref, so that the freeze is defensible.
**Mode:** mvp
**Depends on**: Phase 5
**Requirements**: DOCS-01, DOCS-02, DOCS-03, DOCS-04, ACC-01, FREEZE-01, FREEZE-02, FREEZE-03
**Success Criteria** (what must be TRUE):
  1. `DECISIONS.md` §E/§F is rewritten from the tree, keeping D9's personal-tool intent while recording that the release pipeline exists; `TESTING.md` §2/§6 has no OIDC test counts and no stale sign-in line; `KNOWN-ISSUES.md` records the superseded `deepseek-v0.1-snapshot` tag, the `--permission-mode plan` correction, the sandbox findings and the Windows tier.
  2. The user guide covers the Flash default with its caveat and revert path, the estimate labelling, headless permission failure, container run flags, the measured cost levers (`/model`, `--effort low`) and the Windows tier.
  3. **Blocking gate:** the author has used the TUI for 20–30 minutes on a clone of a real repo and the signed acceptance note is committed; **FREEZE-01/02/03 do not run until that commit exists**, so no branch and no marker tag can precede the sign-off.
  4. Only after the acceptance note is committed, the pre-freeze checklist passes — completeness-asserted clean suite, `check-egress.sh --strict` exit 0 with a dated record (the three test-only strings must be resolved by a recorded decision, not by weakening the gate silently), `branding_surface`, `key-leak-check.sh`, `cargo-deny`, the runtime proof artifacts and the Phase 4 dry-run evidence.
  5. Branch **`deepseek-build`** exists at the freeze commit on `main`; a non-triggering marker tag sits on that commit with `git rev-parse <tag>^{}` equal to the branch tip; the pre-existing `deepseek` branch and `deepseek-v0.1-snapshot` tag at `4b305bd0` are unchanged and recorded as superseded; the marker triggered no workflow run and `v0.1.0` was not published.
**Plans**: 5/5 plans complete

Plans:
- [x] 06-01: `DECISIONS.md` §E/§F rewritten from the tree; `TESTING.md` §2/§6 corrected
- [x] 06-02: `KNOWN-ISSUES.md` reconciled — superseded snapshot tag, plan-mode correction, sandbox findings, Windows tier
- [x] 06-03: User guide — Flash default with caveat and revert path, estimate label, headless denial, container flags, cost levers, Windows tier
- [ ] 06-04: Human acceptance — 20–30 minutes in the TUI on a real repo clone, signed note committed (blocks 06-05)
- [ ] 06-05: Pre-freeze checklist, branch `deepseek-build`, non-triggering marker tag verified against the branch tip, snapshot refs untouched

## Sequencing Constraints

The order is load-bearing, not conventional. Each line is a reason a later step cannot move up.

- **Sandbox fix before the clean-suite gate, inside Phase 1:** the only known red tests are the bwrap pair, and only the product fix may zero them — not a skip, not `#[ignore]`, not a widened predicate. A gate that goes green because the test was weakened certifies nothing.
- **Phase 1 before Phase 2:** a loop item cannot be "proven" while the suite is red, and Phase 1 owns the evidence convention (path normalization, `docs/evidence/` layout) that every Phase 2 artifact is written into. Phase 2's proofs are the first real consumers of it.
- **Phase 2 before Phase 3:** the tarball and image a dry run proves are built from the product tree, so the product tree must be final first. Proving artifacts of a tree you then change means the freeze ships something that was never proven.
- **Phase 3 before Phase 4 — decoupling:** `container` currently depends on the whole four-leg matrix, so a slow macOS leg skips the image. A dry run before the split can therefore produce no image at all, or an image whose tag is decided by the metadata-action list that has no empty-list guard.
- **Phase 3 before Phase 4 — version and tag shape:** a dry run is a real push. It exercises the version parse, the prerelease classification and `install.sh` validation against a tag, so the tag shape must be settled first; otherwise the dry run can move `:latest` or publish a snapshot as a stable release.
- **Phase 4 before Phase 5:** Windows packaging is built from the post-split workflow and the settled version, so a Windows `.zip` cannot be produced by the coupled pipeline Phase 3 exists to remove, and it cannot carry a version the tag contract has not settled.
- **Phase 5 before Phase 6:** a freeze claim that omits a supported platform is incomplete, and DOCS-03/DOCS-04 must describe the Windows tier and installer that actually exist. The sandbox-honesty fix must land before any Windows artifact ships, because a `.zip` is the moment the silent no-op becomes a false claim.
- **Phase 6 last, and acceptance before the freeze marker:** docs reconciliation must describe the finished tree, the pre-freeze gates must be re-run after every other phase landed, and ACC-01 is a **blocking gate** — the author's signed note must be committed before branch `deepseek-build` or the marker tag is created. The freeze marker must also be non-triggering, and `v0.1.0` stays unpublished: the author publishes it.

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 5.1 → 6

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Truth Baseline | 6/6 | Complete    | 2026-11-23 |
| 2. Loop-Item Verification | 6/6 | Complete | 2026-11-25 |
| 3. Pipeline Decoupling & Version | 4/4 | Complete (REL-02 proof deferred to Phase 4) | 2026-11-25 |
| 4. Distribution Proof | 5/5 | REL-04 closed (pulled by digest + run); REL-05 turn proven, sandbox clause unmet | - |
| 5. Windows Support | 4/5 | WIN-02/WIN-04 closed; WIN-01/WIN-05 authored not proven; WIN-03/WIN-06 need a Windows host | - |
| 5.1 Project Directory Rebrand (INSERTED) | 2/2 | Complete | 2026-11-26 |
| 5.2 API Key Entry (INSERTED) | 0/1 | In progress | - |
| 6. Documentation Truth & Freeze | 3/5 | Docs reconciled; ACC-01 (author) blocks 06-04/06-05 | - |
