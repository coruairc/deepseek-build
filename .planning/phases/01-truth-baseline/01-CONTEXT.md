# Phase 1: Truth Baseline - Context

**Gathered:** 2026-11-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Make the finished tree honest before anything is proven on it. Four bounded areas:

1. Sandbox: runtime-socket deny materialization must skip candidates whose parent directory is not traversable, while staying fail-closed for a traversable parent with an unreadable endpoint, so `deny_paths_e2e` passes unmodified.
2. Test gate: one scripted gate that runs the whole workspace plus the `config-docs` feature step, asserts completeness by target count, and resolves or quarantines the `git_gate`/`restore_fetch` flake.
3. Evidence convention: `docs/evidence/<phase>/<item>/` with an asciinema v2 cast, a fixed-schema `report.json`, and documented path normalization, so later phases can append and a reader can re-run.
4. Cost honesty and default model: price the hidden `deepseek-v4-flash` alias at Flash rates, label the TUI figure an estimate, make headless honour `[ui.status_line.pricing]`, and flip the catalog default to `deepseek-flash` with the caveat and revert path documented.

No new product surface, no routing, no new outbound destination. `scripts/sandbox-run.sh` also reports IPv6 destinations instead of `non-ip`.

</domain>

<decisions>
## Implementation Decisions

### Sandbox deny materialization
- Cover both non-traversable shapes: (A) `canonicalize(parent)` fails with `PermissionDenied` (an ancestor is unsearchable), and (B) `symlink_metadata(candidate)` fails with `PermissionDenied` while `symlink_metadata(parent.join("."))` also fails — the parent itself is unsearchable, which is the real `/run/containerd` shape and where `canonicalize` succeeds.
- Everything else stays fatal. A traversable parent with an unreadable endpoint cannot produce EACCES at this call site; the fail-closed arm is expressed against the parent-traversal check and any LSM denial, never against endpoint readability.
- A skipped candidate emits one `tracing::debug!` with the path only — no stderr noise.
- Tests: a unit matrix over missing parent (NotFound), shape A, shape B, mode-000 endpoint under a traversable parent (materialized), symlink endpoint (fatal), and parent-alias dedupe; plus `deny_paths_e2e` green on the author's host.
- The `devbox_genuine_reexec_applies_enforcement` failure is a host gap, not the deny-path bug: with no `/data`, devbox resolves to an empty plan and `bwrap_reexec_for_profile` correctly returns `None`. Fix by retargeting the scenario to a devbox-extending profile with an explicit deny (or `restrict_network = true`) so re-exec has a real obligation on every host — same assertions, no `#[ignore]`, no widened skip predicate.

### Test gate
- New `scripts/test-gate.sh`: `scripts/disk-guard.sh` first, then `flock -w 900 /tmp/team-build.lock`, then `cargo test --workspace --no-fail-fast`, then `cargo test -p xai-grok-shell --lib --features config-docs`.
- Completeness is asserted by counting `test result:` lines against a checked-in expected target count; the expected count is refreshed only via an explicit flag, never silently.
- `git_gate`/`restore_fetch`: attempt a deterministic fix first; if still flaky after two attempts, quarantine with `#[ignore = "<recorded reason>"]` and a `KNOWN-ISSUES.md` entry.
- A recorded run commits a normalized log plus `report.json` under `docs/evidence/01-truth-baseline/test-gate/`.

### Evidence convention
- Casts are produced by the existing pty harness `write_cast` (asciinema v2) — no external `asciinema` binary is installed. Headless-only items use `script(1)` where a pty is required.
- Layout: `docs/evidence/NN-<phase-slug>/<item-slug>/` containing `<item-slug>.cast`, `report.json`, optional `notes.md`.
- `report.json` fixed keys: `item`, `requirements[]`, `date`, `host`, `command`, `steps[]` (`name`, `ok`, `detail`), `artifacts[]` (`path`, `sha256`), `result` (`ok` | `fail` | `skipped`).
- Normalization: absolute repo/home/tmp prefixes become `<REPO>`/`<HOME>`/`<TMP>`; env dumps are never captured; `scripts/key-leak-check.sh` must pass over the committed artifacts.

### Cost honesty and default model
- Price `deepseek-v4-flash` explicitly alongside `deepseek-flash` at Flash rates in the shipped table, so the hidden catalog alias stops inheriting Pro rates.
- The TUI cost item carries an estimate label on every path, provider-reported or table-derived: `~$0.0042 est.` below $0.01, `~$0.37 est.` at/above, floor lowered to $0.0001. The label is unconditional because the segment has no provenance input, and separating it would change the `command`-script payload.
- Headless resolves the same `[ui.status_line.pricing]` table as the TUI for `attach_cost_estimate` and the streaming reducer, instead of `PricingTable::default()`.
- DEF-01 is implemented by making `default_models.json` authoritative in `first_or_fallback`: set `"default": "deepseek-flash"` and have the resolver prefer `xai_grok_models::default_model()` when it is visible. Picker order is unchanged; add tests for default-present, default-hidden, and default-absent.
- Document the at-ceiling caveat (63/63 both models) and the revert path (`/model deepseek-v4-pro`, `-m deepseek-v4-pro`) in the README quickstart, the user guide and `docs/TUNING.md`.

### UI design contract
- `01-UI-SPEC.md` is approved and binding: adaptive precision with fixed width per branch (4 dp below $0.01, 2 dp at/above), a promotion rule that renders `~$0.01 est.` rather than `~$0.0100 est.`, ASCII-only glyphs, the `dim` separator and `muted` text unchanged, no new color or spacing token, and `Cost` never added to `varies_mid_turn`.
- The status-line docs page (`25-status-line.md`) is updated with the spec's exact replacement copy; `/usage`, the raw `cost.total_cost_usd` payload, `README.md` and `14-headless-mode.md` are out of scope for the label.

### the agent's Discretion
- Exact wording and placement of the estimate label, the report.json helper shape, the expected-count file name, and whether the IPv6 report reuses the existing `egress_log.c` shim or extends `sandbox-run.sh`'s inline shim.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs:101` — `materialize_runtime_socket_deny_paths_from`; `dunce::canonicalize(parent)` currently turns EACCES into a fatal error that aborts bwrap startup.
- `crates/codegen/xai-grok-sandbox/src/profiles.rs:366` — `resolve_profile_with_runtime_sockets`, the caller that merges auto socket denies.
- `crates/codegen/xai-grok-sandbox/src/read_deny_verify.rs:433` — `verify_runtime_socket_deny`; already tolerates live/absent endpoints, must keep rejecting symlinks and exposed content.
- `crates/codegen/xai-grok-sandbox/src/runtime_sockets_tests.rs` — existing materialization tests (canonicalization, dedupe, mode-zero mask) to extend.
- `crates/codegen/xai-grok-status-line/src/pricing.rs` — `PricingTable::deepseek_defaults()` names `deepseek-v4-pro` and `deepseek-flash`; `deepseek-v4-flash` falls through to Pro.
- `crates/codegen/xai-grok-pager/src/headless.rs:308` — `attach_cost_estimate` hardcodes `PricingTable::default()`.
- `crates/codegen/xai-grok-pager/src/headless/reducer/messages/usage.rs:116` — the streaming path also hardcodes the default table.
- `crates/codegen/xai-grok-pager-pty-harness/src/lib.rs:485` — `write_cast` emits asciinema v2 (`{"version":2,"width":..,"height":..}` + `[elapsed,"o",data]`).
- `scripts/sandbox-run.sh` — inline `LD_PRELOAD` shim; the AF_INET6 branch allows `::1` but the blocked report defaults to `non-ip`.
- `scripts/disk-guard.sh`, `scripts/key-leak-check.sh`, `scripts/check-egress.sh` — existing gates the new script composes.

### Established Patterns
- Tests live beside the code (`*_tests.rs`) plus integration tests under `tests/`.
- User-visible strings resolve through `xai-grok-brand` constants; the `branding_surface` test gates CLI help/version.
- Config is layered and lenient (`Lenient<T>`), ignored-key collection is surfaced, and `26-config-reference.md` is hand-edited.
- Deletions are real deletions; no disabled stubs.

### Integration Points
- Sandbox: `runtime_sockets.rs` → `profiles.rs` → bwrap launch; the deny handoff is encoded for re-exec.
- Pricing: `xai-grok-status-line` config (`[ui.status_line.pricing]`) → pager status line and headless JSON.
- Default model: `crates/codegen/xai-grok-models/default_models.json` `"default"` field → catalog → CLI/TUI default.
- Test gate: new script at `scripts/`; evidence under `docs/evidence/`.

</code_context>

<specifics>
## Specific Ideas

- The failure is host-environment-specific: a root-owned `/run/containerd` parent is not traversable by the sandbox process, so canonicalization fails. The fix must skip only that condition, not weaken the deny set generally.
- The evidence convention's first consumer is Phase 2's live-session proof, so `report.json` needs a place for per-step assertions, not just a log.
- `deepseek-v4-flash` is a hidden catalog alias; it must keep working as a model id while pricing at Flash rates.

</specifics>

<deferred>
## Deferred Ideas

- Automatic model routing, `[models] routing`, session pins, `Route` status-line item, `/route`, per-model cost accounting — v2 / open-build.
- Installing `asciinema` or any new external tool — the pty harness already emits the format.
- `ci.yml` and debug-symbol sidecars — tracked as REL-V2-01/02.
- Renaming `GROK_*` env names or `xai-grok-*` crates — explicitly out of scope (D2).

</deferred>
