# deepseek-build

## What This Is

A personal, DeepSeek-first terminal coding agent forked from
[xai-org/grok-build](https://github.com/xai-org/grok-build): a fullscreen TUI plus a headless
mode that reads and edits code, runs shell commands under a permission and sandbox system, and
talks to DeepSeek through the OpenAI-compatible Chat Completions API. One `DEEPSEEK_API_KEY`,
one binary, no account and no service behind it. Internal crate names keep the upstream
`xai-grok-*` prefix; only the user-facing surface is `deepseek-build`.

The current released lineage is `v0.0.1-rc5` (Linux x86_64 + aarch64, `SHA256SUMS`,
`install.sh`). The tree is already DeepSeek-only in its adapter, endpoint, and model catalog,
and its egress boundary is closed — Phase 1 is about making that finished thing trustworthy and
freezing it, not about adding a product surface.

## Core Value

One DeepSeek key in, a trustworthy daily coding agent out: the right model on every turn, zero
traffic to any host you did not configure, and a release you can install with confidence.

## Requirements

### Validated

Shipped and confirmed valuable (from the codebase map, `.planning/codebase/`):

- ✓ Chat Completions adapter with DeepSeek semantics — `reasoning_content` round-trip,
  thinking/effort → `tool_choice` downgrade, typographic-quote tool-call repair, cache hit/miss
  accounting — `xai-grok-sampler`, `xai-grok-sampling-types`
- ✓ Single wire protocol — `ApiBackend` has exactly one variant, `ChatCompletions`
- ✓ Endpoints resolved to `https://api.deepseek.com` with config/env overrides —
  `xai-grok-config/src/endpoints.rs`
- ✓ Closed egress — `scripts/check-egress.sh --strict` exits 0 (HARD OK, SOFT zero); runtime
  traffic observed only to the provider, the `web_fetch` host, and AF_UNIX sockets
- ✓ Rebrand — config dir `~/.deepseek-build`, ACP namespace, binary `deepseek-build`, Monokai
  theme, brand constants in `xai-grok-brand`; legacy `GROK_*` fallbacks removed
- ✓ Deletion of exfil/telemetry/auto-update/voice/upload/computer-hub code — real deletions, no
  disabled stubs
- ✓ Release pipeline — 4-target native build workflow, `SHA256SUMS`, `install.sh` with 39/39
  harness assertions, `cargo-deny` clean
- ✓ TUI reasoning folding, `/model`, `/effort`, `/theme`, configurable status line with `cost`
- ✓ Headless single turn with `plain`/`json`/streaming emitters and
  `estimated_cost_usd` + `estimate: true`
- ✓ Eval harness — `scripts/evals/`, 21 auto-checked tasks; E2/E3 results in `docs/TUNING.md`
- ✓ Adapter proof — wiremock suite plus live smoke 8/8 PASS

### Active

Building toward the Phase 1 freeze. Each is a hypothesis until shipped and verified:

- [ ] Model routing is wired, not latent — `route_turn`/`ModelRoutingPolicy` decides per turn
  (Flash for simple turns, Pro for hard ones) from the real session path, and the decision is
  visible on the status line
- [ ] An explicit `/model` or `/effort` assignment pins the session, routing respects the pin
  until it is cleared, and clearing returns to routing
- [ ] Session resume, plan mode, permission prompts, and cost/cache display are verified against
  real sessions, fixed where broken, and polished where the verification exposes UX gaps
- [ ] The sandbox profile decision lands: runtime-socket denies are materialized only where the
  parent path is traversable, fail-closed behavior stays everywhere else, sandbox stays on by
  default
- [ ] `cargo test --workspace` exits 0 with zero failures on the author's host
- [ ] The release workflow is green including both macOS arches, and produces tarballs that
  pass the installer checksum test
- [ ] Installer and container image are proven from a throwaway pre-release tag, including
  pulling the published image
- [ ] Manifests are at `0.1.0` and `release.yml` fires on the DeepSeek tag shape
- [ ] Every loop item carries committed evidence: an automated test where feasible plus a
  scripted pty/headless session transcript
- [ ] `DECISIONS.md`, `TESTING.md`, and `KNOWN-ISSUES.md` are brought in line with the tree
- [ ] A `deepseek` branch exists with tag `deepseek-v0.1-snapshot` on it, marking the freeze

### Out of Scope

- **Phase 2 "open-build"** — provider registry, bring-your-own-provider and key, native Anthropic
  and Responses backends, key-to-host binding, product rename. Explicitly deferred; this plan
  covers Phase 1 only. Phase 2 branches from the freeze tag.
- **Renaming the ~435 internal `GROK_*` env names** (1,529 occurrences, 284 files,
  `docs/INTERNAL-GROK-ENV-NAMES.md`) — Phase 2 work by decision. Renaming them here would churn
  every adapter and config crate the freeze depends on.
- **Renaming `xai-grok-*` crate and package names** — decision D2 keeps them; only the
  user-facing surface is `deepseek-build`.
- **Restoring any deleted exfil, telemetry, upload, voice, or computer-hub path** — those
  deletions are the egress guarantee. Comments that still say "computer hub" are leftover
  naming, not a license to restore crates.
- **Publishing `v0.1.0`** — the author publishes it. Phase 1 produces the tree, the pipeline
  proof, and the freeze tag; it does not cut the public release.
- **New outbound destinations of any kind** — no model-catalog fetch, no telemetry endpoint, no
  pricing scrape.
- **OAuth, accounts, or a hosted service** — the product is a key in an environment variable.

## Context

- **Origin:** fork of grok-build (Apache-2.0). All xAI-owned server surfaces (OAuth/OIDC,
  telemetry, auto-update, announcements, voice, Imagine, `web_search`, plugin marketplace source,
  file upload) were deleted rather than disabled. `NOTICE` and `THIRD-PARTY-NOTICES` are shipped
  and must stay.
- **Phase-1 history is done:** the adapter core, rebrand, backend removal, release pipeline, and
  eval harness are complete on `main` at `221a1a59`. Do not re-litigate them.
- **Documentation drift is a known hazard:** `DECISIONS.md` §E/§F and `TESTING.md` §2/§6 still
  describe states that no longer exist (rebrand undone, OIDC tests, no binary). Trust the code,
  then `AGENTS.md`/`KNOWN-ISSUES.md`. Reconciling these is in scope.
- **Latent routing is the biggest correctness gap:** `ModelRoutingPolicy` defaults to `Off`,
  `SamplingClient::route_turn` has no caller outside its own module, and every shell construction
  site passes `model_routing: Default::default()`. `/model` and `/effort` are the only working
  routing today, so the Flash-vs-Pro cost control measured in `docs/TUNING.md` cannot be turned
  on and is not documented as a user setting.
- **Release is Linux-only today:** `v0.0.1-rc4` and `rc5` shipped Linux tarballs. The macOS
  aarch64 job hit the 90-minute job timeout and cancelled the run, which skipped the container
  job — so no GHCR image exists. rc4's x86_64 install was verified from the real release.
- **Test reality:** last full workspace run was 31,939 passed / 2 failed / 388 ignored across 307
  targets. Both failures are host-environment `xai-grok-sandbox --test deny_paths_e2e` bwrap
  failures on a root-owned `/run/containerd`. A follow-up full run on `cc6e6948` was started and
  never recorded as finished.
- **Toolchain:** Rust 1.94.0 pinned in `rust-toolchain.toml`. `protoc` on PATH. `strace` absent,
  so runtime egress is measured with the `LD_PRELOAD` logger in `scripts/egress-check.sh`.
- **Unexercised surfaces:** the interactive TUI has only ever been driven under a pty, never by a
  human; no macOS artifact exists; the container image has never been built or pulled.
- **Author's environment:** Linux x86_64, one shared `CARGO_TARGET_DIR`, 150 GB disk floor
  enforced by `scripts/disk-guard.sh`.

## Constraints

- **Toolchain:** Rust 1.94.0, do not bump. `cargo fmt --all --check` clean; clippy at 0 errors
  (16 pre-existing warnings are tolerated).
- **Egress invariants (D6/D10/D13):** the only outbound destinations are the configured model
  provider and user-configured MCP servers. `scripts/check-egress.sh --strict` must stay exit 0
  and `scripts/key-leak-check.sh` green. A change that adds a destination is stop-and-ask.
- **Secrets:** API keys never reach logs, saved sessions, panic text, artifacts, or commits.
- **Branding (D1/D2):** user-visible strings resolve through `xai-grok-brand` constants;
  `branding_surface` integration test gates the surface.
- **Prompt templates:** XOR-obfuscated into `src/prompt/prompt_encrypted.rs`; after any template
  edit run `python3 scripts/encrypt_templates.py` or the process aborts on invalid UTF-8.
- **Disk:** `.cargo/config.toml` keeps `incremental = false`; dev/test debug is
  `line-tables-only` with `debug = false` for dependencies. One shared cargo target dir.
- **Deliberate deletions stay deleted:** Phase-1 stub paths are absent from the tree. Comments
  and log-filter names are leftovers, not live network.
- **Copyright:** `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES`, `third_party/NOTICE` must ship in
  every tarball. Upstream attribution and non-affiliation statements stay in the README.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Phase 1 commits land on `main`; freeze cuts branch `deepseek` + tag `deepseek-v0.1-snapshot` from its tip | One linear history to prove; no divergence between what is tested and what is frozen | — Pending |
| Routing override = status line shows the routed model per turn; `/model` or `/effort` pins the session and routing never overrides a pin | Cost control must be visible and reversible; a silent router would undermine trust | — Pending |
| `route_turn` is wired from the session sampler path rather than deleted | Measured Flash-vs-Pro savings in `docs/TUNING.md` are only realizable if the turn loop calls it | — Pending |
| Sandbox stays on by default; runtime-socket denies are materialized only where the parent is traversable | Keeps the stronger default and the fail-closed invariant while removing a host-dependent startup failure | — Pending |
| "Clean" full workspace run means zero failures on the author's host | A documented-but-red suite cannot support a trustworthy freeze | — Pending |
| macOS ships both arches in v0.1.0 | The timeout is a pipeline defect to fix, not a platform to abandon | — Pending |
| Manifests bump to `0.1.0` and `release.yml` learns the DeepSeek tag shape in Phase 1 | The author's publish becomes a rerun, not a code change | — Pending |
| Evidence per loop item = automated test where feasible + committed scripted session transcript | A freeze is only defensible if each claim has an artifact | — Pending |
| Phase 2 (open-build) branches from the freeze tag and is not planned in this roadmap | Keeps Phase 1 shippable; provider abstraction is a different product decision | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-11-21 after initialization*
