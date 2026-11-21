# Feature Research

**Domain:** Personal, DeepSeek-only terminal coding agent (Rust; TUI + headless) at the `v0.1.0` freeze
**Researched:** 2026-11-21
**Confidence:** MEDIUM-HIGH (every claim below is grounded in a file in this tree; recall-based claims are marked `UNVERIFIED`)

## Scope and method

This is **not** a greenfield feature survey. The product already ships a fullscreen TUI, headless
`plain`/`json`/`streaming` output, shell/read/edit/search tools, permission modes, sandbox, MCP,
sessions, compaction, subagents, hooks, plugins, slash commands, status line, installer and a
release pipeline (`.planning/PROJECT.md` §Validated). Re-researching those as gaps would be wrong.

The milestone question is narrower: **for five daily-driver loops, what does "good" look like, what
is missing or wrong today, and how is each behaviour surfaced so it is visible and reversible rather
than silent?**

Loops: model routing, permission prompting (incl. headless cannot-approve), session resume, plan
mode, cost/cache transparency. Plus the sandbox profile decision and the evidence requirement.

Grounded by reading: `crates/codegen/xai-grok-sampler/src/model_routing.rs`,
`.../src/client.rs`, `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs`,
`.../status_line.rs`, `.../session_mode.rs`, `crates/codegen/xai-grok-status-line/src/{config,context,pricing}.rs`,
`crates/codegen/xai-grok-pager/src/views/status_line/segments.rs`,
`crates/codegen/xai-grok-pager/src/app/status_line_policy.rs`, `.../src/headless.rs`,
`.../src/slash/commands/{model,effort,plan,resume}.rs`, `.../src/app/dispatch/settings/setters.rs`,
`.../src/views/{permission_view,plan_approval_view}.rs`,
`crates/codegen/xai-grok-workspace/src/permission/prompter.rs`,
`crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs`, `docs/TUNING.md`,
`crates/codegen/xai-grok-models/default_models.json`, and the user guide under
`crates/codegen/xai-grok-pager/docs/user-guide/`.

No web search was available. Nothing here proposes telemetry, upload, or a new outbound
destination; nothing proposes renaming `xai-grok-*` crates or `GROK_*` env names; nothing
re-platforms the TUI or ACP.

---

## The one table that matters: is the behaviour visible and reversible?

| Behaviour | Visible where today | Reversible how today | Verdict |
|---|---|---|---|
| Automatic routing decision | **Nowhere.** `StatusLineContext` carries no route field; `segments.rs:66-69` has an explicit `TODO(auto-routing)` saying exactly this | N/A (policy defaults `Off`, so nothing to reverse) | **Missing** |
| Manual model switch | Scrollback `Switched to {name} ({effort} effort)` from the `model_changed` notification (`app/dispatch/session/lifecycle.rs:1889-1917`); status-line `model` item if enabled | `/model <other>`; but bare `/model` also writes `default_model` for future sessions (`setters.rs:1567-1650`) | Works; **no unpin**, and "session pin" and "new-session default" are the same action |
| Reasoning effort | Status-line `effort` item (opt-in row); `/effort` usage error prints `current: <level>` | `/effort <other>` | Works; row is **off by default** (`StatusLineType` default `Disabled`, `config.rs:311-318`) |
| Permission prompt scope | Prompt lists "Yes, proceed" / "allow all edits during this session" / "Reject" with a security warning prepended for high-signal findings (`prompter.rs:333-380`, `:832-848`) | Grants are session-scoped; permission mode is a persisted setting (`PermissionModeKind`, `app/actions.rs:947-1000`) | Works; interactive UI never driven by a human |
| Headless cannot-approve | stderr hint + non-zero exit (`headless.rs:459-478`, `:1493-1500`) | Re-run with `--always-approve` or `--permission-mode auto` | Works and fails closed; under-documented next to headless |
| Plan mode | Prompt-border tint, transient banner, `/plan`, approval surface | `/plan` off, or approve/request-changes/quit on the plan (`plan_approval_view.rs`) | Works; human interactive pass outstanding |
| Cost | Status-line `cost` item, hidden under $0.005 (`segments.rs:18,104-108`); headless `estimated_cost_usd` + `estimate: true` | Config `[ui.status_line.pricing.*]` | Works, but **estimate is not labelled in the TUI**, ignores per-model usage under routing, and headless ignores the user's pricing overrides |
| Cache hit | Status-line `cache N%` and `tokens` buckets | Disable the row / drop the item | Works; opt-in row |
| Sandbox profile | `sandbox.profile` setting + `--sandbox`; `sandbox.auto_allow_bash` | Setting / flag | On by default, but a root-only `/run/containerd` parent aborts startup (see PITFALLS-adjacent note below) |

---

## Feature Landscape

### Table Stakes (must have at freeze, or the daily-driver claim is false)

| # | Feature | Why expected | Complexity | Notes / current state |
|---|---|---|---|---|
| T1 | **Routing decides per turn from the real session path** | A cost-control policy the turn loop never reads is a lie; `PROJECT.md` names this the biggest correctness gap | MEDIUM | `ModelRoutingPolicy` defaults `Off` (`model_routing.rs:26`); `SamplingClient::route_turn` (`client.rs:611-623`) has **no caller outside its own module**; all four construction sites pass `model_routing: Default::default()` (`sampler_turn.rs:729`, `agent/config.rs:4328`, `agent/subagent/mod.rs:779`, `tools/config.rs:212`). `request_builder.rs:84` takes `model` from chat-state sampling config, so the route must be applied to the built request (or a per-turn config) and **not** by mutating the session's sampling config |
| T2 | **The routed model is shown per turn** | Cost control must be visible; a silent router undermines trust (`PROJECT.md` Key Decisions) | MEDIUM | Needs a new field on `StatusLineContext` (additive; `context.rs:12-14` says additions do not bump `schema_version`) plus a `route` segment replacing the `TODO(auto-routing)` at `segments.rs:66-69`. Note the row is **disabled by default**, so also surface it in the turn status/scrollback path or accept "opt-in visibility" as the documented contract |
| T3 | **A pin is explicit, respected, and clearable** | "Routing respects the pin until it is cleared" is the reversibility half of T2 | MEDIUM-HIGH | `ModelRoutingConfig.manual_override` exists (`model_routing.rs:69`) and wins over policy (`:190-192`), but nothing sets it and no caller reads it. Today `/model` does two things at once: switches the session and persists `default_model` (`setters.rs:1567-1650`). Decide and document: session pin (routing) vs new-session default (config). A clear path (`/model auto`, or a `/route` command) does not exist |
| T4 | **Routing is opt-in and documented as a user setting** | A setting only a source reader can find is not a feature | MEDIUM | No `routing`/`model_routing` key exists in the user config, `FEATURES` registry, or `26-config-reference.md` (grepped: zero hits). Adding one requires the registry entry **and** a hand-edited row, enforced by the `config_docs` tests (`--features config-docs`) |
| T5 | **One routing decision per turn, stable across sampler rounds** | A turn is several sampler calls; flipping model mid-turn thrashes the prefix cache and mis-attributes cost | LOW-MEDIUM | `classify_turn` returns `Complex` as soon as the conversation contains any tool activity (`model_routing.rs:159-167`). Resolving per sampler call would flip a turn that started trivial (Flash) to Pro at round 2. Resolve once at turn start, hold for the turn |
| T6 | **Interactive permission prompt is complete and legible** | A daily driver asks for approval constantly; a prompt without scope or reason trains blind approval | LOW (verify) | Already implemented: allow-once, "allow all edits during this session", reject-once (`prompter.rs:20`, `:333-380`); a fixed-text `⚠ Security warning:` is prepended to the prompt title in **every** mode for high-signal findings (`prompter.rs:832-848`, `security_findings.rs:154-169`). Gap is exercise by a human, not code |
| T7 | **Headless fails closed and says what to do** | Silent auto-approve in a script is a safety hole; a mute failure is a usability hole | LOW | `RequestPermission` outside `--always-approve` answers `Cancelled` (`headless.rs:1899-1919`); the turn ends non-zero with a hint naming `--always-approve` and `--permission-mode auto` (`:459-478`, `:1493-1500`). Polish: document beside headless (guide `14-headless-mode.md` covers flags but not this failure), and make sure the JSON error path carries the same hint |
| T8 | **Permission mode is visible and survives resume** | Users must know whether the agent is asking, auto-reviewing, or never asking | LOW (verify) | `PermissionModeKind` (default/ask/auto/always-approve) persists to config and projects onto the runtime yolo flag (`app/actions.rs:947-1000`); on load the mode is re-derived from config (`session_setup.rs:1358-1367`). Visible as an info-line label (`PermissionLabel::display_name`). Verify the label matches after resume |
| T9 | **Resume restores what the session was doing** | A resume that changes model or mode silently is worse than no resume | MEDIUM (verify) | Persisted: `CurrentModel { model_id, agent, reasoning_effort, context_window }` and `PlanModeState` (`session/persistence.rs:261-269`, `:1159-1170`); `awaiting_plan_approval` re-parks `exit_plan_mode` on load (`session_setup.rs:1345-1352`). Permission mode comes from config, not the session — that is defensible but must be stated in docs |
| T10 | **Resume under routing is deterministic** | The resumed session must not silently run a different model than the one it was saved with | MEDIUM | Depends on T3. A saved `current_model_id` should be treated as a pin until cleared, or routing must be re-evaluated explicitly and shown. Pick one and test it |
| T11 | **Plan mode is read-only except the plan file, in every mode** | The whole point of plan mode is the guarantee | LOW (verify) | Documented at guide `19-plan-mode.md:16`; enforced through the plan edit gate. Verify the guarantee holds under `always-approve` and `auto`, which is where a regression would hide |
| T12 | **`--permission-mode plan` really arms plan mode headless** | `KNOWN-ISSUES.md` still claims the flag does nothing; the tree disagrees | LOW (verify) | `apply_headless_permission_mode` sends `SetSessionModeRequest` for `plan` (`headless.rs:829-853`) and the comment says that is what arms the reminder and edit gate. Verify with a headless transcript, then correct `KNOWN-ISSUES.md` rather than "fixing" it with a second set-mode call (`CONCERNS.md` Fragile Areas) |
| T13 | **Plan approval survives quit/resume** | Losing a drafted plan on quit is a daily-driver failure | LOW (verify) | A pty scenario already asserts this end to end (`xai-grok-pager-pty-harness/src/scenarios/plan_approval_resume.rs`), and the shell re-issues the reverse request on resume. Evidence exists; promote it into the milestone's committed transcript set |
| T14 | **Cost is labelled as an estimate and is priced per model** | A wrong number the user trusts is worse than no number | MEDIUM-HIGH | TUI fills `cost.total_cost_usd` from `[ui.status_line.pricing]` using **the session model id for all session usage** (`status_line_policy.rs` `fill_estimated_cost`), while the ledger already keeps `by_model` totals (`xai-chat-state/src/usage.rs:109`). Under routing, a mixed Flash/Pro session is priced at one rate. Headless uses `PricingTable::default()`, ignoring the user's overrides (`headless.rs:313-340`), so TUI and JSON can disagree. No `estimate` marker in the TUI segment |
| T15 | **Cache hit and token buckets are visible and correct** | The 97% cache-read share dominates DeepSeek spend (`docs/TUNING.md` baseline) | LOW (verify) | `cache` item renders `cache N%` from `StatusLineSessionUsage::cache_hit_rate()` (`context.rs:196-200`, `segments.rs:118-125`); `tokens` renders `in · cache · out · think`. Verify the buckets stay disjoint after a routed turn (the debug assert at `status_line.rs:83-89` is the canary) |
| T16 | **Headless JSON cost contract stays stable** | Scripts consume it; `estimate: true` is the honest marker | LOW (verify) | `estimated_cost_usd` + `estimate: true`, omitted when no tokens were recorded (`headless.rs:308-340`); documented at guide `14-headless-mode.md:194` |
| T17 | **Sandbox stays on by default; runtime-socket denies skip what they cannot traverse** | Startup must not fail on a host where `/run/containerd` is root-only, and must not weaken elsewhere | MEDIUM | `materialize_runtime_socket_deny_paths_from` canonicalizes the parent and returns `Err` for any error other than `NotFound` (`runtime_sockets.rs:114-172`), so `EACCES` on a root-only parent aborts bwrap. Fix: skip candidates whose parent is not traversable, keep fail-closed for denies the process can see but cannot mask; add the test that a root-only parent does not abort (`CONCERNS.md` Known Bugs) |
| T18 | **Every loop item carries committed evidence** | A freeze is only defensible with artifacts (`PROJECT.md` Key Decisions) | LOW-MEDIUM | Test surfaces exist: mock inference server records each request body (`xai-grok-test-support/src/request_log.rs`, `inference_request.rs`) so a routed-turn test can assert `model` per foreground request; pty harness can `write_cast()` an asciinema transcript (`lib.rs:484`), and **no `.cast` is committed today**, so the artifact convention must be chosen |

### Differentiators (worth doing in this milestone vs not)

| # | Feature | Value proposition | Complexity | Notes |
|---|---|---|---|---|
| D1 | **"Why this model?" on demand** | Trust: the router explains its own decision without a new surface | LOW-MEDIUM | `classify_turn` is a pure function; return `TurnComplexity` + the matched marker/flag alongside the route, put it on the status-line payload and in the turn log. No extra inference |
| D2 | **Per-model spend in the TUI** | Makes the Flash-vs-Pro saving observable instead of theoretical | MEDIUM | Ledger already carries `by_model`; headless `modelUsage` already emits it. TUI `/usage` is billing-oriented today. Cheapest useful version: a `models` line in `/usage`, not a new dashboard column |
| D3 | **A `/route` command** (`off / low / high / auto / status`) | One obvious place for policy, leaving `/model` as the pin | MEDIUM | Complements T3/T4. Only worth it if T3's "clear the pin" lands here rather than in `/model auto` |
| D4 | **Explicit off-peak pricing switch** | DeepSeek off-peak is half price; users currently must halve three rates by hand | LOW | `[ui.status_line.pricing.<model>]` already overrides (`pricing.rs:85-101`); an `off_peak = true` that applies the documented half-rate is a config-only change. `D14` in `DECISIONS.md` is the governing decision. Must **not** fetch prices |
| D5 | **Route decision recorded in the session log** | A resumed or exported session can show what ran and why | LOW-MEDIUM | Additive field in persisted turn metadata; keep it local-only (no new destination) |
| D6 | **Flash for research subagents, automatically** | Matches the cost story; `AGENTS.md` lists it as a Phase-4 leftover | LOW (config) / HIGH (automatic) | Already achievable by config: `[subagents.models].<agent>` and `AgentDefinition.model` (`agent/subagent/mod.rs:594-667`). Automatic coupling to the router is a **later** step; document the config path now |

### Anti-Features (deliberately NOT in this milestone)

| # | Anti-feature | Why it looks appealing | Why problematic | Alternative |
|---|---|---|---|---|
| A1 | **LLM-based turn-complexity classifier** | "Smart" routing | Adds a model call and latency to every turn, adds a second inference path to audit, and makes routing unexplainable | Keep the local heuristic; add D1 so the decision is explainable |
| A2 | **Silent routing** | Fewer surfaces | `PROJECT.md` rejects it outright: an unexplained model change destroys trust in cost and quality | T2/T3: show the route, make the pin explicit |
| A3 | **Re-routing every sampler round** | Reacts to tool activity | One turn would straddle two models: cache thrash (the prompt is ~53k cached tokens/task), prompt-label drift, incoherent cost rows | T5: one decision per turn |
| A4 | **Auto-escalate Flash→Pro mid-turn on failure** | "Self-healing" | Doubles cost exactly when the user is already confused, hides model-quality problems, and `docs/TUNING.md` E3c found no task class where Flash fails and Pro succeeds, so the boundary is unmeasured | Defer until a harder task set locates the boundary (`docs/TUNING.md` explicitly says this) |
| A5 | **Provider-agnostic routing registry** | "Future-proof" | Phase 2 (open-build) work; would touch endpoints, catalog, and auth crates this freeze depends on | DeepSeek-only policy now; Phase 2 branches from the freeze tag |
| A6 | **Runtime pricing fetch / price scrape** | Always-fresh costs | New outbound destination — forbidden by D6/D10/D13 and `PROJECT.md` Out of Scope | Config override + documented estimate semantics (T14, D4) |
| A7 | **Hidden auto-approve in headless** | "Just make `-p` work" | Deletes the fail-closed guarantee that makes unattended runs safe | Keep cancel + hint; document `--always-approve` and `--permission-mode auto` (T7) |
| A8 | **A full undo/checkpoint product** | Users ask for "undo everything" | `CONCERNS.md` warns the `[ui.contextual_hints].undo` flag is not a user-facing undo of every tool; building one is a re-platform | Verify existing rewind/checkpoint behaviour; do not grow it this milestone |
| A9 | **Any routing/usage telemetry to a remote endpoint** | "Measure routing quality" | Violates the egress invariant and the product premise | Local logs and the committed transcripts only |
| A10 | **New TUI surfaces** (route picker modal, dashboard column, new pane) | Looks like progress | This milestone fixes and polishes; `PROJECT.md` forbids re-platforming the TUI | One status-line segment + one config key |
| A11 | **"Fix" `--permission-mode plan` with a second set-mode call** | Reads like the obvious patch for the stale bug report | `CONCERNS.md` Fragile Areas warns the tree already sends the call; a second one would double-apply | Verify, then correct `KNOWN-ISSUES.md` (T12) |
| A12 | **Renaming `GROK_*` readers or `xai-grok-*` crates** | "Finish the rebrand" | Explicitly deferred; would churn every adapter crate the freeze depends on | Out of scope by `PROJECT.md` and `AGENTS.md` |

---

## Feature Dependencies

```
T1 routing wired
    ├──requires──> T5 one decision per turn (otherwise T1 ships a straddling turn)
    ├──requires──> T4 a user-facing setting (otherwise T1 is unreachable)
    └──enhances──> T2 routed model visible
                       └──requires──> T3 pin state + clear path (else "visible" has nothing to show
                                      when the user has pinned, and "reversible" has no off switch)

T3 pin ──conflicts──> current `/model` semantics (session switch AND default_model persist in one action)
T3 ──requires──> a decision: pin is session-scoped, default is config-scoped

T14 per-model cost ──requires──> T1 (only matters once two models share a session)
T14 ──enhances──> D2 per-model spend
T14 ──requires──> per-model usage on the status-line payload (ledger has it; payload does not)

T10 resume-under-routing ──requires──> T3 (a saved model must be either a pin or re-routed, not ambiguous)
T9 resume restores state ──enhances──> T13 plan approval survives resume

T17 sandbox socket fix ──independent──> everything (do it early; it unblocks the full-suite gate)

T18 evidence ──requires──> a chosen artifact format (cast vs text+JSON) before any loop item can be "done"
T6/T7/T8, T11/T12/T13, T15/T16 ──are──> verify-and-record work, not new code
```

### Dependency notes

- **T1 → T4 → T2/T3 is one vertical slice.** Shipping T1 alone changes nothing a user can turn on;
  shipping T2 without T3 means the row shows a route the user cannot override or stop.
- **T3 conflicts with today's `/model`.** Bare `/model` currently persists `default_model` *and*
  switches the session (`setters.rs:1567-1650`). Routing needs a session-scoped pin that can be
  cleared without changing the user's new-session default. Resolve this before writing the pin.
- **T14 only becomes wrong once T1 lands.** Today there is one model per session, so pricing all
  usage at the session model is correct. Routing makes it an estimate of a mixed bill.
- **T17 is independent and blocking.** A root-only `/run/containerd` parent fails sandbox startup
  (`runtime_sockets.rs` canonicalize error path), which is also one of the two recorded workspace
  test failures. Fixing it is a prerequisite for a clean `cargo test --workspace`.
- **T18 gates the definition of done for every other row.** Pick the artifact format first.

---

## MVP Definition

### Launch with (the freeze)

- [ ] **T1 + T4 + T5** — routing is reachable, wired from the turn path, decided once per turn, and
      configured by a documented key. Without these the milestone's headline claim is false.
- [ ] **T2 + T3** — the route is on the status line, the pin is explicit and clearable. Without these
      routing is silent and irreversible, which `PROJECT.md` forbids.
- [ ] **T6–T8, T11–T13, T15–T16** — verify-and-record. No new code expected; each ends in a test or
      transcript plus a corrected doc line.
- [ ] **T14** — at minimum: label the TUI number as an estimate, make headless honour
      `[ui.status_line.pricing]`, and price per model when the ledger has more than one.
- [ ] **T17** — sandbox socket materialization skips non-traversable parents; fail-closed elsewhere;
      test proves a root-only parent does not abort bwrap.
- [ ] **T18** — one committed artifact per loop item, format decided up front.

### Add after validation (v0.1.x)

- [ ] **D1** "why this model" — the cheapest trust multiplier once T2 exists.
- [ ] **D3** `/route` — if T3's clear path does not fit naturally in `/model`.
- [ ] **D4** off-peak switch — config-only, and it closes a known 2× overstatement.
- [ ] **D2** per-model spend in `/usage`.
- [ ] **D5** route recorded in the session log.

### Future consideration (v2 / Phase 2)

- [ ] **D6-automatic** Flash-for-research subagents tied to the router (config path ships now).
- [ ] Any provider-agnostic routing (A5) — Phase 2, from the freeze tag.
- [ ] Learned escalation (A4) — needs a harder eval set first.

---

## Feature Prioritization Matrix

| # | Feature | User value | Implementation cost | Priority |
|---|---|---|---|---|
| T1 | Routing wired from the turn path | HIGH | MEDIUM | P1 |
| T2 | Routed model visible | HIGH | MEDIUM | P1 |
| T3 | Explicit, clearable pin | HIGH | MEDIUM-HIGH | P1 |
| T4 | Routing as a documented setting | MEDIUM | MEDIUM | P1 |
| T5 | One decision per turn | HIGH (correctness) | LOW-MEDIUM | P1 |
| T6 | Permission prompt complete | HIGH | LOW (verify) | P1 |
| T7 | Headless fail-closed + hint | HIGH | LOW | P1 |
| T8 | Permission mode visible/persistent | MEDIUM | LOW (verify) | P1 |
| T9 | Resume restores state | HIGH | MEDIUM (verify) | P1 |
| T10 | Resume deterministic under routing | HIGH | MEDIUM | P1 |
| T11 | Plan-mode read-only guarantee | HIGH | LOW (verify) | P1 |
| T12 | Headless plan mode verified | MEDIUM | LOW (verify) | P1 |
| T13 | Plan approval survives resume | MEDIUM | LOW (verify) | P1 |
| T14 | Cost labelled and per-model | HIGH | MEDIUM-HIGH | P1 |
| T15 | Cache/tokens visible and correct | MEDIUM | LOW (verify) | P1 |
| T16 | Headless cost contract stable | MEDIUM | LOW (verify) | P1 |
| T17 | Sandbox socket materialization | HIGH (blocks clean suite) | MEDIUM | P1 |
| T18 | Committed evidence per item | HIGH (freeze defence) | LOW-MEDIUM | P1 |
| D1 | "Why this model" | MEDIUM | LOW-MEDIUM | P2 |
| D4 | Off-peak switch | MEDIUM | LOW | P2 |
| D2 | Per-model spend in `/usage` | MEDIUM | MEDIUM | P2 |
| D3 | `/route` command | MEDIUM | MEDIUM | P2 |
| D5 | Route in session log | LOW-MEDIUM | LOW-MEDIUM | P3 |
| D6 | Flash-for-research (config path) | MEDIUM | LOW | P2 (doc), P3 (automatic) |

**Priority key:** P1 = must have at freeze; P2 = should have, add when possible; P3 = nice to have.

---

## Competitor Feature Analysis

All external-product claims below are limited to what this repository's own docs assert; anything
beyond that is marked `UNVERIFIED` and was not web-checked.

| Feature | Claude Code (per this repo's docs) | Codex (per this repo's docs) | Our approach |
|---|---|---|---|
| Headless machine-readable output | deepseek-build deliberately matches the Messages-shaped JSON contract, including `system/init` and `result` lines (guide `14-headless-mode.md:247-296`) | `UNVERIFIED` | Keep the contract; add nothing (T16) |
| Session identity / anti-overwrite | `-r/-c/-s/--fork-session` semantics "match Claude Code's anti-overwrite model" (guide `17-sessions.md:216`) | `UNVERIFIED` | Keep; only verify restore fidelity (T9) |
| Permission mode naming | `bypassPermissions` naming is adopted (guide `14-headless-mode.md:38`, `22-permissions-and-safety.md`) | `UNVERIFIED` | Keep; do not add modes (T8) |
| Edit tool format | `UNVERIFIED` | `apply_patch` via `--agent codex`, measured at +5% cost and rejected (`docs/TUNING.md` E3b) | Keep `search_replace` default |
| Model routing | `UNVERIFIED` | `UNVERIFIED` | Heuristic + user policy + visible pin. No learned router (A1), no mid-turn escalation (A4) |
| Cost reporting | `UNVERIFIED` | `UNVERIFIED` | Local estimate from a shipped table, `estimate: true`, never a scrape (A6) |

---

## Sources

Repository files (all read for this document; line references inline above):

- `crates/codegen/xai-grok-sampler/src/model_routing.rs`, `src/client.rs`, `src/config.rs`, `src/lib.rs`
- `crates/codegen/xai-grok-shell/src/session/acp_session_impl/{sampler_turn,status_line,session_mode,model_switch,turn}.rs`
- `crates/codegen/xai-grok-shell/src/agent/{config.rs,subagent/mod.rs,handlers/model_switch.rs,mvp_agent/session_setup.rs}`
- `crates/codegen/xai-grok-shell/src/{tools/config.rs,session/persistence.rs,session/plan_mode.rs}`
- `crates/codegen/xai-chat-state/src/{actor/request_builder.rs,usage.rs}`
- `crates/codegen/xai-grok-status-line/src/{config,context,pricing}.rs`
- `crates/codegen/xai-grok-pager/src/views/status_line/segments.rs`, `src/app/status_line_policy.rs`
- `crates/codegen/xai-grok-pager/src/{headless.rs,app/actions.rs,app/dispatch/settings/setters.rs}`
- `crates/codegen/xai-grok-pager/src/slash/commands/{model,effort,plan,resume,usage}.rs`
- `crates/codegen/xai-grok-pager/src/views/{permission_view,plan_approval_view}.rs`
- `crates/codegen/xai-grok-workspace/src/permission/{prompter.rs,auto_mode/security_findings.rs}`
- `crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs`
- `crates/codegen/xai-grok-models/default_models.json`
- `crates/codegen/xai-grok-test-support/src/{mock_server,request_log,inference_request}.rs`
- `crates/codegen/xai-grok-pager-pty-harness/src/{lib.rs,scenarios/plan_approval_resume.rs}`
- `crates/codegen/xai-grok-pager/docs/user-guide/{14-headless-mode,16-subagents,17-sessions,19-plan-mode,22-permissions-and-safety,25-status-line,26-config-reference}.md`
- `docs/TUNING.md`, `.planning/PROJECT.md`, `.planning/codebase/{CONCERNS,ARCHITECTURE}.md`

---
*Feature research for: personal DeepSeek-only terminal coding agent, v0.1.0 freeze*
*Researched: 2026-11-21*
