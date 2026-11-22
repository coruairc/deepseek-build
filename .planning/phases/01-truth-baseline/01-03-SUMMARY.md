---
phase: 01-truth-baseline
plan: 03
subsystem: cost-reporting
tags: [pricing, status-line, headless, tui, deepseek-flash, estimate]

# Dependency graph
requires:
  - phase: none
    provides: "the rc5 tree (base 711524dd): xai-grok-status-line pricing table, the status-line cost segment, both headless output paths"
provides:
  - "`deepseek-v4-flash` priced at Flash rates (0.006 / 0.30 / 1.20) instead of inheriting the v4 Pro entry"
  - "the TUI `cost` item rendered as `~$0.37 est.` / `~$0.0042 est.` with a $0.0001 floor"
  - "`HeadlessEmitter` and `MessagesReducer` pricing from the resolved `[ui.status_line.pricing]` table"
  - "a cross-surface test proving the TUI figure and both headless figures are the same float"
  - "the UI-SPEC replacement copy in `25-status-line.md`, one factual sentence in `14-headless-mode.md`"
affects: [01-04 (default model), any later phase reading `estimated_cost_usd` or the status row]

# Actuals (#2632) — chars/4 over the realized diff, not a harness token count.
actuals:
  tokens: 8126
  tasks: 3
  commits: 0
  plan_head_before: 711524dd454225eeb2bcbbb3924f7d8a83f2268e
  plan_head_after: 711524dd454225eeb2bcbbb3924f7d8a83f2268e

tech-stack:
  added: []
  patterns:
    - "resolve-once injection: a surface that needs the price table takes it as a constructor argument; `new()` keeps the shipped default for the existing test call sites"
    - "pure formatting helper (`format_cost`) so branch selection is unit-testable without a `StatusLineContext`"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-status-line/src/pricing.rs
    - crates/codegen/xai-grok-pager/src/views/status_line/segments.rs
    - crates/codegen/xai-grok-pager/src/views/status_line/segments_tests.rs
    - crates/codegen/xai-grok-pager/src/views/status_line/mod.rs
    - crates/codegen/xai-grok-pager/src/app/status_line_tests.rs
    - crates/codegen/xai-grok-pager/src/headless.rs
    - crates/codegen/xai-grok-pager/src/headless_tests.rs
    - crates/codegen/xai-grok-pager/src/headless/reducer/mod.rs
    - crates/codegen/xai-grok-pager/src/headless/reducer/messages/mod.rs
    - crates/codegen/xai-grok-pager/src/headless/reducer/messages/usage.rs
    - crates/codegen/xai-grok-pager/src/headless/reducer/messages/tests/result_usage.rs
    - crates/codegen/xai-grok-pager/docs/user-guide/25-status-line.md
    - crates/codegen/xai-grok-pager/docs/user-guide/14-headless-mode.md

key-decisions:
  - "The alias stays a separate table entry; `for_model` does not alias the two ids, so an override written for one does not silently move the other (the doc comment is the contract)."
  - "Headless derives the table from the already-loaded `raw_config` ([ui] -> UiConfig -> status_line.pricing()); it never calls `load_initial_ui_config()`, which would re-read the config file."
  - "`MessagesReducer::new()` and `HeadlessEmitter::new()` remain as default-table delegates, so ~18 existing call sites are untouched."
  - "Promotion is decided on the rounded 4-dp value (`(usd*10_000).round() < 100`), which is what turns 0.00996 into `~$0.01 est.` rather than `~$0.0100 est.`."

requirements-completed: [COST-01, COST-02, COST-03]

coverage:
  - id: D1
    description: "`deepseek-v4-flash` prices at Flash rates (0.006 / 0.30 / 1.20) and its worked example totals 0.001206 USD"
    requirement: COST-01
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-status-line --lib pricing#default_table_prices_both_labelled_models_at_peak"
        status: pass
      - kind: unit
        ref: "cargo test -p xai-grok-status-line --lib pricing#flash_alias_worked_example_is_exact"
        status: pass
    human_judgment: false
  - id: D2
    description: "the TUI cost item renders the UI-SPEC strings (`~$0.37 est.`, `~$0.0042 est.`, promotion at 0.00996, floor $0.0001), stays Dim, never splits"
    requirement: COST-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib status_line#cost_renders_the_estimate_at_fixed_width_per_branch"
        status: pass
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib status_line#composes_every_segment_in_order"
        status: pass
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib status_line#the_numbers_the_guide_states_are_pinned"
        status: pass
    human_judgment: false
  - id: D3
    description: "`25-status-line.md` carries the UI-SPEC replacement copy and keeps every guided item/key named"
    requirement: COST-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib status_line#every_builtin_item_is_named_in_the_guide"
        status: pass
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib status_line#every_config_key_the_section_serializes_is_named_in_the_guide"
        status: pass
    human_judgment: false
  - id: D4
    description: "both headless formats honour `[ui.status_line.pricing]`, and TUI + `json` + `streaming-messages-json` report the same float for one usage and one override"
    requirement: COST-03
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib headless#json_result_honours_status_line_pricing_override"
        status: pass
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib headless#messages_result_honours_status_line_pricing_override"
        status: pass
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib headless#tui_and_headless_report_the_same_cost_for_one_override"
        status: pass
    human_judgment: false

# Metrics
duration: ~58 min (not instrumented; two full `xai-grok-pager` rebuilds dominate)
completed: 2026-11-21
status: complete
---

# Phase 01 Plan 03: Cost Honesty Summary

**The hidden `deepseek-v4-flash` alias now prices at Flash rates, the TUI cost item is a labelled estimate with the UI-SPEC's exact precision, and both headless formats price from the same resolved `[ui.status_line.pricing]` table — with a test that fails if the surfaces ever diverge.**

## Performance

- **Started:** 2026-11-21T18:04+01:00 (plan read)
- **Completed:** 2026-11-21T19:03+01:00
- **Tasks:** 3/3
- **Files modified:** 13 (11 source/test, 2 user-guide pages)
- **Uncommitted by design** — team rule: the reviewer gates the diff, the orchestrator commits.

## What changed, per task

### Task 1 — price the hidden Flash alias, drive it through the headless JSON result (COST-01 + COST-03 first half)

`crates/codegen/xai-grok-status-line/src/pricing.rs`
- Added `pub const FLASH_ALIAS_PRICING_MODEL: &str = "deepseek-v4-flash";` beside `FLASH_PRICING_MODEL`, with a doc
  comment fixing the contract: identical rates, a **separate** entry, `with_overrides` replaces outright, and
  `for_model` does not alias the two ids — so an override for `deepseek-flash` does not apply to the alias.
- `PricingTable::deepseek_defaults()` inserts it with `ModelPricing::flash()`. Alias rates: **0.006 / 0.30 / 1.20**
  (cache-hit / cache-miss / output, USD per 1M).
- `default_table_prices_both_labelled_models_at_peak` now also asserts the alias's three rates;
  `flash_alias_worked_example_is_exact` prices `usage(0, 500, 1_000, 2_000)` at **0.001206 USD** (±1e-12), where the
  Pro entry it used to inherit charges 0.004664.

`crates/codegen/xai-grok-pager/src/headless.rs`
- `HeadlessEmitter` gains `pricing: xai_grok_status_line::PricingTable`. `new(format, schema)` is unchanged in
  signature and delegates to `new_with_pricing(..., PricingTable::default())`, so the ~13 existing call sites did
  not move.
- `attach_cost_estimate` prices from `self.pricing`.
- New private `status_line_pricing(&toml::Value)`: `get("ui")` → `try_into::<UiConfig>()` → `.status_line.pricing()`,
  falling back to `UiConfig::default()` on a missing/malformed `[ui]` — the same three steps
  `load_initial_ui_config` performs, without the second file read.
- `run_single_turn` builds the emitter *below* the `load_effective_config()` call and passes that resolved table
  (nothing between the old and new position used the emitter).

`crates/codegen/xai-grok-pager/src/headless_tests.rs`
- `json_result_honours_status_line_pricing_override`: same usage as the shipped-table test, Flash output doubled to
  2.40 → emitted `estimated_cost_usd` = 0.001524 (exact), `estimate: true`. The two pre-existing estimate tests pass
  unchanged.

### Task 2 — render the cost segment as the fixed-width estimate (COST-02)

`crates/codegen/xai-grok-pager/src/views/status_line/segments.rs`
- `MIN_DISPLAYED_COST_USD` is now `pub(crate) const … = 0.0001`; the `Cost` arm calls a new pure helper
  `fn format_cost(usd: f64) -> Option<String>` returning the whole segment text:
  `None` when not finite or below the floor; otherwise `(usd * 10_000.0).round() < 100.0` selects
  `format!("~${usd:.4} est.")`, else `format!("~${usd:.2} est.")`. Tone stays `StatusSegment::dim`, one segment, no
  `fit_columns`, ASCII only (`~`, `$`, `.`, space, letters, digits).

Rendered strings verified against every row of the UI-SPEC table:

| Input | Rendered |
|---|---|
| `0.00004`, `0.0`, `None`, `NaN`, `±inf`, `-1.0` | *(omitted)* |
| `0.0001` | `~$0.0001 est.` |
| `0.0042` | `~$0.0042 est.` |
| `0.0099` | `~$0.0099 est.` |
| `0.00996` | `~$0.01 est.` (promotion) |
| `0.01` | `~$0.01 est.` |
| `0.3745` | `~$0.37 est.` |
| `1.006` | `~$1.01 est.` |
| `12.34` | `~$12.34 est.` |
| `1234.5` | `~$1234.50 est.` (no thousands separator) |

- `views/status_line/mod.rs` re-exports the floor as `pub(crate) use segments::MIN_DISPLAYED_COST_USD;`.
- `segments_tests.rs`: full row is now
  `project │ deepseek-build │ 42% ctx │ ~$0.37 est. │ 1m23s │ status_line work`; the old boundary test is renamed
  (`omits_segments_whose_data_is_missing_or_below_the_floor`) and now pins `0.004 → ~$0.0040 est.`,
  `0.006 → ~$0.0060 est.`, `0.00004 → omitted`; new `cost_renders_the_estimate_at_fixed_width_per_branch`,
  `cost_below_the_floor_or_not_finite_is_omitted`, `cost_segment_stays_dim_at_every_value` (asserts exactly one
  segment and `SegmentTone::Dim` at 0.0042/0.01/0.3745/12.34).
- `app/status_line_tests.rs`: `the_numbers_the_guide_states_are_pinned` asserts the floor is `0.0001`.
- `25-status-line.md`: the `cost` row, the "Whatever the source…" paragraph, the two-id rate row and the alias note
  are the UI-SPEC copy verbatim and nothing else. `StatusLineItem::varies_mid_turn` untouched (`Cost` still false).

### Task 3 — the streaming reducer honours the same table, and the surfaces are proven equal (COST-03 second half)

- `headless/reducer/messages/mod.rs`: `MessagesReducer` gains `pricing`; `new()` delegates to
  `new_with_pricing(PricingTable::default())`, so the reducer tests built via the `messages()` helper are untouched.
- `headless/reducer/messages/usage.rs`: `estimated_cost_usd` reads `self.pricing`.
- `headless/reducer/mod.rs`: new `reducer_for_with_pricing(format, pricing)`; `reducer_for(format)` delegates with
  the default table. `AcpReducer` is unchanged (the `streaming-json` format carries no cost field).
- `HeadlessEmitter::new_with_pricing` passes its table into the factory for `StreamingMessagesJson`.
- `result_usage.rs`: `messages_result_honours_status_line_pricing_override` — model `deepseek-flash`, all three rates
  doubled, result line prices 0.00007092.
- `headless_tests.rs`: `tui_and_headless_report_the_same_cost_for_one_override` builds one `StatusLineConfig`
  (deserialized from JSON, so its `BTreeMap` override is the real config path), derives the expected figure from
  `.pricing().cost_usd(...)` — exactly what `fill_estimated_cost` reads — then asserts exact equality
  (`assert_eq!` on `f64`) for the `json` emitter **and** the `streaming-messages-json` reducer. All three paths run
  the same arithmetic on the same projected buckets (85 fresh + 5 creation, 10 hit, 7 output), so the comparison is
  exact rather than tolerance-based: **0.00007092 on all three**.
- `14-headless-mode.md`: one factual sentence — the estimate honours `[ui.status_line.pricing]`, the same table the
  status line uses. No `~`/`est.` marker, no formatted-wire-value claim.

## Verification

All cargo commands under `flock -w 900 /tmp/team-build.lock`. `scripts/disk-guard.sh` ran first:
`1009 GB free on the target filesystem (minimum 60 GB)` → exit 0.

| # | Command | Result |
|---|---------|--------|
| 1 | `cargo test -p xai-grok-status-line --lib pricing` | **8 passed**, 0 failed |
| 2 | `cargo test -p xai-grok-pager --lib headless` | **192 passed**, 0 failed |
| 3 | `cargo test -p xai-grok-pager --lib status_line` | **103 passed**, 0 failed |
| 4 | `cargo test -p xai-grok-status-line --lib` | **25 passed**, 0 failed |
| 5 | `cargo test -p xai-grok-pager --lib` | **9885 passed**, 0 failed, 4 ignored |
| 6 | `cargo fmt --all --check` | exit 0 (clean; `cargo fmt -p xai-grok-pager -p xai-grok-status-line` applied first) |
| 7 | `cargo clippy -p xai-grok-status-line -p xai-grok-pager --all-targets` | exit 0, **0 errors**; warnings all pre-existing, none in a touched file |
| 8 | `cargo test -p xai-grok-pager --lib -- honours_status_line_pricing_override tui_and_headless_report the_same_cost` | **3 passed** (the three new headless tests) |
| 9 | `git diff --stat` reviewed | 14 entries incl. pre-existing `.planning/STATE.md`; **no `Cargo.toml`, `Cargo.lock`, `deny.toml` or `.cargo/` change** |

Guide-guard tests confirmed green by name: `every_builtin_item_is_named_in_the_guide`,
`every_field_on_the_wire_is_named_in_the_guide`, `every_config_key_the_section_serializes_is_named_in_the_guide`,
`the_numbers_the_guide_states_are_pinned`.

NOT RUN: no live TUI session and no live model call — the plan's verification is unit-level only, and the headless
cost path is asserted through `build_json_result`/`reducer.finish` rather than a spawned agent. No workspace-wide
`cargo test` (that is plan 01-05's test gate).

## Files Created/Modified

Modified (all inside the plan's `files_modified`):

- `crates/codegen/xai-grok-status-line/src/pricing.rs` — alias const + table entry + two tests
- `crates/codegen/xai-grok-pager/src/views/status_line/segments.rs` — floor, `format_cost`, Cost arm
- `crates/codegen/xai-grok-pager/src/views/status_line/segments_tests.rs` — strings and new cases
- `crates/codegen/xai-grok-pager/src/views/status_line/mod.rs` — `pub(crate)` floor re-export
- `crates/codegen/xai-grok-pager/src/app/status_line_tests.rs` — floor pinned against the guide
- `crates/codegen/xai-grok-pager/src/headless.rs` — `pricing` field, `new_with_pricing`, `status_line_pricing`, emitter move
- `crates/codegen/xai-grok-pager/src/headless_tests.rs` — override + cross-surface tests
- `crates/codegen/xai-grok-pager/src/headless/reducer/mod.rs` — `reducer_for_with_pricing`
- `crates/codegen/xai-grok-pager/src/headless/reducer/messages/mod.rs` — `pricing` field, `new_with_pricing`
- `crates/codegen/xai-grok-pager/src/headless/reducer/messages/usage.rs` — `self.pricing`
- `crates/codegen/xai-grok-pager/src/headless/reducer/messages/tests/result_usage.rs` — override test
- `crates/codegen/xai-grok-pager/docs/user-guide/25-status-line.md` — UI-SPEC copy
- `crates/codegen/xai-grok-pager/docs/user-guide/14-headless-mode.md` — one sentence

Not touched by this plan (pre-existing in the working tree at the baseline):
`.planning/STATE.md` (modified) and `.planning/state.json` (untracked).

## Decisions Made

- Separate alias entry, no id aliasing in `for_model` — a user override for one id must not silently move the other.
  The doc comment on `FLASH_ALIAS_PRICING_MODEL` is the contract.
- Headless resolves the table from `raw_config`; it deliberately does **not** call `load_initial_ui_config()`
  (a `OnceLock` read of the config file) because `run_single_turn` already owns the loaded copy.
- Promotion uses the rounded 4-dp value, not a 0.01 comparison, which is what renders `0.00996` as `~$0.01 est.`.
- `new()` constructors stay as default-table delegates so the existing test call sites (and any other caller) keep
  compiling unchanged; the injection is purely additive.
- The cross-surface test compares `f64`s with `assert_eq!`: all three paths call the same `PricingTable::cost_usd`
  on the same projected buckets, so a tolerance would hide the very drift the test exists to catch.

## Deviations from Plan

Two mechanical slips, both corrected before verification and with no residual change:

1. A `HeadlessEmitter::new` line was duplicated while moving the construction below the `raw_config` load; the
   duplicates were removed and the final `run_single_turn` constructs the emitter exactly once, after the config
   load, as the plan specifies.
2. `reducer_for` was dropped from the `headless.rs` import list when `reducer_for_with_pricing` was added, which
   broke two pre-existing tests that call it; both names are now imported.

No architectural change, no new dependency, no new outbound destination, no scope widening: every edited path is in
the plan's `files_modified`.

## Issues Encountered

- `FLASH_PRICING_MODEL` is not re-exported at the `xai-grok-status-line` crate root (only `DEFAULT_PRICING_MODEL`
  is). Rather than widening scope into `lib.rs`, the new tests reference it as
  `xai_grok_status_line::pricing::FLASH_PRICING_MODEL` (the module is public).
- The cross-surface test needed a `StatusLineConfig` with a pricing override, and `StatusLineConfig`'s `pricing`
  field is private with no `with_pricing` fixture. It is deserialized from JSON instead, which exercises the real
  config path and keeps `config_test_support.rs` untouched.

## Runtime risks

- `run_single_turn` now deserializes `[ui]` into `UiConfig` once more per process (in addition to the config load
  itself). It is a `toml::Value` → struct conversion of an already-parsed tree, not a file read, and it cannot fail
  the run; the cost is negligible next to process startup, but it is new work on the headless path.
- The cost segment is 5–8 columns larger than before (11–13 for `[$0.0001, $999.99]`). On a narrow terminal the
  row's existing right-edge `…` elision absorbs it, and `StatusLineDisplay::line_count` stays 1 — but a user with
  several other items will see more elision than before.
- `~$1234.50 est.` is 14 columns and grows one column per further integer digit, by design (no magnitude suffix).
- Both headless surfaces now depend on `[ui.status_line.pricing]` being parseable; a malformed table falls back to
  `UiConfig::default()` (the shipped peak rates), so the failure mode is a wrong-but-plausible number rather than
  an error — the same trade the TUI already makes.

---
*Phase: 01-truth-baseline*
*Completed: 2026-11-21*

## Commit

The frontmatter records `commits: 0` because the dispatch forbids committing;
the reviewed change landed afterwards as `e834f647`. This note is the post-phase
record, not a change to the plan's reported state.

