---
last_mapped_commit: 221a1a591ef3af5f90d59d73ae7899d110f28b37
last_mapped_at: 2026-11-21
---
# Testing Patterns

**Analysis Date:** 2026-11-21

## Test Framework

**Runner:**
- Cargo's built-in libtest, Rust **1.94.0** (`rust-toolchain.toml`). No `cargo-nextest` config, no `tarpaulin`, no `cargo-llvm-cov` config in the tree.
- Async tests use `tokio::test` (dev-dependency `tokio` with `test-util` where the crate needs it, e.g. `crates/codegen/xai-grok-pager/Cargo.toml`).
- Property testing is not used (`proptest` / `quickcheck` have no call sites). Table tests use `rstest` 0.25 (workspace dependency).
- Snapshot tests use `insta` 1 (workspace dependency), inline in the module that renders the text.
- Benchmarks use `criterion` 0.6 with `harness = false` (`[[bench]]` in `crates/codegen/xai-grok-pager/Cargo.toml`, `crates/codegen/xai-grok-shell/Cargo.toml`, `crates/codegen/xai-grok-markdown/Cargo.toml`, `crates/codegen/xai-fsnotify/Cargo.toml`, `crates/codegen/xai-ratatui-inline/Cargo.toml`).
- Fuzzing is cargo-fuzz on nightly, only for markdown: `crates/codegen/xai-grok-markdown/fuzz/` (`render_all` target). It is not part of `cargo test`.
- Concurrency models use `loom` 0.7 in `crates/codegen/xai-grok-login/src/manager/lock/flock_wait_loom_tests.rs`. Do not mix `loom::sync` into production paths; the production file cfg-switches the import (`flock_wait.rs`).

**Assertion Library:**
- `pretty_assertions::assert_eq` (and `assert_ne`) in crates that declare the dev-dependency (`crates/codegen/xai-dirs/src/lib.rs`, pager, shell, dashboard-store). Use it for any multi-line or struct equality. Plain `assert_eq!` remains in older modules; new tests in those crates should import `pretty_assertions`.
- `assert_matches` 1.5 for enum shape (`crates/codegen/xai-grok-sampling-types/src/conversation/chat_completions_tests.rs`).
- `insta::assert_snapshot!` for rendered TUI/diff text (`crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/edit.rs`).

**Run Commands:**
```bash
# Routine: one crate. Do this during a change, not a full workspace run.
cargo test -p xai-grok-sampling-types --lib
cargo test -p xai-grok-sampler --lib stream::chat_completions
cargo test -p xai-grok-sampler --test chat_completions_wire

# Filter
cargo test -p xai-dirs --lib env_wins_over_os_home

# Opt-in ignored tests (PTY, prebuilt binary, perf)
cargo test -p xai-grok-pager-pty-harness --test pty_auto_mode -- --ignored

# Feature-gated docs sync
cargo test -p xai-grok-shell --features config-docs --lib config_docs

# Prompt template staleness (after editing templates/)
python3 scripts/encrypt_templates.py
cargo test -p xai-grok-agent --lib prompt::template::tests::test_encrypted_templates_not_stale

# Branding surface of the real binary
cargo test -p xai-grok-pager-bin --test branding_surface

# Format / clippy (clippy must be 0 errors; the tree carries pre-existing warnings)
cargo fmt --all --check
cargo clippy -p xai-grok-pager-bin --all-targets -- -D warnings

# Final gate only. Last recorded full run (AGENTS.md, 2026-10-09): 31939 passed,
# 2 failed, 388 ignored, 307 targets. The 2 failures are host bwrap
# (xai-grok-sandbox deny_paths_e2e), not logic. There is no cargo test job in
# .github/workflows/release.yml — release CI builds artifacts only.
cargo test --workspace --no-fail-fast

# Non-cargo gates (no coverage tool; these are the product gates)
scripts/check-egress.sh --strict          # HARD + branding, no key
scripts/key-leak-check.sh                 # fake key must not land in logs/sessions
sh scripts/test-install.sh                # 39 installer assertions, loopback only
scripts/smoke-test.sh                     # live DeepSeek; needs DEEPSEEK_API_KEY
```

There is no `cargo test` watch mode configured. Re-run the crate filter.

**Operational constraints (do not skip):**
- Before any workspace build or `cargo test --workspace`, run `scripts/disk-guard.sh`. It exits non-zero below 150 GB free. Stop if it fails.
- `.cargo/config.toml` sets `RUST_MIN_STACK = "67108864"` because `SessionActor` turn futures overflow libtest's default 2 MiB stack. Do not unset it when running shell/session tests.
- `[profile.test]` debug info is `line-tables-only` (root `Cargo.toml`). Do not raise it.
- Use one shared `CARGO_TARGET_DIR` (the main checkout's `target/`). A second target dir will fill the disk.
- `protoc` must be on `PATH` for crates that build prost code.

## Test File Organization

**Location:**
- Unit tests are co-located. Small suites are `#[cfg(test)] mod tests` at the bottom of the production file (`crates/codegen/xai-dirs/src/lib.rs`, `crates/codegen/xai-grok-brand/src/lib.rs`).
- Large suites are a sibling file included by path so the production file stays readable:

```rust
#[cfg(test)]
#[path = "detect_tests.rs"]
mod tests;
```

  From `crates/codegen/xai-grok-sandbox/src/command/detect.rs`. The sibling is not a separate cargo target; it compiles only as part of the parent module. Follow this instead of a `tests/` directory for crate-private unit tests.
- Integration tests are `crates/<crate>/tests/*.rs`. Each file is its own binary unless `[[test]]` overrides `path`. Shell process-isolated suites stay as implicit `tests/*.rs` binaries (`crates/codegen/xai-grok-shell/Cargo.toml`). The comment there mentioning `tests/all.rs` / `mod suite` does not match the tree: there is no `tests/all.rs`. Add a new integration test as `tests/<behavior>.rs`, and set `required-features = ["test-support"]` when it needs the test seams.
- PTY / spawn-only pager tests live in `crates/codegen/xai-grok-pager-pty-harness/tests/`, one `[[test]]` per family so they can be scheduled separately (`pty_e2e_smoke`, `pty_e2e_queue`, `scripted_scenarios`, …). They are `#[ignore]`d for ordinary `cargo test`.
- Shared doubles live in `crates/codegen/xai-grok-test-support` (mock inference server, ACP client, `TestSandbox`, `TestProcess`, managed-gateway and storage-upload doubles). Shared git/runfiles helpers live in `crates/common/xai-test-utils`. Do not copy a second mock server into a product crate.
- Dropped tests are recorded in `docs/DROPPED-TESTS.md`. If a behavior was intentionally removed, delete the test and add a ledger row. Do not `#[ignore]` a test whose behavior is gone.

**Naming:**
- Unit module: `tests` (the `#[cfg(test)]` module name), file `*_tests.rs`.
- Test function: snake_case behavior sentence (`empty_env_falls_through_to_os_home`).
- Integration binary: snake_case topic (`chat_completions_wire.rs`, `branding_surface.rs`). A `test_` prefix on the filename is legacy; do not add it to new files.
- `rstest` cases use `#[case::case_name(...)]` so the failure names the row (`crates/codegen/xai-grok-shell/src/agent/config_tests.rs`).

**Structure:**
```
crates/codegen/<crate>/
├── src/
│   ├── foo.rs                  # production + optional inline tests
│   └── foo_tests.rs            # #[path] unit tests
├── tests/
│   └── behavior.rs             # integration binary (external crate API only)
├── benches/                    # criterion, harness = false
└── Cargo.toml                  # [dev-dependencies], [[test]], [[bench]]
crates/codegen/xai-grok-test-support/   # shared doubles
crates/common/xai-test-utils/           # git, runfiles, tracing capture
scripts/                                # egress, key-leak, installer, live smoke
```

Integration tests cannot see `pub(crate)` items. If a test needs a crate-private function, keep it as a `#[cfg(test)]` unit test, or expose a narrow `#[cfg(feature = "test-support")]` seam. Do not make the function `pub` only for the test.

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn env_wins_over_os_home() {
        let resolved =
            resolve_grok_home_from(Some(OsStr::new("/custom/home")), Some(Path::new("/home/u")));
        assert_eq!(
            resolved,
            Some((PathBuf::from("/custom/home"), GrokHomeSource::EnvOverride))
        );
    }
}
```

From `crates/codegen/xai-dirs/src/lib.rs`. Async wire tests follow the same shape with `#[tokio::test]` and a local fixture (`mock_server`, `client`, `sse`) in `crates/codegen/xai-grok-sampler/tests/chat_completions_wire.rs`.

**Patterns:**
- Arrange fixtures as functions in the test module (`fn client(server: &MockServer) -> SamplingClient`), not as a global. Constants for shared literals (`const MODEL: &str = "deepseek-v4-pro"`).
- One assertion theme per test. A wire test may assert several fields of one response; do not also assert an unrelated error path in the same function.
- `expect("why this cannot fail")` on setup. `assert_eq!` / `assert_matches!` on the behavior. Panic messages in branding tests tell the operator how to build the missing binary (`crates/codegen/xai-grok-pager-bin/tests/branding_surface.rs`).
- Table-driven rows use `rstest`, not a hand-rolled loop, when each row should fail independently:

```rust
#[rstest::rstest]
#[case::neither(None, &[], None)]
fn context_window_choices_resolve(
    #[case] context_window: Option<u64>,
    #[case] listed: &[u64],
    #[case] expected: Option<(u64, Vec<u64>)>,
) { /* ... */ }
```

From `crates/codegen/xai-grok-shell/src/agent/config_tests.rs`.

- No `Drop` glue that panics after a `?` already failed, if the panic would hide the error. `MockInferenceServer` drop checks script violations; a test that returns `Result` must call `finish_scripts()` before an early return (`crates/codegen/xai-grok-test-support/README.md`).

**Teardown:**
- Temp dirs: `tempfile::tempdir()` and let it drop (`xai-dirs` tests). Do not `remove_dir_all` a path outside the temp dir.
- Subprocesses: `TestProcess` / `TestProcessTree` own the process group and kill on drop (`crates/codegen/xai-grok-test-support`). A raw `Command` in a test must be waited on and killed on timeout (pattern in `branding_surface.rs`). Never `Command::spawn` in production code.
- Mock servers shut down on drop (`MockInferenceServer`, `wiremock::MockServer`). Keep the server binding alive until assertions finish.

## Mocking

**Framework:** No `mockall`. Doubles are hand-written types plus HTTP servers.

**Patterns:**
```rust
async fn mock_server(body: String, content_type: &str) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", content_type)
                .set_body_raw(body, content_type),
        )
        .mount(&server)
        .await;
    server
}
```

From `crates/codegen/xai-grok-sampler/tests/chat_completions_wire.rs`. Assert the bytes the mock received, not only the in-process struct. The module docs say so; follow that for any new adapter test.

Scripted multi-turn agent tests use `MockInferenceServer` + `Conversation`, not wiremock:

```rust
server.set_conversations(vec![
    Conversation::nth(1)
        .calls([
            MockToolCall::new(Tool::Read, json!({ "target_file": "a.rs" }))
                .result_contains("fn main"),
        ])
        .reply("done"),
]);
```

From `crates/codegen/xai-grok-test-support/README.md`. Read that README before adding a mock capability; it is the API contract and must be updated in the same change as `src/`.

`mockito` 1 is used for small HTTP middleware tests (`crates/codegen/xai-grok-auth/src/retry_middleware.rs`). Prefer `wiremock` for new sampler/HTTP wire tests; prefer `MockInferenceServer` when the system under test is the agent, shell, or pager.

**What to Mock:**
- The model provider. Never call `api.deepseek.com` from `cargo test`. Live calls belong in `scripts/smoke-test.sh`.
- Managed-config and storage upload endpoints, via `MockInferenceServer`'s managed-gateway and storage routes and `GatedUploadProxy` in `xai-grok-test-support`.
- Timeouts and stalls via the scripted failure setters (`.stall`, `.refuse`, `.cut`, `.fail_stream`, `.drop_connection`), not `sleep`.

**What NOT to Mock:**
- Filesystem and env for process tests. Use `TestSandbox` (`crates/codegen/xai-grok-test-support/src/sandbox.rs` via the crate root). It owns a temp root, isolated `HOME` / `USERPROFILE`, loopback `NO_PROXY`, and telemetry pins (`DISABLE_TELEMETRY=1`, `GROK_TRACE_UPLOAD=false`, `GROK_INSTRUMENTATION=disabled`) so a test cannot reach production. Do not `std::env::set_var` on the process for a value another test reads.
- The installed binary's help/version text. `branding_surface` runs the real `deepseek-build` binary (`CARGO_BIN_EXE_deepseek-build`, then a sibling of the test exe).
- Permission and sandbox policy decisions that are the unit under test. Drive them through the real manager with a temp workspace.
- `dunce` / path canonicalization. Assert the real `dunce::canonicalize` result (`empty_env_falls_through_to_os_home`).

**Process-global state:**
- If a test must mutate a process-global (env, a `static` inject), serialize it:

```rust
#[serial(bwrap_env)]
```

  Used throughout `crates/codegen/xai-grok-sandbox/src/lib.rs`. `#[serial_test::serial]` without a key is a global lock (`crates/codegen/xai-grok-pager-render/src/terminal/image/tests.rs`). Prefer a `Mutex` inject that the production code only sees under `#[cfg(test)]` (`INJECT_STATE` in `crates/codegen/xai-grok-workspace/src/worktree/create_root.rs`) over env mutation. `TestSandbox` docs forbid new process-global env mutation.

## Fixtures and Factories

**Test Data:**
```rust
fn sse(events: &[Value]) -> String {
    let mut body = String::new();
    for event in events {
        body.push_str("data: ");
        body.push_str(&event.to_string());
        body.push_str("\n\n");
    }
    body.push_str("data: [DONE]\n\n");
    body
}
```

From `crates/codegen/xai-grok-sampler/tests/chat_completions_wire.rs`. SSE fixtures must include the `[DONE]` sentinel. Tool-call arguments in fixtures are JSON strings, matching the wire (`"arguments": "{\"path\":\"a\"}"`).

ACP values go through `acp_fixtures` (`session_id`, `tool_call_id`, `text_block`, …) so a protocol bump touches one module. Do not construct ACP structs by hand in a new test.

Git repos: `xai_test_utils::git::init_git_repo` / `git_commit_all`, or `TestSandbox::builder().git()`. `git_workdir()` returns a git-initialized sandbox (`xai-grok-test-support` `env` module).

**Location:**
- Inline fixture fns next to the tests that use them.
- Shared fixtures: `crates/codegen/xai-grok-test-support/src/` and `crates/common/xai-test-utils/src/`.
- Checked-in wire samples: `crates/codegen/xai-grok-workspace/fixtures/sandbox_wire/`, `crates/common/xai-tool-protocol/fixtures/`.
- Fuzz seeds: `crates/codegen/xai-grok-markdown/fuzz/seeds/` (committed). `fuzz/corpus/` is generated and gitignored.
- Installed-doc and config fixtures used by pager tests come from the crate under test, not from `~/.deepseek-build`.

## Coverage

**Requirements:** None enforced. No coverage threshold, no CI coverage upload, no `llvm-cov` or `tarpaulin` config in the repo.

**View Coverage:**
```bash
# Not configured. Do not add a coverage tool as part of an unrelated change.
# The gates that are actually enforced locally:
scripts/check-egress.sh --strict
cargo test -p <crate>
cargo fmt --all --check
```

Ignored tests do not count as coverage. A `#[ignore]` must name why and how to opt in:

```rust
#[ignore = "spawns real pager PTY; run with cargo test -- --ignored"]
```

From `crates/codegen/xai-grok-pager-pty-harness/tests/pty_auto_mode.rs`. Do not ignore a test to hide a failure. Host-only sandbox failures (`bwrap` against root-owned `/run/containerd`) are the known exception recorded in `AGENTS.md` / `KNOWN-ISSUES.md`; do not paper over them with `#[ignore]` unless the ledger says to.

## Test Types

**Unit Tests:**
- Pure functions and parsers. No network, no home directory, no clock sleeps. Inject inputs (`resolve_grok_home_from(env, os_home)` is the testable core; `resolve_grok_home` reads the process env).
- Scope: one module. Run with `cargo test -p <crate> --lib`.
- Crate-private items are in scope via `use super::*;`.

**Integration Tests:**
- `tests/*.rs` binaries against the crate's public API, often with `wiremock` or `xai-grok-test-support`.
- Sampler wire suite: `crates/codegen/xai-grok-sampler/tests/` (`chat_completions_wire.rs`, `shared_http_wire.rs`, `request_query_and_headers.rs`, `test_actor.rs`, …). These pin streaming, `reasoning_content` replay, tool-call JSON repair, and partial-stream failure.
- Shell e2e: `crates/codegen/xai-grok-shell/tests/`. Suites that need in-process seams declare `required-features = ["test-support"]` in `Cargo.toml`. Run them with `--features test-support` if cargo does not enable it automatically (it does for `required-features` when you name the test target).
- Branding: `crates/codegen/xai-grok-pager-bin/tests/branding_surface.rs` fails if help, version, completions, or keyless headless output prints `grok`, `xAI`, or `x.ai` outside the allowlist.
- Config reference sync: `crates/codegen/xai-grok-shell/src/config_docs/` is compiled only with `--features config-docs` (`src/lib.rs`). Update `crates/codegen/xai-grok-pager/docs/user-guide/26-config-reference.md` in the same change as a registry edit.
- Installer: `sh scripts/test-install.sh` (POSIX sh, fake release on 127.0.0.1, 39 assertions). Not a cargo test.
- Egress grep gate: `scripts/check-egress.sh` / `--strict`. A change that adds an outbound destination is a stop-and-ask, not a test to update into green.

**E2E Tests:**
- PTY harness: `crates/codegen/xai-grok-pager-pty-harness`. Cases are `#[ignore]` because they spawn the real pager. Opt in with `--ignored`. CI/Bazel historically supplied `PAGER_BINARY`; this tree's GitHub workflow does not run them.
- Live model e2e is `scripts/smoke-test.sh` (needs `DEEPSEEK_API_KEY` or `DEEPSEEK_BUILD_API_KEY`; the script must not print the key). Runtime egress is `scripts/egress-check.sh` (LD_PRELOAD connect/send logger; `strace` is not required).
- Eval harness (not cargo): `scripts/evals/` — 21 auto-checked tasks, JSONL metrics (`scripts/README.md`).
- Markdown fuzz: `cargo +nightly fuzz run render_all` from `crates/codegen/xai-grok-markdown` (`fuzz/README.md`). Nightly only. Not the default toolchain.

## Common Patterns

**Async Testing:**
```rust
#[tokio::test]
async fn streaming_emits_text_and_reasoning_channels() {
    let server = mock_server(
        sse(&[
            chunk(json!({"reasoning_content": "thinking..."}), json!(null)),
            chunk(json!({"content": "hello"}), json!(null)),
            chunk(json!({}), json!("stop")),
        ]),
        "text/event-stream",
    )
    .await;
    let response = collect(&client(&server), request).await;
    // assert channels, then assert the mock received the expected body
}
```

Shape from `crates/codegen/xai-grok-sampler/tests/chat_completions_wire.rs`. Use `MockServer::start().await`, not a fixed port. Timeouts in assertions go through the harness budget (`xai_grok_test_support::scaled(base)`, which applies `GROK_TEST_TIMEOUT_SCALE`), not a bare `tokio::time::sleep`, so a slow CI machine scales the same way.

ACP stdio clients must run inside a tokio `LocalSet` (`run_in_local_set` in `xai-grok-test-support`). The connection is not `Send`.

**Error Testing:**
```rust
assert_matches!(
    parsed,
    Err(SamplingError::/* variant */)
);
```

Use `assert_matches` (or `matches!` inside `assert!`) for error variants. For `anyhow` / IO boundaries, assert `err.to_string()` contains the `.context("...")` text you added, so a refactor that drops the context fails the test. Do not snapshot a full `Debug` of an `anyhow` error; it includes host paths.

Auth and key tests use a fake key and an isolated `HOME` (`scripts/key-leak-check.sh`, `TestSandbox`). Never put a real `DEEPSEEK_API_KEY` in a fixture, assertion, or committed file.

**Ignored / host-gated tests:**
- PTY and prebuilt-binary tests: `#[ignore = "... run with cargo test -- --ignored"]`.
- Perf regressions: `#[ignore = "perf regression tier; ... --ignored --nocapture"]` (`crates/codegen/xai-grok-shell/tests/subagent_bootstrap_regression.rs`).
- Sandbox bwrap tests are not ignored; they fail on hosts where `/run/containerd` is root-owned. Treat that as an environment failure, not a cue to weaken the assertion.

**Feature and cfg gates:**
- `test-support` is how product crates expose test seams (`WorkspaceOps::for_test` and similar). Depend on it from `[dev-dependencies]` with `features = ["test-support"]` (`crates/codegen/xai-grok-pager/Cargo.toml`). Do not enable it in `[dependencies]` default features.
- Platform-specific tests sit behind the same `cfg` as the code (`#[cfg(target_os = "macos")]` seatbelt tests under `crates/codegen/xai-grok-sandbox/src/command/backend/seatbelt_tests.rs`).
- `dhat-heap` is an opt-in allocator feature for heap profiles (`xai-grok-shell`). Do not turn it on for ordinary tests.

**Docs and generated artifacts that tests pin:**
- After editing `crates/codegen/xai-grok-agent/templates/`, regenerate `src/prompt/prompt_encrypted.rs` with `python3 scripts/encrypt_templates.py` and keep `test_encrypted_templates_not_stale` green. The failure string already names the script.
- After editing config registries, update `26-config-reference.md` and run the `config-docs` tests. Removed-flow keys go in `UNDOCUMENTED_REMOVED_FLOW_KEYS`, not the page.
- Do not regenerate user-guide pages from code. The config reference is hand-edited (`crates/codegen/xai-grok-shell/AGENTS.md`).

---

*Testing analysis: 2026-11-21*
