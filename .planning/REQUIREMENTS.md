# Requirements: deepseek-build

**Defined:** 2026-11-21
**Revised:** 2026-11-21 — routing and per-model cost cut from this milestone
**Core Value:** One DeepSeek key in, a trustworthy daily coding agent out: the right model on every turn, zero traffic to any host you did not configure, and a release you can install with confidence.

**Scope note:** Phase 1 only — the freeze. This is a subsequent milestone on a mature tree: the
adapter, rebrand, deletions, eval harness, and Linux release pipeline already exist and are not
re-specified here.

**Revision note:** Automatic routing (`route_turn`/`ModelRoutingPolicy`), the `[models] routing`
key, session pins, the `Route` status-line item, `/route`, and per-model cost accounting were
removed from v1 by decision. Routing is the first item after the freeze and is designed in the
open-build phase, where it generalizes across providers. `/model` is the manual switch until then.
What survives from the cost work is the honesty bug fix: Flash must be priced correctly and
estimates must be labelled and consistent.

## v1 Requirements

Requirements for the v0.1.0 freeze. Each maps to exactly one roadmap phase.

### Sandbox

- [x] **SBOX-01**: Runtime-socket deny materialization skips candidates whose parent path is not traversable, while staying fail-closed for denies the process can see but cannot mask (traversable parent with unreadable endpoint still errors)
- [x] **SBOX-02**: A test proves a root-only deny-path parent does not abort bwrap startup, and `scripts/sandbox-run.sh` resolves and reports IPv6 destinations instead of `non-ip`

### Test Gate

- [x] **TEST-01**: `cargo test --workspace` exits 0 with zero failures on the author's host, asserted for completeness (target count) rather than read from a tail line
- [x] **TEST-02**: The test gate runs the `config-docs` feature tests, which `cargo test --workspace` does not reach today because `xai-grok-shell` has `default = []`
- [x] **TEST-03**: The recorded `git_gate`/`restore_fetch` timing flake is either made deterministic or explicitly quarantined, so "zero failures" is not a single lucky run

### Evidence

- [x] **EVID-01**: Every loop item carries a committed asciinema cast plus `report.json` under `docs/evidence/`, with absolute artifact paths normalized out
- [x] **EVID-02**: The evidence convention is documented and referenced by each item's verification step, so a later reader can re-run it

### Cost Honesty

- [x] **COST-01**: `deepseek-v4-flash` exists in the pricing table, so the default model no longer silently prices at Pro rates (~4× overstatement)
- [x] **COST-02**: The TUI cost figure is labelled as an estimate
- [x] **COST-03**: Headless honours `[ui.status_line.pricing]` overrides instead of `PricingTable::default()`, so the TUI and the JSON output report the same number

### Default Model

- [x] **DEF-01**: `deepseek-flash` is the catalog default, documented with the at-ceiling-benchmark caveat (63/63 for both models, so the eval set cannot prove Flash sufficient for hard work) and the revert path (`/model deepseek-v4-pro`, `-m deepseek-v4-pro`)

### Runtime Proof

- [x] **PROOF-01**: A live smoke run on a real model-driven session with a throwaway key passes, with the transcript committed as evidence
- [x] **PROOF-02**: Runtime egress on that same session is measured and shows only the configured provider, the `web_fetch` host, and AF_UNIX sockets — zero violations — with the measurement committed as evidence
- [ ] **PROOF-03**: `scripts/key-leak-check.sh` is green over the tree and over the committed artifacts, no key material appears in logs, saved sessions, or evidence files, and the throwaway key is revoked afterwards

### Permissions

- [x] **PERM-01**: The interactive permission prompt is exercised end to end and shows allow-once, allow-for-session, and reject, with the security warning on high-signal findings
- [x] **PERM-02**: Headless permission denial fails closed with an actionable hint naming `--always-approve` and `--permission-mode auto`, on both the stderr and JSON error paths
- [ ] **PERM-03**: Permission mode is visible in the UI and matches after resume
- [ ] **PERM-04**: `--permission-mode plan` is verified to arm plan mode headless, and `KNOWN-ISSUES.md` is corrected rather than the code being changed

### Sessions And Plan Mode

- [ ] **SESS-01**: Resume restores the session's model, reasoning effort, and plan state
- [ ] **SESS-02**: Plan mode is read-only except the plan file in every permission mode, including `always-approve` and `auto`
- [ ] **SESS-03**: A drafted plan survives quit and resume, promoted from the existing pty scenario into committed evidence
- [ ] **SESS-04**: The provenance of each resumed setting is documented — what comes from the session versus from config

### Release Pipeline

- [ ] **REL-01**: The build matrix is split into `build-linux` and `build-macos`, and `container` depends on `build-linux` only, so a slow macOS leg can no longer skip the image
- [ ] **REL-02**: Both macOS targets (aarch64 and x86_64) produce tarballs inside the job timeout, including one cache-warming run if cold-cache is the cause
- [ ] **REL-03**: The macOS binary passes a smoke check from the extracted tarball, and its signature remains valid after any strip or debug-profile change
- [ ] **REL-04**: The GHCR image is built, published, and pulled from a throwaway pre-release tag
- [ ] **REL-05**: The container image runs a real headless tool-executing turn with the documented run flags that keep the sandbox on, and the README states the image is headless-only with the mounts it needs
- [ ] **REL-06**: `install.sh` is proven against a real pre-release tag, and the unpinned `latest` path's actual behaviour is established rather than assumed from the fake-tree harness
- [ ] **REL-07**: `release-dist` ships no DWARF (`strip`/`debug` resolved, no `.debug`/`.dSYM` sidecar assumption), with the macOS signature risk verified
- [ ] **REL-08**: The `docker/metadata-action` tag list is guarded so an empty list fails loudly instead of producing a silent no-op manifest

### Packaging And Versioning

- [ ] **PKG-01**: Manifests report `0.1.0` via `[workspace.package] version` with the four remaining crates inheriting it, and `Cargo.lock` is regenerated
- [ ] **PKG-02**: The version is asserted on both paths — the CI `GROK_VERSION` stamp and a source build's `CARGO_PKG_VERSION`
- [ ] **PKG-03**: `release.yml` is verified against the `v0.1.0` tag shape (trigger, version parse, prerelease classification, `install.sh` validation) with no `deepseek-*` trigger added

### Documentation Truth

- [ ] **DOCS-01**: `DECISIONS.md` §E/§F is rewritten from the tree, keeping D9's personal-tool intent while recording that the release pipeline exists
- [ ] **DOCS-02**: `TESTING.md` §2/§6 is corrected — no OIDC test counts, no stale sign-in line
- [ ] **DOCS-03**: `KNOWN-ISSUES.md` is reconciled, including the superseded `deepseek-v0.1-snapshot` tag, `--permission-mode plan`, and the sandbox findings
- [ ] **DOCS-04**: The user guide covers the Flash default with its caveat and revert path, the estimate labelling, headless permission failure, container run flags, and the measured cost levers (`/model`, `--effort low`)

### Human Acceptance

- [ ] **ACC-01**: The author uses the TUI for 20–30 minutes on a clone of a real repo, reports back, and signs off; the acceptance note is committed and the freeze does not proceed without it

### Freeze

- [ ] **FREEZE-01**: Branch `deepseek` exists at the freeze commit on `main`
- [ ] **FREEZE-02**: A non-triggering freeze marker tag sits on that commit, and the pre-existing `deepseek-v0.1-snapshot` tag at `4b305bd0` is recorded as superseded rather than moved
- [ ] **FREEZE-03**: The pre-freeze checklist passes — clean suite, egress gate, branding surface, key-leak check, deny check, runtime proof artifacts, and the release dry-run evidence — after the author's sign-off

## v2 Requirements

Deferred. Tracked but not in the current roadmap.

### Routing (first item after the freeze, designed in open-build)

- **ROUT-V2-01**: Routing decided once per turn from the real session path, with the sampler-side entry point deleted
- **ROUT-V2-02**: `[models] routing` as a documented user setting
- **ROUT-V2-03**: Whole-session pin (model + effort) that routing respects, with a clear path
- **ROUT-V2-04**: `Route` status-line item showing the model and the reason
- **ROUT-V2-05**: `/route off|low|high|auto|status`
- **ROUT-V2-06**: Route decision recorded in local-only persisted turn metadata
- **ROUT-V2-07**: Escalation that actually escalates (the `pro_model` overwrite removed)
- **ROUT-V2-08**: Subagent turns follow the policy
- **ROUT-V2-09**: Resume deterministic under routing
- **ROUT-V2-10**: Provider-agnostic routing registry (open-build)

### Cost (follows routing)

- **COST-V2-01**: Per-model pricing of a mixed session from the ledger's `by_model` totals
- **COST-V2-02**: Per-model spend in `/usage`
- **COST-V2-03**: Off-peak pricing switch
- **COST-V2-04**: Cache/token bucket disjointness proof for routed turns

### Pipeline And Product

- **REL-V2-01**: A `ci.yml` running fmt/clippy/test on push
- **REL-V2-02**: Debug symbol sidecars published alongside tarballs
- **PROD-V2-01**: A full undo/checkpoint product beyond the existing rewind behaviour
- **PROD-V2-02**: Automatic (rather than configured) Flash-for-research subagent coupling

## Out of Scope

| Feature | Reason |
|---------|--------|
| Automatic model routing of any kind — `route_turn`, `[models] routing`, pins, `Route` item, `/route` | Cut by decision: routing generalizes across providers and is designed in open-build; `/model` is the manual switch until then |
| Per-model cost accounting for a mixed session | Meaningless while only one model can run in a session |
| **Renaming the ~435 internal `GROK_*` env names** | **Explicitly out of scope for this milestone** — not a requirement, not a phase, not a "while we're here" cleanup. Recorded in `PROJECT.md` §Out of Scope and §Key Decisions |
| Renaming `xai-grok-*` crates and packages | Locked decision D2 |
| Phase 2 "open-build" — provider registry, BYO provider/key, native Anthropic and Responses backends, product rename | Deferred; branches from the freeze tag |
| Restoring deleted telemetry, upload, voice, or computer-hub paths | Those deletions are the egress guarantee |
| Runtime pricing fetch or price scraping | New outbound destination — forbidden by D6/D10/D13 |
| Any routing or usage telemetry to a remote endpoint | Violates the egress invariant and the product premise |
| Hidden auto-approve in headless | Deletes the fail-closed guarantee that makes unattended runs safe |
| `deepseek-*` release tag support | Publish tag is `v0.1.0`, which today's `v*` trigger already handles; adding the shape would risk classifying a snapshot as stable |
| Moving or deleting the existing `deepseek-v0.1-snapshot` tag | Destructive remote ref change; superseded in docs instead |
| Publishing `v0.1.0` itself | The author publishes it; this milestone produces the tree, the proof, and the freeze marker |
| OAuth, accounts, or a hosted service | The product is a key in an environment variable |

## Traceability

Which phases cover which requirements. One phase per requirement, no duplicates, no orphans.

| Requirement | Phase | Status |
|-------------|-------|--------|
| SBOX-01 | Phase 1 — Truth Baseline | Complete |
| SBOX-02 | Phase 1 — Truth Baseline | Complete |
| TEST-01 | Phase 1 — Truth Baseline | Complete |
| TEST-02 | Phase 1 — Truth Baseline | Complete |
| TEST-03 | Phase 1 — Truth Baseline | Complete |
| EVID-01 | Phase 1 — Truth Baseline | Complete |
| EVID-02 | Phase 1 — Truth Baseline | Complete |
| COST-01 | Phase 1 — Truth Baseline | Complete |
| COST-02 | Phase 1 — Truth Baseline | Complete |
| COST-03 | Phase 1 — Truth Baseline | Complete |
| DEF-01 | Phase 1 — Truth Baseline | Complete |
| PROOF-01 | Phase 2 — Loop-Item Verification | Complete |
| PROOF-02 | Phase 2 — Loop-Item Verification | Complete |
| PROOF-03 | Phase 2 — Loop-Item Verification | Pending |
| PERM-01 | Phase 2 — Loop-Item Verification | Complete |
| PERM-02 | Phase 2 — Loop-Item Verification | Complete |
| PERM-03 | Phase 2 — Loop-Item Verification | Pending |
| PERM-04 | Phase 2 — Loop-Item Verification | Pending |
| SESS-01 | Phase 2 — Loop-Item Verification | Pending |
| SESS-02 | Phase 2 — Loop-Item Verification | Pending |
| SESS-03 | Phase 2 — Loop-Item Verification | Pending |
| SESS-04 | Phase 2 — Loop-Item Verification | Pending |
| REL-01 | Phase 3 — Pipeline Decoupling & Version | Pending |
| REL-02 | Phase 3 — Pipeline Decoupling & Version | Pending |
| REL-03 | Phase 3 — Pipeline Decoupling & Version | Pending |
| REL-07 | Phase 3 — Pipeline Decoupling & Version | Pending |
| REL-08 | Phase 3 — Pipeline Decoupling & Version | Pending |
| PKG-01 | Phase 3 — Pipeline Decoupling & Version | Pending |
| PKG-02 | Phase 3 — Pipeline Decoupling & Version | Pending |
| PKG-03 | Phase 3 — Pipeline Decoupling & Version | Pending |
| REL-04 | Phase 4 — Distribution Proof | Pending |
| REL-05 | Phase 4 — Distribution Proof | Pending |
| REL-06 | Phase 4 — Distribution Proof | Pending |
| DOCS-01 | Phase 5 — Documentation Truth & Freeze | Pending |
| DOCS-02 | Phase 5 — Documentation Truth & Freeze | Pending |
| DOCS-03 | Phase 5 — Documentation Truth & Freeze | Pending |
| DOCS-04 | Phase 5 — Documentation Truth & Freeze | Pending |
| ACC-01 | Phase 5 — Documentation Truth & Freeze | Pending |
| FREEZE-01 | Phase 5 — Documentation Truth & Freeze | Pending |
| FREEZE-02 | Phase 5 — Documentation Truth & Freeze | Pending |
| FREEZE-03 | Phase 5 — Documentation Truth & Freeze | Pending |

**Coverage:**
- v1 requirements: 41 total
- Mapped to phases: 41
- Unmapped: 0 ✓
- Duplicates: 0 ✓

**Per phase:**

| Phase | Requirements | Count |
|-------|--------------|-------|
| 1 — Truth Baseline | SBOX-01, SBOX-02, TEST-01, TEST-02, TEST-03, EVID-01, EVID-02, COST-01, COST-02, COST-03, DEF-01 | 11 |
| 2 — Loop-Item Verification | PROOF-01, PROOF-02, PROOF-03, PERM-01, PERM-02, PERM-03, PERM-04, SESS-01, SESS-02, SESS-03, SESS-04 | 11 |
| 3 — Pipeline Decoupling & Version | REL-01, REL-02, REL-03, REL-07, REL-08, PKG-01, PKG-02, PKG-03 | 8 |
| 4 — Distribution Proof | REL-04, REL-05, REL-06 | 3 |
| 5 — Documentation Truth & Freeze | DOCS-01, DOCS-02, DOCS-03, DOCS-04, ACC-01, FREEZE-01, FREEZE-02, FREEZE-03 | 8 |
| **Total** | | **41** |

---
*Requirements defined: 2026-11-21*
*Last updated: 2026-11-21 after cutting routing and per-model cost from the milestone*
