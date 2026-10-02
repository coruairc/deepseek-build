# DECISIONS — `deepseek-build`

Canonical decision log for turning the xAI Grok Build clone into a DeepSeek-first coding
agent named **`deepseek-build`**. Re-read this at the start of every phase. If a needed
decision is not covered here, stop and ask the stakeholder.

Last updated: 2026-10-02.

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
| O3 | ~~DeepSeek exact model IDs and parameters.~~ **Resolved** against official docs (2026-10-02): base URL `https://api.deepseek.com`; models `deepseek-v4-pro` and `deepseek-flash` (legacy `deepseek-v4-flash` accepted but retired/billed as Flash); context 1M, max output 384K; `thinking: {type: enabled|disabled}`; `reasoning_effort: none|low|high|max` (`minimal`→low, `medium`/`xhigh`→high); usage `prompt_tokens_details.cached_tokens` == `prompt_cache_hit_tokens`, plus `prompt_cache_miss_tokens` and `completion_tokens_details.reasoning_tokens`; streaming deltas carry `reasoning_content`; `tool_choice` `required`/named is rejected in thinking mode. | Done. |
| O4 | Whether to rename all internal `xai-grok-*` crates later. | Deferred by D2. |

## F. Phase log

| Phase | Status | Notes |
|-------|--------|-------|
| 0 — Audit & plan | Complete | `PLAN.md` written; audit done. |
| 1 — Cleanup & rebrand | Partial | Egress-policy guard + `NOTICE` added. Full removal of telemetry/upload/login/cloud-config/update/voice/imagine/marketplace and the user-facing rebrand are **not done** (large mechanical refactor). |
| 2 — DeepSeek adapter | Partial | Core committed (`cdbf5aa3`): thinking control, reasoning_content round-trip + sanitizer, quote repair, cache helpers, DeepSeek default models. Not yet done: end-to-end API-key credential path validation, mock-server test suite, auto-router, stray-thinking normalization, non-stream reasoning capture. |
| 3 — DeepSeek-native TUI | Not started | |
| 4 — Adapt existing competitive features | Not started | |
| 5 — Quality bar | Skipped (D9) | |
