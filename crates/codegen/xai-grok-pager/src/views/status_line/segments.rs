//! The `builtin` row: one segment per [`StatusLineItem`] the session asked for, each already cut to the columns it may use.

use std::time::Duration;

use xai_grok_status_line::{StatusLineContext, StatusLineItem, StatusLineSessionUsage};

use super::fit_columns;

pub const SEGMENT_SEPARATOR: &str = " │ ";

const CONTEXT_WARN_PCT: u8 = 80;

// Columns, not bytes: a byte budget halves a CJK or emoji name.
const CWD_COLS: usize = 40;
const MODEL_COLS: usize = 30;
const SESSION_NAME_COLS: usize = 40;

// The smallest value four decimals can show as non-zero; below it the segment is omitted rather than shown as a
// zero. Derived from the precision, not tuned.
pub(crate) const MIN_DISPLAYED_COST_USD: f64 = 0.0001;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentTone {
    Dim,
    Warn,
}

/// A `builtin` segment, already cut to the columns it may use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSegment {
    // Not `pub`: a struct literal elsewhere would skip the control-character filter in [`Self::new`]
    // Read through [`Self::text`]
    pub(super) text: String,
    pub(super) tone: SegmentTone,
}

impl StatusSegment {
    fn toned(text: String, tone: SegmentTone) -> Self {
        Self::new(text, tone)
    }

    /// Read access for tests in other modules; the fields stay closed so a literal cannot skip [`Self::new`].
    #[cfg(test)]
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    fn dim(text: impl Into<String>) -> Self {
        Self::new(text, SegmentTone::Dim)
    }

    pub(crate) fn warn(text: impl Into<String>) -> Self {
        Self::new(text, SegmentTone::Warn)
    }

    /// Control characters are dropped here rather than at the painter.
    /// A segment carries the user's own text: a cwd, a model name, a config value they typed.
    /// Only [`SanitizedText`](super::SanitizedText) filters the path a script's output takes.
    fn new(text: impl Into<String>, tone: SegmentTone) -> Self {
        let text: String = text.into();
        Self {
            text: text.chars().filter(|c| !c.is_control()).collect(),
            tone,
        }
    }
}

// TODO(auto-routing): Phase 3 asks the row to show the auto-routing decision (selected route/model) and any manual
// override. The classifier and its policy live on another branch; no `StatusLineContext` field carries a route yet, so
// there is nothing to display. When that field lands: add a `route` segment here that renders the route only when the
// payload names one, and mark the manual override (a pinned model/effort) distinctly from the routed choice.
#[must_use]
pub fn compose_builtin(
    ctx: &StatusLineContext,
    turn_elapsed: Option<Duration>,
    items: &[StatusLineItem],
) -> Vec<StatusSegment> {
    items
        .iter()
        .filter_map(|item| match item {
            StatusLineItem::Cwd => {
                let short = ctx.cwd.rsplit(['/', '\\']).find(|s| !s.is_empty())?;
                Some(StatusSegment::dim(fit_columns(short, CWD_COLS)))
            }
            StatusLineItem::Model => {
                let model = ctx
                    .model
                    .display_name
                    .as_deref()
                    .filter(|s| !s.is_empty())?;
                Some(StatusSegment::dim(fit_columns(model, MODEL_COLS)))
            }
            StatusLineItem::Context => {
                let window = &ctx.context_window;
                let pct = window.used_percentage?;
                let warn_at = window
                    .auto_compact_threshold_percent
                    .unwrap_or(CONTEXT_WARN_PCT);
                let tone = if pct >= warn_at {
                    SegmentTone::Warn
                } else {
                    SegmentTone::Dim
                };
                Some(StatusSegment::toned(format!("{pct}% ctx"), tone))
            }
            StatusLineItem::Cost => {
                Some(StatusSegment::dim(format_cost(ctx.cost.total_cost_usd?)?))
            }
            StatusLineItem::Effort => {
                let level = ctx.effort.as_ref()?.level.as_str();
                let level = level.trim();
                (!level.is_empty()).then(|| StatusSegment::dim(format!("effort {level}")))
            }
            StatusLineItem::Tokens => {
                let usage = ctx.context_window.session_usage.as_ref()?;
                Some(StatusSegment::dim(format_tokens(usage)))
            }
            StatusLineItem::Cache => {
                let rate = ctx
                    .context_window
                    .session_usage
                    .as_ref()?
                    .cache_hit_rate()?;
                Some(StatusSegment::dim(format!("cache {:.0}%", rate * 100.0)))
            }
            StatusLineItem::TurnTimer => {
                let secs = turn_elapsed?.as_secs();
                if secs == 0 {
                    return None;
                }
                Some(StatusSegment::dim(crate::views::dock::fmt_elapsed(secs)))
            }
            StatusLineItem::SessionName => {
                let name = ctx.session_name.as_deref().filter(|s| !s.is_empty())?;
                Some(StatusSegment::dim(fit_columns(name, SESSION_NAME_COLS)))
            }
        })
        .collect()
}

/// The cost segment's whole text: `~$0.37 est.` at or above one cent, four decimals below it, or nothing when the
/// value is unknown, not finite, negative, or below the smallest figure four decimals can show.
///
/// Precision is fixed per branch, so the segment keeps a stable width as the number moves, and a value that rounds
/// up to a full cent at four decimals (`0.00996`) promotes to the two-decimal form rather than reading `$0.0100`.
/// The `~` and `est.` markers carry the imprecision; the figure is a running estimate, never a bill.
fn format_cost(usd: f64) -> Option<String> {
    if !usd.is_finite() || usd < MIN_DISPLAYED_COST_USD {
        return None;
    }
    let sub_cent = (usd * 10_000.0).round() < 100.0;
    Some(if sub_cent {
        format!("~${usd:.4} est.")
    } else {
        format!("~${usd:.2} est.")
    })
}

/// Session token buckets, compact enough for one row: `in 12.3k · cache 40.0k · out 4.5k · think 1.2k`.
/// The cache figure is the cache-hit subset of the prompt, shown beside the fresh `in` count rather than summed with it.
fn format_tokens(usage: &StatusLineSessionUsage) -> String {
    format!(
        "in {} \u{00b7} cache {} \u{00b7} out {} \u{00b7} think {}",
        fmt_tokens(usage.input_tokens),
        fmt_tokens(usage.cache_read_input_tokens),
        fmt_tokens(usage.output_tokens),
        fmt_tokens(usage.reasoning_tokens),
    )
}

/// `999`, `1.2k`, `100k`, `1.2m`. Truncates rather than rounds up so the segment never overstates the bill.
fn fmt_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}m", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        format!("{n}")
    }
}

#[cfg(test)]
#[path = "segments_tests.rs"]
mod tests;
