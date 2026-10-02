# AGENTS.md — deepseek-build handoff

Read this first when resuming. It records exact state, what remains, and the
commands to continue. Companion docs: [`DECISIONS.md`](DECISIONS.md) (locked
decisions; wins on conflict), [`PLAN.md`](PLAN.md) (audit + phased plan),
[`README.md`](README.md).

## 0. Snapshot

- **Branch:** `dsb/integration` (the integration branch; all work merged here).
- **HEAD:** `b68327ca` — "chore(egress): scrub legacy xAI/host strings to clear HARD gate".
- **Remote:** `origin git@github.com:coruairc/deepseek-build.git`.
- **Toolchain:** Rust **1.94.0** via rustup (pinned in `rust-toolchain.toml`). Do not bump.
  `protoc` on PATH (`/usr/bin/protoc`); `dotslash` not needed.
- **Binary name (still upstream):** package `xai-grok-pager-bin`, artifact
  `target/release/xai-grok-pager`. Rebrand to `deepseek-build` is **not** done.
- **Egress gate:** `scripts/check-egress.sh` → **HARD OK**; SOFT report = **3452**
  branding/ACP-namespace hits (must become 0 after the rebrand slice).
- **Build:** `cargo check -p xai-grok-pager-bin` is green (~2 min warm).
  `cargo test --workspace` currently **fails to compile** (see §5).
- **Release build:** was started, then aborted by the user. No release binary exists yet.

## 1. How to resume (another machine)

```sh
git clone git@github.com:coruairc/deepseek-build.git
cd deepseek-build
git switch dsb/integration          # or: git switch -c dsb/integration origin/dsb/integration

# toolchain
rustup toolchain install 1.94.0 --component rustfmt clippy
source "$HOME/.cargo/env"

# sanity
cargo check -p xai-grok-pager-bin
scripts/check-egress.sh
```

Slice branches are pushed too (`dsb/telemetry`, `dsb/tools`, `dsb/autoupdate`,
`dsb/exfil`, `dsb/auth`, `dsb/misc`, `dsb/cloud`, `dsb/hard`, `dsb/hard2`);
they are already merged, so you normally stay on `dsb/integration`. To
re-parallelize, create fresh worktrees off `dsb/integration`:

```sh
git worktree add ../wt/<name> -b dsb/<name> dsb/integration
```

The slice worktrees on the original machine live under `/home/cein_orourke/wt/dsb-*`
(not part of the repo).

## 2. What is DONE

- **Phase 0:** `PLAN.md` audit + phased plan; `DECISIONS.md` locked decisions; `NOTICE`.
- **Phase 1 (partial, committed via slice branches, merged on `dsb/integration`):**
  - `mixpanel` crate deleted (`988bad45`).
  - Telemetry: xAI OTLP stack + `xai-grok-otel` deleted, Sentry sink deleted, external
    OTEL stream stubbed; `xai-mixpanel` gone (`dsb/telemetry`, `83d26ac7`).
  - xAI voice / Imagine image+video / `web_search` tools deleted; xAI plugin-marketplace
    official source removed (`dsb/tools`, `eb165260`).
  - Auto-update (`xai-grok-update`), remote announcements (`xai-grok-announcements`),
    subagent bundle (`xai-grok-bundle`) crates deleted (`dsb/autoupdate`, `6cd2277d`).
  - Upload/exfil: `xai-file-utils` deleted (GCS/S3/storage client/queue); heap_profile
    deleted; `xai-grok-workspace/src/upload` deleted; feedback/share upload handlers
    deleted (`dsb/exfil`, `bb949731`). Some shell upload/feedback paths were **stubbed**,
    not fully deleted — see §5.
  - Auth: xAI OAuth/OIDC/device-code/JWKS/refresh network stack deleted; API-key path
    (`DEEPSEEK_API_KEY` / `DEEPSEEK_BUILD_API_KEY`) added (`dsb/auth`, `3f56baf7`).
  - Misc: announcement banner UI + SuperGrok billing/upsell removed; computer-hub **not**
    removed (`dsb/misc`, `ef7e4d89`).
  - `xai-grok-cloud-config` crate deleted, code vendored inert into
    `xai-grok-shell/src/cloud_config/` (`dsb/cloud`, `02a551c0`).
  - **HARD egress gate cleared to zero** across production + tests
    (`dsb/hard`, `f0c2d43d`; `dsb/hard2`, `b68327ca`).
  - `prod/mc/cli-chat-proxy-types` renamed to `prod/mc/model-api-types`.
- **Endpoints foundation (me):** compiled endpoints → `https://api.deepseek.com` with
  loopback relay/gateway placeholders (`xai-grok-env`, `xai-grok-config/endpoints.rs`);
  xAI model entries removed; default model `deepseek-v4-pro`, plus `deepseek-flash` and a
  hidden `deepseek-v4-flash` alias (`633d80df`).
- **Phase 2 adapter core (me):** `ThinkingConfig` + `thinking` field, thinking/effort →
  `tool_choice` downgrade, `reasoning_content` final-pass sanitizer (backfills all prior
  assistant turns when tools are present), typographic-quote tool-call JSON repair,
  `TokenUsage` cache hit/miss helpers, DeepSeek default catalog (`cdbf5aa3`, `28296f7e`).
  Tests: `cargo test -p xai-grok-sampling-types --lib` (297), sampler stream tests (13).

## 3. What is LEFT (ordered)

1. **Backend removal (D5):** delete the `Responses` and `Messages` backends and their
   wiring so only `ChatCompletions` remains. In `xai-grok-sampling-types` (`ApiBackend`,
   `conversation/responses.rs`, `conversation/messages.rs`, `rs` types) and
   `xai-grok-sampler` (`client.rs` responses/messages paths, `stream/{responses,messages}.rs`).
   This is adapter-owned — keep it single-owner.
2. **Rebrand (D1/D2) → SOFT gate zero:** centralize name / config-dir / env-prefix in ONE
   module; rename binary to `deepseek-build`; config dir `~/.deepseek-build`; env prefix
   `DEEPSEEK_BUILD_*`; ACP extension namespace `x.ai/*` → `deepseek-build/*`; user-visible
   strings. Then `scripts/check-egress.sh --strict` must be zero.
3. **Finish real deletion of Phase-1 stubs** (see §5): shell `src/upload/*`,
   `session/repo_changes`, `feedback_manager`, feedback UI, `share`, vendored
   `shell/src/cloud_config/**`, `xai-computer-hub-{core,sdk,mcp-adapter}` and consumers,
   shell `src/remote/**` relay clients, `agent/relay.rs`.
4. **Phase 2 proof:**
   - `wiremock` suite: streaming, a **3+ turn tool-call chain replaying `reasoning_content`**,
     missing `reasoning_content` backfill, malformed tool-call JSON, partial stream,
     429/5xx retry with backoff.
   - non-stream reasoning capture + retries + clear error messages.
   - cache-stable prefix test (system prompt + tool defs byte-identical / stable order across
     turns; dynamic state at the tail).
   - **Live smoke test (STOP condition): needs `DEEPSEEK_API_KEY`** — GET `/models` to confirm
     `deepseek-v4-pro` / `deepseek-flash`, a plain chat, a streamed chat, a 3-turn thinking
     tool-call chain, and confirm `prompt_tokens_details.cached_tokens` /
     `prompt_cache_hit_tokens` and `reasoning_tokens` in usage. Fix + record any doc mismatch.
5. **Phase 3 TUI:** collapsible reasoning block (keybind), status bar (model, thinking level,
   tokens, cache hit rate, running cost from a config price table, context usage), slash
   commands/hotkeys to switch model + `reasoning_effort {none,low,high,max}` + toggle thinking,
   auto model routing (Flash/no-reasoning vs Pro/high) + manual override, per-turn/session
   cost + cache breakdown.
6. **Phase 4 (verify then fix):** plan mode, permission modes + dangerous-command detection,
   read-before-write enforcement, cache-aware compaction, session resume, subagents (Flash for
   research), MCP client, AGENTS.md loading, checkpoints/undo, headless mode.
7. **Phase 5:** release build + one-line run command, `TESTING.md` (20–30 min manual checklist),
   `scripts/sandbox-run.sh` (only `api.deepseek.com` reachable), `KNOWN-ISSUES.md`, `strace`
   runtime egress test, independent verification sub-agent.

## 4. Gates (from the task) and current status

1. `cargo build --release`, `cargo test --workspace`, clippy, fmt clean — **FAIL** (tests
   don't compile yet; release build not finished).
2. HARD egress zero — **PASS**. SOFT zero after rebrand — **FAIL** (3452 pending rebrand).
3. Runtime egress test via `strace -f -e trace=connect` — **NOT RUN**.
4. Adapter wiremock + live smoke — **NOT DONE** (live needs the key).
5. Independent verification sub-agent — was run once earlier for the adapter; **re-run at the
   end**.

Stop and ask only if: a gate fails twice, a decision is not in `DECISIONS.md`, a change would
add an outbound destination, or the live smoke test needs a key not provided. **The key is
currently not provided**, so the live smoke test is blocked.

## 5. Known gotchas / incomplete (do not paper over)

- **Tests do not compile.** Many `#[cfg(test)]` modules and `tests/**` files still reference
  symbols deleted in Phase 1 (`WebSearchInput`, `MediaGenOutput`, `ImageGenConfig`,
  `emit_announcements`, `heap_profile`, `external::config`, etc.). `cargo check` (non-test) is
  green; `cargo test` is not. Budget a dedicated pass.
- **Stubs left (no network, but present):** `xai-grok-telemetry/src/{external.rs,otel_layer.rs,
  trace_context.rs}` (no-op OTEL), shell `src/file_utils_compat.rs`, shell
  `session/repo_changes/mod.rs` (pure serde types), shell `session/feedback_manager.rs`,
  shell `src/upload/**` (restored inert, routed to compat), pager voice stub
  (`pager/src/xai_grok_voice.rs`), pager `views/announcements.rs` (no-op).
- **Not removed:** `xai-computer-hub-{core,sdk,mcp-adapter}` + consumers; shell `src/remote/**`
  relay/gateway clients; `agent/relay.rs`; vendored `shell/src/cloud_config/**`; remote
  settings/campaigns types (fetch is inert).
- **`xai-grok-cloud-config`** exists only as `xai-grok-shell/src/cloud_config/` now.
- **`xai-grok-*` crate/package names remain** (decision D2); only the user-facing surface will
  be renamed.
- Do not edit `xai-grok-env`, `xai-grok-config/src/endpoints.rs`, `xai-grok-models`,
  `xai-grok-sampling-types`, `xai-grok-sampler` in a generic scrub — those are adapter/endpoint
  owned.
- `THIRD-PARTY-NOTICES`, `third_party/NOTICE`, `LICENSE`, `NOTICE` must stay.

## 6. Useful commands

```sh
source "$HOME/.cargo/env"

cargo check -p xai-grok-pager-bin --message-format=short
cargo test  -p xai-grok-sampling-types --lib
cargo test  -p xai-grok-sampler --lib stream::chat_completions
cargo build --release -p xai-grok-pager-bin

scripts/check-egress.sh           # HARD gate
scripts/check-egress.sh --strict  # + branding (post-rebrand)
```
