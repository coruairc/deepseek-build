# cost-estimate

Proves COST-02 and COST-03: the status-line cost figure is labelled an
estimate at a fixed width per branch, and the headless surfaces honour the
same `[ui.status_line.pricing]` table as the TUI.

## What the cast shows

`cost-estimate.cast` is a `script -T` capture of two filtered runs of the
pager lib suite, in one session:

```sh
cargo test -p xai-grok-pager --lib status_line
cargo test -p xai-grok-pager --lib headless
```

Both run with the three API-key variables unset and color off
(`NO_COLOR=1 CARGO_TERM_COLOR=never --color never`), under the shared build
lock. The `status_line` filter runs `views::status_line::segments_tests`,
which pins the exact rendered strings; the `headless` filter runs
`headless_tests` and the streaming reducer tests, which pin the estimate
arithmetic and the pricing-override path.

**No pty session renders the status line today.** There is no pty e2e test
that drives the cost segment (RESEARCH 4b), and this item does not invent a
TUI session: the cast shows the suite that pins the strings, and
`report.json` carries the strings themselves in the step details.

The strings pinned by `cost_renders_the_estimate_at_fixed_width_per_branch`
(`crates/codegen/xai-grok-pager/src/views/status_line/segments_tests.rs`):

| USD | Rendered |
|---|---|
| 0.3745 | `~$0.37 est.` |
| 0.0042 | `~$0.0042 est.` |
| 0.01 | `~$0.01 est.` |
| 0.0001 | `~$0.0001 est.` |

Four decimals below $0.01, two at/above, with the promotion rule
(0.00996 → `~$0.01 est.`) and the $0.0001 floor pinned by the same test and
`cost_below_the_floor_or_not_finite_is_omitted`.

The cross-surface equality test is
`headless_tests::tui_and_headless_report_the_same_cost_for_one_override`:
one usage, one `[ui.status_line.pricing]` override, one float — the TUI's
`StatusLineConfig::pricing()` derivation, the `json` emitter and the
`streaming-messages-json` reducer must agree exactly. The override path is
also pinned per surface by
`json_result_honours_status_line_pricing_override` and
`messages_result_honours_status_line_pricing_override`.

## Steps

The first step runs the capture itself (`script -qec … -T`), so its `ok`
is the real exit status of both cargo runs. The remaining steps `grep` the
captured typescript for the test names (proving the suites ran them) and
the test source for the exact rendered strings (proving the suites pin
them). A regression that drops a test or changes a string fails the step
and the item.

## Re-run

```sh
flock -w 900 /tmp/team-build.lock env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY \
    cargo test -p xai-grok-pager --lib status_line
flock -w 900 /tmp/team-build.lock env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY \
    cargo test -p xai-grok-pager --lib headless
```

Expect both to exit 0 with `test result: ok`. To regenerate the cast:

```sh
ITEM=docs/evidence/01-truth-baseline/cost-estimate
script -qec 'flock -w 900 /tmp/team-build.lock env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY NO_COLOR=1 CARGO_TERM_COLOR=never cargo test -p xai-grok-pager --lib --color never status_line -- --color never && flock -w 900 /tmp/team-build.lock env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY NO_COLOR=1 CARGO_TERM_COLOR=never cargo test -p xai-grok-pager --lib --color never headless -- --color never' \
    -T "$ITEM/run.timing" "$ITEM/run.typescript"
scripts/evidence.py cast --typescript "$ITEM/run.typescript" \
    --timing "$ITEM/run.timing" --out "$ITEM/cost-estimate.cast"
```

The raw typescript/timing intermediates are not committed; the cast and the
step logs are, and both are normalized (`<REPO>/<HOME>/<TMP>`).