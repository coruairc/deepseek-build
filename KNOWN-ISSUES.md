# KNOWN ISSUES — deepseek-build

Honest list of what is incomplete or broken at this revision. Nothing here is
hidden behind a "temporary" flag; where code remains it is called out. Claims are
against `main` at the Phase 5.1 close (2026-11-26, `0d276f85`). The test state is
last recorded 2026-11-23 (and its expected count is now stale, see below); the
distribution, container and Windows notes were refreshed 2026-11-26.

## Build & tests

- **`process_key_from_model_env_key` can print an exported `DEEPSEEK_API_KEY`
  (pre-existing, found 2026-11-21).** With `DEEPSEEK_API_KEY` exported,
  `agent::config::tests::process_key_from_model_env_key` resolves the exported
  key where it expects the model's `env_key` token, so the `assert_eq!` failure
  message prints the real key value. The gate (`scripts/test-gate.sh`) unset the
  three key variables for every cargo step; a hand-run of the shell suite must do
  the same (`env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY`).
  Redacting the value in the assertion is a follow-up outside this phase.

- **`xai-grok-shell` env-global test rotation: fixed (2026-11-21).**
  `agent::folder_trust::tests::record_and_lookup_round_trip` now carries
  `#[serial_test::serial]`, and
  `util::config::consent::tests::set_consent_answer_is_monotonic_per_account`
  dropped its *named* key (`serial(DEEPSEEK_BUILD_HOME)`), so both share the same
  serial key and exclude each other. The `config-docs` gate step then passed
  inside `scripts/test-gate.sh`. Separately, three config tests fail when
  `DEEPSEEK_API_KEY` is exported; the gate unsets it (see the entry above).

- **The full workspace run is green under the gate (seeded 2026-11-21,
  confirmed 2026-11-23).** `scripts/test-gate.sh` exits 0: step 1
  `cargo test --workspace --no-fail-fast -- --test-threads=4` PASS, step 2
  `cargo test -p xai-grok-shell --lib --features config-docs -j 1` PASS, leak
  scan PASS; `results=308`, `binaries=219`, `doctests=89`, `expected=308`
  (checked in at `scripts/test-gate.expected`). Completeness is asserted
  arithmetically — `results` counts one top-level `test result:` line per cargo
  target block — and never inferred from the log tail. Raw `test result:`
  lines are 309 because `xai_tty_utils`'s
  `stderr_redirect_roundtrip_subprocess` re-runs its own test binary with
  inherited stdout. Evidence: `docs/evidence/01-truth-baseline/test-gate/`.
  **The 2026-10-09 figure "31,939 passed / 2 failed / 388 ignored across 307
  targets" is stale** (a different tree) and must not be quoted. Quarantined
  tests: none. **Confirmation run #3 (2026-11-23, plain `scripts/test-gate.sh`,
  no `--refresh-expected`) reproduced the same numbers and exit 0**: step 1
  31,913 passed / 0 failed / 386 ignored; the `config-docs` step 6,201 passed /
  0 failed / 4 ignored (artifact-backed by the supplementary excerpt
  `docs/evidence/01-truth-baseline/test-gate/step2-config-docs-result-line.log`,
  added post-review for IN-04); the expected file was not edited. The run's executor
  session was killed by a server restart mid-compilation; the detached,
  evidence-wrapped gate process survived and completed, and the continuation
  executor adopted it (step record carries the real exit status; counts
  re-derived independently from the log). The `test-gate` evidence item is
  complete (cast, `report.json`, normalized log, notes). **`scripts/test-gate.expected` was refreshed from a green full run on
  2026-11-26** (`be923015`): `results=310`, `binaries=221`, `doctests=89`, clean
  tree at `e79aef5b`; both steps (workspace + `config-docs`) and the leak scan
  passed. It previously read 308/219 at `c159a3e7`.
- **`xai-grok-sandbox --test deny_paths_e2e` is green: 14 passed / 0 failed /
  1 ignored (2026-11-21).** The previously failing
  `devbox_genuine_reexec_applies_enforcement` now runs a
  `[profiles.genuine] extends = "devbox"` profile with an explicit `deny`, so
  the genuine re-exec has a non-empty plan on a host with no `/data`; the single
  ignored test is the pre-existing `subprocess_entry` harness entry point and no
  new `#[ignore]` was added. `read_deny_empty_set_verifies_inside_bwrap` passes
  here because `/run/containerd` is absent.
- **The same bwrap test still fails on a host that cannot create namespaces
  (build box, Step 3e 2026-10-23).** `devbox_genuine_reexec_applies_enforcement`
  fails inside the containerised build box because bwrap re-exec cannot create
  namespaces there; the failure is host-environment, not code, and the author's
  host is green. Repro:
  `cargo test -p xai-grok-sandbox --test deny_paths_e2e -j 1 -- devbox_genuine_reexec_applies_enforcement`.
- **`xai-grok-login`: green (last recorded 2026-10-09: 395 passed / 0 failed /
  1 ignored).** The lane was red because inert OAuth/OIDC scaffolding tests
  could never pass, and one e2e test deadlocked. The dead xAI login/OIDC surface
  was deleted (dropping 18 tests of removed behavior, ledgered), the sleep-gate
  test was rewritten to drive the real drain with a controllable refresher, and
  the API-key env vars are guarded so a developer's exported key can neither
  change results nor leak. See `docs/DROPPED-TESTS.md`. The count is historical;
  do not compare it with a current run.
- **`xai-grok-shell`: 6197 passed / 0 failed / 4 ignored** on the final tree (was
  6221; the 24-test drop is inside the deleted login surface). `xai-grok-pager`
  9876 / 0 / 4 and `xai-grok-pager-minimal` 94 / 0 unchanged.
- **All test targets compile** (`cargo check --workspace --tests` green).
- **`xai-grok-workspace` test target: compiles and passes** (1695 passed in the
  post-fix 2026-11-23 repetition; the 2026-11-21 runs reported 1694 before the
  death-poll regression test existed). The `git_gate` waits were widened in plan
  01-05 (root-cache sleep 100 → 500 ms; walk-timeout TTL 20 → 100 ms with sleep
  40 → 250 ms; late-OK walk timeout 40 → 100 ms with sleep 50 → 250 ms; joiner
  TTL 500 ms → 5 s; sustained-invalidate outer timeout ×3 → ×10).
  **TEST-03 determinism — corrected 2026-11-23 by plan 01-06.** The earlier
  five-run sample quoted here was executor-reported (raw logs never committed)
  and is superseded: the Phase-1 verification's own 35-run repetition of
  `cargo test -p xai-grok-workspace --lib -- --test-threads=4` on `1308903a`
  (`restore_fetch_tests.rs` is identical at `98c2f1b8`) produced **2 failures
  in 35**; the captured one was
  `restore_fetch::tests::fetch_timeout_sigkills_term_immune_grandchild` —
  `grandchild <pid> must not remain live` at `restore_fetch_tests.rs:731`. The
  test read `/proc/<pid>` immediately after teardown while SIGKILL delivery and
  reaping are asynchronous. Fixed by a deadline-bounded poll
  (`wait_until_dead`: 5 s deadline, 20 ms interval) on all five process-death
  assertion sites; the prompt-return bounds are still asserted before any
  polling, no assertion direction changed and no `assert_timeout_about` band
  moved. Post-fix repetition: **35/35 full-crate runs green** (`1695 passed; 0
  failed` each), every run's `test result:` line recorded in
  `docs/evidence/01-truth-baseline/determinism/repetition-35-runs.log`
  (item `docs/evidence/01-truth-baseline/determinism/`, requirement TEST-03).
  Quarantined tests: none. **Coverage gap to re-home:** the local harness
  `SessionToolHandle`/`create_local_harness` runs the toolset without the
  deleted hub pre-run/sandbox integration, so equivalent local coverage must be
  re-established (see `docs/DROPPED-TESTS.md`).
- **Per-crate test results (last recorded per-crate runs, 2026-10-08):**
  sampling-types 240, sampler 186, telemetry 156, status-line 24, brand 2,
  version 2, config 467, tools-api 15, chat-state 391, session-events 16,
  active-sessions 6, foreign-sessions 53, config-types 45, http 14, mcp 279,
  hooks 279, agent 576 — all 0 failures **in those runs**. These do not cover
  `xai-grok-login`, `xai-grok-pager` (last 9876) or `xai-fast-worktree`. Because
  workspace feature unification compiles feature-gated tests that per-crate
  runs miss (the `xai-fast-worktree` case below), a per-crate pass is necessary
  but not sufficient evidence for the workspace run.
- **Non-test build is green:** `cargo check -p xai-grok-pager-bin` and
  `cargo build --release -p xai-grok-pager-bin` succeed; the binary runs
  (`target/release/deepseek-build`, 147 MiB at the last measured build
  `46aef0920857`, `deepseek-build 1.0.45 (<commit>)`).
- **Step 6 mutation check:** breaking the sandbox truncation cap (`5_000`→
  `6_000`) made `sandbox_host_caps_task_output_polls_at_5k` fail, confirming the
  sandbox tests still detect the behavior; the change was reverted.
- **Tags/releases (2026-11-26):** the release pipeline exists and has run.
  Prerelease tags `v0.0.1-rc2` … `v0.0.1-rc5` and `v0.1.0-proof1` are on
  `origin`; `v0.1.0-proof1` (commit `b14d28eb`) built all four targets plus the
  GHCR image in run 45724842211 (7/7 jobs) and published a pre-release with six
  assets. **`v0.1.0` is not published** (the author publishes it). The old
  `last-known-good-2026-10-06` tag (`b3847ff8`) still contains egress code and is
  not safe to run.
- **`deepseek-v0.1-snapshot` is superseded, not moved (FREEZE-02).** The
  pre-existing `deepseek` branch and the `deepseek-v0.1-snapshot` tag both sit at
  `4b305bd0c1ab2fbb55468df14ab6a7e94d7c8d0b`. The freeze marker is a new,
  non-triggering tag; it must not move either ref. The `v0.1.0-proof1` tag push
  left `origin/main` (`6e9d8d88…`) and both of those refs where they were
  (`docs/evidence/04-distribution-proof/tag-push/`).

## Network / egress

- **`scripts/check-egress.sh --strict` is red on this tree (baseline,
  re-checked 2026-11-26).** It exits 1 with 3 HARD hits, all pre-existing
  source/test strings:
  `crates/codegen/xai-grok-shell-base/src/util/mod.rs:422-423` (`api.x.ai`,
  `grok.com`) and `crates/codegen/xai-grok-shell/src/agent/config_tests.rs:2768`
  (`gs://bucket`). Phase 1 does not fix the gate (out of scope); the red
  baseline is recorded so the test gate's green cannot be mistaken for a green
  egress gate. `scripts/egress-check.sh` (runtime audit) is unchanged.
- **Extended runtime egress verified (2026-10-07)** with
  `scripts/egress-check.sh` — an LD_PRELOAD logger that allows-and-logs
  `connect`/`sendto`/`sendmsg`/`sendmmsg`/`getaddrinfo` from the binary and its
  dynamic children. A real model-driven session (shell, file edit, `web_fetch`
  of `https://example.com/`, a local stdio MCP tool, forced auto-compaction,
  resume, headless) observed **only**:
  - `api.deepseek.com:443` over IPv6/NAT64 (`64:ff9b::3ad:153f`),
  - `example.com:443` for the `web_fetch` call,
  - AF_UNIX sockets (`/run/systemd/resolve/io.systemd.Resolve`,
    `/var/run/dbus/system_bus_socket`).
  `EGRESS_VIOLATIONS=0`. Non-loopback public destinations must be on port 443.
- **Evidence normalizer overlaps `mktemp`'s `/tmp/tmp.*` prefix (observed 2026-11-25, plan 02-01).**
  `scripts/evidence.py` replaces every `/tmp` substring. A work directory created
  as `/tmp/tmp.<random>` therefore renders as `<TMP><TMP>.<random>` in the
  normalized transcript and cast. The live-smoke item records this; the scripts
  were not edited (read-only this phase). It is a display overlap, not a second
  directory and not a key leak. Follow-up: replace the longest prefix once, or
  stop at a path boundary, so `/tmp/tmp.X` becomes `<TMP>/tmp.X`.
- **`scripts/sandbox-run.sh` is stricter**: it allows only the provider +
  loopback, so the `web_fetch` host is refused (the session still completes).
  Its shim resolves the allow host over both families and names IPv6 refusals by
  destination (fixed in plan 01-02; `scripts/test-sandbox-run.sh` proves it
  offline). `non-ip` remains only for a genuinely non-IP family such as
  `AF_NETLINK`.
- **LD_PRELOAD limits (inherent, not fixed):** raw `syscall(SYS_connect|sendto)`,
  `io_uring`, statically-linked or setuid children, and children that scrub
  `LD_PRELOAD` are not intercepted; allowlisting is by resolved IP, so another
  hostname on the same shared CDN edge IP would pass. `strace` is absent in this
  environment. For kernel-level ground truth use
  `strace -f -e trace=connect,sendto,sendmsg` where available.
- **Removed and verified gone:** `xai-grok-shell/src/agent/session_registry_client.rs`
  and all remote session-registry writers; `common/xai-tracing` and the OTLP
  exporter stack (`opentelemetry`, `opentelemetry-otlp`, `opentelemetry-proto`,
  `opentelemetry-http`, `tracing-opentelemetry`, `fastrace-opentelemetry`) —
  `Cargo.lock` has 0 `opentelemetry` entries; `xai-grok-http::shared_upload_client()`.
  `tonic` remains (tools/tools-api/workspace/login).
- **User-configured HTTP hooks** (`xai-grok-hooks`) are intentionally kept.

## API key handling

- **Env-only auth:** `DEEPSEEK_API_KEY` (primary) or `DEEPSEEK_BUILD_API_KEY`
  (fallback). No interactive key flow. Missing key prints one line:
  `Not signed in: set DEEPSEEK_API_KEY (or DEEPSEEK_BUILD_API_KEY) to authenticate.`
- **Never logged:** `SamplerConfig`'s `Debug` redacts `api_key`; the sampler's
  header-conversion errors and the new-session debug line no longer echo it.
  `scripts/key-leak-check.sh` runs the binary with a fake key and scans debug logs
  and saved sessions — currently PASS. A unit test (`debug_redacts_api_key`)
  pins the redaction.
- **Default (no-arg) `key-leak-check.sh` still fails open on grep exit ≥ 2
  (pre-existing; deferred 2026-11-23).** The live-run scan uses
  `if grep …; then hits=…; fi`, so an unreadable file or I/O error leaves
  `hits=0` and prints PASS without a complete scan. The `--scan` mode (the
  gate's step 3 and the committed-evidence scan) now fails closed on grep
  errors (CR-01, fixed 2026-11-23); default-mode behaviour is frozen by
  plan 01-05 task 3(b) for this phase. Follow-up: apply the same rc 0/1/else
  branch (diagnostics only, never matched lines) and pin it in TESTING step 10.
  Disposition: `01-REVIEW-DISPOSITION.md` row IN-07 (`deferred`).
- **Live smoke (Step 1, 2026-10-07):** `scripts/smoke-test.sh` = 8/8 PASS on the
  author's machine. See the smoke results below.

## Step 1 — live adapter results

> All results below are from the author's machine (2026-10-07) and have not been
> reproduced on another host.

- `GET /models` returns exactly `deepseek-flash`, `deepseek-v4-pro`.
- Plain and streamed headless chat OK; the hidden `deepseek-v4-flash` alias maps
  to `deepseek-flash`.
- A 5-turn thinking tool-call chain ran (`num_turns=5`, `modelCalls=5`,
  `reasoning_tokens>0`).
- Usage mapping confirmed: harness `input_tokens` = DeepSeek
  `prompt_cache_miss_tokens`, `cache_read_input_tokens` = `prompt_cache_hit_tokens`,
  `reasoning_tokens` = `completion_tokens_details.reasoning_tokens`,
  `total_tokens` = input + cache_read + output. On a cache hit
  `cached_tokens == prompt_cache_hit_tokens` (verified 2432). No adapter mismatch.
- `tool_choice=required` in thinking mode → HTTP 400 (documented in O3).

## Step 4 — real-use results (scratch repo, headless)

> Author's machine, 2026-10-07; not reproduced elsewhere.

- **Worked:** simple edit; fixing a failing test (it ran `python3 test_calc.py`);
  a multi-file rename (`mul`→`multiply` across 4 files); session resume with
  context recall. Warm-turn cache hit rate 98–99%.
- **Plan mode (corrected 2026-11-25):** the note that stood here described an
  older tree. The current headless path sends `session/set_mode` with `plan`
  before the prompt, so the shell injects the plan-mode reminder
  (`Plan mode is active. Do not make any edits or writes to the system.`) into
  the turn's request, persists `plan_mode.json` with `state: "Active"`, and
  keeps the plan edit gate armed. Proven from the captured request body and the
  session's own `plan_mode.json` by
  `docs/evidence/02-loop-item-verification/permission-mode/` (PERM-04), which
  also drives the flag end to end instead of asserting it.
- **Headless cannot prompt:** a permission-gated action (e.g. `rm -rf`) ends as
  `permission_cancelled` with a one-line stderr message naming both
  `--always-approve` and `--permission-mode auto`, and a non-zero exit
  (`d3a9c176`; the exact line is pinned in
  `crates/codegen/xai-grok-pager-bin/tests/headless_loop_items.rs`). The JSON
  error path carries the same hint. `--always-approve` is required for unattended
  actions.
- **Cost:** the provider still reports no `cost_usd_ticks`, so `total_cost_usd`
  is omitted. The headless `--output-format json` result carries a local estimate
  instead: `estimated_cost_usd` + `estimate: true`, priced from the shipped
  DeepSeek peak-rate table (D14), omitted when no tokens were recorded. The eval
  harness records it per task (`scripts/evals/`).

## Phase 2 — adapter

- **Backend removal (D5): done.** `ApiBackend` has only `ChatCompletions`.
- **Wiremock suite present** (`crates/codegen/xai-grok-sampler/tests/chat_completions_wire.rs`):
  streaming text + reasoning, 3-turn tool chain replaying `reasoning_content`,
  missing-`reasoning_content` backfill, quote repair, partial/malformed stream;
  retry/backoff in `tests/test_actor.rs`.
- **Automatic model routing is not wired** (`model_routing` defaults to `off`);
  manual `/model` + `/effort` is what works. Catalog defines `deepseek-v4-pro`,
  `deepseek-flash`, and the hidden `deepseek-v4-flash` alias.

## Phase 3 — TUI

- Implemented: reasoning folding (`e`/`E`/`Ctrl+E`), `/think`, `/model`,
  `/effort`, `/theme`, Monokai theme with whale logo, configurable status line
  (`[ui.status_line]`, items include `model`, `context`, `effort`, `tokens`,
  `cache`, `cost`).
- The status row does not surface the auto-routing decision (moot until routing
  is wired).

## Phase 4 — hardening

- Read-before-write enforced; dangerous commands (`rm -rf`-class) stay gated with
  a warning.
- Compaction, MCP, resume, headless were exercised headlessly (Step 2). Plan
  mode and permission *prompts* were not exercised interactively.

## Unexercised surfaces (no human, no CI)

- **Interactive TUI by a human: never.** It has been driven under a pty with
  scripted keystrokes (2026-10-07: welcome logo, `/model`, `/effort`, `/think`,
  status line), but no person has used it interactively. §4 items in
  [`TESTING.md`](TESTING.md) are the checklist for that pass.
- **Release workflow in CI: green on its four targets (2026-11-26).** Run
  45724842211 (`v0.1.0-proof1`) is 7/7 jobs: two Linux, two macOS (47.8 min and
  61.5 min against the 240-minute budget), container, publish. The release is a
  pre-release with six assets, and both macOS binaries are ad-hoc signed,
  verified from the extracted tarball. `v0.0.1-rc4`'s macOS timeout is history;
  see `docs/evidence/04-distribution-proof/tag-run/`.
- **Container image: built, published and pulled (2026-11-26).** The `container`
  job depends on `build-linux` only, so the macOS legs cannot skip it. The image
  was pulled by index digest and run; see [Container image](#container-image-2026-11-26).
  Neither `docker/Dockerfile` nor `docker/Dockerfile.localtest` has been
  exercised by a human locally, but the published image is the CI-built one from
  `docker/Dockerfile`.
- **Live scripts on any machine but the author's: never.** `scripts/smoke-test.sh`
  (8/8) and `scripts/egress-check.sh` (`EGRESS_VIOLATIONS=0`) passed there on
  2026-10-07 and have not been reproduced elsewhere.
- **`scripts/test-install.sh`: green locally** (39/39 assertions) against a
  loopback fake release; it never touches the real release pipeline.

## Release CI capacity (2026-11-26)

- **macOS arm64 runners are not always acquirable.** Run 2 (`v0.1.0-proof2`) lost its
  `build-macos (aarch64-apple-darwin)` job three times to
  `The job was not acquired by Runner of type hosted` (capacity). `publish` waits for all four
  builds, so no release was created; the image still published, because `container` depends on
  `build-linux` only. A retry acquired a runner, and `v0.1.0-proof2` is now published with all five artifacts
  (four tarballs + the Windows zip) plus `SHA256SUMS` and both installers; the zip verifies
  against `SHA256SUMS`.

## Acceptance findings (2026-11-26)

The author signed off the daily loop on `0.1.0 (bc95bc47)` (`docs/acceptance-2026-11-26.md`) and
reported two follow-ups, both being fixed after the sign-off:

- **`Ctrl+E` did not expand the thought in front of you.** It is bound to
  `ExpandAllThinking` (scrollback-focused) and toggles every thinking block. The fix makes it toggle
  the selected block and keep the expand-all behaviour only when nothing is selected.
- **Focus did not return to the prompt after `Esc` closed a thought.** `Esc` in the scrollback is
  deliberately swallowed by the input-flow contract; the fix returns focus to the prompt when `Esc`
  actually dismisses a selection, and leaves the double-Esc rewind and the mid-turn swallow alone.

## Fixed regressions (recorded, 2026-11-27)

Fixed bugs are kept here for a short while because the *way* they were missed matters more than
the fix. Issues are enabled again; this one is tracked as
[#1](https://github.com/coruairc/deepseek-build/issues/1).

### `/agent` always restarted the TUI even though in-place switching existed

- **Symptom:** `/agent` → pick → the notice appeared and `dsb` closed and reopened, every time.
- **Root cause:** the handler was reachable in tests but not in production. `acp_agent.rs` routes
  ext methods through an **explicit match list** that forwards a subset to
  `extensions::session_admin::handle`. `deepseek-build/session/switch_agent` was missing from that
  list, so every request answered `method_not_found`; the pager correctly classified that as
  "this shell cannot switch in place" and took its relaunch fallback.
- **Why the tests missed it:** `switch_agent_tests` called `session_admin::handle` directly, so the
  handler was green while the top-level router 404'd. The pager's fallback test also used a
  synthetic error, so it never exercised the real method either.
- **Fix:** `b7296ca6` adds the method to the router arm. The pager also records every switch
  outcome to `$TMPDIR/deepseek-build-agent-switch.log` (`ok-in-place` / `failed-fallback reason=…`),
  which is what identified the 404 after two earlier guesses were wrong.
- **Remaining gap:** there is still no test at the router level. `ext_method` is a trait method
  (`E0449` forbids a visibility qualifier on it), so the pragmatic follow-up is a table-driven
  router — one list consumed by both the dispatcher and a test — rather than a new test per method.
- **Related UX:** the picker now closes on selection (`bbe4c24b`), so the prompt is ready instead
  of leaving the modal open.

## Container image (2026-11-26)

- **The image runs, and the sandbox cannot apply inside it.** `ghcr.io/coruairc/deepseek-build@sha256:8f5c484e…`
  was pulled and run: `--version` prints `0.1.0-proof1 (b14d28eb2bc7)`, and a headless turn ran
  `printf 'in-image-ok\n'` and returned it with `estimated_cost_usd`. The image contains **no
  `bubblewrap`**, so the in-process sandbox is unavailable there; the container boundary is the
  isolation. `--sandbox` is accepted but not enforced, and the README now says so. Evidence:
  `docs/evidence/04-distribution-proof/image-run/`.
- **Adding bwrap would not be enough.** bwrap needs user namespaces, which a default unprivileged
  container blocks; enabling it requires `--cap-add SYS_ADMIN` or `--privileged`, i.e. weakening the
  container's own isolation. That trade-off is deliberately not made.

## Windows tier (2026-11-26)

- **Windows packaging is in the tree but not yet released or accepted.** The release workflow
  carries a `build-windows` job (`windows-latest`, `x86_64-pc-windows-msvc`, 90-minute budget)
  that stages a `.zip` (binary + `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES`, `README.txt`) and
  `install.ps1` and its harness `scripts/test-install.ps1` exist at the repo root; all three
  were uncommitted working-tree changes at the time of writing. The harness has **not been run**
  (the authoring host has no PowerShell), no tagged run has produced a Windows `.zip`, and
  WIN-01…06 are **pending** in `.planning/REQUIREMENTS.md` (Phase 5 is 0/5). Do not tell a user
  that a Windows `.zip` is downloadable today.
- **The sandbox is honest on Windows.** There is no Windows sandbox backend. Only `--sandbox off`
  is accepted; any other profile (including one from `config.toml` or
  `DEEPSEEK_BUILD_SANDBOX`) is a startup error naming the unsupported platform, and
  `/session-info` reports it unsupported. Tests: `windows_only_accepts_off`,
  `windows_absent_request_defaults_to_off`, `unsupported_platform_error_names_windows_exactly`
  (`crates/codegen/xai-grok-sandbox/src/profiles.rs`). The user-guide page `18-sandbox.md` and
  the README state the tier; no artifact claims "sandboxed" on Windows.
- **The compile spike is recorded, not finished.** `.planning/phases/05-windows-support/05-SPIKE-LOG.md`
  records a `windows-latest` `cargo check` spike on throwaway branches
  (`ci/windows-spike`, `ci/windows-spike2`): four blockers found, three fixes landed on `main`
  (`find-msvc-tools` 0.1.15, `xai-proto-build` temp files instead of `/dev/*`, pty `cfg(unix)`
  gates). The run that would prove the whole workspace clean on Windows is not recorded as
  green, so a Windows build is unsupported-but-plausible, not proven.

## Rebrand / config

- Binary (`deepseek-build`), config dir (`~/.deepseek-build`), ACP namespace
  (`deepseek-build/*`), theme, user-facing strings, and the shipped user-guide
  docs' user-home paths are rebranded. `install.sh` additionally creates a `dsb`
  shortcut symlink (install-time only — the binary name itself is unchanged, D1).
- **Legacy env-var fallbacks are gone (2026-10-12).** Only the branded names
  are read: `DEEPSEEK_BUILD_HOME`, `DEEPSEEK_BUILD_SANDBOX`,
  `DEEPSEEK_BUILD_AGENT_DASHBOARD`, `DEEPSEEK_BUILD_THEME` /
  `LC_DEEPSEEK_BUILD_THEME`, and `DEEPSEEK_BUILD_APPEARANCE` /
  `LC_DEEPSEEK_BUILD_APPEARANCE`. The old `GROK_HOME`, `GROK_SANDBOX`,
  `GROK_AGENT_DASHBOARD`, `GROK_THEME`, and `GROK_APPEARANCE` names (including
  the `LC_` forms) are ignored, with tests pinning that. No automatic
  migration from `~/.grok`: set `DEEPSEEK_BUILD_HOME` to reuse it. Every
  `GROK_*` name named in the docs, `--help`, or product error text was renamed
  to `DEEPSEEK_BUILD_*` (2026-10-12); the remaining ~435 internal-only names
  (1,529 occurrences across 284 files) are listed in
  [`docs/INTERNAL-GROK-ENV-NAMES.md`](docs/INTERNAL-GROK-ENV-NAMES.md) for the
  open-build branch.
- **User-guide swept (2026-10-12).** The shipped guide no longer uses
  `grok <command>` examples, and the pages/sections describing removed flows
  (OAuth/OIDC/login, external auth providers, remote settings, managed config,
  external OpenTelemetry) are deleted (`02-authentication.md`,
  `24-monitoring-usage.md`, plus pruned sections elsewhere). Remaining prose
  reads `deepseek-build <command>`.

- **Project directory is `.deepseek/`, with `.grok/` as a compatibility
  fallback (BRAND-01…03, 2026-11-26).** `xai-grok-brand::PROJECT_DIR_NAME =
  ".deepseek"`, `LEGACY_PROJECT_DIR_NAMES = [".grok"]`, and
  `project_dir_names()` return `.deepseek` first. Agents, plugins, skills,
  workflows, project `config.toml`, hooks, `lsp.json`, sandbox profiles and
  `workspaced.toml` all resolve through it; a repository with only `.grok/`
  still works, and `.deepseek` wins when both define the same name. Writes go to
  `.deepseek`; an existing `.grok` file is never migrated or rewritten.
  - **Retained residuals:** `cwd/.grok/rewind-checkpoints` stays where it is
    (session durability; splitting the mirror could stop existing sessions
    rehydrating), and `.grok-plugin/` stays (marketplace index convention).
  - **Open-build:** flip `PROJECT_DIR_NAME` to `.ob`; the fallback list then
    reads `.ob` → `.deepseek` → `.grok`.
- **Some user-guide project paths still say `.grok/`.** The canonical pages
  `05-configuration.md`, `12-project-rules.md` and `26-config-reference.md`
  document `.deepseek/` with the fallback, but a few pages (`08-skills.md`,
  `09-plugins.md`, `10-hooks.md`) still show `.grok/` in project-scope paths.
  This is a documentation gap, not a code gap.
- **Version string (0.1.0).** `[workspace.package] version = "0.1.0"`, inherited
  by the remaining crates. Release builds take the version from the tag via
  `GROK_VERSION`; the `v0.1.0-proof1` image prints
  `deepseek-build 0.1.0-proof1 (b14d28eb2bc7)`, and a source build without the
  stamp prints `deepseek-build 0.1.0 (<commit>)`.
- **Leftover sign-in surface.** `login` / `logout` subcommands and the `/login`
  slash command are still registered, and `login --help` still describes
  OAuth via `api.deepseek.com`, although the OAuth/OIDC stack was deleted and
  auth is env-only (D13). Use `DEEPSEEK_API_KEY`; do not rely on `login`.
- **Grove/clone config keys documented but inert.** The `clone` subcommand and
  `worktree redirect` commands do not exist (their guide pages were removed or
  replaced 2026-10-21), but `26-config-reference.md` still lists `[cli] grove`
  / `[clone]` keys and `DEEPSEEK_BUILD_CLONE` / `DEEPSEEK_BUILD_GROVE`.
- **Building from source needs `make`.** `tikv-jemalloc-sys` compiles jemalloc
  and its build script fails without `make` (observed 2026-10-21 on a host that
  had `cc` but no `make`).

## TUI (Step 2 checks, 2026-10-22)

- **Esc leaves picker text in the prompt.** Closing a `/model` or `/effort`
  picker with Esc keeps the typed command, so the next slash command is
  appended to it (`/model /effort`). Clear the prompt by hand.
- **No separate first-run notice.** A first launch shows only the welcome card
  ("Thanks for trying deepseek-build"); there is no one-time setup notice.

## Binary size (Step 3d, 2026-10-23)

- **Release profile left unchanged; LTO does not fit this build box.** Baseline
  `cargo build -j 2 --release --bin deepseek-build` at `61757a9c` (strip +
  panic=abort, no LTO): 152,670,552 bytes, 28m 38s.
  - `lto = "fat"` + `codegen-units = 1` at `-j 2`: rustc was killed by the OOM
    killer compiling `xai-grok-shell` (16 GB box).
  - `lto = "thin"` (default codegen-units) at `-j 1`: still compiling after the
    40-minute timeout (reached the `xai-grok-pager` crates, never linked); no
    size measured.
  - Retry thin or fat LTO on a machine with more RAM and cores, e.g. in CI via
    the existing `release-dist` profile (already thin LTO).

## Test-suite failures on the build box (Step 3e, 2026-10-23)

`cargo test --workspace -j 1` at `03393baa`: 31,901 passed, 8 failed, 388
ignored. One failure is a real upstream bug (pager reset overlay, being fixed on
`fe/fix-reset-overlay-dim`). The other 7 come from the box environment, not from
our changes:

- **3 `xai-grok-shell` credential tests fail when an API key is exported.**
  `agent::config::tests::has_own_credentials_guards_session_vs_external_key`,
  `agent::config::tests::process_key_from_model_env_key` and
  `sampling::error::tests::forbidden_subscription_error_no_hint_without_api_key`
  assume no key is set in the environment. They pass with the keys unset:
  `env -u DEEPSEEK_API_KEY -u XAI_API_KEY cargo test -p xai-grok-shell --lib -j 1 -- has_own_credentials_guards_session_vs_external_key process_key_from_model_env_key forbidden_subscription_error_no_hint_without_api_key`
- **`xai-grok-hooks` `runner::http::tests::validate_hook_url_accepts_and_rejects`
  (`http.rs:972`).** The box's fake-IP DNS resolves `nonexistent.invalid` to
  `198.18.0.1`, so the "unresolvable host" case resolves. Passes on normal DNS.
  Repro: `cargo test -p xai-grok-hooks --lib -j 1 -- validate_hook_url_accepts_and_rejects`
- **`xai-grok-sandbox` `devbox_genuine_reexec_applies_enforcement`.** bwrap
  re-exec can't create namespaces inside this container. Repro:
  `cargo test -p xai-grok-sandbox --test deny_paths_e2e -j 1 -- devbox_genuine_reexec_applies_enforcement`
- **2 `xai-proto-build` tests:** `debug_redact::tests::classifies_marked_fields_by_oneof_membership`
  and `debug_redact::tests::first_marked_field_finds_annotations_and_ignores_comments`.
  They look for protoc via `dotslash`, which isn't installed; CI uses apt protoc.
  Repro with system protoc:
  `PROTOC=/usr/bin/protoc cargo test -p xai-proto-build --lib -j 1`

### Ticket: SSRF guard does not block 198.18.0.0/15

`is_blocked_ip` in `xai-grok-hooks` does not treat `198.18.0.0/15` (RFC 2544
benchmarking range, used by fake-IP DNS resolvers) as blocked. A hook URL whose
host resolves there is allowed. Not an rc2 blocker; add the range to the
blocklist with a regression test in a follow-up.

## Licensing

- `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES`, and `third_party/NOTICE` are
  preserved. Upstream files are not relicensed. Legal sign-off (DECISIONS O1) is
  still recommended before any public distribution.
