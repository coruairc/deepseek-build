# Requirements: deepseek-build

**Defined:** 2026-11-21
**Core Value:** One DeepSeek key in, a trustworthy daily coding agent out: the right model on every turn, zero traffic to any host you did not configure, and a release you can install with confidence.

**Scope note:** Phase 1 only. This is a subsequent milestone on a mature tree — the adapter,
rebrand, deletions, eval harness, and Linux release pipeline already exist and are not
re-specified here. Phase 2 (open-build) is out of scope and branches from the freeze tag.

## v1 Requirements

Requirements for the v0.1.0 freeze. Each maps to exactly one roadmap phase.

### Routing Core

- [ ] **ROUT-01**: Routing is decided from the real session path (`SessionActor::run_turn_via_sampler` at turn entry), and the sampler-side routing entry point (`SamplingConfig.model_routing`, `ClientDefaults.model_routing`, `SamplingClient::route_turn`) is deleted so exactly one owner exists
- [ ] **ROUT-02**: The routing policy is a documented user setting at `[models] routing`, present in the `FEATURES` registry and the hand-edited `26-config-reference.md`, with the `config_docs` tests green
- [ ] **ROUT-03**: One routing decision is made per turn and held for every sampler round of that turn; a turn never sends two different models
- [ ] **ROUT-04**: Escalation actually escalates — `route_turn` no longer overwrites `pro_model` with the session's configured model, so a Flash session can reach Pro
- [ ] **ROUT-05**: Subagent turns follow the routing policy (no implicit exemption)

### Pin And Override

- [ ] **PIN-01**: `/model` or `/effort` sets a whole-session pin holding both model and effort, stored as one `Option<ModelRoute>` on the session and consumed as `ModelRoutingConfig.manual_override`
- [ ] **PIN-02**: Routing never overrides a pin, and the pinned state is visible on the status line
- [ ] **PIN-03**: A pin can be cleared, returning the session to routing, without changing the persisted new-session default model
- [ ] **PIN-04**: A resumed session is deterministic under routing — a saved model is either treated as a pin or explicitly re-routed and shown, never silently swapped

### Visibility And Control

- [ ] **DISP-01**: A status-line `Route` item shows the model chosen for the current turn plus the reason (`auto`, `auto · pinned`, `policy low`)
- [ ] **DISP-02**: The reason behind a routing decision ("why this model") is available from the same decision — on the status-line payload and in the turn log — with no extra inference
- [ ] **DISP-03**: The route decision is recorded in persisted turn metadata, local-only, so a resumed or exported session shows what ran and why
- [ ] **DISP-04**: `/route off|low|high|auto|status` controls the policy from one obvious place, leaving `/model` as the pin

### Cost And Cache

- [ ] **COST-01**: The TUI cost figure is labelled as an estimate
- [ ] **COST-02**: Headless honours `[ui.status_line.pricing]` overrides instead of `PricingTable::default()`, so TUI and JSON agree
- [ ] **COST-03**: Usage is priced per model when the ledger holds more than one model, using the existing `by_model` totals
- [ ] **COST-04**: `deepseek-v4-flash` exists in the pricing table and no longer silently falls back to Pro rates
- [ ] **COST-05**: An off-peak pricing switch applies the documented half rate, config-only, with no price fetch at runtime
- [ ] **COST-06**: `/usage` reports per-model spend from the ledger's existing `by_model` totals
- [ ] **COST-07**: Cache-hit and token buckets stay disjoint and correct after a routed turn

### Permissions

- [ ] **PERM-01**: The interactive permission prompt is exercised end to end and shows allow-once, allow-for-session, and reject, with the security warning on high-signal findings
- [ ] **PERM-02**: Headless permission denial fails closed with an actionable hint naming `--always-approve` and `--permission-mode auto`, on both the stderr and JSON error paths
- [ ] **PERM-03**: Permission mode is visible in the UI and matches after resume
- [ ] **PERM-04**: `--permission-mode plan` is verified to arm plan mode headless, and `KNOWN-ISSUES.md` is corrected rather than the code being changed

### Sessions And Plan Mode

- [ ] **SESS-01**: Resume restores the session's model, reasoning effort, and plan state
- [ ] **SESS-02**: Plan mode is read-only except the plan file in every permission mode, including `always-approve` and `auto`
- [ ] **SESS-03**: A drafted plan survives quit and resume, promoted from the existing pty scenario into committed evidence
- [ ] **SESS-04**: The provenance of each resumed setting is documented — what comes from the session versus from config

### Sandbox

- [ ] **SBOX-01**: Runtime-socket deny materialization skips candidates whose parent path is not traversable, while staying fail-closed for denies the process can see but cannot mask (traversable parent with unreadable endpoint still errors)
- [ ] **SBOX-02**: A test proves a root-only deny-path parent does not abort bwrap startup, and `scripts/sandbox-run.sh` resolves and reports IPv6 destinations instead of `non-ip`

### Test Gate

- [ ] **TEST-01**: `cargo test --workspace` exits 0 with zero failures on the author's host, asserted for completeness (target count) rather than read from a tail line
- [ ] **TEST-02**: The test gate runs the `config-docs` feature tests, which `cargo test --workspace` does not reach today because `xai-grok-shell` has `default = []`
- [ ] **TEST-03**: The recorded `git_gate`/`restore_fetch` timing flake is either made deterministic or explicitly quarantined, so "zero failures" is not a single lucky run

### Evidence

- [ ] **EVID-01**: Every loop item carries a committed asciinema cast plus `report.json` under `docs/evidence/`, with absolute artifact paths normalized out
- [ ] **EVID-02**: The evidence convention is documented and referenced by each item's verification step, so a later reader can re-run it

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
- [ ] **DOCS-04**: The user guide covers the routing key, the `Route` item, headless permission failure, container run flags, and off-peak pricing

### Freeze

- [ ] **FREEZE-01**: Branch `deepseek` exists at the freeze commit on `main`
- [ ] **FREEZE-02**: A non-triggering freeze marker tag sits on that commit, and the pre-existing `deepseek-v0.1-snapshot` tag at `4b305bd0` is recorded as superseded rather than moved
- [ ] **FREEZE-03**: The pre-freeze checklist passes — clean suite, egress gate, branding surface, key-leak check, deny check, and the release dry run evidence

## v2 Requirements

Deferred. Tracked but not in the current roadmap.

### Routing

- **ROUT-V2-01**: LLM-based turn-complexity classifier (rejected for v1: adds inference and makes routing unexplainable)
- **ROUT-V2-02**: Mid-turn Flash→Pro escalation on failure (blocked on an eval set where Flash fails and Pro succeeds; `docs/TUNING.md` E3c found none)
- **ROUT-V2-03**: Provider-agnostic routing registry (Phase 2)

### Pipeline

- **REL-V2-01**: A `ci.yml` running fmt/clippy/test on push
- **REL-V2-02**: Debug symbol sidecars published alongside tarballs

### Product

- **PROD-V2-01**: A full undo/checkpoint product beyond the existing rewind behaviour
- **PROD-V2-02**: A route picker modal or dedicated dashboard pane
- **PROD-V2-03**: Automatic (rather than configured) Flash-for-research subagent coupling

## Out of Scope

| Feature | Reason |
|---------|--------|
| Phase 2 "open-build" — provider registry, BYO provider/key, native Anthropic and Responses backends, product rename | Explicitly deferred; Phase 2 branches from the freeze tag |
| Renaming the ~435 internal `GROK_*` env names | Deferred by decision; would churn every adapter crate the freeze depends on |
| Renaming `xai-grok-*` crates and packages | Locked decision D2 |
| Restoring deleted telemetry, upload, voice, or computer-hub paths | Those deletions are the egress guarantee |
| Runtime pricing fetch or price scraping | New outbound destination — forbidden by D6/D10/D13 |
| Any routing or usage telemetry to a remote endpoint | Violates the egress invariant and the product premise |
| Hidden auto-approve in headless | Deletes the fail-closed guarantee that makes unattended runs safe |
| `deepseek-*` release tag support | Publish tag is `v0.1.0`, which today's `v*` trigger already handles; adding the shape would risk classifying a snapshot as stable |
| Moving or deleting the existing `deepseek-v0.1-snapshot` tag | Destructive remote ref change; superseded in docs instead |
| Publishing `v0.1.0` itself | The author publishes it; this milestone produces the tree, the proof, and the freeze marker |
| OAuth, accounts, or a hosted service | The product is a key in an environment variable |

## Traceability

Which phases cover which requirements. Populated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| *(pending roadmap)* | — | — |

**Coverage:**
- v1 requirements: 53 total
- Mapped to phases: 0
- Unmapped: 53 ⚠️ (expected before roadmap creation)

---
*Requirements defined: 2026-11-21*
*Last updated: 2026-11-21 after initial definition*
