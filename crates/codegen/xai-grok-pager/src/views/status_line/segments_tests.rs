use super::*;

const DIR: &str = "/home/user/project";

fn context() -> StatusLineContext {
    let mut ctx = crate::app::status_line::test_context(DIR);
    ctx.session_name = Some("status_line work".into());
    ctx.model.display_name = Some("deepseek-build".into());
    ctx.cost.total_cost_usd = Some(0.3745);
    ctx.context_window.used_percentage = Some(42);
    ctx.context_window.auto_compact_threshold_percent = Some(80);
    ctx
}

fn plain(ctx: &StatusLineContext, turn_elapsed: Option<Duration>) -> String {
    compose_builtin(ctx, turn_elapsed, StatusLineItem::ALL)
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join(SEGMENT_SEPARATOR)
}

#[test]
fn composes_every_segment_in_order() {
    assert_eq!(
        plain(&context(), Some(Duration::from_secs(83))),
        "project │ deepseek-build │ 42% ctx │ ~$0.37 est. │ 1m23s │ status_line work"
    );
}

#[test]
fn omits_segments_whose_data_is_missing_or_below_the_floor() {
    let mut ctx = context();
    ctx.cost.total_cost_usd = Some(0.004);
    assert_eq!(
        plain(&ctx, Some(Duration::from_millis(400))),
        "project │ deepseek-build │ 42% ctx │ ~$0.0040 est. │ status_line work"
    );

    ctx.cost.total_cost_usd = Some(0.006);
    assert!(plain(&ctx, None).contains("~$0.0060 est."));

    ctx.cost.total_cost_usd = Some(0.00004);
    assert!(
        !plain(&ctx, None).contains("$"),
        "a figure below the floor is omitted rather than shown as a zero"
    );

    ctx.context_window.used_percentage = None;
    assert!(compose_builtin(&ctx, None, &[StatusLineItem::Context]).is_empty());
}

#[test]
fn name_past_its_budget_is_cut_by_painted_columns() {
    let mut ctx = context();
    ctx.session_name = Some("辺".repeat(SESSION_NAME_COLS));
    let composed = compose_builtin(&ctx, None, &[StatusLineItem::SessionName]);
    let Some(cut) = composed.first().map(|s| s.text.as_str()) else {
        panic!("expected a session-name segment: {composed:?}");
    };

    let width = super::super::painted_width(cut);
    assert!(
        cut.ends_with('…'),
        "a cut name has to say it was cut: {cut}"
    );
    assert!(
        width <= SESSION_NAME_COLS,
        "{width} columns overruns the {SESSION_NAME_COLS} the segment was given"
    );
    // The cut fills the budget to within one cluster; a byte or character cut would leave more unused
    assert!(
        width + 2 > SESSION_NAME_COLS,
        "{width} columns leaves more than a cluster of the budget unused"
    );
}

#[test]
fn cost_the_session_does_not_have_omits_its_segment() {
    let mut ctx = context();
    ctx.cost.total_cost_usd = None;
    assert!(compose_builtin(&ctx, None, &[StatusLineItem::Cost]).is_empty());
}

/// Every row of the approved rendered-states table, in the string the row paints.
#[test]
fn cost_renders_the_estimate_at_fixed_width_per_branch() {
    for (usd, rendered) in [
        (0.0001, "~$0.0001 est."),
        (0.0042, "~$0.0042 est."),
        (0.0099, "~$0.0099 est."),
        // 0.00996 rounds to a full cent at four decimals, so it promotes to the two-decimal form.
        (0.00996, "~$0.01 est."),
        (0.01, "~$0.01 est."),
        (0.3745, "~$0.37 est."),
        (1.006, "~$1.01 est."),
        (12.34, "~$12.34 est."),
        // No thousands separator and no magnitude suffix.
        (1234.5, "~$1234.50 est."),
    ] {
        assert_eq!(
            format_cost(usd).as_deref(),
            Some(rendered),
            "{usd} should render {rendered}"
        );
    }
}

#[test]
fn cost_below_the_floor_or_not_finite_is_omitted() {
    for usd in [
        0.0,
        0.00004,
        -1.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        assert_eq!(format_cost(usd), None, "{usd} should render nothing");
    }
    assert_eq!(
        format_cost(MIN_DISPLAYED_COST_USD),
        Some("~$0.0001 est.".into())
    );
}

/// The estimate marker carries the imprecision; the cost segment never gains a warning tint.
#[test]
fn cost_segment_stays_dim_at_every_value() {
    for usd in [0.0042, 0.01, 0.3745, 12.34] {
        let mut ctx = context();
        ctx.cost.total_cost_usd = Some(usd);
        let segments = compose_builtin(&ctx, None, &[StatusLineItem::Cost]);
        assert_eq!(segments.len(), 1, "one segment, never two: {segments:?}");
        let Some(segment) = segments.first() else {
            panic!("expected a cost segment for {usd}");
        };
        assert_eq!(segment.tone, SegmentTone::Dim, "{usd}");
    }
}

#[test]
fn context_segment_warns_near_compaction() {
    let mut ctx = context();
    let tone = |ctx: &StatusLineContext| {
        let segs = compose_builtin(ctx, None, &[StatusLineItem::Context]);
        let Some(seg) = segs.first() else {
            panic!("expected a context segment: {segs:?}");
        };
        seg.tone
    };

    ctx.context_window.used_percentage = Some(90);
    assert_eq!(tone(&ctx), SegmentTone::Warn);
    ctx.context_window.used_percentage = Some(50);
    assert_eq!(tone(&ctx), SegmentTone::Dim);

    ctx.context_window.auto_compact_threshold_percent = Some(65);
    ctx.context_window.used_percentage = Some(70);
    assert_eq!(tone(&ctx), SegmentTone::Warn);
}

fn turn_timer(secs: u64) -> String {
    compose_builtin(
        &context(),
        Some(Duration::from_secs(secs)),
        &[StatusLineItem::TurnTimer],
    )
    .first()
    .expect("TurnTimer segment")
    .text
    .clone()
}

#[test]
fn turn_timer_values() {
    assert_eq!(turn_timer(59), "59s");
    assert_eq!(turn_timer(194 * 60 + 4), "3h14m");
}
