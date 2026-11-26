# TESTING — deepseek-build

A manual checklist for the personal `deepseek-build` agent. Every step states
what to run and what to expect; expected results are exact strings, exit codes,
or observable state.

- **Core pass (§1–§2, §4):** ~20–30 minutes with a warm `target/` — build check,
  first run + missing-key path, and the interactive checklist.
- **Live checks (§3) and installer/container/release (§5):** another ~25–40
  minutes; §3 needs a real key, §5 does not.

> Status (2026-11-26): the last **recorded** full gate run was green on
> 2026-11-23 — exit 0, step 1 (workspace) PASS, step 2 (`config-docs`) PASS,
> leak scan PASS; `results=308`, `binaries=219`, `doctests=89`. Test binaries
> landed after that refresh (`xai-grok-pager-bin`'s `headless_loop_items` and
> `version_paths`, among others), so **the checked-in
> `scripts/test-gate.expected` (308 results / 219 binaries) is stale**. Run
> `scripts/test-gate.sh --refresh-expected` to a green finish and commit the new
> counts before quoting a number. Evidence for the 2026-11-23 run is under
> `docs/evidence/01-truth-baseline/test-gate/`. The live DeepSeek smoke test and
> the runtime egress audit were last green on the author's machine (2026-10-07).
> `scripts/check-egress.sh --strict` is red on this tree for three pre-existing
> source strings (see [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md)). Some surfaces
> remain unexercised: see [Not exercised yet](#not-exercised-yet).

## Disk budget

- **One shared target dir.** In any worktree or parallel session, point
  `CARGO_TARGET_DIR` at the main checkout's `target/` (for example
  `export CARGO_TARGET_DIR="$HOME/Projects/deepseek-build/target"`) and remove a
  worktree's own `target/` if one exists. Builds are not shared otherwise.
- **Disk guard.** Before any workspace build run `scripts/disk-guard.sh` (exits
  non-zero below its threshold; pass a lower one as the first argument, e.g.
  `scripts/disk-guard.sh 60`, on a smaller disk). If it fails, stop and clean up first.
- **Lean dev/test profile.** `.cargo/config.toml` sets `[build] incremental =
  false`; `[profile.dev]` and `[profile.test]` use `debug = "line-tables-only"`,
  with `debug = false` for third-party crates. Measured 2026-10-12: a fresh
  `cargo build --workspace --tests` is ~24 GB in ~7 min; the same workspace had
  accumulated 244 GB of `target/debug` under the old settings. Delete
  `target/debug` after a gate run if space is tight.
- **Routine.** During work use per-crate tests (`cargo test -p <crate>`); keep the
  full `cargo test --workspace` for the final gate; then reclaim space with
  `cargo clean` (or `rm -rf target/debug`).
- **Emergency.** `ps aux | grep -E 'cargo|rustc'` for stray builds, then
  `git worktree remove <stale>` and `cargo clean`.

## 0. Prerequisites (once)

- Rust **1.94.0** via rustup (pinned by `rust-toolchain.toml`; do not bump) and
  `protoc` on `PATH` (e.g. `/usr/bin/protoc`).
- `cc` and `make` (the bundled jemalloc is built from source and fails without `make`).
- For the live egress script: a C compiler (`cc`) and `python3`.
- For §5: Docker, and `git`/`gh` for the release dry run.
- A real key, exported in your shell — **never** paste it into a prompt, a
  tracked file, or a transcript:
  `export DEEPSEEK_API_KEY=sk-...` (fallback name `DEEPSEEK_BUILD_API_KEY`).

## 1. Build (3 min once toolchain is warm)

| # | Run | Expect |
|---|-----|--------|
| 1 | `source "$HOME/.cargo/env"` | no output |
| 2 | `cargo build --release -p xai-grok-pager-bin` | exit 0, ends `Finished release ...` (a cold first build takes 10+ min of compile time — not counted in the 20–30 min pass) |
| 3 | `ls -lh target/release/deepseek-build` | an executable; last measured 147M (153,624,024 bytes) |
| 4 | `./target/release/deepseek-build --version` | `deepseek-build 0.1.0 (<12-hex commit>)` for a source build, exit 0. A CI binary built from a tag stamps that tag (`0.1.0-proof1 (b14d28eb2bc7)` for `v0.1.0-proof1`). |

## 2. Key handling and first run (3 min)

| # | Run | Expect |
|---|-----|--------|
| 5 | `[ -n "${DEEPSEEK_API_KEY:-}" ] && echo key-present` (or check `DEEPSEEK_BUILD_API_KEY`) | `key-present`; if nothing prints, export the key before continuing |
| 6 | Missing-key path, no key involved: `T=$(mktemp -d); env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY HOME="$T/home" DEEPSEEK_BUILD_HOME="$T/dbh" ./target/release/deepseek-build -p hi; echo "exit=$?"; rm -rf "$T"` | stderr `Not signed in: set DEEPSEEK_API_KEY (or DEEPSEEK_BUILD_API_KEY) to authenticate.`; `exit=1`; fail-closed before any model request (verified 2026-10-09 against a release binary built at `46aef092`) |
| 7 | Fresh first run: `BIN="$PWD/target/release/deepseek-build"; T=$(mktemp -d); (cd /tmp && HOME="$T/home" "$BIN"); rm -rf "$T"` | `$T/home/.deepseek-build/` (or `$DEEPSEEK_BUILD_HOME/`) is created with `sessions/`, `docs/user-guide/` (25 pages, from `USER_GUIDE`; the source directory also carries a README), `config.toml`, `logs/`; the TUI starts with the DeepSeek whale welcome logo. With the key exported: "Logged in with API key" and the prompt accepts input. Without a key: stays on the welcome screen, whose retry row reads `Retry sign-in (set DEEPSEEK_API_KEY)`; auth is env-only (no OIDC/OAuth path). There is **no interactive key entry** — quit and export the key instead. `q` quits cleanly. |

## 3. Live scripts (8–12 min; a real key needed for steps 8, 9 and 13)

| # | Run | Expect |
|---|-----|--------|
| 8 | `scripts/smoke-test.sh` | 8 checks, `8 passed, 0 failed`, exit 0. Covers: `/models` lists `deepseek-v4-pro` + `deepseek-flash`; plain + streamed chat; a 3-turn thinking tool chain (`num_turns >= 3`); usage carries `reasoning_tokens` + cache fields; `deepseek-flash` and the hidden `deepseek-v4-flash` alias accepted; `cached_tokens == prompt_cache_hit_tokens` on a cache hit. Without a key: `FAIL  API key present — set DEEPSEEK_API_KEY or DEEPSEEK_BUILD_API_KEY`, then `0 passed, 1 failed`, exit 1. The key is never printed. |
| 9 | `scripts/egress-check.sh` (5–10 min) | `EGRESS-CHECK PASS`, `EGRESS_VIOLATIONS=0`, exit 0. Runs a real model-driven session (shell, edit, `web_fetch`, local MCP, forced compaction, resume) under an LD_PRELOAD connect/send logger; the destination table shows only `api.deepseek.com:443`, the `web_fetch` host, loopback, and AF_UNIX. Without a key it exits 2 with `FAIL API key missing`. LD_PRELOAD limits apply — see `KNOWN-ISSUES.md`; `strace -f -e trace=connect,sendto,sendmsg` is the kernel-level alternative where available. |
| 10 | `scripts/key-leak-check.sh` (no real key) | `PASS fake key absent from N saved file(s) and captured output`, exit 0. Runs the binary with a fake key in an isolated `HOME` and scans logs/sessions/stdout/stderr. |
| 11 | `scripts/check-egress.sh --strict` (no key) | **Known red baseline on this tree:** exit 1, `HARD EGRESS VIOLATION: forbidden destination/host strings found:` with `(3 hard hits total; …)` — three pre-existing source/test strings (`api.x.ai`, `grok.com`, `gs://bucket`; see [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md)). The script exits at the HARD stage, before the SOFT branding scan runs. Phase 5 / FREEZE-03 owns the fixture fix; until then exit 1 here is the expected baseline, not a broken tree (matches the status banner above). |
| 12 | `scripts/sandbox-run.sh --version` (no key) | prints the version; needs no network. |
| 13 | `scripts/sandbox-run.sh -p "print hello"` (key) | the session completes; any destination other than the provider + loopback is refused and printed as `sandbox-run: BLOCKED connect ...` with the destination address (IPv4 or IPv6). To see the `web_fetch` host refused, ask it in the same session to fetch `https://example.com/` (`web_fetch` must be enabled) |

## 4. Interactive checklist (12–18 min; a real key)

Work in a scratch git repo: `mkdir -p /tmp/dsb-scratch && cd /tmp/dsb-scratch && git init`.
Launch the binary and approve tool calls as prompted.

| # | Do | Expect |
|---|----|--------|
| 14 | Ask: "create hello.py that prints hello" | a file-write tool call → permission prompt → after approve the file exists and a diff is shown |
| 15 | Multi-file refactor: "rename `mul` to `multiply` everywhere in this repo" | every affected file edited; `grep -rn 'mul(' .` finds nothing; any tests still pass |
| 16 | Plan mode: `Shift+Tab` until Plan (or `/plan`) | the mode indicator changes; the agent investigates read-only and asks before acting; launching with `--no-plan` disables it. Known gap: headless `--permission-mode plan` is read-only but does not emit a plan for a bare edit request |
| 17 | Permission modes: `Shift+Tab` cycling, `/auto`, and `--allow`/`--deny`/`--permission-mode` flags | modes change behavior; a dangerous `rm -rf`-class command still prompts with a warning even in auto modes |
| 18 | Reasoning: `/think` | toggles every reasoning block in the transcript (same toggle as `Ctrl+E`). Fold keys: `e` folds/unfolds the selected entry, `E` every entry, `Ctrl+E` all thinking blocks |
| 19 | `/model` (alias `/m`) | menu lists `deepseek-v4-pro` + `deepseek-flash`; the hidden `deepseek-v4-flash` alias is not shown but is accepted by `-m deepseek-v4-flash`; the switch changes the model shown |
| 20 | `/effort <level>` | offers `none`/`low`/`high`/`max`; the next turn honors it (`none` stops reasoning) |
| 21 | `/usage` (alias `/cost`) | opens the usage modal (tabs Context usage / Usage limit / Session info); the session block shows `Input tokens: N (M cached)`, `Output tokens: N (R reasoning)`, `Total tokens`, `Model calls`, and `Cost:` (currently `not available (not reported)` — the provider reports no cost). In minimal mode the same block prints to the scrollback |
| 22 | Status line: add `[ui.status_line]` with `type = "builtin"` and `items = ["cwd","model","context","effort","tokens","cache","cost","turn-timer","session-name"]` to `~/.deepseek-build/config.toml`, restart | the row renders those segments. Item names are kebab-case (`turn-timer`, `session-name`) |
| 23 | Resume: quit and relaunch with `-c` (most recent for this directory) or `-r <id>`, or use `/resume` inside the TUI | history reloads; `-r <id>` reuses the same session id |
| 24 | Headless: `./target/release/deepseek-build -p "Reply with exactly: pong"` | reply on stdout, exit 0 |
| 25 | Headless JSON: `./target/release/deepseek-build -p "say hi" --output-format json` | one JSON object with `text`, `stopReason`, `sessionId`, `requestId`; when tokens were recorded: `usage` (`input_tokens`, `cache_read_input_tokens`, `cache_creation_input_tokens`, `output_tokens`, `reasoning_tokens`, `total_tokens`), `num_turns`, `modelUsage.<model>.modelCalls`, plus `estimated_cost_usd` and `"estimate": true` (local peak-rate estimate, D14). `total_cost_usd` appears only when the provider reported a complete cost — the DeepSeek API currently reports none, so it is omitted; `usage_is_incomplete` / `cost_is_partial` flag degraded payloads |
| 26 | Headless permission cancel: `./target/release/deepseek-build -p "run rm -rf /tmp/dsb-does-not-exist"` (no `--always-approve`) | the gated tool is cancelled in headless mode (no TTY to prompt); stderr carries the one-line hint naming `--always-approve` and `--permission-mode auto`; exit is non-zero. Re-run with `--always-approve` to see it proceed |
| 27 | MCP: add a small stdio server to `~/.deepseek-build/config.toml` (`[mcp_servers.echo]` with `command`/`args`), relaunch, run `/mcps` | the server shows connected; its tool is listed to the model and can be called; a broken server shows an error rather than hanging |
| 28 | `web_fetch` ask-gate: enable `[features] web_fetch = true`, keep the default domain allowlist, then ask "fetch https://example.com/ and give me the page title" | a permission prompt naming the URL appears first (the host is not on the static allowlist); approve → the page is fetched; deny → the tool is refused and the turn continues |

## 5. Installer, container, release (10–15 min; no key)

| # | Run | Expect |
|---|-----|--------|
| 29 | `sh scripts/test-install.sh` | `ALL PASS (39 assertions)`, exit 0. Builds a fake release tree and serves it on 127.0.0.1; covers success, latest-resolution, checksum mismatch (installs nothing), unsupported OS, tarball 404, uninstall (incl. the `dsb` shortcut), rerun-update, unwritable dir, and a pre-existing `dsb` file or foreign `dsb` symlink. No network beyond loopback. |
| 30 | Container — published image: the CI-built `ghcr.io/coruairc/deepseek-build` image from `v0.1.0-proof1` was pulled by index digest `sha256:8f5c484e…` and run on 2026-11-26 (registry facts and the in-image turn are in `docs/evidence/04-distribution-proof/`). It is `debian:bookworm-slim`, runs as uid 1000, and `ENTRYPOINT`s `deepseek-build` with licenses in `/usr/share/doc/deepseek-build/`; it ships **no `bubblewrap`**. To rebuild locally: `docker buildx build --platform linux/amd64 -f docker/Dockerfile -t deepseek-build:test <ctx>`, where `<ctx>` is staged from the release tarballs | `docker run --rm ghcr.io/…@sha256:8f5c484e… --version` prints `deepseek-build 0.1.0-proof1 (b14d28eb2bc7)`; a headless turn with `--permission-mode auto` runs a shell command and returns `estimated_cost_usd` |
| 31 | Container — local smoke (`docker/Dockerfile.localtest`): stage a directory with your built binary copied as `deepseek-build`, plus `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES` and a `README.txt`, then `docker build -f docker/Dockerfile.localtest -t deepseek-build:localtest .` — **NOT YET EXERCISED** | `docker run --rm deepseek-build:localtest --version` works; `docker run -it --rm -e DEEPSEEK_API_KEY -v "$PWD":/work deepseek-build:localtest` starts the TUI |
| 32 | Release proof — **EXERCISED** on `v0.1.0-proof1` (run 45724842211, 7/7 jobs, 2026-11-26) | all four native builds succeeded; each tarball has `deepseek-build` + `LICENSE` + `NOTICE` + `THIRD-PARTY-NOTICES` + `README.txt` at top level; `SHA256SUMS` lists all four; the pre-release has six assets (four tarballs + `SHA256SUMS` + `install.sh`); the pinned installer prints `0.1.0-proof1 (b14d28eb2bc7)`. Evidence: `docs/evidence/04-distribution-proof/`. `v0.0.1-rc1` … `rc5` are published prereleases in the same workflow |

## Not exercised yet

> - **Interactive TUI by a human.** It has been driven under a pty (scripted keystrokes, 2026-10-07), never used by a person.
> - **Live scripts on any machine but the author's.** `smoke-test.sh` (8/8) and `egress-check.sh` (`EGRESS_VIOLATIONS=0`) passed there on 2026-10-07 and have not been reproduced elsewhere.
>
> The release workflow (run 45724842211) and the published container image (pulled by digest) were exercised on 2026-11-26; the remaining unproven surfaces are Windows (no artifact), a local `docker/Dockerfile.localtest` build, and a `brew install` on macOS.

## 6. Test suite status

- **The gate is `scripts/test-gate.sh`; run it before any release claim.** It
  runs three named steps: (1) `cargo test --workspace --no-fail-fast --
  --test-threads=4`; (2) **TEST-02**, the `config-docs` step
  `cargo test -p xai-grok-shell --lib --features config-docs -j 1` (the
  workspace run never enables `config-docs` because `xai-grok-shell` has
  `default = []`); (3) a key-leak scan over `docs/evidence`. The verdict is the
  arithmetic (`results == binaries + doctests == expected`), never the tail of
  the log; `results` counts one top-level `test result:` line per cargo target
  block. Both cargo steps run with `DEEPSEEK_API_KEY`,
  `DEEPSEEK_BUILD_API_KEY` and `XAI_API_KEY` unset. The last recorded full run
  was 2026-11-23 (exit 0; `results=308`, `binaries=219`, `doctests=89`; step 1
  31,913 passed / 0 failed / 386 ignored; `config-docs` 6,201 passed / 0 failed /
  4 ignored). Raw `test result:` lines were 309 because `xai_tty_utils`'s
  `stderr_redirect_roundtrip_subprocess` re-runs its own test binary with
  inherited stdout. Evidence:
  [`docs/evidence/01-truth-baseline/test-gate/`](docs/evidence/01-truth-baseline/test-gate/).
  **Those counts are stale for the current tree:** test binaries added after the
  refresh (`xai-grok-pager-bin`'s `headless_loop_items` and `version_paths`,
  among others) change the target count, so re-run
  `scripts/test-gate.sh --refresh-expected` green and commit the new
  `scripts/test-gate.expected` before quoting a suite size.
- **The expected count is checked in** at `scripts/test-gate.expected`, with its
  provenance header (refresh command, commit, UTC date, the three numbers). Only
  a fully green `scripts/test-gate.sh --refresh-expected` rewrites it; edit it by
  hand and the full-mode comparison fails.
- **Determinism (TEST-03, corrected 2026-11-23).** The two process-global
  `xai-grok-shell` tests (`agent::folder_trust::tests::record_and_lookup_round_trip`
  and `util::config::consent::tests::set_consent_answer_is_monotonic_per_account`)
  share one bare `#[serial_test::serial]` key, and the `git_gate` waits were
  widened (root-cache sleep 100 → 500 ms; walk-timeout TTL 20 → 100 ms, sleep
  40 → 250 ms; late-OK walk timeout 40 → 100 ms, sleep 50 → 250 ms; joiner TTL
  500 ms → 5 s; sustained-invalidate outer timeout ×3 → ×10); no assertion
  changed direction. The earlier five-run sample is superseded: the phase
  verification's 35-run repetition of `cargo test -p xai-grok-workspace --lib
  -- --test-threads=4` on `1308903a` (identical file also at `98c2f1b8`)
  produced **2 failures in 35**; the captured one was
  `restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild`
  (`grandchild <pid> must not remain live`, `restore_fetch_tests.rs:731`) — an
  instantaneous `/proc` read after an asynchronous SIGKILL. Plan 01-06 fixed it
  with a deadline-bounded death poll (5 s deadline, 20 ms interval) on all five
  process-death sites, prompt-return bounds still asserted before any polling.
  Post-fix: **35/35** full-crate runs green (`1695 passed; 0 failed` each),
  each run's `test result:` line in
  [`docs/evidence/01-truth-baseline/determinism/repetition-35-runs.log`](docs/evidence/01-truth-baseline/determinism/repetition-35-runs.log)
  (item [`docs/evidence/01-truth-baseline/determinism/`](docs/evidence/01-truth-baseline/determinism/) —
  cast, `report.json`, notes). No test is quarantined.
- `cargo check --workspace --tests` is green (all test targets compile).
  `xai-grok-login` is green; the dead OIDC/OAuth surface has been deleted, so
  its old pre-deletion counts are not a current number. See
  [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md) and `docs/DROPPED-TESTS.md` before
  quoting numbers.
- Branding gate: `cargo test -p xai-grok-pager-bin --test branding_surface`
  runs the real binary's `--help`, `agent --help`, `--version`, `completions
  bash`, and a keyless headless run, and fails on `grok` / `xai` / `x.ai`.
- `scripts/smoke-test.sh` is the live end-to-end check.
