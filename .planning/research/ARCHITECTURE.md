# Architecture Research — Placement of Five Changes at the v0.1.0 Freeze

**Domain:** existing Rust terminal coding agent (95 workspace members, 81 product crates)
**Researched:** 2026-11-21
**Base:** `main` @ `221a1a59` (the rc5 lineage)
**Confidence:** HIGH for every placement claim (each is grounded in a file read at the line cited);
MEDIUM for the pin-persistence choice and the status-line segment shape, which are design decisions the
planner must lock, not facts in the tree.

---

## Scope: this document does not propose an architecture

The system is fixed and already documented in `.planning/codebase/ARCHITECTURE.md`. This document answers
one question only:

> For each of the five changes, which component owns the decision, what crosses a boundary, and in what
> order should they land so nothing ends up with two sources of truth?

Everything below is a **placement decision** on the existing seams:

```
xai-grok-pager  ──ACP──▶  xai-grok-shell (SessionActor)  ──▶  xai-chat-state (request assembly)
   │                              │                                    │
   │                              ▼                                    ▼
   │                        xai-grok-sampler  ◀── SamplingConfig ── ChatStateHandle
   │                        (SamplerActor → SamplingClient → POST /chat/completions)
   │
   └── status line ◀── XaiSessionUpdate::SessionStatus(StatusLineContext) ── build_status_context()

xai-grok-sandbox  ◀── tool dispatch (child process launch)
xai-grok-pager-pty-harness / xai-grok-test-support  ◀── evidence only, never linked by product code
```

### The five seams, at a glance

| # | Decision to place | Owner (after the change) | What crosses a boundary |
|---|---|---|---|
| 1 | Which model/effort this sampling request uses | `SessionActor::run_turn_via_sampler` (one call site) | the mutated `ConversationRequest` → `SamplerHandle` |
| 2 | Whether routing may act at all (pin), and what the row shows | pin: `SessionActor` (set by the two `/model` `/effort` handlers); display: `StatusLineContext.route` | pin → routing input; route decision → ACP `SessionStatus` → pager row |
| 3 | What the session cost | `UsageLedger.by_model` (usage) × `PricingTable` (price) | `PromptUsage.model_usage` → `StatusLineContext` / headless `modelUsage` |
| 4 | Whether a runtime socket can be masked | `runtime_sockets::materialize_runtime_socket_deny_paths_from` | materialized `PathBuf`s → `SandboxProfile.deny` → bwrap argv |
| 5 | What counts as evidence | `xai-grok-pager-pty-harness` / `xai-grok-test-support` (producers) → `docs/evidence/` (committed) | harness artifacts → repo files |

---

## Item 1 — Model routing: where the decision is made

### What exists today (verified)

- `resolve_route(&ModelRoutingConfig, &ConversationRequest) -> Option<ModelRoute>` is a pure function
  (`crates/codegen/xai-grok-sampler/src/model_routing.rs:186`). `classify_turn` is also pure and reads only
  the request (`:159`).
- `ModelRoutingConfig` carries `policy`, `flash_model`, `pro_model`, `pro_effort`, `flash_effort`,
  `manual_override` (`:57-70`).
- `SamplingClient::route_turn` (`client.rs:611`) is the only caller of `resolve_route`, and **nothing calls
  `route_turn`**. It builds `ModelRoutingConfig { pro_model: self.defaults.model, ..Default::default() }`.
- The policy reaches the sampler as `SamplingConfig.model_routing` (`sampler/src/config.rs:52` →
  `ClientDefaults.model_routing`, `client.rs:263,572`). Every construction site sets
  `Default::default()` — `shell/src/session/acp_session_impl/sampler_turn.rs:729`,
  `shell/src/agent/config.rs:4328`, `shell/src/agent/subagent/mod.rs:779`,
  `shell/src/tools/config.rs:212`. **No config-file reader exists** for the key (grep for `model_routing`
  outside `xai-grok-sampler/src` returns only those four literals).
- The request is assembled in `xai-chat-state/src/actor/request_builder.rs:79-104`, which sets
  `model: Some(sampling_config.model)` (`:84`) and `reasoning_effort: sampling_config.reasoning_effort`
  (`:99`).
- The sampler honours a per-request model over the client default: `if request.model.is_none() { request.model = Some(self.defaults.model.clone()) }` (`client.rs:795`, again at `:1178`).
- The wire body takes `reasoning_effort` from the **request**, and derives `thinking` from it
  (`sampling-types/src/conversation/chat_completions.rs:338-346,387`). There is no second effort source in
  the body builder.
- The main-loop funnel is `SessionActor::run_turn_via_sampler` (`sampler_turn.rs:1692`); it has exactly
  one production caller, `turn.rs:2951`, and it forwards to `submit_turn_request` (`:1757`) which calls
  `sampler_handle.submit_and_collect_with_metadata(request_id, request)` (`:1782`). Side calls
  (compaction, the auto-mode classifier) build their own clients and never enter this funnel.

### Placement

**Owner: `SessionActor::run_turn_via_sampler`, at entry, before `prepare_sampler_for_turn`.**

```rust
// sampler_turn.rs, run_turn_via_sampler entry (~1692)
let (request, route) = self.apply_turn_route(request).await;   // mutates model + reasoning_effort
self.record_last_turn_route(route);                            // item 2's slot
if !park.is_parked() { self.prepare_sampler_for_turn().await; }
```

Why this exact spot and not the alternatives:

| Candidate | Verdict |
|---|---|
| `turn.rs` right after `build_request` (`:2902`) | Works, but it is one caller of the funnel. A future caller (e.g. a subagent path, a retry re-entry) would bypass the decision. The funnel is where "one decision per sampling request" is enforceable. |
| Inside `submit_turn_request` (`:1757`) | Re-decides on every resubmit of the same request (`run_turn_via_sampler` loops `request.clone()` at `:1724`). Deterministic today, but it makes the decision "per HTTP attempt" rather than "per sampling request" and it has no place to record the outcome once. |
| Inside the sampler (`SamplingClient`) | Rejected. The sampler would then own a decision the session cannot observe, so the status line would have to re-derive it — the exact divergence this milestone forbids. It also cannot see the pin, which lives session-side. |
| In `xai-chat-state` at `build_request` | Rejected. Chat state owns the pinned/config model (`request_builder.rs:84`); if it also routed, the pin and the route would be the same field and "clear the pin" would be unexpressible. |

### What crosses the boundary, and in which direction

```
chat state ──(request with pinned model)──▶ SessionActor::run_turn_via_sampler
                                                   │
                              resolve_route(policy, pin, request)
                                                   │
                     request.model = route.model        (mutate, by value)
                     request.reasoning_effort = route.reasoning_effort
                                                   │
                                                   ├──▶ SamplerHandle ──▶ SamplingClient ──▶ POST body
                                                   └──▶ last_turn_route slot (item 2)
```

The mutation is local: `run_turn_via_sampler` takes `request: ConversationRequest` **by value**
(`sampler_turn.rs:1694`), so the routed fields are the only fields the sampler can see. Divergence is
structurally impossible — there is no other model source for that request (`client.rs:795` only fills an
absent model).

### Configuration: where the policy comes from

Follow the existing `resolve_*_from_disk` convention (`shell/src/util/config/resolve/auto_mode.rs:177`):

1. Add the key to the config layer. Recommended home: `ModelsConfig` in `shell/src/agent/config.rs:387`
   (`[models] routing = "auto"`, plus optional `flash` / `pro` / `pro_effort` overrides), because
   `default` and `default_reasoning_effort` already live there (`:389`, `:402`).
2. New module `shell/src/util/config/resolve/model_routing.rs`, re-exported from `resolve/mod.rs`
   (glob pattern at `resolve/mod.rs:1-30`).
3. Resolve **once at spawn** into a `SessionActor` field (the `max_retries: u32` pattern,
   `acp_session.rs:852`, resolved in `spawn.rs:536`), then read it in `run_turn_via_sampler`. Do not
   re-read config layers per turn.
4. Do **not** add the policy to `xai_grok_sampling_types::SamplingConfig`: that type is
   `Serialize/Deserialize` and persisted per session (`types.rs:1214`), so a config choice would stick to
   old sessions.

### Delete the second entry point

Because the session now decides, the sampler must have no opinion:

- Remove `SamplingConfig.model_routing` (`sampler/src/config.rs:52`), `ClientDefaults.model_routing`
  (`client.rs:263`, assigned at `:572`), `SamplingClient::route_turn` (`:611`) and
  `model_routing_policy` (`:601`).
- That deletes all four `model_routing: Default::default()` literals listed above, so there is exactly one
  place a policy is read.
- `xai-grok-sampler` is adapter-owned (`AGENTS.md` §5). This is a deliberate adapter change, not a scrub;
  keep it in its own commit so the adapter diff is reviewable.

### Add the reason (needed by item 2)

`ModelRoute` (`model_routing.rs:46-54`) carries no reason. Add
`pub reason: RouteReason` to `ModelRoute` and fill it inside `resolve_route` — the only place that already
knows which arm won:

| Arm | Reason |
|---|---|
| `manual_override` wins (`:190`) | `Pinned` |
| `Low` (`:195`) | `PolicyLow` |
| `High` (`:196`) | `PolicyHigh` |
| `Auto` + `Trivial` (`:198`) | `AutoTrivial` |
| `Auto` + `Complex` (`:199`) | `AutoComplex` |

One field, one derivation, no second `classify_turn` call anywhere.

### Failure modes this placement removes

- Policy set in config but never read (today's state).
- A route decided twice (session + status line) and disagreeing.
- A route decided in the sampler and invisible to the session.

---

## Item 2 — Pin state and route visibility

### What exists today (verified)

- `/model` crosses ACP as `SetSessionModelRequest` (`pager/src/app/effects/mod.rs:1909`,
  `headless.rs:806`) → `SessionCommand::SetSessionModel` (`session/commands.rs:468`) →
  `handle_set_session_model` (`session/acp_session_impl/model_switch.rs:5`), which writes the model into
  `chat_state.sampling_config` (`:79-101`).
- `/effort` crosses as `SessionCommand::SetReasoningEffort` (`commands.rs:474`, sent from
  `agent/handlers/model_switch.rs:318`) → `handle_set_reasoning_effort` (`model_switch.rs:241`), which
  writes `cfg.model` (possibly remapped by `model_for_effort`, `:255`) and
  `cfg.reasoning_effort` (`:258`), then updates chat state (`:260`) and wakes the status line (`:271`).
- Both handlers already call `emit_status_snapshot_detached()` (`:171`, `:271`), so the row refreshes
  after an explicit assignment with no new wiring.
- The status line payload is built **only** by `SessionActor::build_status_context`
  (`session/acp_session_impl/status_line.rs:118`). It reads `model` and `reasoning_effort` from
  `chat_state.get_sampling_config()` (`:119-127`) — i.e. the pinned/config model, never the routed one.
  It is pushed as `XaiSessionUpdate::SessionStatus` (`:269`) and stored by the pager at
  `app/acp_handler/session_notification.rs:1328`.
- `StatusLineContext` is a public wire contract with three coupled files: the type
  (`status-line/src/context.rs:18`), the fixture `status-line/testdata/status_wire.json`, and the guide
  `pager/docs/user-guide/25-status-line.md`. A test fails if a wire field is not named in the guide
  (`pager/src/app/status_line_tests.rs:356-396`).
- The row already reserves space for this feature:
  `// TODO(auto-routing): … add a `route` segment here that renders the route only when the payload names
  one, and mark the manual override … distinctly` (`pager/src/views/status_line/segments.rs:66-69`).
- `StatusLineItem` is a strum enum (`status-line/src/config.rs:345-358`) with a `varies_mid_turn`
  classifier (`:363-374`).

### Placement: the pin

**Owner: `SessionActor`, set in the two handlers that already receive an explicit assignment, read by the
router.** One field, one writer pair, one reader:

```rust
// SessionActor
pin: std::sync::Mutex<Option<xai_grok_sampler::ModelRoute>>,   // Some(_) = routing suspended
```

- `handle_set_session_model` sets `pin = Some(ModelRoute { model: <new model>, reasoning_effort: <current effort>, thinking: <derived>, reason: Pinned })` in the same call that writes chat state (`model_switch.rs:79-101`).
- `handle_set_reasoning_effort` sets the same pin from the resulting `cfg.model` + `effort` (`:255-260`).
- `run_turn_via_sampler` passes `pin.clone()` as `ModelRoutingConfig.manual_override`
  (`model_routing.rs:69`), which already wins over the policy (`:190`).

Why the pin belongs on the session actor and not in config, chat state, or the sampler:

| Candidate | Why not |
|---|---|
| Config layer (`[models]`) | A pin is per session, not per user. Writing it to config would make `/model` a global default change, which is what `set_default_model` already does. |
| `xai_grok_sampling_types::SamplingConfig` | Persisted (`types.rs:1214`). The pinned model is already there; adding a second "pinned" copy of the same fact is the duplication to avoid. |
| Sampler / `SamplerConfig` | The sampler is rebuilt every turn from chat state (`prepare_sampler_for_turn` → `reconstruct_full_config`, `sampler_turn.rs:1011,631`). A pin there would be reconstructed from config and lost on the next prepare. |
| `xai-chat-state` | Would need a new command and a second copy of model+effort that must be kept in step with `sampling_config`. |

**Do not store `manual_override` separately from the pin.** The pin *is* the override; `manual_override`
is derived at the call site. One fact, one field.

### Placement: the display

**Owner: the route decision slot on `SessionActor`, read by `build_status_context`. The status line must
never re-run `classify_turn`.**

```rust
// SessionActor
last_turn_route: std::sync::Mutex<Option<xai_grok_sampler::ModelRoute>>,  // written by run_turn_via_sampler
```

- Written once per sampling request, by the router (item 1), immediately after `resolve_route`.
- Read by `build_status_context` (`status_line.rs:118`) and emitted as a new additive payload field.
- Wake the emitter once per user turn, not once per sampling request: `build_status_context` costs a git
  discovery plus four chat-state round trips (`status_line.rs:119,136,141,160`). Track a
  `turn_route_announced: AtomicBool` reset at turn start; the existing turn-end wake
  (`turn_end.rs:116,446`) covers the rest.

New payload field (additive; `STATUS_LINE_SCHEMA_VERSION` stays `1` — the type's own comment says adding
a field never bumps it, `context.rs:12-14`):

```rust
// status-line/src/context.rs
pub struct StatusLineRoute {
    pub model: String,          // the model this request asked for
    pub reason: StatusLineRouteReason,   // pinned | policy_low | policy_high | auto_trivial | auto_complex
    pub pinned: bool,           // the session is pinned; routing is suspended
}
// on StatusLineContext:  pub route: Option<StatusLineRoute>,
```

Semantics to hold the two model names apart:

| Field | Meaning | Never |
|---|---|---|
| `model.id` / `model.display_name` | the session's pinned/configured model — the identity `/model` shows | used for pricing (item 3) |
| `route.model` | the model the last sampling request actually asked for | recomputed by the pager |
| `route.pinned` | routing is suspended for this session | inferred from `model != default` |
| `effort.level` | from the route when routed, else the pinned effort | read from chat state when a route exists |

### The segment

Implement the TODO at `segments.rs:66-69`, with one adjustment to avoid printing two model names on one
row:

- `StatusLineItem::Model` renders `ctx.route.model` when present, else `ctx.model.display_name`
  (today: `segments.rs:83-90`).
- New `StatusLineItem::Route` renders only the reason: `auto`, `auto · pinned`, `policy low`, etc. Add it
  to the enum (`status-line/src/config.rs:345`), mark `varies_mid_turn() == true` (`:363`), and add the
  guide row.
- `StatusLineItem::Effort` (`segments.rs:109-113`) needs no change; it already reads `ctx.effort`.

### Clearing the pin

`handle_set_session_model` receives `SetSessionModelRequest` which already carries `meta`
(`headless.rs:806` shows `.meta(meta)`). Recommended clear path, cheapest first:

1. A `_meta` flag on the existing `set_session_model` request (no new ACP method, no vendored-protocol
   change).
2. A `/model auto` (or `/model reset`) form in the pager slash controller that sends the same request with
   the flag.

Two decisions the planner must record in `DECISIONS.md` (not derivable from code):

- **Pin granularity.** Recommended: any explicit `/model` *or* `/effort` pins the whole session (model +
  effort) until cleared. A field-level pin would require synthesising a full `ModelRoute` from a partial
  one at decision time — a second derivation, which is the failure mode this milestone exists to remove.
  Note `/effort` does not actually change the model id for DeepSeek: `model_at` falls back to `self.model`
  when no variant matches (`agent/config.rs:3505-3522`) and the catalog entries carry no variants.
- **What clearing restores.** Recommended: clearing drops the pin and leaves `sampling_config.model` as
  the user's last explicit choice. With `policy = off` that means "clearing" keeps the pinned model
  visible; document it rather than silently resetting to the catalog default.

### Persistence (MEDIUM confidence — a decision, not a fact)

`PersistenceMsg::CurrentModel` (`session/persistence.rs:260-267`) persists `model_id`, `agent`,
`reasoning_effort`, `context_window`. If a pin must survive resume, add an optional
`pinned: Option<bool>` with `#[serde(default)]` there, or accept "a resumed session whose model differs
from the configured default is pinned". The first is explicit; the second infers and can flip when a
release changes the default. Recommend the explicit field.

---

## Item 3 — Cost and cache from the real usage

### What exists today (verified)

**Usage has one source and it is real.** `UsageLedger` keeps `totals` **and** `by_model:
IndexMap<String, UsageTotals>` (`xai-chat-state/src/usage.rs:107-114`). The writer is
`record_model_call_usage` (`actor/mutations.rs:449-471`), fed from the response by
`record_response_token_usage` (`sampler_turn.rs:2189-2205`) with
`response.assistant().model_id` — the id the provider echoed in the SSE chunk
(`stream/chat_completions.rs:135,293`). `TokenUsage` is the wire usage
(`sampling-types/src/conversation.rs:773-788`), not an estimate.

**Price has one implementation** — `PricingTable::cost_usd(model_id, &StatusLineSessionUsage)`
(`status-line/src/pricing.rs:109-125`) — but **three call sites, all of which price the whole session at
one model id**:

| Site | Model it uses | File:line |
|---|---|---|
| TUI status line | `ctx.model.id` (session/pinned model) | `pager/src/app/status_line_policy.rs:275-290` |
| Headless JSON result | `self.session_model` (from the init context) | `pager/src/headless.rs:313-340` (model at `:330`, set at `:184`) |
| Headless streaming-messages reducer | `self.session.model` | `pager/src/headless/reducer/messages/usage.rs:103-117` |

**And the price table has a hole for the third catalog id.** `PricingTable::deepseek_defaults()` seeds
only `deepseek-v4-pro` and `deepseek-flash` (`pricing.rs:78-83`), while
`crates/codegen/xai-grok-models/default_models.json` ships three entries: `deepseek-v4-pro`,
`deepseek-flash`, and the hidden `deepseek-v4-flash`. `for_model` falls back to the Pro default for an
unmatched id (`pricing.rs:97-102`), so a turn on `deepseek-v4-flash` would be estimated at Pro rates —
~4x overstatement. This is a live defect the moment routing can select more than one model.

### Placement

**Owner of usage: `UsageLedger.by_model` (already correct — do not touch it).
Owner of price: `PricingTable` (already correct — do not add arithmetic anywhere else).
The change is plumbing the per-model buckets to the three consumers and making each consumer price per
bucket.**

1. **Add one pricing method** in `status-line/src/pricing.rs`:

   ```rust
   pub fn cost_usd_by_model(&self, buckets: &[(String, StatusLineSessionUsage)]) -> Option<f64>
   ```

   It sums `cost_usd(model, usage)` over the buckets and returns `None` when no bucket has tokens. Every
   existing `cost_usd` test still applies; this is composition, not a second formula.

2. **Carry the buckets in the payload.** Add to `StatusLineContext` (additive, same schema version):

   ```rust
   // on StatusLineContextWindow
   pub session_usage_by_model: Option<Vec<StatusLineModelUsage>>,  // {model, usage}
   ```

   `xai-grok-status-line` depends only on `serde` + `strum` (`status-line/Cargo.toml`), so use a `Vec`,
   not `IndexMap`. Fill it in `build_status_context` from `usage.model_usage` — the shell already has it
   (`PromptUsage.model_usage`, `extensions/notification.rs:113`, projected from the ledger at `:308-312`);
   today `build_context_window` uses only `totals` (`status_line.rs:64-106`).

3. **Change the three consumers to use the buckets, with the aggregate as fallback** (a session with one
   model, or an older payload, must keep working):

   | Consumer | Change |
   |---|---|
   | `status_line_policy.rs:275` | price `session_usage_by_model` via `cost_usd_by_model`; fall back to today's `cost_usd(ctx.model.id, session_usage)` |
   | `headless.rs:313` | price the `modelUsage` rows already present in the result object (`notification.rs:391-424`); fall back to `session_model` |
   | `reducer/messages/usage.rs:103` | same, from the `modelUsage` rows it already maps (`:122-146`) |

4. **Fill the price-table hole.** Add `deepseek-v4-flash` to `deepseek_defaults()` at Flash rates, or
   alias it to the `deepseek-flash` entry. Otherwise per-model pricing silently reintroduces the Pro
   fallback for the hidden alias.

### What must NOT change

- **Do not price from `route.model`.** The route is what was *asked for*; the ledger is what the provider
  *reported*. If they disagree (alias echo, provider substitution), money follows the ledger. This is the
  one place where "the decision" is deliberately not authoritative.
- **Do not touch the tokens or cache segments.** `format_tokens` and `cache_hit_rate()` are
  model-agnostic aggregates over disjoint buckets (`segments.rs:141-151`, `context.rs:188-200`). They stay
  session-wide. Only `cost.total_cost_usd` is model-dependent.
- **Do not add a runtime price fetch.** D14 and PROJECT.md forbid it; the table stays a local estimate
  with `estimate: true` in headless output.

### Data flow

```
SSE chunk.model + usage ──▶ ConversationResponse.usage / assistant.model_id
        │
        ▼
record_response_token_usage (sampler_turn.rs:2189)
        │
        ▼
ChatState.record_model_call_usage(model_id, usage, ...)  (mutations.rs:449)
        │
        ├──▶ UsageLedger.totals          ──▶ StatusLineContext.context_window.session_usage (aggregate)
        └──▶ UsageLedger.by_model[model] ──▶ PromptUsage.model_usage
                                               ├──▶ StatusLineContext…session_usage_by_model (new)
                                               └──▶ headless `modelUsage` rows (already)
        │
        ▼
PricingTable::cost_usd_by_model  ──▶ cost.total_cost_usd / estimated_cost_usd
```

One ledger, one table, one sum.

---

## Item 4 — Sandbox runtime-socket denies

### What exists today (verified)

- The only resolver is `materialize_runtime_socket_deny_paths_from`
  (`sandbox/src/runtime_sockets.rs:101-157`). It canonicalizes the **parent** (`:127`), then stats the
  endpoint (`:141`), rejects symlinks (`:146`), and dedupes (`:152`).
- Every error other than a `NotFound` parent (or a missing endpoint) is returned:
  `Err(error) => return Err(with_context(error))` (`:138`). That includes `PermissionDenied` on a parent
  the process cannot traverse — the `/run/containerd` case in `CONCERNS.md`.
- The error propagates to `resolve_profile_with_runtime_sockets` → `append_socket_denies`
  (`sandbox/src/profiles.rs:366-399`), which maps it to an error, so the profile fails to resolve and the
  sandbox refuses to start (fail-closed).
- Inside bwrap the paths are not re-discovered: they are handed over as JSON in
  `__GROK_BWRAP_RUNTIME_SOCKET_DENY` (`runtime_sockets.rs:13,75-98`, encoded at
  `sandbox/src/lib.rs:364-376`) and re-validated against the static policy
  (`:168-194`). An empty handoff yields no denies (`:92`).
- bwrap is invoked **without** `--unshare-user` or `--uid`, with `--cap-drop ALL` and `--bind / /`
  (`sandbox/src/lib.rs:328-330`). The child therefore runs as the same uid with the same filesystem
  permissions: a parent the launching process cannot traverse is equally untraversable inside bwrap. That
  is the security argument for skipping, and it is visible in the argv.
- There is already a test that non-absence parent errors propagate
  (`runtime_sockets_tests.rs:260-278`, using a symlink loop = `ELOOP`).

### Placement

**Owner: `materialize_runtime_socket_deny_paths_from`, and only there.** The traversability decision is a
property of resolving a candidate, so it belongs beside the `dunce::canonicalize(parent)` call
(`:127`), not in `profiles.rs` (which decides *which* candidates) and not in the bwrap argv builder
(which only consumes materialized paths).

```rust
// runtime_sockets.rs, inside the candidate loop
let canonical_parent = match dunce::canonicalize(parent) {
    Ok(parent) => parent,
    // Parent exists but this process may not traverse it: the same uid inside bwrap
    // (no --unshare-user) cannot reach the endpoint either, and a bwrap --ro-bind
    // would abort the launch. Skip it like an absent endpoint.
    Err(error) if error.kind() == io::ErrorKind::PermissionDenied => continue,
    Err(error) if error.kind() == io::ErrorKind::NotFound => { /* existing block, unchanged */ }
    Err(error) => return Err(with_context(error)),   // ELOOP, EIO, … stay fail-closed
};
```

### The fail-closed boundary (state it in the code comment)

| Situation | Behaviour | Why |
|---|---|---|
| Parent missing (`NotFound`) | skip | nothing to mask |
| Endpoint missing | skip | nothing to mask |
| Endpoint is a symlink | **error** | a symlink could redirect the mask; never silently follow |
| Parent not traversable (`EACCES`) | **skip (new)** | the child, same uid and no user namespace, cannot reach it either; a bind would abort the whole sandbox |
| Parent resolution fails otherwise (`ELOOP`, `EIO`) | **error** | the process can see *something* it cannot classify; fail closed |
| Inside bwrap, handoff malformed or outside the policy | **error** | provenance check, unchanged (`:168-194`) |

The distinction to preserve: **skip only what the process provably cannot reach; error on anything it can
see but cannot mask.**

### One residual, flag it rather than fix it blind

`merge_runtime_socket_denies` (`:264-294`) calls `normalize_existing_parent_alias`, which canonicalizes
the nearest existing ancestor (`:218-241`). If a user *explicitly* configured a deny path whose parent is
`EACCES`, that path still reaches this function with `auto_sockets` empty and would error. That is
arguably correct (an explicit deny the process cannot mask should stop the launch), but it should be a
recorded decision, not an accident.

### Tests and evidence

- Unit: create a temp dir, `chmod 0o000` a child dir, assert
  `materialize_runtime_socket_deny_paths_from([dir/docker.sock])` is `Ok(vec![])`. Skip when
  `geteuid() == 0` (root ignores mode bits) — the existing `temp_runtime_root` helper
  (`runtime_sockets_tests.rs:105`) is the base.
- Keep `materialized_socket_paths_propagate_parent_resolution_errors` green (`:262`): `ELOOP` must still
  error.
- E2E: `deny_paths_e2e.rs:1062` and `:1142` (the two recorded host failures) should go green on the
  author's host without editing them. If they still fail, the fix is wrong — do not relax
  `skip_if_enforcement_unavailable` (`:84`).
- This item is what makes the "zero failures on the author's host" gate reachable, so it has independent
  value even if the other four slip.

---

## Item 5 — Where the evidence lives and what produces it

### What exists today (verified)

- `xai-grok-pager-pty-harness` drives the real binary under a PTY. `PtyHarness::write_cast` writes an
  asciinema v2 cast (`pty-harness/src/lib.rs:484-514`); `ScriptedScenarioRunner` writes `report.json`
  plus `text` / `html` / `svg` / `styled_json` captures (`scripted.rs:1041-1065`, report shape at
  `:1136-1250`).
- The `pty-scenario` binary already exposes the artifact directory and a binary override:
  `--scenario`, `--binary`, `--artifacts` (default `target/pty-scenarios`)
  (`pty-harness/src/bin/pty_scenario.rs:26-33`).
- The mock inference server records every request body: `LogEntry.body: Option<Value>`
  (`test-support/src/request_log.rs:33`), surfaced through `flows::inference_requests`
  (`pty-harness/src/flows.rs:44`). That is how a scenario asserts **which model went on the wire**,
  independently of the status line.
- Headless runs: `test-support/src/headless.rs:32` `run_headless` returns stdout/stderr/status against a
  `MockInferenceServer`; `test-support/src/acp_transcript.rs` records the ACP-level transcript.
- There is no evidence directory in the tree, and `target/` plus `scripts/evals/results/` are the only
  ignored artifact paths (`.gitignore`).

### Placement

**Producers: the harness crates (never product code).**
**Destination: a new committed `docs/evidence/<item>/` directory.**

| Layer | Owner | Why there |
|---|---|---|
| Scenario definition | `pty-harness/src/scenarios/*.rs` (programmatic) or `pty-harness/tests/scenarios/*.yaml` (declarative) | Both are already supported; YAML keeps a reviewable one-file scenario |
| Run + artifacts | `pty-scenario --artifacts target/pty-scenarios/<item>` | Keeps build output out of the tree |
| Headless transcript | `xai-grok-test-support` `run_headless` + `MockInferenceServer` | Same mock server, no second driver |
| Committed evidence | `docs/evidence/<item>/` | `docs/` is the human-facing artifact home; `.planning/` is milestone state that gets archived at `complete-milestone` |

Why not the alternatives: `scripts/evals/results/` is gitignored by design; `target/` is build output;
`.planning/` is process state, not a durable artifact, and the freeze tag must carry the evidence.

Layout:

```
docs/evidence/
├── README.md                     # item → scenario → exact command → expected assertion
├── routing-wired/
│   ├── scenario.yaml
│   ├── report.json               # normalized (see below)
│   ├── screen.txt                # final ScreenTracker text
│   └── requests.json             # model field of each /chat/completions body
├── route-visibility-pin/
├── cost-per-model/
└── sandbox-runtime-sockets/
```

### The one trap in this item

`ScriptedRunReport` embeds absolute paths — `artifact_dir: PathBuf` (`scripted.rs:1146`) and each
`VisualArtifact.path` (`:1236-1239`). Committed as-is, they leak the author's home directory and make the
evidence non-reproducible. Add a normalizer (a small `scripts/normalize-pty-report.py`, consistent with
the Python already under `scripts/evals/`) that rewrites those fields to repo-relative paths before the
file is copied into `docs/evidence/`. Hand-edited evidence is a second, fictional source of truth and
defeats the purpose.

### What each item's evidence asserts

| Item | Assertion | Where it is checked |
|---|---|---|
| 1 | the request body's `model` equals the routed model for a trivial turn and the pro model for a complex one | `inference_requests` body, `request_log.rs:33` |
| 1 | the routed `reasoning_effort` reaches the body | same body, field at `chat_completions.rs:387` |
| 2 | the row shows the routed model + reason, and shows the pin after `/model` | `ScreenTracker` text capture |
| 2 | after a pin, the request body's `model` never leaves the pinned value | `inference_requests` across turns |
| 3 | `cost` reflects the per-model sum (Flash turn cheaper than the same tokens on Pro) | headless `estimated_cost_usd` + the status line row |
| 4 | sandbox starts on a host with a root-only `/run/containerd` and the child cannot reach the socket | pty/headless run + the e2e test |

---

## Build order

```
        ┌──────────────────────────────┐
        │ 5a: evidence convention      │  docs/evidence/ + normalizer + one worked example
        └──────────────┬───────────────┘
                       │ (no code dependency; every later phase appends its artifact)
   ┌───────────────────┴────────────────────────┐
   ▼                                            ▼
┌──────────────────────────────┐   ┌────────────────────────────────────────┐
│ 4: sandbox deny skip (EACCES)│   │ 1: routing decision wired              │
│ independent; unblocks the    │   │ run_turn_via_sampler + policy config + │
│ 2 recorded e2e failures      │   │ delete the sampler-side entry points   │
└──────────────┬───────────────┘   └───────────────────┬────────────────────┘
               │                                       │
               │                                       ▼
               │                       ┌────────────────────────────────────────┐
               │                       │ 2: pin + route visibility              │
               │                       │ StatusLineRoute + /model //effort pin  │
               │                       └───────────────────┬────────────────────┘
               │                                           │
               │                                           ▼
               │                       ┌────────────────────────────────────────┐
               │                       │ 3: per-model cost                      │
               │                       │ cost_usd_by_model + 3 consumers + table│
               │                       └───────────────────┬────────────────────┘
               │                                           │
               └───────────────┬───────────────────────────┘
                               ▼
                 ┌──────────────────────────────┐
                 │ 5b: evidence sweep          │  one transcript per item, index in docs/evidence/README.md
                 └──────────────────────────────┘
```

### Why this order, and what breaks if it is reordered

| Order | Reason |
|---|---|
| 4 anytime, early | It shares no state with 1–3. It removes the only known red tests, which the freeze gate needs, so landing it first de-risks the whole milestone. |
| 5a before everything | Every other item must produce an artifact. Building the convention first means each phase ends with evidence instead of a retrofit. |
| **1 before 2** | 2 displays a decision 1 creates and passes a pin into 1's config assembly. Landing 2 first gives the row nothing to show and the pin nothing to suppress. |
| **2 before 3** | Both edit `StatusLineContext` + `testdata/status_wire.json` + `25-status-line.md` (the guide test at `status_line_tests.rs:356`). Doing 2's `route` field first means 3's `session_usage_by_model` is a second additive change to an already-extended contract. Landing them in one phase with two plans is acceptable; landing 3 first would require re-pricing work that 1 invalidates. |
| 3 after 1 (hard) | Per-model cost is meaningless while only one model can be sent. Before 1, the three pricing sites are correct by accident. |
| 3 after 1 also fixes a latent bug | `deepseek-v4-flash` is unpriced (`pricing.rs:78-83` vs `default_models.json`). Routing makes that reachable; 3 must land with 1 in the same release. |

### Minimum safe commit sequence

1. `docs/evidence/` + normalizer + index (no product code).
2. Sandbox: skip `EACCES` parent + unit test + the two e2e tests green.
3. Routing: add `RouteReason`, add the policy config + resolver, apply the route in
   `run_turn_via_sampler`, delete `SamplingClient::route_turn` / `model_routing_policy` /
   `SamplingConfig.model_routing` and the four `Default::default()` literals. Policy still defaults to
   `off`, so behaviour is unchanged until step 4.
4. Routing on: `[models] routing` default `auto` (or documented opt-in), plus the wire-body assertion
   test. Do not ship 3 before 4 — a routed turn must not be priced at the wrong model.
5. Pin: `SessionActor.pin`, set in the two handlers, passed as `manual_override`; clear path.
6. Visibility: `StatusLineRoute` + guide + fixture + segment; wake once per turn.
7. Cost: `cost_usd_by_model`, `session_usage_by_model`, three consumers, `deepseek-v4-flash` pricing.
8. Evidence sweep.

Steps 3–6 are one phase (1 and 2 are one requirement in PROJECT.md: "the decision is visible on the
status line"). Step 7 is the next phase. Step 2 is independent and can run in parallel.

---

## Anti-patterns (the second-source-of-truth traps)

### Anti-pattern 1: the status line recomputes the route

**What people do:** `build_status_context` calls `classify_turn` on the last request to show the reason.
**Why it is wrong:** it runs on a different input (the last request may have been mutated, or the turn may
have ended) and can disagree with what was sent. Two derivations of one decision.
**Do this instead:** store the `ModelRoute` the router returned; read it.

### Anti-pattern 2: a pin that is a copy of the model field

**What people do:** keep `sampling_config.model` and a separate `pinned_model`, and try to keep them in
step.
**Why it is wrong:** `/model` writes one, `/effort` writes the other, and a resume restores a third
combination.
**Do this instead:** one `Option<ModelRoute>` on the session, written by the two handlers, consumed as
`manual_override`.

### Anti-pattern 3: pricing from the routed or configured model

**What people do:** `cost_usd(ctx.model.id, session_usage)` (today) or `cost_usd(ctx.route.model, …)`.
**Why it is wrong:** the first prices a Flash session at Pro rates; the second prices a provider
substitution at the requested model's rates. Both ignore the ledger's own per-model truth.
**Do this instead:** `cost_usd_by_model(ledger buckets)`; the ledger is the money.

### Anti-pattern 4: a policy the sampler also holds

**What people do:** leave `SamplingConfig.model_routing` in place "for later".
**Why it is wrong:** it is a second place to set a policy that nothing reads, and the next person will set
it and wonder why routing does not change.
**Do this instead:** delete it with the sampler-side entry points (step 3 above).

### Anti-pattern 5: a traversability check in `profiles.rs` or the bwrap builder

**What people do:** filter candidates in `resolve_profile_with_runtime_sockets`, or guard the `--ro-bind`
loop.
**Why it is wrong:** the outer materialization and the inner handoff would then disagree about which
paths are in the policy, and the handoff validation (`decode_bwrap_runtime_socket_denies_with_policy`,
`runtime_sockets.rs:168`) would reject a path the outer step accepted.
**Do this instead:** one skip inside the one resolver.

### Anti-pattern 6: hand-written evidence

**What people do:** paste a terminal screenshot into `docs/evidence/` and edit the paths in `report.json`.
**Why it is wrong:** the artifact no longer comes from a run, so it cannot regress or be re-derived.
**Do this instead:** run `pty-scenario` against the built binary, normalize, copy.

---

## Internal boundaries touched (one table for the planner)

| Boundary | Communication | Items | Notes |
|---|---|---|---|
| `xai-chat-state` → `xai-grok-shell` | `ChatStateHandle` commands (actor) | 1, 2, 3 | the request and the ledger both cross here; no new command is needed for 1 or 3 |
| `xai-grok-shell` → `xai-grok-sampler` | `SamplingConfig` + `ConversationRequest` | 1 | after the change the sampler has no routing opinion; the request is the only model source |
| `xai-grok-shell` → `xai-grok-pager` | ACP `XaiSessionUpdate::SessionStatus(StatusLineContext)` | 2, 3 | additive fields only; `schema_version` stays 1 |
| `xai-grok-pager` → `xai-grok-shell` | ACP `SetSessionModelRequest` (+ `_meta`) and `SessionCommand::SetReasoningEffort` | 2 | the clear-pin flag rides the existing request's `meta` |
| `xai-grok-sandbox` internal | `SandboxProfile.deny` → `bwrap_reexec_command_ex` argv | 4 | no cross-crate change; `runtime_sockets.rs` is the whole surface |
| harness → repo files | process artifacts → `docs/evidence/` | 5 | harness crates are never linked by product code (`pty-harness/Cargo.toml` header) |

### Constraints that apply to all five

- Production crates use `#![deny(clippy::indexing_slicing)]` (shell, sampler, pager, chat-state). Use
  `get`/`first`, never `[0]`.
- Paths through `dunce::canonicalize`; home through `xai-dirs`; HTTP only through `xai-grok-extra-ca`.
  Item 4 is the only item that touches path resolution — keep `dunce`.
- Actors own mutable state. `pin` and `last_turn_route` are session-actor state; do not put them in the
  pager, and do not keep a mutable copy in scrollback.
- Egress stays closed. None of the five adds a destination; item 3 explicitly does not fetch prices.
- Config keys require `26-config-reference.md` + the `FEATURES` registry (`config_docs` tests,
  `shell/src/config_docs/mod.rs:209-212`). New status-line fields require
  `testdata/status_wire.json` + `25-status-line.md` (`status_line_tests.rs:356`).
- Prompt templates are untouched by all five, so `scripts/encrypt_templates.py` is not in any plan.

---

## Open questions and unverified points

1. **UNVERIFIED — provider model echo.** The ledger buckets by the SSE chunk's `model` field
   (`chat_completions.rs:135`). Per-model pricing depends on DeepSeek echoing the requested id. If it
   echoes an alias, buckets may split or collapse. Verify with one live smoke turn before relying on
   per-model cost; the fallback (`cost_usd` on the aggregate) is the safety net.
2. **MEDIUM — pin persistence across resume.** Adding `pinned` to `PersistenceMsg::CurrentModel` is a
   session-file schema change. Decide explicitly (see item 2).
3. **MEDIUM — whether subagents route.** Subagent sessions run the same `run_turn_via_sampler` funnel, so
   they inherit the policy. `AGENTS.md` §3 lists "subagents (Flash for research)" as a separate Phase-4
   item. Decide whether to gate on `self.startup_hints.is_subagent` or let them route; do not leave it
   implicit.
4. **MEDIUM — what the row shows between turns.** `last_turn_route` survives a turn end. Showing the last
   routed model with no turn running is defensible; showing the pinned model is also defensible. Pick one
   and put it in `25-status-line.md`.
5. **UNVERIFIED — `route` as a default item.** Adding `StatusLineItem::Route` does not change the default
   `items` list; confirm the default row stays unchanged so existing users see no layout shift.
6. **Not re-litigated here:** the 2 bwrap e2e failures' exact host condition (`CONCERNS.md` records
   root-owned `/run/containerd`). Item 4's unit test proves the resolver; only the author's host proves
   the e2e.

---

## Sources (all read in this session)

| File | Used for |
|---|---|
| `.planning/PROJECT.md` | the five requirements, decisions, out-of-scope |
| `.planning/codebase/ARCHITECTURE.md` | existing seams, anti-patterns, constraints |
| `.planning/codebase/CONCERNS.md` | the four latent defects this milestone fixes |
| `crates/codegen/xai-grok-sampler/src/model_routing.rs` | pure decision, config shape, arms |
| `crates/codegen/xai-grok-sampler/src/client.rs` | `route_turn`, `ClientDefaults`, `request.model` precedence |
| `crates/codegen/xai-grok-sampler/src/config.rs` | `SamplingConfig.model_routing` |
| `crates/codegen/xai-grok-sampler/src/stream/chat_completions.rs` | echoed model, effort → thinking |
| `crates/codegen/xai-grok-sampling-types/src/types.rs` | persisted `SamplingConfig` |
| `crates/codegen/xai-grok-sampling-types/src/conversation.rs` | `ConversationRequest`, `TokenUsage` |
| `crates/codegen/xai-grok-sampling-types/src/conversation/chat_completions.rs` | wire body |
| `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` | funnel, prepare, usage record |
| `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn.rs` | `build_request` → `run_turn_via_sampler` |
| `crates/codegen/xai-grok-shell/src/session/acp_session_impl/status_line.rs` | payload builder |
| `crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs` | `/model`, `/effort` handlers |
| `crates/codegen/xai-grok-shell/src/session/commands.rs`, `run_loop.rs` | command plumbing |
| `crates/codegen/xai-grok-shell/src/util/config/resolve/auto_mode.rs` | config-resolution convention |
| `crates/codegen/xai-grok-shell/src/agent/config.rs` | `ModelsConfig`, `model_at` |
| `crates/codegen/xai-grok-chat-state/src/usage.rs`, `actor/mutations.rs`, `actor/request_builder.rs` | ledger, per-model buckets, request fields |
| `crates/codegen/xai-grok-status-line/src/{context,pricing,config}.rs` | payload contract, price table, item enum |
| `crates/codegen/xai-grok-pager/src/views/status_line/segments.rs` | the `TODO(auto-routing)` and segments |
| `crates/codegen/xai-grok-pager/src/app/status_line_policy.rs` | `fill_estimated_cost` |
| `crates/codegen/xai-grok-pager/src/headless.rs`, `headless/reducer/messages/usage.rs` | the other two price sites |
| `crates/codegen/xai-grok-pager/src/app/status_line_tests.rs` | guide-sync test |
| `crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs`, `profiles.rs`, `lib.rs`, `runtime_sockets_tests.rs`, `tests/deny_paths_e2e.rs` | deny resolution, bwrap argv, tests |
| `crates/codegen/xai-grok-pager-pty-harness/src/{lib,scripted,flows,env}.rs`, `bin/pty_scenario.rs`, `Cargo.toml` | evidence producers |
| `crates/codegen/xai-grok-test-support/src/{headless,request_log,acp_transcript}.rs` | headless driver, request bodies, transcripts |

---

*Architecture research for: deepseek-build v0.1.0 freeze — placement of five changes*
*Researched: 2026-11-21*
