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

| ID | Item | Status |
|----|------|--------|
| O1 | Legal sign-off on re-licensing the aggregate as `deepseek-build` while preserving upstream Apache-2.0 attribution. | Open; not blocking engineering. Required before any public release. |
| O2 | Toolchain install feasibility (rustup + Rust 1.94.0) in this environment; `dotslash` absent though `/usr/bin/protoc` exists. | Being resolved in Phase 1 setup. |
| O3 | ~~DeepSeek exact model IDs and parameters.~~ **Resolved** against official docs (2026-10-02): base URL `https://api.deepseek.com`; models `deepseek-v4-pro` and `deepseek-flash` (legacy `deepseek-v4-flash` accepted but retired/billed as Flash); context 1M, max output 384K; `thinking: {type: enabled|disabled}`; `reasoning_effort: none|low|high|max` (`minimal`→low, `medium`/`xhigh`→high); usage `prompt_tokens_details.cached_tokens` == `prompt_cache_hit_tokens`, plus `prompt_cache_miss_tokens` and `completion_tokens_details.reasoning_tokens`; streaming deltas carry `reasoning_content`; `tool_choice` `required`/named is rejected in thinking mode. **Live-verified 2026-10-07** (`scripts/smoke-test.sh`, 8/8 PASS): `GET /models` returns exactly `deepseek-flash`, `deepseek-v4-pro`; plain + streamed chat OK; 3-turn thinking tool chain OK (`reasoning_tokens=57`); on a cache hit `cached_tokens == prompt_cache_hit_tokens` (2432) and `prompt_tokens == hit + miss`; `tool_choice=required` → HTTP 400 "Thinking mode does not support this tool_choice". **Adapter mapping confirmed, no mismatch:** the harness `usage.input_tokens` = DeepSeek `prompt_cache_miss_tokens`, `cache_read_input_tokens` = `prompt_cache_hit_tokens`, `reasoning_tokens` = `completion_tokens_details.reasoning_tokens`, `total_tokens` = input + cache_read + output. | Done. |
| O4 | Whether to rename all internal `xai-grok-*` crates later. | Deferred by D2. |

## F. Phase log

| Phase | Status | Notes |
|-------|--------|-------|
| 0 — Audit & plan | Complete | `PLAN.md` written; audit done. |
| 1 — Cleanup & rebrand | Partial | **HARD egress gate green.** Deleted/neutralized: mixpanel, telemetry OTLP + Sentry, `xai-grok-otel`, `xai-file-utils` upload stack, heap_profile, workspace upload, `xai-grok-update`, `xai-grok-announcements`, `xai-grok-bundle`, xAI voice/Imagine/web_search, xAI OAuth/OIDC network stack, `xai-grok-cloud-config` (vendored inert into shell), SuperGrok upsell, xAI model entries/endpoints. `prod/mc/cli-chat-proxy-types` → `prod/mc/model-api-types`. **Rebrand not done** (SOFT report 3452); some stubs and `xai-computer-hub`/shell `remote/**` remain. See `AGENTS.md` §5. |
| 2 — DeepSeek adapter | Partial | Core committed (`cdbf5aa3`, `28296f7e`): thinking control, reasoning_content round-trip + sanitizer, quote repair, cache helpers, DeepSeek default models, endpoints foundation (`633d80df`). Not done: backend removal (Responses/Messages), wiremock suite, non-stream reasoning capture, cache-prefix test, live smoke (blocked on key). |
| 3 — DeepSeek-native TUI | Not started | |
| 4 — Adapt existing competitive features | Not started | |
| 5 — Quality bar | Not started | Release build aborted; no binary yet. |
