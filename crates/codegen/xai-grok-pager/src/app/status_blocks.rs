//! Read-only system-block text for `/queue`, `/tasks`, and `/usage`.
//!
//! Plain text committed into scrollback; minimal mode has no interactive panes, so these blocks are its main way to inspect that state.
//! The formatting lives outside `dispatch` so it is easy to unit test.
use crate::app::agent::{BgTaskState, BgTaskStatus};
use crate::app::agent_view::AgentView;
use crate::app::subagent::format_subagent_label;
use crate::util::{format_duration, group_thousands};
use xai_grok_shell::extensions::notification::{PromptUsage, PromptUsageModel, ticks_to_usd};
use xai_grok_status_line::{PricingTable, StatusLineSessionUsage};
impl BgTaskState {
    pub(crate) fn display_kind(&self) -> &'static str {
        if self.is_monitor { "Monitor" } else { "Task" }
    }
}
/// `/queue` body: a read-only list of the queued prompts.
/// Rows from the server's shared queue (minus the prompt already running) come first in broadcast order, then the local queue (`pending_prompts`).
/// This matches [`crate::views::queue_pane::QueuePane::sync_from_merged`]'s ordering.
pub(crate) fn queue_block_text(agent: &AgentView) -> String {
    let running_id = agent.session.current_prompt_id.as_deref();
    let mut rows: Vec<String> = Vec::new();
    let mut pos = 1usize;
    for wire in &agent.shared_queue {
        if running_id == Some(wire.id.as_str()) {
            continue;
        }
        rows.push(format_queue_row(pos, &wire.text));
        pos += 1;
    }
    for prompt in &agent.session.pending_prompts {
        rows.push(format_queue_row(pos, &prompt.text));
        pos += 1;
    }
    if rows.is_empty() {
        "Queue is empty.".to_string()
    } else {
        let header = format!(
            "Queued prompt{} ({}):",
            if rows.len() == 1 { "" } else { "s" },
            rows.len()
        );
        join_header_rows(header, rows)
    }
}
/// `/tasks` body: [`crate::views::tasks_pane::TasksPane`] without its styled rows.
pub(crate) fn tasks_block_text(agent: &AgentView) -> String {
    let mut rows: Vec<String> = Vec::new();
    let mut workflows: Vec<_> = agent.workflow_runs.iter().collect();
    workflows.sort_by(|a, b| {
        b.is_active()
            .cmp(&a.is_active())
            .then(b.received_at.cmp(&a.received_at))
            .then(a.run_id.cmp(&b.run_id))
    });
    for run in workflows {
        let active = run.active_agent_count();
        let agents = match active {
            0 => String::new(),
            1 => " · 1 agent".to_string(),
            n => format!(" · {n} agents"),
        };
        let phase = run
            .current_phase
            .as_deref()
            .map(str::trim)
            .filter(|phase| !phase.is_empty())
            .map(|phase| format!(" · {phase}"))
            .unwrap_or_default();
        rows.push(format!(
            "  {:<9}Workflow · {}{phase}{agents}  ({})",
            if run.is_active() {
                "running".to_string()
            } else {
                run.status.replace('_', " ")
            },
            run.name,
            format_duration(std::time::Duration::from_millis(run.live_elapsed_ms()))
        ));
    }
    let mut subs: Vec<_> = agent
        .subagent_sessions
        .values()
        .filter(|s| s.attempt.workflow_run_id.is_none())
        .collect();
    subs.sort_by(|a, b| {
        b.is_running()
            .cmp(&a.is_running())
            .then(b.attempt.started_at.cmp(&a.attempt.started_at))
            .then(a.child_session_id.cmp(&b.child_session_id))
    });
    for info in subs {
        let (type_label, desc) = format_subagent_label(info);
        let status = if info.attempt.pending_kill {
            "stopping"
        } else if info.is_running() {
            "running"
        } else {
            info.attempt.status.as_deref().unwrap_or("done")
        };
        let label = if desc.is_empty() {
            type_label
        } else {
            format!("{type_label} · {desc}")
        };
        rows.push(format!(
            "  {status:<9}{label}  ({})",
            format_duration(info.display_elapsed())
        ));
    }
    let mut tasks: Vec<_> = agent.session.bg_tasks.values().collect();
    tasks.sort_by(|a, b| {
        let (ar, br) = (
            a.status == BgTaskStatus::Running,
            b.status == BgTaskStatus::Running,
        );
        br.cmp(&ar)
            .then(b.start_time.cmp(&a.start_time))
            .then(a.task_id.cmp(&b.task_id))
    });
    for task in tasks {
        let kind = task.display_kind();
        let one_line = task
            .description
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| first_nonempty_line(&task.command));
        let status = if task.pending_kill {
            "stopping"
        } else {
            match task.status {
                BgTaskStatus::Running => "running",
                BgTaskStatus::Done => "done",
                BgTaskStatus::Failed => "failed",
            }
        };
        rows.push(format!(
            "  {status:<9}{kind} · {one_line}  ({})",
            format_duration(task.elapsed())
        ));
    }
    let mut sched: Vec<_> = agent.session.scheduled_tasks.values().collect();
    sched.sort_by(|a, b| {
        a.tag
            .cmp(&b.tag)
            .then(a.human_schedule.cmp(&b.human_schedule))
            .then(a.task_id.cmp(&b.task_id))
    });
    for info in sched {
        rows.push(format!(
            "  {:<9}{} · {} · {}",
            "scheduled",
            info.tag,
            info.human_schedule,
            first_nonempty_line(&info.prompt)
        ));
    }
    if rows.is_empty() {
        "No background tasks, workflows, or subagents.".to_string()
    } else {
        let header = format!(
            "Task{} ({}):",
            if rows.len() == 1 { "" } else { "s" },
            rows.len()
        );
        join_header_rows(header, rows)
    }
}
/// `/usage` body: per-session token and cost totals, covering the ledger's lifetime (since session start, or since the last `/resume`).
pub(crate) fn session_usage_block_text(
    usage: &PromptUsage,
    pricing: &PricingTable,
    model_id: Option<&str>,
) -> String {
    let t = &usage.totals;
    if t.model_calls == 0 && usage.model_usage.is_empty() {
        return if usage.usage_is_incomplete {
            "Session usage: none recorded, but tracking is incomplete and may under-count."
                .to_string()
        } else {
            "Session usage: no model calls yet in this session.".to_string()
        };
    }
    let mut rows = Vec::new();
    rows.push(format!(
        "  Input tokens:   {} ({} cached)",
        group_thousands(t.input_tokens),
        group_thousands(t.cached_read_tokens),
    ));
    rows.push(format!(
        "  Output tokens:  {} ({} reasoning)",
        group_thousands(t.output_tokens),
        group_thousands(t.reasoning_tokens),
    ));
    rows.push(format!(
        "  Total tokens:   {}",
        group_thousands(t.total_tokens)
    ));
    rows.push(format!(
        "  Model calls:    {} · API time: {}",
        group_thousands(t.model_calls),
        format_duration(std::time::Duration::from_millis(t.api_duration_ms)),
    ));
    rows.push(format!(
        "  Cost:           {}",
        format_cost_cell(session_cost_cell(usage, pricing, model_id))
    ));
    if usage.model_usage.len() > 1 {
        rows.push(String::new());
        rows.push("  By model:".to_string());
        for (model, m) in &usage.model_usage {
            rows.push(format_by_model_row(model, m, pricing));
        }
    }
    if usage.usage_is_incomplete {
        rows.push("  Note: usage is incomplete and may under-count.".to_string());
    }
    join_header_rows(
        "Session usage (since start or last resume):".to_string(),
        rows,
    )
}
/// One model's cost cell: provider-reported ticks, a local estimate, or nothing priceable.
#[derive(Debug, Clone, Copy)]
enum CostCell {
    Reported(f64),
    Estimated(f64),
    Unavailable { partial: bool },
}

/// Formats one `By model:` row. The cost cell is omitted when the model has neither a reported
/// cost nor enough usage to price.
fn format_by_model_row(model: &str, m: &PromptUsageModel, pricing: &PricingTable) -> String {
    let tokens = group_thousands(m.total_tokens);
    match model_cost_cell(model, m, pricing) {
        CostCell::Unavailable { .. } => format!("    {model}: {tokens} Tokens"),
        cell => format!("    {model}: {tokens} Tokens · {}", format_cost_cell(cell)),
    }
}

/// A provider-reported cost always wins; otherwise price the model locally with the same table the
/// status line uses, so `/cost` and the row cannot disagree for the same turn. `None` from
/// [`PricingTable::cost_usd`] means no tokens were recorded, not a free turn.
fn model_cost_cell(model_id: &str, m: &PromptUsageModel, pricing: &PricingTable) -> CostCell {
    if let Some(ticks) = m.cost_usd_ticks {
        return CostCell::Reported(ticks_to_usd(ticks));
    }
    match pricing.cost_usd(model_id, &status_line_usage(m)) {
        Some(estimate) => CostCell::Estimated(estimate),
        None => CostCell::Unavailable {
            partial: m.cost_is_partial,
        },
    }
}

/// The session total, summed from the same per-model cells the `By model:` rows print so the two
/// lines cannot disagree. A single total with no model breakdown (reported totals-only payloads)
/// falls back to pricing the totals row with the session model.
fn session_cost_cell(
    usage: &PromptUsage,
    pricing: &PricingTable,
    model_id: Option<&str>,
) -> CostCell {
    if usage.model_usage.is_empty() {
        return model_cost_cell(model_id.unwrap_or(""), &usage.totals, pricing);
    }
    let mut reported = 0.0;
    let mut estimated = 0.0;
    let mut any_estimated = false;
    let mut any_priced = false;
    for (model, m) in &usage.model_usage {
        match model_cost_cell(model, m, pricing) {
            CostCell::Reported(usd) => {
                reported += usd;
                any_priced = true;
            }
            CostCell::Estimated(usd) => {
                estimated += usd;
                any_estimated = true;
                any_priced = true;
            }
            CostCell::Unavailable { .. } => {}
        }
    }
    if any_estimated {
        CostCell::Estimated(reported + estimated)
    } else if any_priced {
        CostCell::Reported(reported)
    } else {
        CostCell::Unavailable {
            partial: usage.totals.cost_is_partial,
        }
    }
}

/// Ticks are 1e10 per USD; an estimate is labelled so it is never mistaken for provider-reported.
fn format_cost_cell(cell: CostCell) -> String {
    match cell {
        CostCell::Reported(usd) => format!("${usd:.4}"),
        CostCell::Estimated(usd) => format!("${usd:.4} (estimate)"),
        CostCell::Unavailable { partial: true } => {
            "not available (not reported for some calls)".to_string()
        }
        CostCell::Unavailable { partial: false } => "not available (not reported)".to_string(),
    }
}

/// Project an ACP-identity ledger row onto the status line's disjoint token buckets.
///
/// `PromptUsageModel::input_tokens` already includes cache reads and cache creation, while
/// [`StatusLineSessionUsage`] wants fresh input only, so the fresh count is the saturating
/// difference. Both cache buckets map across unchanged.
fn status_line_usage(m: &PromptUsageModel) -> StatusLineSessionUsage {
    StatusLineSessionUsage {
        input_tokens: m
            .input_tokens
            .saturating_sub(m.cached_read_tokens)
            .saturating_sub(m.cache_creation_tokens),
        output_tokens: m.output_tokens,
        cache_creation_input_tokens: m.cache_creation_tokens,
        cache_read_input_tokens: m.cached_read_tokens,
        reasoning_tokens: m.reasoning_tokens,
    }
}
/// First non-empty, trimmed line of `text` (empty string if none). Collapses a multi-line prompt/command to a single display line.
pub(crate) fn first_nonempty_line(text: &str) -> &str {
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
}
/// Format one `/queue` row as `  #N  <first non-empty line>` with a `(+K more lines)` suffix for multi-line prompts.
fn format_queue_row(pos: usize, text: &str) -> String {
    let first_line = first_nonempty_line(text);
    let extra = text.lines().count().saturating_sub(1);
    if extra > 0 {
        format!(
            "  #{pos}  {first_line}  (+{extra} more line{})",
            if extra == 1 { "" } else { "s" }
        )
    } else {
        format!("  #{pos}  {first_line}")
    }
}
fn join_header_rows(header: String, rows: Vec<String>) -> String {
    std::iter::once(header)
        .chain(rows)
        .collect::<Vec<_>>()
        .join("\n")
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use xai_grok_shell::extensions::notification::{PromptUsage, PromptUsageModel};
    use xai_grok_status_line::{ModelPricing, PricingTable};
    fn model_row(input: u64, output: u64, ticks: Option<i64>) -> PromptUsageModel {
        PromptUsageModel {
            input_tokens: input,
            output_tokens: output,
            total_tokens: input + output,
            cached_read_tokens: 0,
            cache_creation_tokens: 0,
            reasoning_tokens: 0,
            model_calls: 1,
            api_duration_ms: 1_000,
            cost_usd_ticks: ticks,
            cost_is_partial: false,
            cost_missing_calls: 0,
        }
    }
    fn single_model_usage(model: &str, row: PromptUsageModel) -> PromptUsage {
        let mut usage = PromptUsage {
            totals: row.clone(),
            ..Default::default()
        };
        usage.model_usage.insert(model.to_string(), row);
        usage
    }
    #[test]
    fn session_usage_block_empty_ledger() {
        let usage = PromptUsage::default();
        assert_eq!(
            session_usage_block_text(&usage, &PricingTable::default(), None),
            "Session usage: no model calls yet in this session."
        );
        let incomplete = PromptUsage {
            usage_is_incomplete: true,
            ..Default::default()
        };
        assert!(
            session_usage_block_text(&incomplete, &PricingTable::default(), None)
                .contains("incomplete")
        );
    }
    /// The ACP ledger folds cache reads and cache creation into `input_tokens`; the status line wants
    /// disjoint buckets, so the fresh count is a saturating subtraction.
    #[test]
    fn status_line_usage_maps_acp_input_to_disjoint_buckets() {
        let row = PromptUsageModel {
            input_tokens: 100,
            output_tokens: 7,
            cached_read_tokens: 80,
            cache_creation_tokens: 50,
            reasoning_tokens: 3,
            ..model_row(0, 0, None)
        };
        let mapped = status_line_usage(&row);
        assert_eq!(mapped.input_tokens, 0, "the subtraction must saturate");
        assert_eq!(mapped.cache_read_input_tokens, 80);
        assert_eq!(mapped.cache_creation_input_tokens, 50);
        assert_eq!(mapped.output_tokens, 7);
        assert_eq!(mapped.reasoning_tokens, 3);

        let row = PromptUsageModel {
            input_tokens: 100,
            cached_read_tokens: 40,
            cache_creation_tokens: 25,
            ..model_row(0, 0, None)
        };
        assert_eq!(status_line_usage(&row).input_tokens, 35);
    }
    #[test]
    fn session_usage_block_estimates_missing_cost_at_the_status_line_rate() {
        let table = PricingTable::default();
        let row = model_row(2_000, 500, None);
        let expected = table
            .cost_usd("deepseek-v4-pro", &status_line_usage(&row))
            .expect("estimate");
        let usage = single_model_usage("deepseek-v4-pro", row);
        let text = session_usage_block_text(&usage, &table, None);
        assert!(
            text.contains(&format!("${expected:.4} (estimate)")),
            "{text}"
        );
        assert!(!text.contains("not available"), "{text}");
    }
    #[test]
    fn session_usage_block_keeps_reported_cost_without_estimate_label() {
        let row = model_row(1_000, 100, Some(12_345_000_000));
        let usage = single_model_usage("deepseek-v4-pro", row);
        let text = session_usage_block_text(&usage, &PricingTable::default(), None);
        assert!(text.contains("$1.2345"), "{text}");
        assert!(!text.contains("(estimate)"), "{text}");
    }
    #[test]
    fn session_usage_block_mixed_costs_sum_and_label_estimate() {
        let table = PricingTable::default();
        let reported = model_row(1_000_000, 0, Some(10_000_000_000));
        let estimated = model_row(1_000_000, 0, None);
        let estimate = table
            .cost_usd("deepseek-flash", &status_line_usage(&estimated))
            .expect("estimate");
        assert!((estimate - 0.30).abs() < 1e-12, "{estimate}");
        let mut usage = PromptUsage {
            totals: model_row(2_000_000, 0, None),
            ..Default::default()
        };
        usage.model_usage.insert("deepseek-v4-pro".into(), reported);
        usage.model_usage.insert("deepseek-flash".into(), estimated);
        let text = session_usage_block_text(&usage, &table, None);
        assert!(
            text.contains("  Cost:           $1.3000 (estimate)"),
            "{text}"
        );
        assert!(
            text.contains("    deepseek-v4-pro: 1,000,000 Tokens · $1.0000"),
            "{text}"
        );
        assert!(
            text.contains("    deepseek-flash: 1,000,000 Tokens · $0.3000 (estimate)"),
            "{text}"
        );
    }
    #[test]
    fn session_usage_block_no_usage_stays_not_available() {
        let usage = PromptUsage {
            totals: model_row(0, 0, None),
            ..Default::default()
        };
        let text = session_usage_block_text(&usage, &PricingTable::default(), None);
        assert!(
            text.contains("  Cost:           not available (not reported)"),
            "{text}"
        );
        assert!(!text.contains("$0.0000"), "{text}");
    }
    /// Proves the `/cost` estimate reads the same table the status line builds from config, overrides included.
    #[test]
    fn session_usage_block_honours_pricing_overrides() {
        let mut overrides = BTreeMap::new();
        overrides.insert(
            "deepseek-flash".to_string(),
            ModelPricing {
                cache_hit: 0.0,
                cache_miss: 10.0,
                output: 0.0,
            },
        );
        let table = PricingTable::with_overrides(overrides);
        let usage = single_model_usage("deepseek-flash", model_row(1_000_000, 0, None));
        let text = session_usage_block_text(&usage, &table, None);
        assert!(text.contains("$10.0000 (estimate)"), "{text}");
        assert!(!text.contains("$0.3000"), "{text}");
    }
    #[test]
    fn session_usage_block_formats_tokens_and_cost() {
        let mut totals = model_row(1_234_567, 45_678, Some(12_345_000_000));
        totals.cached_read_tokens = 1_000_000;
        totals.reasoning_tokens = 12_000;
        totals.model_calls = 42;
        totals.api_duration_ms = 192_000;
        let usage = PromptUsage {
            totals,
            ..Default::default()
        };
        let text = session_usage_block_text(&usage, &PricingTable::default(), None);
        insta::assert_snapshot!("session_usage_block_full", text);
    }
    #[test]
    fn session_usage_block_lists_models_when_multiple() {
        let mut usage = PromptUsage {
            totals: model_row(150, 15, None),
            ..Default::default()
        };
        usage
            .model_usage
            .insert("grok-build".into(), model_row(100, 10, None));
        usage
            .model_usage
            .insert("deepseek-4".into(), model_row(50, 5, None));
        let text = session_usage_block_text(&usage, &PricingTable::default(), None);
        insta::assert_snapshot!("session_usage_block_by_model", text);
    }
    #[test]
    fn session_usage_block_by_model_reports_or_estimates_each_model() {
        let mut usage = PromptUsage {
            totals: model_row(123_556, 1_010, Some(20_000_000)),
            ..Default::default()
        };
        usage.model_usage.insert(
            "deepseek-4.7-build".into(),
            model_row(123_456, 1_000, Some(20_000_000)),
        );
        usage
            .model_usage
            .insert("deepseek-4".into(), model_row(100, 10, None));
        let text = session_usage_block_text(&usage, &PricingTable::default(), None);
        assert!(
            text.contains("  Cost:           $0.0022 (estimate)"),
            "{text}"
        );
        let rows: Vec<&str> = text
            .lines()
            .skip_while(|line| *line != "  By model:")
            .skip(1)
            .collect();
        assert_eq!(
            rows,
            [
                "    deepseek-4.7-build: 124,456 Tokens · $0.0020",
                "    deepseek-4: 110 Tokens · $0.0002 (estimate)",
            ],
            "{text}"
        );
    }
    #[test]
    fn session_usage_block_absent_cost_is_unknown_not_free() {
        let usage = PromptUsage {
            totals: model_row(0, 0, None),
            ..Default::default()
        };
        let text = session_usage_block_text(&usage, &PricingTable::default(), None);
        assert!(text.contains("not available (not reported)"), "{text}");
        assert!(!text.contains("$0.0000"), "{text}");
    }
    #[test]
    fn session_usage_block_estimates_partial_cost_and_flags_incomplete() {
        let mut totals = model_row(100, 10, None);
        totals.cost_is_partial = true;
        let usage = PromptUsage {
            totals,
            usage_is_incomplete: true,
            ..Default::default()
        };
        let text = session_usage_block_text(&usage, &PricingTable::default(), None);
        assert!(text.contains("(estimate)"), "{text}");
        assert!(text.contains("usage is incomplete"), "{text}");
    }
    #[test]
    fn group_thousands_groups_digits() {
        assert_eq!(group_thousands(0), "0");
        assert_eq!(group_thousands(999), "999");
        assert_eq!(group_thousands(1_000), "1,000");
        assert_eq!(group_thousands(1_234_567), "1,234,567");
    }
    #[test]
    fn first_nonempty_line_skips_blank_leading_lines() {
        assert_eq!(first_nonempty_line("\n  \n  hello \nworld"), "hello");
        assert_eq!(first_nonempty_line("   "), "");
        assert_eq!(first_nonempty_line(""), "");
        assert_eq!(first_nonempty_line("only"), "only");
    }
    #[test]
    fn format_queue_row_single_line() {
        assert_eq!(format_queue_row(1, "fix the bug"), "  #1  fix the bug");
    }
    #[test]
    fn format_queue_row_multiline_reports_extra_lines() {
        assert_eq!(
            format_queue_row(2, "first\nsecond"),
            "  #2  first  (+1 more line)"
        );
        assert_eq!(
            format_queue_row(3, "first\nsecond\nthird"),
            "  #3  first  (+2 more lines)"
        );
    }
}
