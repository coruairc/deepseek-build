---
last_mapped_commit: 221a1a591ef3af5f90d59d73ae7899d110f28b37
last_mapped_at: 2026-11-21
---
# Technology Stack

**Analysis Date:** 2026-11-21

## Languages

**Primary:**
- Rust 1.94.0, edition 2024 — the entire product. Workspace root is `Cargo.toml`; the toolchain pin is `rust-toolchain.toml` (`channel = "1.94.0"`, components `rustfmt` and `clippy`). Do not bump the toolchain. `rustc 1.94.0` / `cargo 1.94.0` are the matching host tools.

**Secondary (build, scripts, and wire schemas — not the agent runtime):**
- POSIX `sh` — `install.sh` (release installer; `shellcheck --shell=sh`).
- Bash — policy and smoke scripts such as `scripts/check-egress.sh`, `scripts/egress-check.sh`, `scripts/smoke-test.sh`.
- Python 3 — template encryption (`scripts/encrypt_templates.py`), eval harness (`scripts/evals/harness.py`, `scripts/evals/experiment.py`, `scripts/evals/report.py`, `scripts/evals/validate.py`), and inline helpers inside smoke/install tests. No `pyproject.toml` and no pinned Python version.
- Protocol Buffers — compiled at build time. The live schema is `crates/codegen/xai-grok-tools-api/proto/grok-tools.proto` (consumed by `crates/codegen/xai-grok-tools-api/build.rs` via `prost`). `protoc` must be on `PATH`. This is local tool-wire codegen, not a remote gRPC service.
- TOML and JSON — user config and the embedded model catalog (`crates/codegen/xai-grok-models/default_models.json`).

There is no Node, Go, or JVM application runtime in this repo.

## Runtime

**Environment:**
- Native process. The user-facing binary is `deepseek-build`, package `xai-grok-pager-bin` version `0.0.1-rc5` (`crates/codegen/xai-grok-pager-bin/Cargo.toml`, `default-run = "deepseek-build"`).
- Async runtime is Tokio (`tokio` 1, `features = ["full"]` in the workspace; the binary enables `rt-multi-thread`). Release and dev profiles set `panic = "abort"` in `Cargo.toml`.
- Default allocator on Unix can be jemalloc (`tikv-jemallocator` 0.7) when the `jemalloc` feature is on. It is not the unconditional default.
- TLS is rustls 0.23 with `aws-lc-rs`, except Windows ARM64 which installs the `ring` provider (`crates/codegen/xai-grok-extra-ca/src/lib.rs`). Do not build HTTP clients with native-tls; construct them through `xai_grok_extra_ca`.

**Package Manager:**
- Cargo (Rust 1.94.0). Workspace resolver `"2"` in `Cargo.toml`.
- Lockfile: present (`Cargo.lock`, committed).
- One crates.io git patch: `async-openai` is overridden in `[patch.crates-io]` to `https://github.com/our-forks/async-openai.git` at rev `95b52ebdedf42143083cf3d6f0e0be7c84e9c808`. Do not drop that patch without re-validating Chat Completions types.
- Another git dep: `nucleo` from `https://github.com/helix-editor/nucleo.git` rev `5b74652` (fuzzy search).
- License gate: `deny.toml` (cargo-deny). First-party license is Apache-2.0 (`[workspace.package]`). Vendored libgit2 is the documented GPL-2.0-only-with-exception case; do not add GPL-3/AGPL/LGPL.

## Frameworks

**Core:**
- Tokio 1 — async runtime, process signals, timers. Entry: `crates/codegen/xai-grok-pager-bin/src/main.rs`.
- Ratatui 0.29 + crossterm 0.28 — fullscreen TUI (`crates/codegen/xai-grok-pager`, `crates/codegen/xai-grok-pager-render`). Scrollback-native mode is a separate crate (`crates/codegen/xai-grok-pager-minimal`) wired by fn-pointer from the binary so the pager library does not cycle.
- Clap 4 (`derive`, `env`) — CLI / subcommand dispatch in the binary.
- Reqwest 0.12 (`rustls-tls`, `stream`, `json`, `http2`, `socks`, `default-features = false`) — model HTTP and `web_fetch`. MCP is quarantined on reqwest 0.13 inside `crates/codegen/xai-grok-mcp` because `rmcp` 3.4 requires it. Do not unify those two reqwest majors.
- `async-openai` 0.33 (patched fork, feature `responses` still declared on the workspace dep) — types around the sampler. The live wire protocol is only Chat Completions; see `ApiBackend` in `crates/codegen/xai-grok-sampling-types/src/types.rs`.
- `agent-client-protocol` 0.10.4 (`unstable`) — ACP stdio agent. Local protocol, not a hosted service. Namespace strings are `deepseek-build/*` (for example `deepseek-build/fs/*` in `crates/codegen/xai-grok-workspace-types`).
- `rmcp` 3.4 — MCP client transports (`transport-streamable-http-client-reqwest`, `transport-async-rw`, `auth`). Owned only by `crates/codegen/xai-grok-mcp`.
- Serde 1 + `serde_json` + `toml` 0.9 / `toml_edit` 0.22 — config and wire JSON.
- `tracing` 0.1 + `tracing-subscriber` 0.3 (`env-filter`) — logs. Not an external APM product.
- Rusqlite 0.37 with `bundled` (SQLite ≥ 3.50.2, self-contained) — sessions, memory, FTS. Optional `sqlite-vec` 0.1.7-alpha.2 in `crates/codegen/xai-grok-memory`.
- `git2` 0.20 (vendored libgit2 in several crates) and `gix` 0.83 — local VCS. Plugin installs may clone a user-specified git remote.
- Tree-sitter 0.26 — local parsing. Syntect 5.3 + `two-face` — syntax highlighting in the TUI.
- Axum 0.8 — local listeners only (diagnostics server, MCP OAuth callback). Not a public HTTP API.

**Testing:**
- `cargo test` is the runner. No separate Jest/pytest product suite.
- Wiremock 0.6 and mockito 1 — HTTP fixtures. Sampler wire tests live under `crates/codegen/xai-grok-sampler/tests/`. The shared mock inference server is `crates/codegen/xai-grok-test-support`.
- Insta 1, rstest 0.25, serial_test 3, pretty_assertions 1, criterion 0.6 — snapshots, fixtures, benches.
- PTY e2e: `crates/codegen/xai-grok-pager-pty-harness`. Drive the TUI under a pty; do not assume a human interactive session.

**Build/Dev:**
- `rustfmt` and `clippy` (pinned components). Repo clippy policy is `clippy.toml` (nearest file wins; it duplicates the repo-root bans). Production crates opt into `#![deny(clippy::indexing_slicing)]`. Banned calls: raw `canonicalize` (use `dunce::canonicalize`), raw child spawn (use `xai_tty_utils::ProcessScope`), `std::env::home_dir` / `dirs::home_dir` outside `xai-dirs`.
- Linker wrapper: `scripts/cargo-linker.sh` for `x86_64-unknown-linux-gnu` (`.cargo/config.toml`).
- `cargo-deny` via `deny.toml`.
- Protobuf compiler (`protoc`) for `prost-build` / `xai-proto-build` (`crates/build/xai-proto-build`).
- Release packaging: `.github/workflows/release.yml` builds profile `release-dist` (thin LTO, `codegen-units = 1`) on native runners. Default `--release` is `strip = "symbols"` and is the local artifact, not the dist profile.
- Prompt templates under `crates/codegen/xai-grok-agent/templates/` are XOR-obfuscated into generated Rust by `python3 scripts/encrypt_templates.py`. After any template edit, regenerate and keep `test_encrypted_templates_not_stale` green.

## Key Dependencies

**Critical:**
- `reqwest` 0.12 + `rustls` 0.23 + `xai-grok-extra-ca` — every outbound HTTPS client. Extra roots: `GROK_EXTRA_CA_BUNDLE`, fallback `SSL_CERT_FILE` (`crates/codegen/xai-grok-extra-ca/src/lib.rs`). A bad bundle is skipped; it must not fail client construction.
- `async-openai` (patched) + `eventsource-stream` 0.2 — Chat Completions request/SSE parse in `crates/codegen/xai-grok-sampler/src/client.rs`. Endpoint path is `chat/completions` (joined onto the configured base URL).
- `xai-grok-brand` — user-facing identity. Binary `deepseek-build`, config dir `.deepseek-build`, env prefix `DEEPSEEK_BUILD`, header prefix `x-deepseek-build` (`crates/codegen/xai-grok-brand/src/lib.rs`). Internal crate names stay `xai-grok-*`.
- `xai-grok-models` catalog `default_models.json` — default model `deepseek-v4-pro`, plus `deepseek-flash` and hidden alias `deepseek-v4-flash`. All three set `base_url` to `https://api.deepseek.com` and `api_backend` to `chat_completions`.
- `rmcp` 3.4 + `oauth2` 5 — user-configured MCP only. Do not use `oauth2` to recreate a first-party login against DeepSeek.
- `rusqlite` 0.37 `bundled` — do not switch consumers to the system SQLite. Keep the pin aligned with `crates/codegen/xai-sqlite-journal`.
- `git2` with `vendored-libgit2` where the crate already vendors it — the binary must not depend on a distro libgit2.

**Infrastructure:**
- `clap` 4 — CLI.
- `ratatui` 0.29 / `crossterm` 0.28 / `vt100`-family crates (`alacritty_terminal` 0.26, `portable-pty` 0.9) — TUI and embedded terminals.
- `tokio` 1 — runtime.
- `anyhow` / `thiserror` 2 — errors.
- `blake3`, `sha2`, `ring` 0.17 — hashing and crypto (installer verifies SHA-256; TLS uses rustls).
- `axum` 0.8 — loopback/Unix diag server (`crates/codegen/xai-grok-diag-server`) and MCP OAuth callback.
- `prometheus` 0.14 — in-process counters inside workspace crates (for example `crates/codegen/xai-grok-workspace/src/permission/hub_gate.rs`). There is no remote Prometheus push client in the workspace manifest.
- `jsonschema` 0.30 (`default-features = false`) — local schema checks. Do not enable features that fetch `$ref` over the network.
- `tree-sitter` 0.26, `syntect` 5.3, `pulldown-cmark` 0.13, `htmd` 0.5 — local document/code processing. `web_fetch` converts HTML with `htmd` (`crates/codegen/xai-grok-tools/src/implementations/grok_build/web_fetch/mod.rs`).

**Explicitly absent (do not reintroduce):**
- No `opentelemetry`, `sentry`, `mixpanel`, `aws-sdk`, or `google-cloud-storage` crates in workspace `Cargo.toml`. `xai-grok-telemetry` describes itself as local structured logging (`crates/codegen/xai-grok-telemetry/Cargo.toml`). `track()` in `crates/codegen/xai-grok-telemetry/src/client.rs` does not POST.

## Configuration

**Environment:**
- No `.env` file is part of the repo. Secrets are process env vars or files under the config dir. Never commit or print key values.
- Config home: `$DEEPSEEK_BUILD_HOME`, else `<home>/.deepseek-build` (`crates/codegen/xai-dirs/src/lib.rs`, constant `CONFIG_DIR_NAME` in `crates/codegen/xai-grok-brand/src/lib.rs`). System dir: `/etc/deepseek-build`.
- User file: `$DEEPSEEK_BUILD_HOME/config.toml` (`USER_CONFIG_FILENAME` in `crates/codegen/xai-grok-config/src/loader.rs`). Also `managed_config.toml`, `requirements.toml`, `sandbox.toml`, `trusted_folders.toml`, `trusted-plugins`. Credentials file: `auth.json` under the same home (`crates/codegen/xai-grok-login`). MCP tokens: `mcp_credentials.json` (`crates/codegen/xai-grok-mcp/src/lib.rs`).
- Layer precedence (low to high) is `ConfigLayers` in `crates/codegen/xai-grok-config/src/config_layers.rs`: system managed, user managed, user `config.toml`, then `DEEPSEEK_BUILD_CONFIG` / `DEEPSEEK_BUILD_CONFIG_PATH` overlay, then requirements (user, system, macOS MDM highest). The env overlay is fail-closed for auth, egress, trust, and code-exec tables.
- Project MCP definitions also come from `.grok/config.toml` (`crates/codegen/xai-grok-config/src/mcp_servers.rs`). User config wins over project files.
- Model API key: `DEEPSEEK_API_KEY`, fallback `DEEPSEEK_BUILD_API_KEY` (`crates/codegen/xai-grok-login/src/auth_method.rs`). Catalog entries repeat those names in `env_key` (`default_models.json`).
- Inference base URL default `https://api.deepseek.com` (`XAI_API_BASE_URL_DEFAULT` in `crates/codegen/xai-grok-config/src/endpoints.rs` and `PRODUCTION_ENDPOINTS` in `crates/codegen/xai-grok-env/src/lib.rs`). Override with `DEEPSEEK_BUILD_XAI_API_BASE_URL` or `[endpoints] xai_api_base_url`. Relay/gateway/asset URLs in `xai-grok-env` are loopback placeholders, not cloud hosts.
- Build-time version stamp used by the release workflow is `GROK_VERSION` (see `.github/workflows/release.yml`). That name is internal.

**Build:**
- `Cargo.toml` — workspace members, profiles, shared deps.
- `Cargo.lock` — locked graph.
- `rust-toolchain.toml` — 1.94.0; listed targets are `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`. macOS release builds use the same channel on the CI runner.
- `.cargo/config.toml` — `[build] incremental = false` (this overrides `[profile.dev] incremental = true` in `Cargo.toml`; do not remove the `[build]` setting). Dev/test debug info is `line-tables-only` with third-party `debug = false`.
- `clippy.toml`, `deny.toml`.
- `docker/Dockerfile` — runtime image `debian:bookworm-slim`. It does not compile Rust; CI copies the release tarball binary. Local test image: `docker/Dockerfile.localtest`.

## Platform Requirements

**Development:**
- Rust 1.94.0 via rustup, with `rustfmt` and `clippy`.
- `protoc` on `PATH` for protobuf build scripts.
- Cargo, a C linker, and (on Linux x86_64) `scripts/cargo-linker.sh` (lld preferred, `cc` fallback).
- Python 3 for template encryption and evals.
- Disk: `.cargo/config.toml` disables incremental caches because debug artifacts grow fast. Use one shared `CARGO_TARGET_DIR`. Run `scripts/disk-guard.sh` before a workspace build.
- Linux sandbox tests need bubblewrap (`bwrap`). Host-owned `/run/containerd` can fail `xai-grok-sandbox` e2e; that is an environment limit, not a missing crate. macOS sandbox backend is Seatbelt (`crates/codegen/xai-grok-sandbox/src/command/backend.rs`).

**Production:**
- Native binaries from `.github/workflows/release.yml`: `ubuntu-22.04` (linux gnu x86_64, older glibc), `ubuntu-22.04-arm`, `macos-15`, `macos-15-intel`. Artifacts are `deepseek-build-<ver>-<target>.tar.gz` plus `SHA256SUMS` and `install.sh`, attached to a GitHub release on a `v*` tag. `workflow_dispatch` builds but does not publish a release.
- Install path: `${INSTALL_DIR:-$HOME/.local/bin}` via `install.sh`. Downloads only from `github.com` (see `install.sh` header). Also installs a `dsb` symlink.
- Container: `ghcr.io/<owner>/deepseek-build`, linux/amd64 and linux/arm64, Debian bookworm-slim, non-root uid 1000, entrypoint `deepseek-build` (`docker/Dockerfile`, container job in `.github/workflows/release.yml`). Image tools: `ca-certificates`, `git`, `ripgrep`, `curl`, `openssh-client`, `bash`, `procps`.
- Runtime needs a `DEEPSEEK_API_KEY` (or `DEEPSEEK_BUILD_API_KEY`) and outbound HTTPS to the configured model host (default `api.deepseek.com:443`). No other vendor host is compiled in.

---

*Stack analysis: 2026-11-21*
