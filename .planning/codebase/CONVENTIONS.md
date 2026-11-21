---
last_mapped_commit: 221a1a591ef3af5f90d59d73ae7899d110f28b37
last_mapped_at: 2026-11-21
---
# Coding Conventions

**Analysis Date:** 2026-11-21

## Naming Patterns

**Files:**
- Snake_case Rust sources. A module `foo.rs` owns production code; its tests live either in an inline `#[cfg(test)] mod tests` or, when large, a sibling `foo_tests.rs` pulled in with `#[cfg(test)] #[path = "foo_tests.rs"] mod tests;` (see `crates/codegen/xai-grok-sandbox/src/command/detect.rs`).
- Integration binaries are `tests/<name>.rs`. Historical names often start with `test_` (`crates/codegen/xai-grok-shell/tests/test_leader_stdio_integration.rs`); new unit modules should not add that prefix.
- Package names use hyphens (`xai-grok-sampling-types` in `crates/codegen/xai-grok-sampling-types/Cargo.toml`). Rust paths use underscores (`xai_grok_sampling_types`). Do not rename internal `xai-grok-*` / `xai-*` crates (decision D2 in `AGENTS.md`).

**Functions:**
- `snake_case`, verb-led, specific: `resolve_grok_home_with_source`, `streaming_emits_text_and_reasoning_channels`.
- Test functions name the behavior, not the method: `env_wins_over_os_home` in `crates/codegen/xai-dirs/src/lib.rs`. A `test_` prefix is the exception (`test_encrypted_templates_not_stale` in `crates/codegen/xai-grok-agent/src/prompt/template.rs`).
- Constructors are `new` / `start` / `with_*` builders (`MockToolCall::new(...).result_contains(...)` in `crates/codegen/xai-grok-test-support`).

**Variables:**
- `snake_case`. Numeric literals use underscores: `256_000`, `128_000` (`crates/codegen/xai-grok-sampler/tests/chat_completions_wire.rs`).
- Environment and brand names are constants, never string literals at call sites. User-facing names come from `crates/codegen/xai-grok-brand/src/lib.rs` (`NAME`, `CONFIG_DIR_NAME`, `ENV_PREFIX`, `HOME_ENV_VAR`).

**Types:**
- `PascalCase` structs and enums. Error enums end in `Error` (`SamplingError`, `StoreError`).
- Crate-local result aliases: `pub type Result<T> = std::result::Result<T, StoreError>;` in `crates/codegen/xai-grok-dashboard-store/src/error.rs`. Do not shadow `std::result::Result` inside a `Deserialize` impl; spell `std::result::Result` there (`SentCredential` in `crates/codegen/xai-grok-sampling-types/src/error.rs`).
- Enum variants are `PascalCase`. Serde rename is explicit and local to the wire:
  - TOML / internal JSON: `#[serde(rename_all = "snake_case")]` (`EmptyReason` in `crates/codegen/xai-grok-sampling-types/src/error.rs`).
  - ACP / protocol JSON that already uses camelCase: `#[serde(rename_all = "camelCase")]` (`crates/codegen/xai-prompt-queue/src/types.rs`).
  - Mode strings: `#[serde(rename_all = "lowercase")]` (`crates/codegen/xai-grok-config-types/src/permission.rs`).
- Unknown wire values degrade to a closed variant (`SentCredential::Unknown`) instead of failing the whole payload. Copy that pattern for any externally tagged enum a newer peer might extend.

**Constants:**
- `SCREAMING_SNAKE_CASE` for `pub const` (`COMPACT_SYSTEM_PROMPT`, `REASONING_PLACEHOLDER`).
- Process-lifetime caches are `static` `OnceLock` (`DEEPSEEK_BUILD_HOME` in `crates/codegen/xai-dirs/src/lib.rs`).

## Code Style

**Formatting:**
- Tool: `rustfmt` from the pinned toolchain in `rust-toolchain.toml` (Rust **1.94.0**, components `rustfmt` and `clippy`).
- Workspace edition is **2024** (`[workspace.package] edition = "2024"` in root `Cargo.toml`). Per-crate manifests set `edition.workspace = true`.
- `rustfmt.toml` sets only `use_field_init_shorthand = true`. Everything else is rustfmt default (including `max_width = 100`). Run `cargo fmt --all --check` before considering a change done. Do not hand-wrap against rustfmt.
- Struct update uses field init shorthand (`..Default::default()` is the usual override style; see `SamplerConfig` construction in `crates/codegen/xai-grok-sampler/tests/chat_completions_wire.rs`).

**Linting:**
- `clippy` via the same toolchain. Workspace allows live in root `Cargo.toml` under `[workspace.lints.clippy]`: `doc_lazy_continuation`, `doc_overindented_list_items`, `needless_lifetimes`, `single_range_in_vec_init`, `too_many_arguments`, `uninlined_format_args`, `useless_format`. Crates opt in with:

```toml
[lints]
workspace = true
```

  Example: `crates/codegen/xai-grok-sampling-types/Cargo.toml`. Without that block, `cargo clippy --workspace -- -D warnings` is stricter than CI-style allows (comment in `crates/codegen/xai-grok-pager/Cargo.toml`). New crates must include it.
- Production crates that ship indexing-sensitive code start with `#![deny(clippy::indexing_slicing)]` (`crates/codegen/xai-grok-pager-bin/src/main.rs`, `crates/codegen/xai-grok-sampling-types/src/lib.rs`, and most other `lib.rs` files). Prefer `get`, iterators, or `split_at` checked APIs. Tests may index: `allow-indexing-slicing-in-tests = true` in root `clippy.toml`. Inline `#[cfg(test)]` modules that clippy sees as production (Bazel aspect note in `clippy.toml`) must not index; put those tests in a `*_tests.rs` sibling or an integration binary.
- A few crates also deny panic in non-test code:

```rust
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
```

  That is the rule in `crates/codegen/xai-grok-lifecycle/src/lib.rs`. Do not add `unwrap` / `expect` / `panic` to that crate's production paths. Other crates still use `unwrap_or_else` and occasional `expect` on invariants; match the crate you are editing.
- Method bans in root `clippy.toml` (`disallowed-methods`). Do not call these; the replacement is mandatory:
  - `std::fs::canonicalize`, `Path::canonicalize`, `tokio::fs::canonicalize` → `dunce::canonicalize` (or `spawn_blocking` + `dunce` off the async runtime).
  - `std::env::home_dir` and `dirs::home_dir` → `xai_dirs::home_dir` or `xai_grok_config` grok-home helpers. The one sanctioned `std::env::home_dir` call is `xai_dirs::home_dir` in `crates/codegen/xai-dirs/src/lib.rs`, marked `#[allow(deprecated, clippy::disallowed_methods)]`.
  - `std::process::Command::spawn`, `tokio::process::Command::spawn`, `portable_pty::SlavePty::spawn_command` → `xai_tty_utils::ProcessScope::enroll` (or `enroll_terminal_pid` for a PTY). A test-only child that is waited on and killed on timeout may `#[allow(clippy::disallowed_methods)]` with a reason, as in `crates/codegen/xai-grok-pager-bin/tests/branding_surface.rs`.
  - `reqwest::Client::new`, `ClientBuilder::build`, and the blocking equivalents → `xai_grok_extra_ca::build_reqwest_client` / `build_blocking_reqwest_client`. Localhost-only or test clients allow with a reason.
- `large-error-threshold = 256` in `clippy.toml`. Keep error types small; box large payloads.
- Clippy uses the nearest `clippy.toml` and does not merge. There is one file, at the repo root. If a nested `clippy.toml` is added, it must duplicate these bans (the header comment in `clippy.toml` says so).
- Do not bump the toolchain. `rust-toolchain.toml` pins `1.94.0` and the linux gnu targets.

**Profiles (do not raise debug info):**
- `.cargo/config.toml` sets `[build] incremental = false`. That overrides `[profile.dev] incremental = true` in root `Cargo.toml`. Leave both as they are.
- `[profile.dev]` and `[profile.test]` use `debug = "line-tables-only"`; `[profile.dev.package."*"]` sets `debug = false`. Do not raise these without a disk plan (`AGENTS.md` §8).

## Import Organization

**Order:**
1. `std` / `core` / `alloc`
2. blank line
3. external crates (`anyhow`, `serde`, `tracing`, `wiremock`, …)
4. blank line
5. workspace crates (`xai_grok_brand`, `xai_dirs`, …) then `crate::`

Observed in `crates/codegen/xai-grok-sampling-types/src/error.rs` and `crates/codegen/xai-grok-workspace/src/worktree/create_root.rs`. `rustfmt` does not regroup imports (`group_imports` is unset), so keep this order by hand when adding a use.

Inside `#[cfg(test)] mod tests`, import `super::*` first, then assertion helpers (`pretty_assertions::assert_eq`), then `std` if needed (`crates/codegen/xai-dirs/src/lib.rs`).

**Path Aliases:**
- None. No `crate`-level path alias map. Use crate names as declared.
- Prefer `use anyhow::Context as _;` when only the trait method is needed (`crates/codegen/xai-grok-pager/src/app/mod.rs`), and `use anyhow::Context;` when the name should be visible (`crates/codegen/xai-grok-workspace/src/worktree/create_root.rs`).

**Visibility imports:**
- Re-export the public surface from `lib.rs` with `pub use`. Downstream crates should depend on the facade, not deep modules, when a `pub use` already exists (`crates/codegen/xai-grok-sampling-types/src/lib.rs`).
- `pub use xai_grok_login::*` style globs exist (`crates/codegen/xai-grok-shell/src/lib.rs` `auth` module). Do not add new glob re-exports unless the module is an explicit compatibility facade.

## Error Handling

**Patterns:**
- Library boundaries return a typed `thiserror::Error` enum. Each variant has an `#[error("...")]` that a user or log line can read without the source chain. Attach sources with `#[from]` / `#[source]`. Model: `StoreError` in `crates/codegen/xai-grok-dashboard-store/src/error.rs`.
- Composition and IO orchestration return `anyhow::Result` and add context at the await/join point:

```rust
pub(crate) async fn resolve_create_source_layout(
    source: &Path,
    grove_requested: bool,
) -> anyhow::Result<CreateSourceLayout> {
    // ...
    .await
    .context("create source layout join failed")?
}
```

  From `crates/codegen/xai-grok-workspace/src/worktree/create_root.rs`. Context strings name the operation that failed, not the callee.
- Distinguish "missing is success" from "missing is an error". `remove_member` is idempotent and does not return `MemberNotFound`; `update_member_metadata` does (`StoreError` docs). Match that split instead of using one error for both.
- Fail closed on auth and path containment. A 401 with no credential header is `SentCredential::Missing`, not evidence the key is bad (`crates/codegen/xai-grok-sampling-types/src/error.rs`). `dunce::canonicalize` keeps the `\\?\` form when it cannot simplify, so containment checks fail closed (`clippy.toml`).
- Do not log, persist, or include API keys in error `Display` text. `scripts/key-leak-check.sh` scans logs and saved sessions.
- `expect("...")` in tests is fine and should say why the invariant holds (`SamplingClient::new(...).expect("client constructs")` in `crates/codegen/xai-grok-sampler/tests/chat_completions_wire.rs`). In production, prefer `?` or a typed error. `unwrap_or_else` for a documented fallback is the usual non-panicking default (`grok_home_in` in `crates/codegen/xai-dirs/src/lib.rs`).

## Logging

**Framework:** `tracing` (workspace dependency). `log` is also a workspace dependency but new code uses `tracing`.

**Patterns:**
- Structured fields first, message last:

```rust
tracing::warn!(path = %home.display(), %err, "failed to create the deepseek-build home directory");
```

  From `crates/codegen/xai-dirs/src/lib.rs`. Use `%` for `Display`, `?` for `Debug`.
- `warn` for a degraded but continuing path (failed mkdir, unmatched tool allowlist). `debug` for expected absence (`ffmpeg not available`). `info` for a user-visible skip (`skipping browser open`). Examples in `crates/codegen/xai-grok-pager-render/src/link_opener.rs`.
- Redact URLs and credentials before they become fields (`url = %redacted` in `link_opener.rs`). Do not attach `DEEPSEEK_API_KEY` or Authorization headers to spans.
- `fastrace::trace` macros expand into `format!("{}", param)`. The workspace allows `useless_format` for that reason (`Cargo.toml`). Do not "fix" those expansions by hand.
- Shell timing helpers `teprintln!` / `timed` / `tprintln!` are re-exported from `xai_tracing_macros` in `crates/codegen/xai-grok-shell/src/lib.rs`. Use them for existing shell instrumentation; do not invent a second timer macro.

## Comments

**When to Comment:**
- Crate and module rustdoc (`//!`) states what the module owns, what it does not own, and the invariants. Follow `crates/codegen/xai-grok-lifecycle/src/lib.rs` (one-trigger-at-a-time, deadline, caps) and `crates/codegen/xai-dirs/src/lib.rs` (which getter to call).
- Public functions get a `///` line when the name is not enough, especially when several near-duplicate getters exist (`grok_home` vs `user_grok_home` vs `resolve_grok_home`).
- Comments explain constraints the type system cannot: Windows `\\?\` paths, process-global vs thread-local (`INJECT_LOCK` comment in `create_root.rs`), fail-closed auth.
- Do not narrate the next statement. `TODO` is acceptable when it names the follow-up (`clippy.toml` tonic issue, `xai-dirs` getter collapse). Do not leave a TODO that hides a live network call.

**JSDoc/TSDoc:**
- Not applicable. Rust doc comments only. `cargo` rustdoc is not a release gate. Prost-generated doc lists trip clippy; those two doc lints stay allowed at the workspace level.

## Function Design

**Size:**
- No line cap is enforced (`too_many_arguments` is allowed). Still keep a function to one decision. When a test module outgrows the production file, move it to `*_tests.rs` rather than appending another thousand lines to `lib.rs`.
- Async functions that hop threads take owned data (`PathBuf`) into `spawn_blocking`, not borrowed `&Path` (`resolve_create_source_layout`).

**Parameters:**
- Borrow for read-only input (`&Path`, `&str`). Own when the function stores or sends across a thread.
- Boolean flags that change a probe (`grove_requested: bool`) are acceptable when there are two modes; do not add a third boolean. Use an enum.
- Builder setters are chainable and return `Self` on test and config types (`Conversation`, `MockToolCall`, `StatusFailure` in `crates/codegen/xai-grok-test-support`).

**Return Values:**
- `Option` when absence is a normal answer (`resolve_grok_home() -> Option<PathBuf>`).
- `Result` when the caller must handle failure. Never return a stringly error.
- `#[must_use]` on guards and spawn results that are wrong to drop (documented for `SpawnedAgent` in `crates/codegen/xai-grok-test-support/README.md`). Apply the same attribute to new RAII guards.

## Module Design

**Exports:**
- `lib.rs` lists `pub mod` then `pub use` of the stable surface. Downstream code imports from the crate root when a re-export exists.
- Default visibility for helpers is private. Cross-module helpers inside a crate are `pub(crate)`.
- Test-only seams are `#[cfg(test)]` modules (`test_support` in `crates/codegen/xai-grok-shell/src/lib.rs`) or a `test-support` cargo feature that is **off** by default (`crates/codegen/xai-grok-shell/Cargo.toml`). Do not put test seams in the default feature set. `config-docs` is also off by default; hermetic CI/Bazel turns it on via `default-bazel`, not `default`.
- `#[doc(hidden)]` marks modules that must be public for macros or benches but are not API (`waterfall` in `xai-grok-shell`).
- User-visible strings, config directory names, and `DEEPSEEK_BUILD_*` env names go through `xai_grok_brand`. Internal `GROK_*` identifiers that are not user-facing stay (see `docs/INTERNAL-GROK-ENV-NAMES.md`). Do not reintroduce legacy `GROK_*` env fallbacks; they were removed and tests pin that.

**Barrel Files:**
- `lib.rs` is the barrel. Do not add `mod.rs` re-export shims that only forward one type unless the directory already uses `mod.rs` (existing directories such as `crates/codegen/xai-grok-sandbox/src/command/` keep `mod.rs`).
- Prompt templates are not ordinary modules. Sources live in `crates/codegen/xai-grok-agent/templates/*.md` and are XOR-obfuscated into `src/prompt/prompt_encrypted.rs`. After any template edit run `python3 scripts/encrypt_templates.py`. The staleness check is `prompt::template::tests::test_encrypted_templates_not_stale`.

**Config docs:**
- `crates/codegen/xai-grok-pager/docs/user-guide/26-config-reference.md` is hand-edited. When a key is added or removed from `FEATURES`, `UNMIRRORED_BOOLEAN_FEATURES`, or `KNOWN_MCP_SERVER_FIELDS`, update that page. Removed-flow keys go in `UNDOCUMENTED_REMOVED_FLOW_KEYS`, not the page (`crates/codegen/xai-grok-shell/AGENTS.md`).

**Platform cfgs:**
- Unix-only dependencies use `[target.'cfg(unix)'.dependencies]`; macOS and Windows the same way (`crates/codegen/xai-grok-pager/Cargo.toml`). Gate the code with the same `cfg`, do not `#[cfg]` inside a function that also compiles the other OS's types unless the type itself is cfg'd (`registered_backend` in `crates/codegen/xai-grok-sandbox/src/command/detect.rs`).

---

*Convention analysis: 2026-11-21*
