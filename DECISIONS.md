# DECISIONS — `deepseek-build`

Canonical decision log for turning the xAI Grok Build clone into a DeepSeek-first coding
agent named **`deepseek-build`**. Re-read this at the start of every phase. If a needed
decision is not covered here, stop and ask the stakeholder.

Last updated: 2026-10-07.

---

## A. Product decisions (locked)

| ID | Decision | Rationale / notes |
|----|----------|-------------------|
| D1 | **Project name = `deepseek-build`.** Binary `deepseek-build`; config dir `~/.deepseek-build`; env prefix `DEEPSEEK_BUILD_*`; ACP extension namespace `deepseek-build/*`. | Stakeholder answer. |
| D2 | **Phase 1 rename scope = user-facing surface only.** Rename binary, config dir, env prefix, and all user-visible strings. Leave internal `xai-grok-*` crate/package names unchanged for now. | Lower risk, smaller diff. A full internal crate rename may come later. |
| D3 | **Auto-update removed entirely.** No update check, no binary download. | Reduces egress; personal tool. |
| D4 | **Drop `web_search`; keep `web_fetch`, permission-gated (ask).** | `web_search` is xAI-hosted. |
| D5 | **Provider backend = `ChatCompletions` only.** Remove `Responses` and `Messages` backends and wiring. | DeepSeek and custom OpenAI-compatible hosts use Chat Completions. |
| D6 | **Delete, never disable,** all data-exfiltration / telemetry / remote-control code. | User requirement. |
| D7 | **Keep upstream licensing.** Keep `LICENSE` and all NOTICE/attribution files. Do not relicense upstream files. Add a `NOTICE` attribution to `xai-org/grok-build` (Apache-2.0). | Apache-2.0 §4 obligations. |
| D8 | **Toolchain = Rust 1.94.0 via rustup. Do not bump.** | Repo `rust-toolchain.toml` pins it. |
| D9 | **Phase 5 skipped** (benchmarks, release docs, release pipeline). | Personal tool. |
| D10 | **Final egress allowlist:** the configured model provider (default `https://api.deepseek.com`) and user-configured MCP servers. Nothing else. | Core security invariant. |
| D11 | **DeepSeek adapter + `reasoning_content` round-trip + cache-stable prefix is authored by ONE agent, never parallelized.** | Correctness-critical, shared state. |
| D12 | **An independent verification sub-agent** that did not write the reviewed code re-checks gates 2 & 3 and the PLAN.md deletion checklist. | Anti-groupthink. |
| D13 | **The API key must never appear in logs, panic/error text, saved sessions, crash output, commits, or command output.** Auth is env-only (`DEEPSEEK_API_KEY` / `DEEPSEEK_BUILD_API_KEY`); a missing key prints one clear line naming both vars. `SamplerConfig`'s `Debug` redacts `api_key`; header-conversion errors do not echo it. `scripts/key-leak-check.sh` runs with a fake key and scans logs + saved sessions. | Stakeholder requirement; defense against accidental secret egress into local artifacts. |
| D14 | **Cost estimates use a configurable price table with peak-rate defaults.** Three rates per model, USD per 1,000,000 tokens: `cache_hit` input, `cache_miss` input, `output` (reasoning tokens are inside `output`, never double-charged). Source: DeepSeek official pricing page, <https://api-docs.deepseek.com/quick_start/pricing>, **fetched 2026-10-07**. Peak defaults: `deepseek-flash` = 0.006 / 0.30 / 1.20; `deepseek-v4-pro` = 0.044 / 1.32 / 3.96. **Peak hours are 01:00-04:00 and 06:00-10:00 UTC Mon-Fri excluding Chinese public holidays; all other hours are off-peak, at exactly half the peak rates.** The table has no clock, so it always prices at peak (conservative, never under-estimates); an override can halve the rates to price off-peak. User-overridable via `[ui.status_line.pricing.<model>]` in `~/.deepseek-build/config.toml`; an unnamed model falls back to the v4 Pro default rather than zero. Shown in the TUI `cost` status item and in the headless result as `estimated_cost_usd` + `estimate: true` (estimated locally; any provider-reported `total_cost_usd` is left untouched). | Personal tool; DeepSeek's usage payload carries tokens but no price. Peak default is conservative. |

## B. Phase scope (locked)

- Implement **Phases 1, 2, 3**, then **Phase 4 items already present that need adapting**.
- **Skip Phase 5** (benchmarks/docs/release polish).

## C. Execution rules (locked)

1. Small, reviewable commits; one logical change each.
2. Parallelize independent work with sub-agents, each on its own git worktree/branch with a
   narrow scope:
   - upload/exfil infrastructure
   - telemetry (Mixpanel / Sentry / OTLP)
   - OAuth + remote config + remote-control relay
   - auto-update
   - xAI web/voice/imagine tools
   - plugin marketplace
   - rename of user-facing surface
   - backend removal (Responses/Messages)
3. One integrator (the lead agent) merges branches in small commits and keeps build + tests
   green after every merge.
4. Deletions must be **real deletions**, not disabled code.
5. Run `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt` clean before declaring a
   phase done.
6. Keep `PLAN.md` updated.
7. Only stop and ask if: a gate fails twice; a needed decision is not in this file; or a
   change would add an outbound network destination.

## D. Automatic gates (run after each phase; stop & report on failure)

1. `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt --check` clean.
2. `grep -rE "gs://|s3://|mixpanel|sentry|x\.ai|grok\.com"` returns nothing outside
   attribution notices.
3. Egress test: run the binary for a short scripted session with only `api.deepseek.com` (or
   a local mock) reachable; confirm no other connection attempts
   (`strace -f -e trace=connect` or equivalent).
4. DeepSeek adapter mock-server tests pass: streaming, tool-call chains, missing
   `reasoning_content`, malformed tool-call JSON, 429/5xx retries.
5. Independent verification sub-agent re-checks gates 2 & 3 and the PLAN.md deletion
   checklist.

## E. Open items

Rewritten from the tree on 2026-11-22 (was: Phase-0 plan-time list).

| ID | Item | Status |
|----|------|--------|
| O1 | Legal sign-off on re-licensing the aggregate as `deepseek-build` while preserving upstream Apache-2.0 attribution. | **Open**; not blocking engineering. Required before any public release. |
| O2 | Toolchain feasibility (rustup + Rust 1.94.0, `protoc`). | **Resolved**: `rust-toolchain.toml` pins 1.94.0; `/usr/bin/protoc` is present; `dotslash` is not needed. |
| O3 | DeepSeek model IDs, parameters and usage mapping. | **Done.** Base URL `https://api.deepseek.com`; models `deepseek-v4-pro` and `deepseek-flash` (hidden legacy `deepseek-v4-flash`); `thinking: {type}`; `reasoning_effort: none|low|high|max`; usage cache fields and `reasoning_tokens`; `reasoning_content` in stream deltas; `tool_choice` required/named rejected in thinking mode. Live-verified 2026-10-07 (`scripts/smoke-test.sh`, 8/8). |
| O4 | Whether to rename all internal `xai-grok-*` crates later. | Deferred by D2. |
| O5 | macOS release artifacts and the GHCR image. | **Open.** `v0.0.1-rc5` is Linux-only; macOS legs time out and are now isolated in `build-macos`; the image is decoupled from macOS but has never been built or pulled. Phase 4's proof is blocked in this run (never push/tag). |
| O6 | A clean-host confirmation of the whole-workspace suite. | **Deferred (tests skipped by owner, 2026-11-22).** Last recorded green: 2026-11-21 on the author's host (`results=308`); not re-confirmed elsewhere. |
| O7 | The author's acceptance session (ACC-01) and the freeze marker. | **Open, owner decision.** The signed note must be committed before any `deepseek` branch or marker tag; this milestone publishes no tag. |

## F. Phase log

Rewritten from the tree on 2026-11-22.

| Phase | Status | Notes |
|-------|--------|-------|
| 0 — Audit & plan | Complete | `PLAN.md`, `DECISIONS.md`, `NOTICE`. |
| 1 — Cleanup & rebrand | Complete | Mixpanel, the xAI OTLP/Sentry/OTel stack, `xai-file-utils` upload/exfil, heap_profile, auto-update, announcements, the subagent bundle, xAI voice/Imagine/`web_search`, the OAuth/OIDC network stack and cloud-config were deleted; shell upload/feedback, `remote/**`, `repo_changes` and the computer-hub crates are gone. **HARD and SOFT egress gates are zero**; rebrand done (binary `deepseek-build`, config `~/.deepseek-build`, ACP namespace, brand constants); legacy `GROK_*` env fallbacks removed (internal `GROK_*` names remain, D2). |
| 2 — DeepSeek adapter | Complete | Chat-Completions-only backend; thinking control; `reasoning_content` round-trip and final-pass sanitizer; typographic-quote tool-call repair; cache-hit helpers; DeepSeek catalog with `deepseek-flash` default; wiremock suite; live smoke 8/8 PASS (2026-10-07, author's host). |
| 3 — DeepSeek-native TUI | Complete | Reasoning fold, `/think` `/model` `/effort` `/theme`, configurable status line with the estimate-labelled `cost` item, Monokai theme and whale logo. |
| 4 — Adapt existing features | Complete | Permission modes and dangerous-command detection, read-before-write guard, plan mode, resume, subagents, MCP client, `AGENTS.md` loading, checkpoints/undo and headless mode exist and were hardened through this milestone's Phase 1/2 work. |
| 5 — Quality bar / release | **Present, not certified** | Release pipeline exists (native `build-linux`/`build-macos`, GHCR container job decoupled from macOS, empty-tag guard, `release-dist` ships no DWARF), `install.sh` plus a fake-tree harness, `docs/evidence/` with the Phase-1 convention, and the Linux-only prerelease `v0.0.1-rc5`. Not certified: no macOS artifact, no GHCR image pulled, container/installer proofs blocked (never push/tag), and the suite is deferred (tests skipped by owner, 2026-11-22). **D9's personal-tool intent stands**: none of this is required to use the tool; the pipeline is an author-run path, not a promise of hosted distribution. |
