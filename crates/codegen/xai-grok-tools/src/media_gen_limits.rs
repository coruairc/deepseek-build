//! Per-batch caps for media-generation tools.
//!
//! The media-generation tools have been removed from this build, so no
//! `ToolKind` maps to a media batch budget any more. The public API is kept as
//! a no-op so host call sites (`xai-grok-shell`) continue to compile without
//! change; [`max_calls_per_batch`] always returns `None`.

use crate::types::tool::ToolKind;

pub const DEFAULT_MAX_PARALLEL_IMAGE_GEN: usize = 8;
pub const DEFAULT_MAX_PARALLEL_VIDEO_GEN: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaGenBatchLimits {
    pub max_image: usize,
    pub max_video: usize,
}

impl Default for MediaGenBatchLimits {
    fn default() -> Self {
        Self {
            max_image: DEFAULT_MAX_PARALLEL_IMAGE_GEN,
            max_video: DEFAULT_MAX_PARALLEL_VIDEO_GEN,
        }
    }
}

/// No tool kind has a per-batch media budget in this build.
pub fn max_calls_per_batch(_kind: ToolKind, _limits: &MediaGenBatchLimits) -> Option<usize> {
    None
}

/// A tool name whose count in one model step exceeds its per-name cap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaGenOverCap {
    pub name: String,
    pub total: usize,
    pub max: usize,
}

impl MediaGenOverCap {
    pub fn is_egregious(&self) -> bool {
        self.max > 0 && self.total >= self.max.saturating_mul(2)
    }
}

/// Names (with totals) that exceed their per-name cap. Always empty in this build.
pub fn over_cap_by_name<'a>(
    _calls: impl IntoIterator<Item = (&'a str, Option<ToolKind>)>,
    _limits: &MediaGenBatchLimits,
) -> Vec<MediaGenOverCap> {
    Vec::new()
}

/// First over-cap at 2x (or more) resamples; never true with no media tools.
pub fn should_resample_egregious(
    _over: &[MediaGenOverCap],
    _resamples_used: u32,
    _max_resamples: u32,
) -> bool {
    false
}

/// Reminder after a discarded 2x burst. No media tools remain, so empty.
pub fn resample_reminder(_egregious: &[MediaGenOverCap]) -> String {
    String::new()
}

/// Per-name first-K partitioning. With no media tools every call is admitted.
pub fn partition_media_gen_batch<T>(
    calls: impl IntoIterator<Item = T>,
    _tool_name: impl Fn(&T) -> &str,
    _tool_kind: impl Fn(&T) -> Option<ToolKind>,
    _limits: &MediaGenBatchLimits,
) -> (Vec<T>, Vec<(T, String)>) {
    (calls.into_iter().collect(), Vec::new())
}
