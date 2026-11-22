---
last_mapped_commit: 221a1a591ef3af5f90d59d73ae7899d110f28b37
last_mapped_at: 2026-11-21
---
# External Integrations

**Analysis Date:** 2026-11-21

## APIs & External Services

**Model inference (the only compiled-in product host):**
- DeepSeek Chat Completions — all model turns.
  - SDK/Client: `reqwest` 0.12 via `xai_grok_extra_ca`, request shape in `crates/codegen/xai-grok-sampler/src/client.rs` (`post` to `chat/completions`). Types: `crates/codegen/xai-grok-sampling-types`. `ApiBackend` has a single variant, `ChatCompletions` (`crates/codegen/xai-grok-sampling-types/src/types.rs`). Responses API and Anthropic Messages backends are gone.
  - Default base URL: `https://api.deepseek.com` (`PRODUCTION_ENDPOINTS.cli_chat_proxy_base_url` in `crates/codegen/xai-grok-env/src/lib.rs`; `XAI_API_BASE_URL_DEFAULT` in `crates/codegen/xai-grok-config/src/endpoints.rs`). Catalog copies the same URL per model in `crates/codegen/xai-grok-models/default_models.json`.
  - Models: default `deepseek-flash` (catalog `default`, authoritative in the resolver); `deepseek-v4-pro` is the heavier option; hidden alias `deepseek-v4-flash` prices at Flash rates. The catalog still names `deepseek-flash` for the legacy `web_search`/`image_description`/`session_summary` role fields. `supports_backend_search` is false on all three — there is no hosted web-search tool on the provider.
  - Auth: `Authorization: Bearer <api_key>` (`AuthScheme::Bearer` in `crates/codegen/xai-grok-sampler/src/client.rs`). Env: `DEEPSEEK_API_KEY`, fallback `DEEPSEEK_BUILD_API_KEY`.
  - Override: `[endpoints] xai_api_base_url` or `DEEPSEEK_BUILD_XAI_API_BASE_URL`. A custom base is a user-chosen OpenAI-compatible provider, not a second baked-in vendor. Adding a new hardcoded host is an egress change and needs explicit approval.
  - Streaming: SSE via `eventsource-stream`. Retries live in the sampler actor. Request bodies may be compressed (`zstd` feature on the sampler crate).

**User-initiated fetch:**
- `web_fetch` tool — fetches a URL the model asks for, after permission checks.
  - SDK/Client: `reqwest` built in `crates/codegen/xai-grok-tools/src/implementations/grok_build/web_fetch/http.rs`. HTML-to-markdown via `htmd`. SSRF guards in `web_fetch/ssrf.rs`.
  - Default registration is `WebFetchConfig::Disabled` (`web_fetch/mod.rs`). When enabled, allowed domains, proxy, and timeouts come from `WebFetchParams`.
  - Auth: none by default. An optional `proxy_endpoint` is a user proxy, not a vendor API.
  - This is the path that contacts hosts other than the model provider during a session. It is not a fixed integration.

**MCP (user-configured servers):**
- Model Context Protocol client — tools from servers the user lists in config.
  - SDK/Client: `rmcp` 3.4 inside `crates/codegen/xai-grok-mcp` (reqwest 0.13 quarantined there). Transports: streamable HTTP and stdio (`SafeTokioChildProcess` in `crates/codegen/xai-grok-mcp/src/servers.rs`).
  - Auth: per server. Bearer file, on-disk `mcp_credentials.json` under `$DEEPSEEK_BUILD_HOME`, or browser OAuth (`crates/codegen/xai-grok-mcp/src/oauth.rs`, `oauth2` crate, `webbrowser::open`). OAuth discovery talks to the authorization server advertised by that MCP server (5s timeout). Client name sent in the flow is `deepseek-build`.
  - Config: user `config.toml` then project `.grok/config.toml` (`crates/codegen/xai-grok-config/src/mcp_servers.rs`). There is no official plugin-marketplace HTTP source compiled as a required host.
  - Stdio MCP children are local processes. HTTP MCP is an outbound destination the user configured. Do not add a default MCP host.

**Plugin git clones (optional, user-specified):**
- `crates/codegen/xai-grok-plugin-marketplace` resolves install refs (`owner/repo`, local paths) and can clone a git remote the user names. It does not phone a vendor marketplace API. Historical `xai-org/plugin-marketplace` strings in tests are examples, not a live official source.

**Local language servers:**
- LSP tools spawn a local language server over stdio (`crates/codegen/xai-grok-tools/src/implementations/lsp/`, `async-lsp` 0.2). Not a cloud API.

**Deleted vendor integrations (absent from the tree; do not treat as live):**
- xAI OAuth/OIDC/device-code login, Mixpanel, Sentry, OTLP export crates, GCS/S3 upload clients, computer-hub, voice/Imagine, hosted `web_search`. `scripts/check-egress.sh` hard-fails if those host strings reappear under `crates/`, `prod/`, or `third_party/` (allowlist is the configured model provider, user MCP, and loopback).
- `EndpointsConfig` in `crates/codegen/xai-grok-config/src/endpoints.rs` still has fields for feedback, managed-config URL, and a GCS service-account key. The OTLP and trace-upload endpoint fields were deleted in rc5. `resolve_managed_config_url()` defaults to `{proxy}/deployment/config`. No production caller fetches that URL; managed policy is the local file `managed_config.toml`. `xai-grok-telemetry` `track()` does not POST (`crates/codegen/xai-grok-telemetry/src/client.rs`). Do not wire those leftover fields to a new host.
- Relay and gateway URLs in `crates/codegen/xai-grok-env/src/lib.rs` are `ws://127.0.0.1/...` placeholders. Cloud relay is not a destination.

**GitHub (release channel only, not the agent runtime):**
- `install.sh` downloads the tarball and `SHA256SUMS` from `https://github.com/<repo>/releases` and verifies SHA-256 before extract. It does not call `api.github.com`.
- CI publishes the same assets with `softprops/action-gh-release` and pushes `ghcr.io/<owner>/deepseek-build` (`.github/workflows/release.yml`). The running agent does not check GitHub for updates (auto-update crate was removed).

## Data Storage

**Databases:**
- Embedded SQLite via rusqlite 0.37 `bundled`. No Postgres, MySQL, or hosted DB client.
  - Connection: filesystem under the config home and workspace session dirs. Journal mode helpers: `crates/codegen/xai-sqlite-journal`.
  - Clients:
    - `crates/codegen/xai-grok-session-search` — session FTS.
    - `crates/codegen/xai-grok-memory` — memory store plus optional `sqlite-vec` KNN.
    - `crates/codegen/xai-grok-foreign-sessions` — read-only open of other agents' local stores (Claude, Codex, optional Cursor). Metadata discovery only; it does not call those products' APIs.
    - `crates/codegen/xai-grok-dashboard-store` — local dashboard DB.
    - `crates/codegen/xai-fast-worktree` — optional metadata DB (`metadata` feature).
- Session transcripts and compaction state are also files (`crates/codegen/xai-compaction-transcript`, `crates/codegen/xai-grok-session-events`).

**File Storage:**
- Local filesystem only for product data. Config, sessions, logs, and MCP credentials stay under `~/.deepseek-build` (or `DEEPSEEK_BUILD_HOME`) and the working tree.
- No S3/GCS client. Trace-upload bucket fields in `EndpointsConfig` are residual schema. `scripts/check-egress.sh` rejects `gs://`, `s3://`, and `/v1/storage`.

**Caching:**
- In-process only: `moka` 0.12 and `arc-swap` where crates already use them (HTTP client swap in `web_fetch`, config memos). No Redis, Memcached, or CDN SDK.
- Prompt-cache behavior is a property of the DeepSeek Chat Completions API (prefix cache), not a separate cache service. The sampler does not forward a Chat Completions `prompt_cache_key` (`ApiBackend::forwards_prompt_cache_key` is false).

## Authentication & Identity

**Auth Provider:**
- API key, not an OAuth identity provider. Interactive sign-in is rejected with instructions to set `DEEPSEEK_API_KEY` or configure an external auth provider command (`crates/codegen/xai-grok-login/src/flow.rs`).
  - Implementation:
    1. `DEEPSEEK_API_KEY`, else `DEEPSEEK_BUILD_API_KEY` (`crates/codegen/xai-grok-login/src/auth_method.rs`). A runtime override (`set_runtime_xai_api_key`) is preferred over `std::env::set_var` so other threads are not raced.
    2. `api_key` / `env_key` on the model entry in `config.toml`.
    3. Optional operator command (`auth_provider_command`) that prints a token; `crates/codegen/xai-grok-login/src/refresh/external_refresher.rs` re-runs that local binary. The command's network targets are the operator's, not this repo's.
    4. Cached credentials in `$DEEPSEEK_BUILD_HOME/auth.json` for non-env flows. First-party OIDC refresh is not a live network stack.
  - Wire: Bearer token on the model request. Header prefix for product headers is `x-deepseek-build` (`crates/codegen/xai-grok-brand`).
- MCP OAuth is separate and only for user MCP servers (`crates/codegen/xai-grok-mcp/src/oauth.rs`). Loopback callback, 600s browser timeout, credentials in `mcp_credentials.json`. Do not point this flow at `api.deepseek.com`.
- ACP clients that embed the agent speak stdio JSON-RPC (`crates/codegen/xai-acp-lib`). That is a local trust boundary, not an identity provider.

## Monitoring & Observability

**Error Tracking:**
- None. Sentry is deleted. Panics abort (`panic = "abort"` in `Cargo.toml`). `crates/codegen/xai-crash-handler` is local.

**Logs:**
- `tracing` / `tracing-subscriber` with `RUST_LOG`-style env filter. Telemetry crate writes local structured logs (`crates/codegen/xai-grok-telemetry/src/logs/`). `track()` enriches metadata and returns; it does not emit to Mixpanel or an OTLP collector.
- In-process Prometheus counters exist in workspace crates. They are not scraped by a hosted Prometheus. No pushgateway client.
- Diagnostics HTTP for the workspace daemon is loopback TCP port 6016 or a Unix socket `/tmp/workspace-server.sock` (`crates/codegen/xai-grok-diag-server/src/lib.rs`): `/ready`, `/statusz`, `/logs`. It is not exposed through sandbox port mapping. Treat `/logs` as sensitive.
- Egress policy check (static): `scripts/check-egress.sh` and `scripts/check-egress.sh --strict`. Runtime connect logger: `scripts/egress-check.sh`. Key-leak scan: `scripts/key-leak-check.sh`.

## CI/CD & Deployment

**Hosting:**
- GitHub Releases for tarballs. GHCR for the container (`ghcr.io/<repository_owner>/deepseek-build`). No other artifact host is allowed by the comment block at the top of `.github/workflows/release.yml`.

**CI Pipeline:**
- GitHub Actions, one workflow: `.github/workflows/release.yml`. Trigger: tag `v*` or `workflow_dispatch`.
- Native runners only (no QEMU cross): `ubuntu-22.04`, `ubuntu-22.04-arm`, `macos-15`, `macos-15-intel`.
- Actions are SHA-pinned (`actions/checkout`, `actions/upload-artifact`, `actions/download-artifact`, `Swatinem/rust-cache`, `softprops/action-gh-release`, Docker setup/login/metadata/build-push).
- Workflow token starts with empty permissions. Publish job needs `contents: write`. Container job needs `packages: write`.
- There is no pull-request test workflow in `.github/workflows/`. Tests are local `cargo test`.
- Installer contract test: `scripts/test-install.sh` (local fake HTTP server, no GitHub).

## Environment Configuration

**Required env vars:**
- `DEEPSEEK_API_KEY` — primary model credential. Fallback name: `DEEPSEEK_BUILD_API_KEY`. One of these, or `api_key` in config, is required for a live model call.
- Optional but operational:
  - `DEEPSEEK_BUILD_HOME` — config directory override.
  - `DEEPSEEK_BUILD_XAI_API_BASE_URL` — inference base override (name is historical; value is the OpenAI-compatible base).
  - `DEEPSEEK_BUILD_MODELS_BASE_URL` / `DEEPSEEK_BUILD_MODELS_LIST_URL` — model list override.
  - `DEEPSEEK_BUILD_CONFIG` / `DEEPSEEK_BUILD_CONFIG_PATH` — config overlay (fail-closed for secrets and egress).
  - `GROK_EXTRA_CA_BUNDLE` or `SSL_CERT_FILE` — extra TLS roots.
  - `GROK_VERSION` — stamped at release build time, not a user secret.
- Do not set or document removed `GROK_*` fallbacks as live credentials. Legacy `GROK_*` env fallbacks for user-facing names were removed; remaining `GROK_*` identifiers are internal (listed in `docs/INTERNAL-GROK-ENV-NAMES.md`).

**Secrets location:**
- Process environment (`DEEPSEEK_API_KEY` / `DEEPSEEK_BUILD_API_KEY`).
- `$DEEPSEEK_BUILD_HOME/config.toml` (`api_key` / `env_key`) and `$DEEPSEEK_BUILD_HOME/auth.json`.
- `$DEEPSEEK_BUILD_HOME/mcp_credentials.json` for MCP OAuth.
- No `.env` file in the repository. No checked-in credential files. API keys must not appear in logs, saved sessions, panic text, or commits. Redaction helpers: `crates/codegen/xai-grok-secrets/src/sanitizer.rs` (string scrubber only; it does not store secrets).

## Webhooks & Callbacks

**Incoming:**
- No public webhook server. The agent does not listen on a routable address for vendor callbacks.
- Local only:
  - MCP OAuth loopback callback (`crates/codegen/xai-grok-mcp/src/oauth.rs`, axum).
  - Diagnostics listener, loopback or a `0600` Unix socket (`crates/codegen/xai-grok-diag-server`).
  - ACP over stdio (`crates/codegen/xai-acp-lib`), which is a pipe protocol, not HTTP.
- Hooks (`crates/codegen/xai-grok-hooks`) run local commands. They are not inbound HTTP.

**Outgoing:**
- Model provider: `POST {base}/chat/completions` (default `https://api.deepseek.com`).
- User MCP HTTP/SSE endpoints and, if that server requires OAuth, the discovered authorization and token endpoints.
- `web_fetch` target URLs when the tool is enabled and permitted.
- Optional git remotes for plugin install.
- Release-time only (not the agent): `github.com` release downloads and `ghcr.io` image push.
- Not outgoing: Mixpanel, Sentry, OTLP collectors, feedback upload, trace bucket upload, xAI accounts, auto-update checks. Config keys that still name those flows must not be implemented as new clients.

---

*Integration audit: 2026-11-21*
