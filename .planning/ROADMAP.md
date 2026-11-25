# Roadmap: deepseek-build

## Overview

**Milestone:** `v0.1.0` — the freeze. A subsequent milestone on the already-shipping rc5 lineage:
the adapter, rebrand, deletions, eval harness, TUI, tools, permissions, MCP, compaction and a
Linux-only release pipeline all exist and are not re-planned here. The milestone does not add a
product surface. It makes the finished thing honest, proves it on a real session, proves its
artifacts, and freezes it.

Five phases, two workstreams, sequential. **Truth:** fix the sandbox deny materialization, make the
cost figure and the default model honest, then assert a zero-failure suite and stand up the evidence
convention. **Proof:** exercise the daily loop and measure the runtime egress boundary on a real
model-driven session with a throwaway key, each with a committed artifact. **Distribution:** decouple
the pipeline and settle the version/tag shape before any tag exists, prove the image and installer
from a throwaway pre-release tag, then reconcile the docs, take the author's signed acceptance, and
cut the freeze marker last.

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
- [ ] **Phase 2: Loop-Item Verification** - Permissions, resume and plan mode proven against a real session, with the egress boundary measured
- [ ] **Phase 3: Pipeline Decoupling & Version** - A slow macOS leg can no longer erase the image and the tag shape is settled
- [ ] **Phase 4: Distribution Proof** - Image, installer and macOS artifacts proven from a throwaway tag
- [ ] **Phase 5: Documentation Truth & Freeze** - Docs match the tree, the author signs off, and the freeze marker moves no published tag

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
**Plans**: 3/6 plans executed

Plans:
- [x] 02-01: Live smoke on a real model-driven session with a throwaway key; transcript committed as evidence
- [x] 02-02: Runtime egress measurement on that session — provider, `web_fetch` host, AF_UNIX only; measurement committed
- [ ] 02-03: Key-leak scan over tree and artifacts, no key material in logs/sessions/evidence, key revoked
- [x] 02-04: Permission prompt — the high-signal warning row, the three outcomes on a real screen, and one denial sentence naming both flags on both headless paths
- [ ] 02-05: Resume fidelity — model, effort and plan state restored, a drafted plan surviving quit and resume, provenance documented
- [ ] 02-06: Permission mode across resume, headless plan arming, and plan mode read-only in every permission mode
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
**Plans**: 4 plans

Plans:
- [ ] 03-01: Split `build-linux`/`build-macos`; `container: needs: [version, build-linux]`; empty-tag-list guard
- [ ] 03-02: macOS time budget — measure the job log first, then strip/debug, re-sign, extracted-tarball smoke, no DWARF in `release-dist`
- [ ] 03-03: `[workspace.package] version = "0.1.0"`, four inheriting crates, `Cargo.lock` regenerated
- [ ] 03-04: Dual version-path assertion and `v0.1.0` tag-shape verification

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
**Plans**: 3 plans

Plans:
- [ ] 04-01: Throwaway tag dry run — image build, publish to GHCR, pull from the registry
- [ ] 04-02: In-image headless tool-executing turn with sandbox-on flags; README GHCR block corrected
- [ ] 04-03: Installer proof from the real release, pinned and unpinned, plus cleanup and the evidence cast

### Phase 5: Documentation Truth & Freeze
**Goal:** As the author, I want the docs to describe the tree that exists, my own acceptance session signed off, and a freeze marker on a `deepseek` branch that moves no published tag, so that the freeze is defensible.
**Mode:** mvp
**Depends on**: Phase 4
**Requirements**: DOCS-01, DOCS-02, DOCS-03, DOCS-04, ACC-01, FREEZE-01, FREEZE-02, FREEZE-03
**Success Criteria** (what must be TRUE):
  1. `DECISIONS.md` §E/§F is rewritten from the tree, keeping D9's personal-tool intent while recording that the release pipeline exists; `TESTING.md` §2/§6 has no OIDC test counts and no stale sign-in line; `KNOWN-ISSUES.md` records the superseded `deepseek-v0.1-snapshot` tag, the `--permission-mode plan` finding and the sandbox findings.
  2. The user guide covers the Flash default with its caveat and revert path, the estimate labelling, headless permission failure, container run flags and the measured cost levers (`/model`, `--effort low`).
  3. **Blocking gate:** the author has used the TUI for 20–30 minutes on a clone of a real repo and the signed acceptance note is committed; **FREEZE-01/02/03 do not run until that commit exists**, so no branch and no marker tag can precede the sign-off.
  4. Only after the acceptance note is committed, the pre-freeze checklist passes — completeness-asserted clean suite, `check-egress.sh --strict` exit 0 with a dated record, `branding_surface`, `key-leak-check.sh`, `cargo-deny`, the runtime proof artifacts and the Phase 4 dry-run evidence.
  5. Branch `deepseek` exists at the freeze commit on `main`; a non-triggering marker tag sits on that commit with `git rev-parse <tag>^{}` equal to the branch tip; the pre-existing `deepseek-v0.1-snapshot` tag at `4b305bd0` is unchanged and recorded as superseded; the marker triggered no workflow run and `v0.1.0` was not published.
**Plans**: 5 plans

Plans:
- [ ] 05-01: `DECISIONS.md` §E/§F rewritten from the tree; `TESTING.md` §2/§6 corrected
- [ ] 05-02: `KNOWN-ISSUES.md` reconciled — superseded snapshot tag, `--permission-mode plan`, sandbox findings
- [ ] 05-03: User guide — Flash default with caveat and revert path, estimate label, headless denial, container flags, cost levers
- [ ] 05-04: Human acceptance — 20–30 minutes in the TUI on a real repo clone, signed note committed (blocks 05-05)
- [ ] 05-05: Pre-freeze checklist, branch `deepseek`, non-triggering marker tag verified against the branch tip, snapshot tag untouched

## Sequencing Constraints

The order is load-bearing, not conventional. Each line is a reason a later step cannot move up.

- **Sandbox fix before the clean-suite gate, inside Phase 1:** the only known red tests are the bwrap pair, and only the product fix may zero them — not a skip, not `#[ignore]`, not a widened predicate. A gate that goes green because the test was weakened certifies nothing.
- **Phase 1 before Phase 2:** a loop item cannot be "proven" while the suite is red, and Phase 1 owns the evidence convention (path normalization, `docs/evidence/` layout) that every Phase 2 artifact is written into. Phase 2's proofs are the first real consumers of it.
- **Phase 2 before Phase 3:** the tarball and image a dry run proves are built from the product tree, so the product tree must be final first. Proving artifacts of a tree you then change means the freeze ships something that was never proven.
- **Phase 3 before Phase 4 — decoupling:** `container` currently depends on the whole four-leg matrix, so a slow macOS leg skips the image. A dry run before the split can therefore produce no image at all, or an image whose tag is decided by the metadata-action list that has no empty-list guard.
- **Phase 3 before Phase 4 — version and tag shape:** a dry run is a real push. It exercises the version parse, the prerelease classification and `install.sh` validation against a tag, so the tag shape must be settled first; otherwise the dry run can move `:latest` or publish a snapshot as a stable release.
- **Phase 4 before Phase 5:** FREEZE-03's checklist includes the release dry-run evidence, and DOCS-04 must describe container run flags that were actually exercised. Documentation written before the proof would describe an artifact that may not exist.
- **Phase 5 last, and acceptance before the freeze marker:** docs reconciliation must describe the finished tree, the pre-freeze gates must be re-run after every other phase landed, and ACC-01 is a **blocking gate** — the author's signed note must be committed before branch `deepseek` or the marker tag is created. The freeze marker must also be non-triggering, and `v0.1.0` stays unpublished: the author publishes it.

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Truth Baseline | 6/6 | Complete    | 2026-11-23 |
| 2. Loop-Item Verification | 2/6 | In Progress | - |
| 3. Pipeline Decoupling & Version | 0/4 | Not started | - |
| 4. Distribution Proof | 0/3 | Not started | - |
| 5. Documentation Truth & Freeze | 0/5 | Not started | - |
