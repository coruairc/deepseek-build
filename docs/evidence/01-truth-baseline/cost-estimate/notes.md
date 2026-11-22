# cost-estimate

Proves COST-01, COST-02 and COST-03: the Flash alias prices at Flash rates, the
TUI cost segment is labelled an estimate at a fixed width per branch, and the
headless paths report the same number as the TUI for one pricing override.

## Status: DEFERRED (tests skipped by owner, 2026-11-22)

The owner directed on 2026-11-22 that all test and gate runs stop for the
remaining milestone plans. This item is **`result: skipped`** with no steps and
no cast: no test run was performed, so nothing here certifies a rendered
string.

The code under test is committed:

- `crates/codegen/xai-grok-status-line/src/pricing.rs` prices
  `deepseek-v4-flash` at Flash rates (COST-01).
- `crates/codegen/xai-grok-pager/src/views/status_line/segments.rs`
  `format_cost` emits `~$<n> est.` — four decimals below one cent, two at or
  above, floor `$0.0001` (COST-02).
- `crates/codegen/xai-grok-pager/src/headless.rs` resolves
  `[ui.status_line.pricing]` instead of `PricingTable::default()` (COST-03).

The suite that would pin these is
`crates/codegen/xai-grok-pager/src/views/status_line/segments_tests.rs`
(`cost_renders_the_estimate_at_fixed_width_per_branch` — the rows
`~$0.0001 est.`, `~$0.0042 est.`, `~$0.01 est.`, `~$0.37 est.`) and
`crates/codegen/xai-grok-pager/src/headless_tests.rs`
(`tui_and_headless_report_the_same_cost_for_one_override`,
`json_result_honours_status_line_pricing_override`). There is no pty session
that renders the status line today, so a future recording shows the suite, not
a TUI screenshot.

## Re-run (when testing resumes)

```sh
ITEM=docs/evidence/01-truth-baseline/cost-estimate
script -qec 'cargo test -p xai-grok-pager --lib status_line && \
             cargo test -p xai-grok-pager --lib headless' \
    -T "$ITEM/run.timing" "$ITEM/run.typescript"
scripts/evidence.py cast --typescript "$ITEM/run.typescript" \
    --timing "$ITEM/run.timing" --out "$ITEM/cost-estimate.cast"
scripts/key-leak-check.sh --scan docs/evidence
```

Then record one step per rendered string and one per named test, as the
`ipv6-report` item does.
