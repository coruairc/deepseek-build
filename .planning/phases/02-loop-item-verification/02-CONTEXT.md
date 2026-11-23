# Phase 2: Loop-Item Verification - Context

**Gathered:** 2026-11-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Prove the daily loop on evidence, not on assertions. Three bounded areas:

1. **Live proof (PROOF-01/02/03):** one real model-driven session with a real key that exercises a shell
   command, a file edit and `web_fetch`; the same session measured for runtime egress (provider,
   `web_fetch` host, AF_UNIX sockets only, zero violations); the tree and every committed artifact
   scanned for key material; the key revoked by the author afterwards and the revocation recorded.
2. **Permission surface (PERM-01…04):** the interactive prompt exercised end to end with allow-once,
   allow-for-session and reject plus the high-signal security warning; headless denial failing closed
   with an actionable hint naming `--always-approve` **and** `--permission-mode auto` on both the stderr
   and JSON error paths; permission mode visible in the UI and matching after resume;
   `--permission-mode plan` verified to arm plan mode headless, with `KNOWN-ISSUES.md` corrected rather
   than the code changed.
3. **Resume fidelity (SESS-01…04):** resume restores model, reasoning effort and plan state; plan mode
   is read-only except the plan file in every permission mode including `always-approve` and `auto`;
   a drafted plan survives quit and resume as committed evidence; the provenance of each resumed
   setting (session versus config) is documented.

Every item lands under `docs/evidence/02-loop-item-verification/<item-slug>/` per the Phase 1
convention. No new product surface beyond the smallest defect fix a verification exposes. No new
outbound destination. No routing.

**Environment reality (blocking, taken autonomously because this run could not prompt):** the host this
run executes on has no Rust toolchain, a 31 GB free filesystem (below the 60 GB `disk-guard.sh` floor
and the 150 GB project floor), 3.7 GB RAM, no built binary, no `DEEPSEEK_API_KEY`, no `gh` CLI and no
usable container runtime. Every success criterion needs a compiled binary and a live key, so execution
and verification of this phase must happen on the build host. The phase must still produce a plan the
author can execute there, and must not fabricate any evidence item.

</domain>

<decisions>
## Implementation Decisions

### Live proof and key handling
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

### Permission surface
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

### Resume fidelity
- SESS-01/03 promote the existing `plan_approval_resume` scenario family
  (`crates/codegen/xai-grok-pager-pty-harness/src/scenarios/plan_approval_resume.rs` plus the test
  `tests/plan_approval_resume.rs`) into committed evidence: draft a plan, quit, `--resume`, assert model,
  reasoning effort and plan state are restored, and assert the drafted plan is intact.
- SESS-02 is a parameterized read-only assertion across `default`, `always-approve` and `auto`: in plan
  mode a write outside the plan file is refused while the plan file itself is writable.
- SESS-04 is documentation plus the artifact: a provenance table in the item's `notes.md` and the user
  guide stating which resumed setting comes from the session record and which from config.

### Defect policy
- A defect a verification exposes is fixed minimally, in scope, with a regression test, and recorded in
  the plan SUMMARY — except where the roadmap says otherwise (`--permission-mode plan`: doc correction,
  not code).
- Anything outside the loop items is recorded in `KNOWN-ISSUES.md`, not fixed here.

### Evidence
- One item directory per claim: `live-smoke` (PROOF-01), `runtime-egress` (PROOF-02), `key-leak`
  (PROOF-03), `permission-prompt` (PERM-01/02), `permission-mode` (PERM-03/04), `resume-fidelity`
  (SESS-01/03/04), `plan-read-only` (SESS-02). Each carries `<slug>.cast` where a pty is used,
  `report.json` built by `scripts/evidence.py`, and `notes.md` with the re-run command.
- `report.json` keeps the Phase 1 fixed key set and order, paths normalized to `<REPO>`/`<HOME>`/`<TMP>`.

### Plan sequencing
- 02-01 and 02-02 are one live session captured twice (smoke transcript, then the egress measurement) and
  stay sequential; 02-03 scans what they produced. 02-04 and 02-05 touch different surfaces and may run
  in parallel worktrees only if their `files_modified` sets are disjoint, per the Phase 2 parallel-lane
  decision in STATE.md — dependent work stays sequential.

### Decided autonomously
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

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `scripts/egress-check.sh` — real-session LD_PRELOAD egress audit; allowlists provider, `web_fetch`
  host, loopback and AF_UNIX; documents its own blind spots.
- `scripts/smoke-test.sh` — live smoke self-test (8 checks recorded as passing in `KNOWN-ISSUES.md`).
- `scripts/key-leak-check.sh` — fake-key run in an isolated `HOME` plus `--scan <path>` for committed
  evidence; prints file names only.
- `scripts/evidence.py` — `step` / `finish` recorder that refuses to overstate a run; `report.json`
  schema is documented in `docs/evidence/README.md`.
- `scripts/test-gate.sh`, `scripts/disk-guard.sh` — the completeness-asserted suite gate and its disk
  precondition.
- `crates/codegen/xai-grok-pager-pty-harness/src/lib.rs:485` — `write_cast` emits asciinema v2.
- `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/plan_approval_resume.rs` and
  `tests/plan_approval_resume.rs` — the existing plan-approval-across-resume scenario.
- `crates/codegen/xai-grok-pager/src/headless.rs:479` (`permission_cancel_hint`) and `:1539` — the two
  headless denial error paths PERM-02 asserts.
- `crates/codegen/xai-grok-pager-render/src/appearance/permission_cursor.rs` — `AllowOnce` /
  `Allow always` / `Reject` labels and `DefaultSelectedPermission` parsing of `allow_once`.
- `crates/codegen/xai-grok-permission-rules/src/resolution.rs` — permission-mode resolution and the
  always-approve policy pins.
- `crates/codegen/xai-grok-workspace-types/tests/wire_round_trip.rs` — the `allow_once` wire shape.

### Established Patterns
- Tests live beside the code plus integration tests under `tests/`; pty e2e scenarios drive the real
  binary against the mock inference server.
- Evidence is an asciinema-v2 cast plus a fixed-schema `report.json` plus `notes.md`; logs are normalized
  and leak-scanned.
- `KNOWN-ISSUES.md` is the honest ledger; correcting it is preferred over touching working code when the
  ledger is stale.
- User-visible strings resolve through `xai-grok-brand`; `branding_surface` gates CLI help/version.

### Integration Points
- Headless permission resolution: CLI flags → `HeadlessOptions` → `apply_headless_permission_mode` →
  `session/set_mode` → tool gate.
- Plan mode: pager `plan_mode` setting (per-session, not persisted) plus the shell's plan-file gate.
- Resume: session persistence in `xai-grok-shell` restores model/effort/plan state on `--resume`.
- Egress: `scripts/egress-check.sh` → `egress_log.c` shim → the allowlist verdict.

</code_context>

<specifics>
## Specific Ideas

- The live proof must be one session, not three: the transcript and the egress measurement describe the
  same run so the egress claim is about real traffic rather than about a harness that was allowed to
  differ.
- The `--permission-mode plan` entry in `KNOWN-ISSUES.md` is stale: `TESTING.md` row 16 and
  `.planning/codebase/CONCERNS.md` both say the tree sends `SetSessionModeRequest` for `plan`. Correcting
  the ledger is the deliverable, not a code change.
- The JSON error path's hint is narrower than the stderr hint today; that asymmetry is the one expected
  product fix.
- The key is compromised-by-policy from the moment it is used: the artifact proves the run happened and
  the scan proves no leak; the author's rotation is the closing action.

</specifics>

<deferred>
## Deferred Ideas

- Automatic model routing, per-model cost accounting, `/route` — v2 / open-build.
- `ci.yml` and debug-symbol sidecars — REL-V2-01/02.
- `IN-07` (`key-leak-check.sh` default-mode fail-open on grep rc≥2) — ledgered for v0.1.x; `--scan` fails
  closed, which is the mode this phase uses.
- Any new evidence tooling (asciinema CLI, a cast-to-gif renderer) — not needed; the harness emits v2.

</deferred>
