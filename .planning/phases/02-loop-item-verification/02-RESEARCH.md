# Phase 2: Loop-Item Verification - Research

**Researched:** 2026-11-23
**Domain:** verification (not construction) of an existing Rust TUI + headless agent: permission prompt, headless fail-closed denial, `--permission-mode plan`, resume fidelity, plan-mode read-only gate, live egress, key hygiene, committed evidence.
**Confidence:** HIGH for every in-tree claim below (read this session, file:line cited). MEDIUM for anything host-dependent (live runs, `script(1)`, pty timing) — this planning host has no Rust toolchain.

---

## Summary

Phase 2 adds almost no product code. The tree already implements every loop item; what is missing is (a) **committed proof** that each item works on the real binary and (b) **two narrow defects** that a verification exposes. The evidence machinery from Phase 1 (`scripts/evidence.py`, `docs/evidence/README.md`, the fixed `report.json` schema) is complete and reusable; the pty harness already emits asciinema-v2 casts via `PtyHarness::write_cast` (`crates/codegen/xai-grok-pager-pty-harness/src/lib.rs:484`), so `asciinema` is not needed (and is absent here).

Three findings shape the plan more than anything else:

1. **The high-signal security warning is currently dropped for exactly the prompt PERM-01 wants.** The shell prepends `⚠ Security warning: …` to the ACP tool-call title (`crates/codegen/xai-grok-workspace/src/permission/prompter.rs:837-850`), but the pager **rebuilds the title for execute prompts** from the bash description / binary and throws `acp_title` away (`crates/codegen/xai-grok-pager/src/app/acp_handler/permissions.rs:198-232`). The rendered title row reads `state.title` only (`crates/codegen/xai-grok-pager/src/views/permission_view.rs:474-484`). So a cast of `rm -rf build` will show **no** warning until the warning is split into its own row (the fix the approved UI-SPEC already anticipates).
2. **PERM-04 is a doc correction, not a code change.** `apply_headless_permission_mode` really does send `SetSessionModeRequest` for `plan` before the prompt (`headless.rs:856-875`, called at `headless.rs:1277-1287`), and it only fires for the literal `"plan"`. `KNOWN-ISSUES.md:209-213` is stale; `.planning/codebase/CONCERNS.md:181` already says so.
3. **PERM/SESS items need no API key and no network.** The pty harness (`ContentController`) and the headless runner (`run_headless_in_sandbox_borrowed`) both drive the real binary against a loopback mock inference server with an isolated HOME. Only PROOF-01/02/03 need a key and outbound network. This means the loop-item half of the phase is deterministic and cheap; the live half is the only host-gated work.

**Primary recommendation:** Plan seven items in two lanes: (A) live proof — `scripts/smoke-test.sh` then `scripts/egress-check.sh` on the build host with the real key, then the leak scan; (B) loop items — write the two minimal fixes first (warning row; denial-copy alignment), then promote/extend pty + headless-mock tests into casts and `report.json` items. Every task states its precondition (toolchain, key, disk) and every runtime command is a named, documented line in `notes.md`.

---


<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Live proof and key handling
- PROOF-01/02 and PROOF-03 re-run the two existing real-session scripts rather than writing a third:
  `scripts/egress-check.sh` (already exercises shell, file edits in a scratch git repo, a real
  `web_fetch` over HTTPS, a local stdio MCP tool, headless JSON, a resume of the same session, a forced
  auto-compaction, and then the same shell + `web_fetch` session through `scripts/sandbox-run.sh`) and
  `scripts/smoke-test.sh`. -- **Reversibility:** one-way — the key is committed-by-policy, so any key
  material in an artifact is a security event that cannot be recalled; the run must be leak-scanned
  before anything is committed.
- The key is read only from `DEEPSEEK_API_KEY` / `DEEPSEEK_BUILD_API_KEY` in the process environment. It
  is never echoed, never written to a file, and never captured by `scripts/evidence.py`. `home` in
  `report.json` is `Linux/x86_64`-style system/arch only.
- PROOF-03's revocation step is a human action on platform.deepseek.com, recorded as a dated attestation
  in the item's `notes.md` (no key material, no account identifiers). There is no API to verify it, so
  the artifact records the author's confirmation rather than pretending to check it.
- `scripts/key-leak-check.sh --scan docs/evidence docs/...` runs over the whole committed tree and over
  every new artifact before the phase seals; it prints file names only.

#### Permission surface
- PERM-01 evidence is a pty scenario in `crates/codegen/xai-grok-pager-pty-harness/tests/` driving the
  real binary against the mock inference server, asserting the three canonical outcomes
  (`allow_once`, `allow_always`, `reject`), the three rendered labels, and the high-signal security
  warning text. Its `write_cast` output is the committed cast — the harness already emits asciinema v2
  and no `asciinema` binary is installed.
- PERM-02 is proven on both paths: a plain headless run (stderr) and a `--output-format json` run. The
  stderr hint already names both flags (`permission_cancel_hint` in
  `crates/codegen/xai-grok-pager/src/headless.rs`); the `Err(anyhow::anyhow!("permission-gated tool
  cancelled; pass --always-approve to approve it"))` path around line 1538 names only one. Aligning that
  message is the expected minimal fix; the test asserts the hint on both paths.
- PERM-03 asserts the status-line/plan-mode indicator before quit and again after `--resume` on the same
  session, in the same pty scenario family.
- PERM-04 follows the roadmap: verify that `apply_headless_permission_mode` really arms plan mode
  headless (it sends `SetSessionModeRequest` for `plan`), capture the transcript, and correct
  `KNOWN-ISSUES.md`. Do **not** add a second set-mode call; `.planning/codebase/CONCERNS.md` warns the
  tree already sends it.

#### Resume fidelity
- SESS-01/03 promote the existing `plan_approval_resume` scenario family
  (`crates/codegen/xai-grok-pager-pty-harness/src/scenarios/plan_approval_resume.rs` plus the test
  `tests/plan_approval_resume.rs`) into committed evidence: draft a plan, quit, `--resume`, assert model,
  reasoning effort and plan state are restored, and assert the drafted plan is intact.
- SESS-02 is a parameterized read-only assertion across `default`, `always-approve` and `auto`: in plan
  mode a write outside the plan file is refused while the plan file itself is writable.
- SESS-04 is documentation plus the artifact: a provenance table in the item's `notes.md` and the user
  guide stating which resumed setting comes from the session record and which from config.

#### Defect policy
- A defect a verification exposes is fixed minimally, in scope, with a regression test, and recorded in
  the plan SUMMARY — except where the roadmap says otherwise (`--permission-mode plan`: doc correction,
  not code).
- Anything outside the loop items is recorded in `KNOWN-ISSUES.md`, not fixed here.

#### Evidence
- One item directory per claim: `live-smoke` (PROOF-01), `runtime-egress` (PROOF-02), `key-leak`
  (PROOF-03), `permission-prompt` (PERM-01/02), `permission-mode` (PERM-03/04), `resume-fidelity`
  (SESS-01/03/04), `plan-read-only` (SESS-02). Each carries `<slug>.cast` where a pty is used,
  `report.json` built by `scripts/evidence.py`, and `notes.md` with the re-run command.
- `report.json` keeps the Phase 1 fixed key set and order, paths normalized to `<REPO>`/`<HOME>`/`<TMP>`.

#### Plan sequencing
- 02-01 and 02-02 are one live session captured twice (smoke transcript, then the egress measurement) and
  stay sequential; 02-03 scans what they produced. 02-04 and 02-05 touch different surfaces and may run
  in parallel worktrees only if their `files_modified` sets are disjoint, per the Phase 2 parallel-lane
  decision in STATE.md — dependent work stays sequential.

#### Decided autonomously
- [Evidence] Q: "New evidence tree or extend Phase 1?" → "New `docs/evidence/02-loop-item-verification/` tree; earlier items are never rewritten" — appending a directory is local and cheap to undo.
- [Live proof] Q: "Write a new live-session harness?" → "Reuse `scripts/egress-check.sh` and `scripts/smoke-test.sh`" — they already exercise the required loop; a new script would be untested scope.
- [Cast source] Q: "Use the `asciinema` CLI?" → "Use the pty harness `write_cast` (asciinema v2)" — it is already installed in-tree and the Phase 1 convention names it; no new dependency.
- [Scanning] Q: "New leak scanner?" → "`scripts/key-leak-check.sh` default mode over the tree plus `--scan` over artifacts" — the script already fails closed in `--scan` mode.
- [Key storage] Q: "Read the key from a file?" → "Environment only, never a file" — a file on disk is a second leak surface the scanner would have to trust.
- [PERM-04] Q: "Fix the plan-mode flag in code?" → "Verify and correct `KNOWN-ISSUES.md`" — the roadmap and CONCERNS.md both say the tree already sends the call.
- [Order] Q: "Run the loop items in any order?" → "PROOF-01 → PROOF-02 → PROOF-03, then PERM/SESS" — the scan must cover exactly the artifacts the live run produced.

### the agent's Discretion
- Exact item slugs beyond the seven named, the pty scenario file names, the wording of the security-warning
  assertion, whether SESS-02 is proven by three pty runs or one parameterized run, and the shape of the
  provenance table.

### Deferred Ideas (OUT OF SCOPE)
- Automatic model routing, per-model cost accounting, `/route` — v2 / open-build.
- `ci.yml` and debug-symbol sidecars — REL-V2-01/02.
- `IN-07` (`key-leak-check.sh` default-mode fail-open on grep rc≥2) — ledgered for v0.1.x; `--scan` fails
  closed, which is the mode this phase uses.
- Any new evidence tooling (asciinema CLI, a cast-to-gif renderer) — not needed; the harness emits v2.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PROOF-01 | Live smoke run on a real model-driven session passes; transcript committed | `scripts/smoke-test.sh` (8 checks, key+network preconditions) recorded as `live-smoke` item via `script -T` + `evidence.py cast` |
| PROOF-02 | Runtime egress on that session: provider + `web_fetch` host + AF_UNIX only, zero violations, measured | `scripts/egress-check.sh` prints its own activity assertions **and** the destination table; `EGRESS_VIOLATIONS=0` + `EGRESS-CHECK PASS` are the committed steps |
| PROOF-03 | `key-leak-check.sh` green over tree + artifacts; no key material anywhere; key revoked | default mode (fake key, no file) + `--scan docs/evidence` + tracked-tree scan; dated revocation attestation in `notes.md` |
| PERM-01 | Interactive prompt exercised end-to-end: allow-once, allow-for-session, reject, security warning | New pty test in `pty_e2e_shell_tools` family; label-driven row selection; **requires the warning-row fix** (see Pitfall 1) |
| PERM-02 | Headless denial fails closed with actionable hint naming `--always-approve` and `--permission-mode auto`, on stderr **and** JSON paths | One shared flags constant in `headless.rs`; unit test asserts both strings; new headless-against-mock integration test asserts exit 1 + JSON `stopReason:"cancelled"` |
| PERM-03 | Permission mode visible in UI and matches after resume | pty scenario asserts chip pair pre-quit and post-`--continue`; note provenance: permission chip is config/CLI, plan chip is session |
| PERM-04 | `--permission-mode plan` verified to arm plan mode headless; `KNOWN-ISSUES.md` corrected | `headless.rs:856-875` + call site `1277-1287`; mock-server request body carries the plan-mode reminder; doc fix at `KNOWN-ISSUES.md:209-213` |
| SESS-01 | Resume restores model, reasoning effort, plan state | `session_setup.rs:1675-1727` restores `current_model_id` + `reasoning_effort`; `plan_mode.json` via `persistence.rs:2155-2159` |
| SESS-02 | Plan mode read-only except plan file in every mode incl. `always-approve`, `auto` | Gate runs **before** permission resolution: `tool_calls.rs:1655-1670`, pure predicate `tool_calls.rs:170-190` |
| SESS-03 | Drafted plan survives quit and resume, promoted to committed evidence | `scenarios/plan_approval_resume.rs` already drafts/seeds plan.md, quits, resumes; add cast emission + label assertions |
| SESS-04 | Provenance of each resumed setting documented (session vs config) | `event_loop.rs:1098-1103,1504-1518` (permission = config/CLI); `settings/defs.rs:1109-1127` + `registry.rs:1269-1276` (pager plan toggle not persisted); shell `plan_mode.json` (session) |
</phase_requirements>

## Project Constraints (from AGENTS.md)

Actionable directives from `AGENTS.md` (repo root), which the plan must comply with:

- **Toolchain:** Rust **1.94.0** via rustup, pinned in `rust-toolchain.toml` — do not bump. Keep `cargo fmt --all --check` clean and clippy at 0 errors (16 pre-existing warnings).
- **Build hygiene:** run `scripts/disk-guard.sh` before any workspace build and stop if it fails (<150 GB for the gate); wrap heavy builds/tests in `flock -w 900 /tmp/team-build.lock`; use **one shared** `CARGO_TARGET_DIR`; do not raise debug/incremental settings (`.cargo/config.toml` sets `[build] incremental = false`).
- **Secrets:** the key comes from `DEEPSEEK_API_KEY` / `DEEPSEEK_BUILD_API_KEY` only; never echo, log, save to a session, or commit it; redact on sight; `scripts/key-leak-check.sh` green before commit.
- **Egress invariants (D6/D10/D13):** the only outbound destinations are the configured provider and user-configured MCP servers; **a change that adds an outbound destination is stop-and-ask**; deleted telemetry/upload/voice/computer-hub paths stay deleted (real deletions only, C4).
- **Branding (D1/D2):** user-visible strings resolve through `xai-grok-brand`; `xai-grok-*` crate names and internal `GROK_*` env names stay (out of scope). After touching CLI help/version output, run the `branding_surface` integration test — Phase 2 touches neither.
- **Prompt templates:** any edit under `crates/codegen/xai-grok-agent/templates/` requires `python3 scripts/encrypt_templates.py` and a green `test_encrypted_templates_not_stale` — Phase 2 must not touch templates.
- **Config reference:** `crates/codegen/xai-grok-pager/docs/user-guide/26-config-reference.md` is hand-edited; Phase 2 adds/removes **no** config key, so it must not regenerate or edit it. `config_docs` stays green.
- **Clippy bans:** use `dunce::canonicalize`, build HTTP clients through `xai_grok_extra_ca`, enroll children via `xai_tty_utils::ProcessScope`, resolve home via `xai-dirs`; tests may index/slice freely, production crates opt into `#![deny(clippy::indexing_slicing)]`.
- **Commits/docs:** small logical commits, one change each (C1); green build+tests after every merge (C3); update `KNOWN-ISSUES.md` when status changes; `DECISIONS.md` wins on conflicts.
- **Stop-and-ask conditions:** a gate fails twice, a decision is not in `DECISIONS.md`, a change would add an outbound destination, or the live smoke needs a key that is not provided.

---

## First-Class Environment Finding (blocking for live items only)

Verified on **this** planning host this session:

| Probe | Result |
|---|---|
| `command -v cargo` / `rustc` | `NO_CARGO` / `NO_RUSTC` |
| `ls target` | `NO_TARGET` (no build artifacts at all) |
| `df -h .` | `/dev/mmcblk1p3 52.0G 18.4G 31.0G 37% /` — 31 GB free (below `scripts/disk-guard.sh`'s 60 GB default and Phase 1's 150 GB gate) |
| `free -g` | 4 GB total, ~1 GB available |
| `DEEPSEEK_API_KEY` | unset |
| `command -v gh` | absent |
| `docker` / `podman` | binaries present, no usable engine (brief: not usable) |
| `command -v asciinema` / `script` / `strace` | all absent |
| `python3` / `flock` / `rg` / `git` / `cc` | present (Python 3.14.8, git 2.54.0) |

**Consequence:** no live run, no build, no pty test can execute here. Every plan task that compiles or runs the binary must state, in the task text, that it executes **on the author's build host** with: Rust 1.94.0 (pinned), `protoc`, `cc`, `python3`, `script(1)` (util-linux) for the `script -T` items, a shared `CARGO_TARGET_DIR`, ≥150 GB free (`scripts/test-gate.sh:59-63` refuses below 150), and `DEEPSEEK_API_KEY` exported **only** for the live lane. Do not substitute `asciinema`, `strace`, or `script(1)` alternatives — the in-tree substitutes are `PtyHarness::write_cast` and `scripts/egress-check.sh`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|---|---|---|---|
| Live session + egress measurement | Dev-script tier (`scripts/egress-check.sh`, `scripts/smoke-test.sh`) driving the release binary | External provider (`api.deepseek.com`) + `web_fetch` host | The measurement must wrap a real process; an in-process test cannot observe `connect()` for children |
| Permission decision (allow/deny/ask) | Shell/agent runtime (`xai-grok-shell` + `xai-grok-workspace` permission manager) | Workspace rules/config | The decision, classifier and rules live below the client; the client only renders and answers |
| Permission prompt + warning row | TUI render tier (`xai-grok-pager` views) | — | `build_permission_display` + `permission_view.rs` own the rendered rows; the warning is a render concern |
| Headless denial copy | CLI/headless tier (`xai-grok-pager/src/headless.rs`) + `main`'s `Error:` prefix | — | Headless is the client; it owns its own stderr/JSON contract |
| `--permission-mode plan` arming | CLI → ACP `SetSessionModeRequest` (pager) → shell plan tracker | — | The flag only lands on the agent definition otherwise (`headless.rs:849-855` comment) |
| Plan-mode read-only gate | Shell session (`acp_session_impl/tool_calls.rs` before permission) | — | Must hold in every permission mode, so it cannot live in the permission layer |
| Resume fidelity (model/effort/plan) | Shell persistence (`session/persistence.rs`, `plan_mode.json`) | — | Session record is the only source that survives a process restart |
| Permission chip after resume | Pager config resolution (`event_loop.rs`) | `[ui] permission_mode` in config | Provenance is config, not the session record — SESS-04 must say so |
| Evidence capture | Dev-script + test tier (`scripts/evidence.py`, pty harness) | `docs/evidence/` tree | Phase 1 convention; `report.json` is the artifact a reviewer reads |

---

## Standard Stack

No new dependency of any kind. This phase adds no crate, no script, no CLI tool, and no outbound destination.

### Core (existing, reused)

| Component | Version / location | Purpose | Why standard here |
|---|---|---|---|
| `scripts/evidence.py` | 490 lines, 7 subcommands | `begin`/`step`/`artifact`/`finish`/`cast`/`normalize` recorder | Phase 1 convention; `finish --result ok` refuses over failed/missing steps (`evidence.py:230-268`) |
| `scripts/key-leak-check.sh` | 116 lines | canary run + `--scan <path>...` | `--scan` fails closed on grep rc≥2 and prints file names only (`key-leak-check.sh:61-82`) |
| `scripts/egress-check.sh` | 565 lines | real-session LD_PRELOAD egress audit | Already exercises shell + edit + web_fetch + MCP + resume + compaction and prints `EGRESS_VIOLATIONS=` |
| `scripts/smoke-test.sh` | 177 lines | 8 live adapter checks | Reuses the adapter proof; prints `N passed, M failed` |
| `xai-grok-pager-pty-harness` | in-tree | PTY driver + `ContentController` mock server + `write_cast` | The only cast producer in-tree; emits asciinema v2 (`lib.rs:484-514`) |
| `xai-grok-test-support` | in-tree | `MockInferenceServer`, `run_headless_in_sandbox_borrowed`, `TestSandbox` | Lets a headless test run the real binary with no key and no network |
| Rust `libtest` + `tokio::test` | workspace | all unit/integration tests | Existing infrastructure; nothing to install |

### Supporting

| Component | Version / location | Purpose | When to Use |
|---|---|---|---|
| `xai-grok-shell --features config-docs` | in-tree | config-reference sync | Only if the phase touches registries — it does not (no new key) |
| `scripts/test-gate.sh` | 325 lines | completeness gate | Phase-boundary check; also runs the leak scan over `docs/evidence` (`test-gate.sh:298-300`) |
| `scripts/disk-guard.sh 150` | 24 lines | disk precondition | Every heavy build (`test-gate.sh:59-63` already enforces 150) |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|---|---|---|
| `script -T` + `evidence.py cast` (headless items) | pty harness cast for everything | `script(1)` is the Phase 1 headless convention and records the real command line; the pty path is for TUI. Do not mix. |
| `key-leak-check.sh --scan` | ad-hoc `grep -R sk-` | The script fails closed and prints names only; an ad-hoc grep fails open and is unreviewable. |
| Headless-mock test for PERM-02 | pty test | pty cannot produce a `-p` run; the denial only exists headless. |
| pty cast for SESS-02 | headless `streaming-json` tool-result capture | pty matches the approved UI-SPEC ("assert the string in the cast"); headless is a cheaper *automated* companion, not a replacement for the artifact. |

**Installation:** none.

## Package Legitimacy Audit

No external package is added, upgraded, or removed in this phase. `Package Legitimacy Gate` not applicable (no install). Any plan task that proposes a new dependency or a new outbound destination is out of scope by D6/D10/D13 and must stop.

---

## Architecture Patterns

### System Architecture Diagram — where each loop item is proven

```
                         ┌──────────────────────────────┐
  real key (env only) ──▶│ scripts/smoke-test.sh        │──▶ stdout PASS/FAIL ──┐
                         │  8 adapter checks            │                      │
                         └──────────────────────────────┘                      │
                         ┌──────────────────────────────┐                      ▼
  real key (env only) ──▶│ scripts/egress-check.sh      │──▶ activity PASS lines + destination table
                         │  real session: shell/edit/   │     EGRESS_VIOLATIONS=0
                         │  web_fetch/MCP/resume/compact│                      │
                         │  + sandbox-run comparison    │                      │
                         └──────────────────────────────┘                      │
                                                                               ▼
  ┌────────────────────────────────────────────────────────────┐   ┌─────────────────────────┐
  │ pty harness (real binary, mock inference, isolated HOME)   │   │ script -T typescript    │
  │  PTY keystrokes → TUI frames → screen assertions → cast    │   │ → evidence.py cast      │
  └────────────────────────────────────────────────────────────┘   └─────────────────────────┘
  ┌────────────────────────────────────────────────────────────┐               │
  │ headless runner (real binary -p, mock inference, sandbox)  │               ▼
  │  stdout(stderr) assertions on denial, plan arming, resume  │    ┌──────────────────────┐
  └────────────────────────────────────────────────────────────┘    │ evidence.py begin    │
                                                                    │  step/artifact       │
  key-leak-check.sh (default + --scan) ───────────────────────────▶│  finish → report.json│
                                                                    └──────────────────────┘
                                                                               │
                                                                               ▼
                                                        docs/evidence/02-loop-item-verification/<item>/
                                                          <slug>.cast · report.json · notes.md · *.log
```

Data flows one way: a real process (or the mock-backed real binary) produces output → output is captured
as a cast or a step log → `evidence.py` records real exit statuses → `finish` refuses to overstate →
the leak scan runs over the committed tree.

### Recommended Project Structure (files this phase touches or adds)

```
docs/evidence/02-loop-item-verification/
├── live-smoke/            live-smoke.cast, report.json, notes.md
├── runtime-egress/        runtime-egress.cast, report.json, notes.md
├── key-leak/              report.json, notes.md (+ step logs; no cast required)
├── permission-prompt/     permission-prompt.cast, report.json, notes.md
├── permission-mode/       permission-mode*.cast, report.json, notes.md
├── resume-fidelity/       resume-fidelity*.cast, report.json, notes.md
└── plan-read-only/        plan-read-only*.cast, report.json, notes.md

crates/codegen/xai-grok-pager/
├── src/headless.rs                    # PERM-02 copy fix (one shared const)
├── src/headless_tests.rs              # assert both hint strings
├── src/app/acp_handler/permissions.rs # PERM-01 warning split
├── src/views/permission_view.rs       # PERM-01 warning row + chrome height
└── tests/…                            # (only if a pager-local test is preferred)

crates/codegen/xai-grok-pager-pty-harness/
├── src/scenarios/<new|extended>.rs    # scenarios return Result<()>, may write casts
└── tests/pty_e2e/<new>.rs             # register in tests/pty_e2e_shell_tools.rs (or a new [[test]])
```

### Pattern 1: Evidence item for a script (headless / live) — `script -T`

Established in Phase 1 and documented at `docs/evidence/README.md:79-97`. Shape:

```sh
ITEM=docs/evidence/02-loop-item-verification/runtime-egress
scripts/evidence.py begin --item-dir "$ITEM" --item runtime-egress \
    --requirements PROOF-02 --command "scripts/egress-check.sh (captured with script -T)"
script -qec 'scripts/egress-check.sh' -T "$ITEM/run.timing" "$ITEM/run.typescript"
scripts/evidence.py cast --typescript "$ITEM/run.typescript" --timing "$ITEM/run.timing" \
    --out "$ITEM/runtime-egress.cast"
scripts/evidence.py step --item-dir "$ITEM" --name "egress: zero violations" -- \
    sh -c "grep -F 'EGRESS_VIOLATIONS=0' '$ITEM/run.typescript'"
scripts/evidence.py artifact --item-dir "$ITEM" --path "$ITEM/runtime-egress.cast"
scripts/evidence.py finish --item-dir "$ITEM" --result ok
```

Notes that matter:
- `evidence.py step` **runs** its command and records the real exit status; it merges stdout+stderr into
  `<name-slug>.log` and saves the record before echoing (`evidence.py:162-209`). Use `grep -F` (not `-q`)
  so the step `detail` carries the matched line, as Phase 1's items do.
- The raw `run.typescript` / `run.timing` are intermediates — Phase 1 does not commit them.
- `finish` normalizes step logs and recomputes artifact digests **after** normalization
  (`evidence.py:270-287`); casts produced by `evidence.py cast` are already normalized.
- `report.json` key set/order is fixed: `item, requirements, date, host, command, steps[{name,ok,detail}],
  artifacts[{path,sha256}], result` (`evidence.py:12-23`, `docs/evidence/README.md:25-34`).
- `host` is `platform.system()/machine()` — never a hostname or user (`evidence.py:153`).

### Pattern 2: Evidence item for a pty scenario — `write_cast`

`PtyHarness::write_cast` writes asciinema v2 (header `{"version":2,"width":cols,"height":rows}` then
`[elapsed,"o",data]` records, `lib.rs:484-514`). The tests-only helper
`write_cast_if_requested(harness, "<slug>.cast")` reads `GROK_PTY_CAST_DIR`
(`tests/pty_e2e/common.rs:1560-1575`) and is non-fatal on failure by design.

```sh
ITEM=docs/evidence/02-loop-item-verification/permission-prompt
scripts/evidence.py begin --item-dir "$ITEM" --item permission-prompt \
  --requirements PERM-01 \
  --command "GROK_PTY_CAST_DIR=$ITEM cargo test -p xai-grok-pager-pty-harness --test pty_e2e_shell_tools -- --ignored --nocapture permission_prompt_outcomes_and_warning"
# The step IS the ignored test run: its real exit status is what gets recorded.
GROK_PTY_CAST_DIR="$ITEM" scripts/evidence.py step --item-dir "$ITEM" \
  --name "pty: three outcomes, three labels, security warning" -- \
  cargo test -p xai-grok-pager-pty-harness --test pty_e2e_shell_tools -- \
  --ignored --nocapture permission_prompt_outcomes_and_warning
scripts/evidence.py normalize --path "$ITEM/permission-prompt.cast"
scripts/evidence.py artifact --item-dir "$ITEM" --path "$ITEM/permission-prompt.cast"
scripts/evidence.py finish --item-dir "$ITEM" --result ok
```

Running the test **inside** `evidence.py step` means the recorded exit status is the test's own, and the
step log carries libtest's `test result:` line. (The `--` before `cargo` is required: the step parser
strips a leading `--` and treats the rest as the command, `evidence.py:166-170`.) The cast and the step
therefore come from the same execution, which is Phase 1's rule (STATE.md:67).

Cast normalization is mandatory: the harness writes raw screen bytes, which can contain the sandbox temp
path. `scripts/evidence.py normalize --path "$ITEM/<slug>.cast"` substitutes `<REPO>/<HOME>/<TMP>` in the
text. (The cast is text JSONL; the README's text-only caveat applies, `docs/evidence/README.md:62-66`.)

### Pattern 3: Headless test against the mock server (no key, no network)

`xai-grok-test-support` gives the whole recipe and it is already used by
`crates/codegen/xai-grok-pager-bin/tests/header_capture.rs:11-32`:

```rust
let server = MockInferenceServer::start_with_models(vec![MockModelEntry::new("deepseek-v4-pro")]).await?;
server.set_response("…");
let out = run_headless(&server, &["-p", "…", "--output-format", "json"], &cwd).await;
// out.status, out.stdout, out.stderr, out.timed_out
```

For a tool-call turn, script the SSE tool call on **both** backends the way the pty tests do
(`tests/pty_e2e/common.rs:734-747`): `ScriptedResponse::sse(responses_api_tool_call_events(...))` and
`ScriptedResponse::sse(chat_completions_tool_call_events_with_id(...))`, registered through
`content.expect_agent_turn_with_responses(...)`. To run the binary **in the same sandbox** (same mock URL,
same HOME, same fake key) use `run_headless_in_sandbox_borrowed(cmd, content.sandbox())`
(`xai-grok-test-support/src/headless.rs:75-88`).

This is the cheapest honest way to prove PERM-02, PERM-04 and the SESS-01 half of resume: real binary,
real ACP, real permission manager, deterministic, no key.

### Pattern 4: Answer a permission prompt without hard-coded digits

Row order and numbering are not stable: `resolve_initial_cursor` may put the cursor on the
always-approve row by identity (`permission_cursor.rs:161-183`), and the sticky last-used kind steers the
next prompt. The working in-tree pattern finds the option number on screen and navigates to it:

```rust
let screen = self.harness.screen_contents();
let row = screen.lines().find(|l| l.contains(REJECT_ROW)).expect("reject row");
let index = row.split_whitespace().find_map(|w| w.parse::<usize>().ok()).expect("displayed option number");
let keys = format!("{}{}", "\x1b[A".repeat(9), "\x1b[B".repeat(index - 1));
self.harness.inject_keys(keys.as_bytes())?;
self.harness.wait_until("reject option selected", …,
    |h| h.screen_contents().lines().any(|l| l.contains(REJECT_ROW) && (l.contains("(●)") || l.contains("(•)"))))?;
self.harness.inject_keys(b"\r")?;
```

(`tests/pty_e2e/configured_bash_filename_arguments.rs:131-161`.) Then confirm the outcome by waiting for
the tool's sentinel text (allowed) or the `permission was denied` banner (rejected), as that test does
at `:162-171`.

**UI-SPEC contradiction to resolve in the plan:** §Verification Hooks says "Drives `1` (allow-once), `2`
(allow-for-session), `4` (reject)", while the row order table in the same document puts
`enable-always-approve` first (so allow-once is row 3). Choose label-driven navigation and never assert a
digit.

### Anti-Patterns to Avoid

- **Asserting a digit for a permission row.** Cursor stickiness and the always-approve row make numbering
  vary; parse the label line.
- **Adding a second `SetSessionModeRequest` for `plan`.** The tree already sends one
  (`headless.rs:1277-1287`); CONCERNS.md:181 and FEATURES A11 both warn.
- **Adding a hint field to the JSON result.** `build_json_result` emits exactly `text, stopReason,
  sessionId, requestId` (+ optional `thought`, `usage`, cost, structured output)
  (`headless.rs:302-328`). A new key is a new product surface and is out of scope (UI-SPEC §Copywriting).
- **Committing the egress-check debug logs.** They contain scratch paths and prompt text; the cast and
  the parsed step details are the evidence.
- **Grepping `.` with `key-leak-check.sh --scan`.** `grep -R` would descend into `target/`; pass the
  tracked paths (`docs/evidence`, and a `git ls-files` list).
- **Running a live item with `set -x` or typing the `export` inside the `script` capture.** Both put the
  key into the typescript.
- **Treating `--permission-mode always-approve` as valid.** `--permission-mode` accepts only
  `default | acceptEdits | auto | dontAsk | bypassPermissions | plan` (`app/cli.rs:590-597` +
  `xai-grok-agent/src/config.rs:910-917`). That is the whole reason PERM-02's current text is wrong.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---|---|---|---|
| Recording a run as evidence | A new report format or hand-written JSON | `scripts/evidence.py begin/step/artifact/finish` | Fixed schema, real exit codes, refuses to overstate, normalizes paths |
| Emitting an asciinema cast | `asciinema`, `script`, a cast converter | `PtyHarness::write_cast` / `write_cast_if_requested` | Installed in-tree; `asciinema` absent on both hosts |
| Reproducing the provider | A new fake HTTP server | `MockInferenceServer` / `ContentController` | Supports both wire shapes, SSE scripting, request capture, bogus-key auth baseline |
| Measuring egress | `strace` (absent), `tcpdump`, a new shim | `scripts/egress-check.sh` (+ `scripts/sandbox-run.sh` comparison) | LD_PRELOAD shim logs every connect; blind spots documented in its header |
| Scanning for key material | A new regex | `scripts/key-leak-check.sh` (default + `--scan`) | Fails closed on scan error, prints names only |
| Proving plan-mode read-only | A new gate | The shell gate at `tool_calls.rs:1655-1670` | It runs before permission resolution, so it holds in every mode by construction |
| Proving resume fidelity | A custom session format reader | `session_setup.rs:1675-1727` + `plan_mode.json` | Those are the real restore paths |

**Key insight:** everything this phase needs to *measure* already exists; the work is choosing the
right seam and recording its real output. Any new harness would be untested scope and a second source of
truth for "what happened".

---

## Runtime State Inventory

Not a rename/refactor phase. But resume and plan-mode evidence depend on **runtime state on disk**, and
the plan must name it correctly:

| State | Where it lives | Phase action |
|---|---|---|
| Session summary (model id, reasoning effort, context window) | session summary JSON under the session dir; written by `PersistenceMsg::CurrentModel` (`persistence.rs:260-267`, `2125-2144`) | Read-only; assert restore path |
| Plan mode lifecycle | `<session-dir>/plan_mode.json` (`storage/mod.rs:32`; snapshot `plan_mode.rs:54-66`) | Read-only; assert survives quit/resume |
| Plan file | `<session-dir>/plan.md` (`plan_mode.rs:69-79`) | Read-only; SESS-03 asserts content survives |
| Permission mode | **config**, `[ui] permission_mode` (`event_loop.rs:1098-1103`, `1504-1518`) | SESS-04 must state it is *not* in the session record |
| Pager `plan_mode` toggle setting | pager-local, "per-session, not persisted" (`settings/registry.rs:1269-1276`; `defs.rs:1109-1127`) | Do not claim it persists; the chip comes from the shell snapshot |
| Test-sandbox HOME | `DEEPSEEK_BUILD_HOME=<sandbox home>/.grok` (`xai-grok-test-support/src/sandbox.rs:212,325`) | Evidence notes must show the real per-item path, not assume `~/.deepseek-build` |

---

## Common Pitfalls

### Pitfall 1: The security warning never paints for a shell prompt (PERM-01 blocker)

**What goes wrong:** a pty cast of `rm -rf build` shows `Allow \`rm\`?` / `rm -rf build` and the three
rows, but no `⚠ Security warning:` line, so PERM-01's evidence fails.
**Why:** `with_security_warning` prefixes `fields.title` with `"{warning}\n{title}"`
(`prompter.rs:837-850`), but `build_permission_display` treats any execute prompt as `is_execute` and
rebuilds the title from `bash_input.description` or `Allow \`<bin>\`?`, discarding `acp_title`
(`app/acp_handler/permissions.rs:198-232`). The view renders only `state.title`
(`views/permission_view.rs:474-484`); nothing else carries the warning.
**How to avoid:** fix at the seam the UI-SPEC prescribes — split the first line when it starts with
`⚠ Security warning:` into its own `PermissionViewState` field, render it above the title in
`theme.warning` (normal weight), and add `+1` to `permission_chrome_height` when present
(`views/permission_view.rs:307-329`; note the height math does **not** already account for the row —
it must be incremented). Keep the copy byte-exact; add a unit test beside
`prompter.rs::security_warning_is_prepended_for_high_signal_findings_only` and a pty assertion on the
substring `Security warning:`.
**Warning signs:** unit test at `prompter.rs:1075-1108` passes while the pty screen lacks the row — that
is exactly the current state.

### Pitfall 2: Two denial strings drift (PERM-02)

**What goes wrong:** the stderr hint names both flags (`headless.rs:492-494`) but the final error names
only `--always-approve` (`headless.rs:1538-1540`); a future edit to one leaves the other behind.
**Why:** two independent literals, no shared constant, no test on the text (the existing test only
asserts `Some/None`, `headless_tests.rs:1014-1043`).
**How to avoid:** one `const` for the flags sentence ("`--always-approve, or choose a non-interactive
permission mode such as --permission-mode auto`"), used by both the hint and the `anyhow!` payload
(preserving the capitalised `Re-run` vs lower-case `re-run` at the call sites), plus assertions
`contains("--always-approve") && contains("--permission-mode auto")` on both rendered strings.
**Warning signs:** UI-SPEC's required error line (`Error: permission-gated tool cancelled; re-run with …`)
not byte-matched.

### Pitfall 3: Assuming `--permission-mode always-approve` exists

**What goes wrong:** a "fix" suggests the mode flag for bypass; clap rejects it.
**Why:** `--permission-mode` is gated by `PermissionMode::VALID_VALUES`
(`app/cli.rs:590-597` → `config.rs:910-917`); always-approve is a separate `--always-approve` flag.
**How to avoid:** copy the UI-SPEC lines exactly; never suggest a config key or `/command` in the denial.
**Warning signs:** any denial text containing `--permission-mode always-approve`.

### Pitfall 4: Claiming the permission mode "survives resume"

**What goes wrong:** PERM-03 is written as "always-approve survives resume" and fails, or is "proved" by
a test that passes only because the flag was passed again.
**Why:** the permission chip is resolved from CLI > `[ui] permission_mode` > default Ask each launch
(`event_loop.rs:1098-1103,1504-1518`); only the **plan** chip comes from the session record.
**How to avoid:** run the pre-quit and post-resume launches with the *same* effective config (no CLI
override on either, or the same one), and assert the pair matches. Put the provenance in SESS-04's
`notes.md` so the reader sees why.
**Warning signs:** a resume command that repeats `--permission-mode …`.

### Pitfall 5: Treating "the same session" as trivially true across the two live scripts

**What goes wrong:** PROOF-01 and PROOF-02 are written as if one `smoke-test.sh` run were the measured
session; it is not.
**Why:** `smoke-test.sh` and `egress-check.sh` are separate scripts and separate sessions. Only
`egress-check.sh` measures egress, and it prints its own activity assertions (`shell-egress`, `edited`,
MCP marker, `RESUMED`, compaction) for the session it measured (`egress-check.sh:329-366`).
**How to avoid:** state in `runtime-egress/notes.md` that the measured session is the egress-check
session and its activity proof is in the same cast; state in `live-smoke/notes.md` that the smoke script
is the separate adapter smoke. Both are honest and both are required by CONTEXT.
**Warning signs:** a `notes.md` that says egress was measured over the smoke run.

### Pitfall 6: `script -T` and the key leaking into the typescript

**What goes wrong:** the key appears in the captured cast/log because the `export` line was typed inside
the captured session or the shell had `set -x`.
**Why:** `script` captures everything the pty writes, including echoed input.
**How to avoid:** export the key in the parent shell before `script`; run only the script inside
`script -qec '…'`; never `set -x`; then `scripts/key-leak-check.sh --scan` the item **before** committing.
**Warning signs:** the leak scan names a file under the item dir.

### Pitfall 7: The pty tests are `#[ignore]`d, so the gate does not run them

**What goes wrong:** "green gate" is read as "pty evidence passed".
**Why:** pty cases carry `#[ignore = "PTY e2e; …"]` and are only run with `--ignored`
(`tests/pty_auto_mode.rs:106`, `tests/pty_e2e_shell_tools.rs:1-2`); `scripts/test-gate.sh` runs
`cargo test --workspace` (`test-gate.sh:1-13`), which skips them.
**How to avoid:** the evidence item's `step` must be the explicit `--ignored` run, and the plan must say
the gate is a separate, complementary check.
**Warning signs:** a SUMMARY claiming PERM-01 was verified by `test-gate.sh`.

### Pitfall 8: Writing a command into the plan that this host cannot run

**What goes wrong:** a task runs `cargo build` here and dies.
**Why:** no toolchain, 31 GB disk, 3.7 GB RAM (see Environment Finding).
**How to avoid:** every runtime task is labelled "build host" with its preconditions; the planning host
host can only write files, run `python3`, `git`, `rg`, `sh -n`/`bash -n` (no `shellcheck` here), and
review diffs.
**Warning signs:** a task's `<automated>` command does not say where it runs.

---

## Code Examples

### PERM-02: the required copy (byte-exact, from `02-UI-SPEC.md` §Copywriting)

```
Turn ended: a permission-gated tool was cancelled because headless mode cannot prompt for approval. Re-run with --always-approve, or choose a non-interactive permission mode such as --permission-mode auto.
Error: permission-gated tool cancelled; re-run with --always-approve, or choose a non-interactive permission mode such as --permission-mode auto.
```

Current sources (verified):

```rust
// crates/codegen/xai-grok-pager/src/headless.rs:490-497
match cancellation_category {
    Some(c) if c == PERMISSION_CANCELLED_CATEGORY || c == PERMISSION_REJECTED_CATEGORY => Some(
        "Turn ended: a permission-gated tool was cancelled because headless mode cannot prompt \
         for approval. Re-run with --always-approve, or choose a non-interactive permission \
         mode such as --permission-mode auto.",
    ),
    _ => None,
}

// crates/codegen/xai-grok-pager/src/headless.rs:1533-1541
} else if let Some(hint) =
    permission_cancel_hint(&stop_reason, cancellation_category, options.yolo)
{
    emitter.on_end(&stop_reason, sid, rid);
    eprint_line(hint);
    Err(anyhow::anyhow!(
        "permission-gated tool cancelled; pass --always-approve to approve it"
    ))
}
```

`main` renders the `anyhow!` payload as `Error: {error}` (`xai-grok-pager-bin/src/main.rs:1745`), so the
required rendered line follows from the required message with no other change.

### PERM-04: the proof target (already implemented)

```rust
// crates/codegen/xai-grok-pager/src/headless.rs:856-875
async fn apply_headless_permission_mode(
    acp_tx: &AcpAgentTx,
    session_id: &acp::SessionId,
    permission_mode_flag: Option<&str>,
) -> anyhow::Result<bool> {
    if permission_mode_flag != Some("plan") {
        return Ok(false);
    }
    acp_send(
        acp::SetSessionModeRequest::new(
            session_id.clone(),
            acp::SessionModeId::new(xai_grok_tools::types::SessionMode::Plan.as_id()),
        ),
        acp_tx,
    )
    .await
    .map_err(|e| anyhow::anyhow!("Couldn't enter plan mode: {e}"))?;
    tracing::debug!("headless: plan mode requested via --permission-mode plan");
    Ok(true)
}
```

Called before the prompt at `headless.rs:1277-1287`. Unit proof already exists:
`headless_tests.rs:1045-1087` asserts exactly one `SetSessionMode` with id `plan`.

### PERM-02: fail-closed answer (verified)

```rust
// crates/codegen/xai-grok-pager/src/headless.rs:1939-1958 (verbatim)
AcpClientMessageBox::RequestPermission(req) => {
    if yolo {
        if let Some(resp) = auto_respond_to_permissions(
            &req.request,
            &[
                acp::PermissionOptionKind::AllowOnce,
                acp::PermissionOptionKind::AllowAlways,
            ],
        ) {
            let _ = req.response_tx.send(Ok(resp));
        } else {
            let _ = req.response_tx.send(Ok(acp::RequestPermissionResponse::new(
                acp::RequestPermissionOutcome::Cancelled,
            )));
        }
    } else {
        let _ = req.response_tx.send(Ok(acp::RequestPermissionResponse::new(
            acp::RequestPermissionOutcome::Cancelled,
        )));
    }
}
```

### SESS-02: the gate is above permissions (verified)

```rust
// crates/codegen/xai-grok-shell/src/session/acp_session_impl/tool_calls.rs:1655-1670
let plan_gate = plan_mode_edit_gate(&self.plan_mode.lock(), &tool_input, &access_kind);
if plan_gate != PlanEditGate::Allow {
    …
    let msg = self.plan_mode_edit_rejected_message().await;
    self.handle_tool_not_executed(&call.id, &tool_call_id, msg).await?;
    return Ok(Err(ToolLoop::Continue));
}
```

`plan_mode_edit_gate` allows only `should_auto_approve_edit` (exact plan-file path) and rejects
`ApplyPatch` outright (`tool_calls.rs:170-190`). The rejection text template is
`plan_mode.rs:328-333`; the later permission resolution begins at `tool_calls.rs:1671+`, so
`always-approve`/`auto` cannot bypass it. Existing unit coverage:
`acp_session_tests/plan_mode_edit_gate_tests.rs:41-64` ("despite allow all permissions").

### Resume restore path (verified)

```rust
// crates/codegen/xai-grok-shell/src/agent/mvp_agent/session_setup.rs:1695-1717
let restore_effort = initial_reasoning_effort.or(summary.reasoning_effort);
…
crate::agent::handlers::model_switch::apply(
    self,
    acp::SetSessionModelRequest::new(session_id.to_owned(), model_id),
    crate::agent::handlers::model_switch::SwitchEffort::Set(restore_effort),
    restore_window,
    crate::agent::handlers::model_switch::ConfigNotice::Skip,
).await
```

---

## State of the Art

| Old approach | Current approach | When changed | Impact |
|---|---|---|---|
| Committed evidence by hand ("see screenshot") | `evidence.py` fixed schema + real exit codes + leak scan | Phase 1 (2026-11-21/23) | A report cannot overstate; `finish` refuses over failed/missing steps |
| `asciinema rec` for casts | pty harness `write_cast` emits v2 | Phase 1 convention | No external binary; deterministic geometry |
| `script -T` for headless items | unchanged | — | Still the convention; needed for commands with no TUI (`docs/evidence/README.md:99`) |
| Permission prompt checked by eye | pty screen assertions | existing suite | Label-driven, sticky-cursor aware |

**Deprecated/outdated in this tree (do not repeat):**
- `KNOWN-ISSUES.md:209-213` — "`--permission-mode plan` … does not send `session/set_mode`". Stale; the
  tree sends it. Correct this line (PERM-04), do not "fix" the code.
- `TESTING.md:93` — the same claim in the manual-test table. It belongs to DOCS-02/03 (Phase 5); record
  the drift for that phase rather than editing it here.
- `scripts/check-egress.sh --strict` is red at base on three test fixtures (STATE.md:99-102). It is not
  this phase's gate and must not be "fixed" by deleting fixtures.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The author's build host has `script(1)` (util-linux), ≥150 GB free, and reachable `api.deepseek.com` + `example.com` | Environment / Pattern 1 | Live items cannot be recorded; `script -T` items must fall back to `PtyHarness` (TUI) or `--output-format json` capture, changing the convention |
| A2 | `--permission-mode auto` really avoids the headless denial (i.e. the classifier answers before `RequestPermission`) | PERM-02 copy is "actionable" | Copy stays correct per the approved UI-SPEC, but the claim "choose auto" would need a live/mock check; verify in the PERM-02 test if cheap. **Unverified here** |
| A3 | A mock-driven headless run can script a gated tool call that reaches the permission prompt deterministically | Pattern 3, PERM-02 | If the scripted tool is auto-allowed by rules, the test would need an explicit `ask` rule seeded in config |
| A4 | `--permission-mode plan` in headless also produces `plan_mode.json` (not only the in-memory mode) | PERM-04 | Evidence may need to assert the reminder in the request body instead of the file; both are acceptable, but say which |
| A5 | Cast normalization by `evidence.py normalize` is acceptable for pty casts (text JSONL) | Pattern 2 | If a cast contains binary escapes that break substitution, the item must be re-recorded with a path-free screen |

**If this table is empty:** not applicable — A1–A5 need confirmation or an observed run.

## Open Questions

1. **Does `auto` mode in a headless run actually run?**
   - What we know: `default_auto_mode` is resolved from the flag (`headless.rs:965-970`,
     `permissions.rs:187-220`), and auto is feature-gated ON by default.
   - What's unclear: whether the classifier completes without a TTY and answers before
     `RequestPermission` is sent.
   - Recommendation: verify with one mock-driven headless run in the PERM-02 task; if it cannot be proved
     cheaply, keep the copy (UI-SPEC locks it) and record the copy as the requirement, not the behaviour.

2. **How should the "whole tree" leak scan be expressed?**
   - What we know: `--scan` takes paths and fails closed; `grep -R .` would enter `target/`.
   - Recommendation: `--scan docs/evidence` plus a tracked-file pass over `git ls-files` (file names
     only). Record the exact command in `notes.md`.

3. **One pty scenario or three for SESS-02?**
   - Recommendation: one parameterized test looping `default`, `always-approve`, `auto`, each spawning the
     harness in a fresh sandbox; one cast per mode (`plan-read-only-<mode>.cast`), one `report.json`.

---

## Environment Availability

| Dependency | Required By | Available (this host) | Version | Fallback |
|---|---|---|---|---|
| Rust toolchain (`cargo`, `rustc` 1.94.0) | every build/test | ✗ | — | Build host only (pinned `rust-toolchain.toml`) |
| Built binary `target/release/deepseek-build` | all live/pty/headless items | ✗ (no `target/`) | — | Build host: `cargo build --release -p xai-grok-pager-bin` |
| `DEEPSEEK_API_KEY` | PROOF-01/02/03 | ✗ | — | Build host env only; never a file |
| Disk ≥150 GB | `scripts/test-gate.sh` | ✗ (31 GB) | — | Build host; `scripts/disk-guard.sh 150` |
| `script(1)` | `script -T` evidence | ✗ | — | Build host; else pty/JSON capture (changes convention) |
| `cc` (LD_PRELOAD shims) | egress-check, sandbox-run | ✓ | system cc | — |
| `python3` | evidence.py, egress parser, MCP fixture | ✓ | 3.14.8 | — |
| `flock` | shared build lock | ✓ | — | — |
| `rg` | check-egress | ✓ | — | grep fallback in the script |
| `git` | scratch repos, commits | ✓ | 2.54.0 | — |
| `shellcheck` / `actionlint` | script lint | ✗ | — | Build host (AGENTS.md §6) |
| `gh` | release-adjacent work | ✗ | — | Not needed in Phase 2 |
| `docker` / `podman` | container work | present, unusable | — | Not needed in Phase 2 |
| `asciinema` / `strace` | (not used) | ✗ | — | `write_cast` / `egress-check.sh` are the substitutes |

**Missing dependencies with no fallback:** Rust toolchain and the built binary (all execution items);
`DEEPSEEK_API_KEY` (live items). These are the build host's preconditions, not defects.
**Missing dependencies with fallback:** `script(1)` (fall back to pty/JSON capture, but that changes the
Phase 1 convention — prefer the build host); `shellcheck` (record as NOT RUN on the planning host).

---

## Validation Architecture

`workflow.nyquist_validation` is `true` in `.planning/config.json`, so this section is required.

### Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `libtest` (`cargo test`), `tokio::test` (multi-thread, 2 workers for pty), `#[ignore]` for PTY e2e |
| **Config file** | workspace `Cargo.toml` (all pty roots listed as `[[test]]`, `crates/codegen/xai-grok-pager-pty-harness/Cargo.toml:70-118`); `.cargo/config.toml` sets `[build] incremental = false` |
| **Quick run command** | `flock -w 900 /tmp/team-build.lock cargo test -p <touched-crate> --lib` (or the single new test name) |
| **Full suite command** | `scripts/test-gate.sh` (150 GB guard → `flock` → `cargo test --workspace --no-fail-fast -- --test-threads=4` → `cargo test -p xai-grok-shell --lib --features config-docs -j 1` → `key-leak-check.sh --scan docs/evidence`) |
| **Estimated runtime** | crate-local seconds–minutes; full gate ~30 min on the author's host (Phase 1, 307 targets) |
| **Explicit, NOT in the gate** | PTY e2e: `cargo test -p xai-grok-pager-pty-harness --test pty_e2e_shell_tools -- --ignored --nocapture <name>` |

### Sampling Rate

- **Per task:** the task's own automated command (crate-local test, single ignored pty test, or the script harness).
- **Per wave:** `flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-pager --lib && … -p xai-grok-shell --lib plan_mode && … -p xai-grok-pager-pty-harness --test pty_e2e_shell_tools -- --ignored <new names>`.
- **Before `/gsd-verify-work`:** `scripts/test-gate.sh` green **and** every evidence item's `report.json` present with `result: ok` **and** `key-leak-check.sh --scan docs/evidence` exit 0.
- **Max feedback latency:** crate-local ≤ 120 s; pty case ≤ 120 s each; live items are minutes (network).

### Requirement → Observable Check

| Req | Observable check | Type | Command / artifact | Precondition |
|---|---|---|---|---|
| PROOF-01 | Committed `live-smoke` item whose step asserts `8 passed, 0 failed` from the smoke capture | live script + report | `script -qec 'scripts/smoke-test.sh' -T …` → `evidence.py cast/step/artifact/finish` | Build host, release binary, key exported, network |
| PROOF-02 | Committed `runtime-egress` item with steps `EGRESS_VIOLATIONS=0`, `EGRESS-CHECK PASS`, activity lines, and a cast showing the destination table | live script + report | `script -qec 'scripts/egress-check.sh' -T …` → same recipe | Build host, release binary, key, `cc`, network to provider + `example.com` |
| PROOF-03 | Default-mode canary scan PASS + `--scan docs/evidence` PASS + tracked-tree scan clean + dated revocation attestation | script + report + human attestation | `scripts/key-leak-check.sh`; `scripts/key-leak-check.sh --scan docs/evidence`; `git ls-files … \| xargs grep -lE …` | Build host for default mode; `--scan` anywhere; author revokes afterwards |
| PERM-01 | pty test passes and its cast contains `Yes, proceed`, `Always allow:`, `No, reject (type to add feedback)`, and `Security warning:` | pty e2e + committed cast | `GROK_PTY_CAST_DIR=… cargo test -p xai-grok-pager-pty-harness --test pty_e2e_shell_tools -- --ignored permission_prompt_outcomes_and_warning` | Build host, binary, **warning-row fix landed** |
| PERM-02 | Unit test asserts both rendered strings; headless-mock integration test asserts exit 1 + both stderr lines + JSON `stopReason:"cancelled"` on both invocations | unit + integration | `cargo test -p xai-grok-pager --lib headless`; new headless test (mock, no key) | Build host, binary; no key |
| PERM-03 | pty cast shows the same chip pair in the last pre-quit frame and the first post-`--continue` frame | pty e2e + cast | new pty test in the same family | Build host, binary, no key |
| PERM-04 | Mock-driven headless `--permission-mode plan` run: request body contains the plan-mode reminder (and/or `plan_mode.json` state Active), plus corrected `KNOWN-ISSUES.md:209-213` | integration test + doc diff | `cargo test … --lib headless` (unit already exists) + new headless-mock test + doc edit | Build host, binary; no key |
| SESS-01 | Resume test: request body after `-r <sid>` carries the persisted model and effort; plan reminder still present | integration (mock) + pty | new headless resume test; extend pty scenario | Build host, binary; no key |
| SESS-02 | Gate rejection text present in transcript/tool result while plan file stays writable, for `default`, `always-approve`, `auto`; unit gate tests stay green | unit + integration + pty cast | `cargo test -p xai-grok-shell --lib plan_mode_edit_gate`; new mode-parameterized test; committed `plan-read-only*.cast` | Build host, binary; no key |
| SESS-03 | `resume-fidelity` cast shows the plan body after quit + `--continue`, plus the approval chrome | pty e2e + cast | extended `scenarios/plan_approval_resume.rs` | Build host, binary; no key |
| SESS-04 | Provenance table in `resume-fidelity/notes.md` (model/effort/plan = session; permission mode = config/CLI) + the user-guide paragraph this phase owns | doc + report | `grep` checks over the item and guide | none (planning host can do this) |

### Wave 0 Gaps

- [ ] `crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/permission_prompt_outcomes.rs` — PERM-01 scenario (register in `tests/pty_e2e_shell_tools.rs`).
- [ ] `.../tests/pty_e2e/permission_mode_resume.rs` — PERM-03 (same registration).
- [ ] `.../tests/pty_e2e/plan_mode_read_only.rs` — SESS-02 (same registration).
- [ ] Extension of `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/plan_approval_resume.rs` — SESS-01/03 (model/effort assertions + cast emission).
- [ ] A headless-mock test target for PERM-02/PERM-04/SESS-01 — either a new `[[test]]` in the pty-harness crate (reuses `common.rs` helpers + `run_headless_in_sandbox_borrowed`) or a new `xai-grok-pager-bin/tests/` file.
- [ ] `crates/codegen/xai-grok-pager/src/headless_tests.rs` — text assertions on both denial strings (extend the existing `Some/None` test).
- [ ] `crates/codegen/xai-grok-pager/src/views/permission_view.rs` tests — warning row present, title stripped, chrome height +1.
- [ ] `docs/evidence/02-loop-item-verification/` tree — all seven item dirs with `report.json` + `notes.md`.

*Existing infrastructure covers the rest: mock server, pty driver, evidence recorder, leak scanner, gate.*

### Manual-Only Verifications

| Behavior | Requirement | Why manual | Instructions |
|---|---|---|---|
| Throwaway key revoked on platform.deepseek.com | PROOF-03 | No API; CONTRACT says record the author's confirmation rather than pretend to check | Author rotates/revokes after the live run; write a dated attestation (no key, no account id) in `key-leak/notes.md` |
| Live legibility of the warning row and chip pair | PERM-01/03 | Terminal aesthetics are a human call; the strings are pinned by tests/casts | Run the pty scenario locally and read the frame; the cast is the record |

---

## Security Domain

`security_enforcement: true`, ASVS level 1 (`security_block_on: high`). Most of this phase is verification,
but it manipulates a live credential and commits artifacts, so:

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---|---|---|
| V2 Authentication | yes (proof only) | Key from env only (`DEEPSEEK_API_KEY` / `DEEPSEEK_BUILD_API_KEY`); never in argv, files, logs, sessions, or evidence (`egress-check.sh:62-68`); rotation after use |
| V3 Session Management | yes (verify only) | Resume provenance documented (SESS-04); no new session secret surface |
| V4 Access Control | yes (verify only) | Permission modes and the fail-closed headless path; no hidden auto-approve (`headless.rs:1954-1957`) |
| V5 Input Validation | yes | `--permission-mode` clap allow-list (`cli.rs:590-597`); JSON schema of `report.json` is fixed |
| V6 Cryptography | no | No crypto added; casts/JSON are plain text and leak-scanned |
| V7 Error Handling | yes | Denial fails closed, exit 1, actionable hint; no stack traces or key material in `Error:` lines |
| V14 Config | yes | No new config key; `[ui] permission_mode` already documented (`26-config-reference.md:518`) |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---|---|---|
| API key in a committed cast/log (`script` echoes, `set -x`, verbose debug) | Information Disclosure | Env-only key; never echo; `--debug-file` lives in a mktemp dir; `key-leak-check.sh --scan` before commit |
| Key in a session file or panic text | Information Disclosure | `key-leak-check.sh` default mode writes files under an isolated HOME with a canary; STATE.md:103-106 records the known hygiene bug (gate unsets key vars) |
| Permission denial silently auto-approved in a script | Elevation of Privilege | Headless answers `Cancelled` unless `--always-approve`; no hidden retry (`headless.rs:1939-1958`) |
| Plan mode bypassed by a permissive mode | Tampering | Gate before permission resolution (`tool_calls.rs:1655-1670`); SESS-02 asserts under `always-approve` and `auto` |
| Denial text suggesting an invalid flag (user frustration → blind bypass) | — | Copy names `--always-approve` + `--permission-mode auto` only (UI-SPEC) |
| New outbound destination from a "test" | Information Disclosure | Forbidden (D6/D10/D13); the only network in this phase is the configured provider + the `web_fetch` host named in `egress-check.sh` |

---

## Sources

### Primary (HIGH confidence — read this session)

- `crates/codegen/xai-grok-pager/src/headless.rs` — `permission_cancel_hint` (476-498), `apply_headless_permission_mode` (849-875), call order (1264-1287), `build_json_result` (301-328), denial branch (1494-1595), RequestPermission handling (1939-1958).
- `crates/codegen/xai-grok-pager/src/headless/cli.rs` — `OutputFormat` (8-19).
- `crates/codegen/xai-grok-pager/src/app/cli.rs` — `--permission-mode` allow-list (590-598), `--output-format` (419-421), `--resume`/`--continue`/`--no-plan` (461-520), `--always-approve` (240).
- `crates/codegen/xai-grok-pager/src/headless_tests.rs` — hint test (1014-1043), plan-mode test (1045-1087).
- `crates/codegen/xai-grok-pager/src/app/acp_handler/permissions.rs` — `build_permission_display` (186-269), `permission_description_lines` (287-301), enqueue (60-168).
- `crates/codegen/xai-grok-pager/src/views/permission_view.rs` — reject label (1519-1528), title row (474-484), chrome height (296-329), row selection markers (1500-1515).
- `crates/codegen/xai-grok-pager-render/src/appearance/permission_cursor.rs` — labels (28-50), `resolve_initial_cursor` (159-183).
- `crates/codegen/xai-grok-workspace/src/permission/prompter.rs` — option labels (29, 340-415), `with_security_warning` (834-850), its test (1075-1108).
- `crates/codegen/xai-grok-workspace/src/permission/auto_mode/security_findings.rs` — `user_warning` (156-171).
- `crates/codegen/xai-grok-agent/src/config.rs` — `PermissionMode::VALID_VALUES` (896-920).
- `crates/codegen/xai-grok-shell/src/util/config/permissions.rs` — canonical parse (7-26), `effective_auto_for_launch` (184-220), `DEFAULT_INTERACTIVE_PERMISSION_MODE` (60-67).
- `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tool_calls.rs` — `PlanEditGate` + gate (157-190), gate application (1655-1670), rejection message (3181-3198).
- `crates/codegen/xai-grok-shell/src/session/plan_mode.rs` — snapshot (30-66), plan path (69-79), `should_auto_approve_edit` (141-144), templates (299-344), `PromptMode` (373-389).
- `crates/codegen/xai-grok-shell/src/session/persistence.rs` — `CurrentModel` (260-267), handling (2125-2144), `PlanModeState` (2155-2159).
- `crates/codegen/xai-grok-shell/src/session/storage/mod.rs` — `PLAN_MODE_FILE` (32), `write_plan_mode_state` (1324-1328).
- `crates/codegen/xai-grok-shell/src/agent/mvp_agent/session_setup.rs` — `restore_persisted_model` (1674-1727).
- `crates/codegen/xai-grok-shell/src/session/acp_session_tests/plan_mode_edit_gate_tests.rs` (41-103), `plan_approval_resume_tests.rs`.
- `crates/codegen/xai-grok-pager/src/settings/{defs.rs,registry.rs}` — pager `plan_mode` setting (1109-1127), "per-session, not persisted" (1269-1276).
- `crates/codegen/xai-grok-pager/src/app/event_loop.rs` — launch yolo/auto (1073-1113), display mode (1504-1520).
- `crates/codegen/xai-grok-pager-pty-harness/src/lib.rs` — `write_cast` (483-514), spawn variants (122-269), `wait_for_text`/`wait_until_stable` (400-476).
- `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/plan_approval_resume.rs` — whole scenario (1-187).
- `crates/codegen/xai-grok-pager-pty-harness/tests/plan_approval_resume.rs`, `tests/pty_e2e/common.rs` (`write_cast_if_requested` 1560-1575; `expect_tool_turn` 733-747), `tests/pty_e2e/configured_bash_filename_arguments.rs` (131-171, 258-326), `tests/pty_e2e/permission_prompt_hook_chimes_only_on_real_wait.rs` (82-122).
- `crates/codegen/xai-grok-test-support/src/headless.rs` (23-105), `src/conversation_script.rs` (64-93), `src/sandbox.rs` (208-238, 325), `crates/codegen/xai-grok-pager-bin/tests/header_capture.rs` (11-32).
- `scripts/*` — evidence.py (whole), key-leak-check.sh (whole), egress-check.sh (whole), smoke-test.sh (whole), test-gate.sh (1-63, 298-300), disk-guard.sh (whole), check-egress.sh (whole), sandbox-run.sh (1-35).
- `docs/evidence/README.md` (whole), `docs/evidence/01-truth-baseline/{ipv6-report,cost-estimate}/` (report.json + notes.md), `.planning/phases/01-truth-baseline/01-VALIDATION.md`, `01-02-PLAN.md`.
- `KNOWN-ISSUES.md` (209-220), `TESTING.md` (93), `.planning/codebase/CONCERNS.md` (181).
- `02-CONTEXT.md`, `02-UI-SPEC.md`, `REQUIREMENTS.md`, `STATE.md`, `.planning/research/{SUMMARY,FEATURES}.md`, `.planning/config.json`.

### Secondary (MEDIUM confidence)

- UI-SPEC's two internal inconsistencies (row numbering in §Verification Hooks; "height math already allows" for the warning row) — resolved here by reading the code; the plan should follow the code.

### Tertiary (LOW confidence / unverified here)

- The author's build-host capabilities (`script(1)`, free disk, network reachability) — assumed, not observed.
- Whether `--permission-mode auto` avoids a headless denial in practice — not run.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependency; every tool read.
- Architecture: HIGH — every seam cited at file:line; the two defects reproduced by reading both sides.
- Pitfalls: HIGH for the tree-grounded ones (warning drop, denial drift, flag allow-list, gate ordering); MEDIUM for host-dependent ones (A1, A2).
- Live evidence mechanics: HIGH (scripts read end to end); MEDIUM on the build host's actual environment.

**Research date:** 2026-11-23
**Valid until:** 2026-12-23 (30 days) — re-read `headless.rs`, `permissions.rs`, `permission_view.rs`, `tool_calls.rs` before execution if the tree moves.
