---
phase: 01-truth-baseline
plan: 03
reviewed: 2026-11-21T18:18:41Z
depth: deep
files_reviewed: 13
files_reviewed_list:
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
findings:
  critical: 0
  warning: 3
  info: 3
  total: 6
status: issues_found
---

# Phase 01 Plan 03: Code Review Report (Cost honesty)

**Reviewed:** 2026-11-21T18:18:41Z
**Base:** `711524dd` (all change under review is uncommitted in the working tree)
**Depth:** deep
**Files Reviewed:** 13
**Status:** issues_found (no blockers)

## Summary

I inspected the whole diff, traced the three pricing paths across crate boundaries
(`pricing.rs` → `StatusLineConfig::pricing()` → `AppView::fill_estimated_cost`; `PricingTable` →
`HeadlessEmitter` → both reducer factories), checked the formatter against every row of the approved
`01-UI-SPEC.md` rendered-states table, and re-ran the verification myself.

**The three requirements hold in code.** `deepseek-v4-flash` is an explicit Flash-rate entry, not a
Pro fallback (COST-01). `format_cost` produces the spec's exact strings including the `0.00996 →
~$0.01 est.` promotion and the `$0.0001` floor, stays `Dim`, one segment, and `Cost` is still absent
from `varies_mid_turn` (COST-02). Both headless formats price from the resolved
`[ui.status_line.pricing]` table, the emitter is constructed after the config load it depends on, and
the TUI resolver recomputes per rebuild rather than caching (COST-03). The guide copy is verbatim the
UI-SPEC replacement text, and no other page states the old `$0.005` behaviour. No new dependency, no
new egress, no secret handling, no `unwrap` on a production path, no `indexing_slicing`.

What I found is not incorrect behaviour: the config → table resolver that COST-03 actually turns on has
no test (the three "honours pricing" tests inject a pre-built table and bypass it), two in-code doc
comments still describe the old unconditional-default pricing, and the plan SUMMARY reports a
cross-surface figure that its own test disproves. All three are WARNINGs; none blocks the merge.

## Warnings

### WR-01: `status_line_pricing` — the COST-03 config read — has no test at all

**File:** `crates/codegen/xai-grok-pager/src/headless.rs:903-912` (used at `:938`)
**Issue:** `status_line_pricing(&toml::Value)` is the only code that turns `[ui.status_line.pricing]`
into the table headless prices from. Nothing exercises it. All three pricing tests
(`json_result_honours_status_line_pricing_override`, `messages_result_honours_status_line_pricing_override`,
`tui_and_headless_report_the_same_cost_for_one_override`) build the table by hand and inject it through
`new_with_pricing` / `reducer_for_with_pricing`, so they prove the *injection seam* works, not that
headless reads the config. A regression in the resolver — wrong section key, wrong `UiConfig` type, a
fallback that swallows a valid table — would leave the whole 192-test headless suite green while
COST-03 silently reverted to `PricingTable::default()`. The plan's Task 1 `<done>` ("which
`run_single_turn` builds from the loaded `[ui]`") is therefore asserted by reading, not by a check.
**Fix:** add one unit test beside the emitter test (`PricingTable` already derives `PartialEq`):
```rust
#[test]
fn status_line_pricing_reads_the_ui_section_and_falls_back() {
    let cfg: toml::Value = toml::from_str(
        r#"
[ui.status_line]
type = "builtin"
items = ["cost"]
[ui.status_line.pricing.deepseek-flash]
cache_hit = 0.012
cache_miss = 0.60
output = 2.40
"#,
    )
    .expect("toml");
    let table = status_line_pricing(&cfg);
    assert_eq!(table.for_model("deepseek-flash").map(|p| p.output), Some(2.40));

    // No [ui] at all, and a malformed [ui], both price from the shipped defaults.
    assert_eq!(status_line_pricing(&toml::Value::Table(Default::default())),
               xai_grok_status_line::PricingTable::default());
    let broken: toml::Value = toml::from_str("[ui]\nstatus_line = 7\n").expect("toml");
    assert_eq!(status_line_pricing(&broken),
               xai_grok_status_line::PricingTable::default());
}
```

### WR-02: the SUMMARY's cross-surface dollar figure is wrong, and contradicts its own test

**File:** `.planning/phases/01-truth-baseline/01-03-SUMMARY.md:202,208`
**Issue:** the report states the result line "prices 0.00006792" and that the cross-surface test proves
"**0.00006792 on all three**". The test it describes asserts
`(90.0 * 0.60 + 10.0 * 0.012 + 7.0 * 2.40) / 1_000_000.0` = **0.00007092**, and the code produces that:
the projected buckets are 85 fresh input + 5 cache-creation at the miss rate, 10 cache hits, 7 output.
`0.00006792` is exactly what you get when the 5 cache-creation tokens are dropped (85 × 0.60 = 51.0),
which contradicts the report's own sentence two lines earlier ("85 fresh + 5 creation, 10 hit, 7
output"). I re-ran the suite: all 192 headless tests pass, including
`tui_and_headless_report_the_same_cost_for_one_override`, so the implementation is 0.00007092 and the
artifact carries a fabricated/stale number. This matters because the number is offered as the evidence
that COST-03's equality holds.
**Fix:** change both mentions to `0.00007092`. If `0.00006792` was actually measured at some point, the
code moved after the measurement and the report needs that note instead.

### WR-03: doc comments still say the estimate comes from "the shipped DeepSeek price table"

**File:** `crates/codegen/xai-grok-pager/src/headless/reducer/messages/usage.rs:21`
(and `.../messages/wire.rs:237`)
**Issue:** the `ResultUsage::estimated_cost_usd` field doc says "Estimated cost from the shipped DeepSeek
price table, when any usage was recorded", and the wire-field doc says "Locally estimated USD from the
shipped DeepSeek price table (peak rates)". After this change the figure comes from the resolved
`[ui.status_line.pricing]` table, of which the shipped table is only the default — an override moves it.
`usage.rs` now contradicts itself: the method doc twelve lines below was correctly updated to "the
reducer's resolved `[ui.status_line.pricing]` table". `wire.rs` is the doc for the exact field COST-03
exists to make configurable.
**Fix:** in both places: "Locally estimated USD from the resolved `[ui.status_line.pricing]` table (the
shipped DeepSeek peak rates unless overridden), independent of `total_cost_usd`."

## Info

### IN-01: the floor is pinned to a literal, not to the guide it names

**File:** `crates/codegen/xai-grok-pager/src/app/status_line_tests.rs:441-445`
**Issue:** the new assertion is `assert_eq!(MIN_DISPLAYED_COST_USD, 0.0001, "the guide says …")`, but it
never reads `25-status-line.md` — it pins the constant to a hard-coded literal, exactly as the
pre-existing numbers in that test do. It catches changing the constant without touching the test; it
does not catch editing the guide to `$0.001`, which is the drift the UI-SPEC seam was specified to
prevent. The file already `include_str!`s the guide in the two neighbouring tests.
**Fix:** read the page and assert the figure:
```rust
let guide = include_str!("../../docs/user-guide/25-status-line.md");
assert!(
    guide.contains(&format!("${}", crate::views::status_line::MIN_DISPLAYED_COST_USD)),
    "the guide must state the cost floor"
);
```

### IN-02: headless and TUI can key the estimate on different model strings

**File:** `crates/codegen/xai-grok-pager/src/headless.rs:1205-1208` vs
`crates/codegen/xai-grok-pager/src/app/status_line_policy.rs:283-289`;
claim added at `docs/user-guide/14-headless-mode.md:201-203`
**Issue:** the headless emitter latches `options.model` (the raw `--model` value) when one is given and
only falls back to `session_models.current_model_id_str()`; the TUI prices from the shell-resolved
`ctx.model.id`. Both are exact-match lookups with a Pro fallback, so when the two strings differ for the
same session (a `--model` value that is a name or alias rather than a table key) the two surfaces can
price one turn at different rates. This predates the plan, but the new doc sentence now promises that an
override "moves the TUI figure and this field together", which holds only when the keys agree.
**Fix:** latch `session_models.current_model_id_str()` after the model override is applied, or resolve
`options.model` through the catalog first; and/or narrow the doc sentence to "both read the same table".

### IN-03: the default-table constructors are now test-only

**File:** `crates/codegen/xai-grok-pager/src/headless.rs:120-126`,
`.../headless/reducer/mod.rs:432-434`, `.../headless/reducer/messages/mod.rs:77-79`
**Issue:** after the change, `HeadlessEmitter::new`, `reducer_for` and `MessagesReducer::new` have no
non-test caller — grep finds them only in `[...]_tests.rs` modules. They were deliberately kept (plan
decision, and the crate's `#![allow(dead_code)]` hides the fact), but a future reader cannot tell that
production never takes the default-table path, which is precisely the guarantee COST-03 is about.
**Fix:** optional — gate the default-table constructors with `#[cfg(test)]`, or add "(default-table;
test-only since the resolver landed)" to each doc comment.

## Verification performed

All cargo commands under `flock -w 900 /tmp/team-build.lock`; `scripts/disk-guard.sh` exited 0
(1004 GB free).

| # | Command | Result |
|---|---------|--------|
| 1 | `cargo test -p xai-grok-status-line --lib pricing` | **8 passed**, 0 failed (matches SUMMARY) |
| 2 | `cargo test -p xai-grok-pager --lib -- status_line` | **103 passed**, 0 failed (matches) |
| 3 | `cargo test -p xai-grok-pager --lib -- headless` | **192 passed**, 0 failed (matches) |
| 4 | `cargo test -p xai-grok-status-line --lib` | **25 passed**, 0 failed (matches) |
| 5 | `cargo test -p xai-grok-pager --lib` | **9885 passed**, 0 failed, 4 ignored (matches) |
| 6 | `cargo fmt --all --check` | exit 0 (matches) |
| 7 | `cargo clippy -p xai-grok-status-line -p xai-grok-pager --all-targets` | exit 0, 0 errors; the warnings shown are in untouched files (`app/session_startup.rs:1817`) (matches the "pre-existing" claim) |
| 8 | `git diff --stat HEAD` | 14 paths; no `Cargo.toml`/`Cargo.lock`/`deny.toml`/`.cargo/` change (matches) |

Not run: no live TUI session, no live model call (same scope as the plan). I did not independently
re-run `cargo clippy` on the whole workspace or the `config-docs` feature tests — no config key changed,
so neither is load-bearing here.

Defects ruled out by inspection: NaN/±inf/negative/`None` → segment omitted; floor boundary
(`0.0001` shown, `0.00004` hidden); promotion boundary (`0.0099` four dp, `0.00996` two dp); no thousands
separator; `Dim` tone at every value; exactly one segment; `Cost` absent from `varies_mid_turn`; the
emitter reorder uses the emitter only after construction (nothing between the old and new position
referenced it); `UiConfig` resolution mirrors `load_initial_ui_config` (`[ui]` → `try_into` →
`.status_line.pricing()`, `UiConfig::default()` on a missing/malformed section, no second file read);
`StatusLineConfig::pricing()` is recomputed per rebuild, so no stale cache; the `/usage` modal and the
`cost.total_cost_usd` command-payload float are untouched; no new egress, secret, or panic path with
user data.

## Verdict

**PASS_WITH_NOTES** — the implementation meets COST-01, COST-02 and COST-03 and introduces no
correctness, security or regression defect I can prove. Ship after fixing WR-02 (the SUMMARY's number;
the artifact must not claim a figure its own test contradicts) and, preferably, WR-01 (one test on
`status_line_pricing`, the seam COST-03 actually hinges on). WR-03 and the three INFO items are
follow-ups, not merge conditions.

---

_Reviewed: 2026-11-21T18:18:41Z_
_Reviewer: gsd-code-reviewer_
_Depth: deep_
