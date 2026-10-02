//! Inert stubs for the former remote-announcement banner/upsell surface.
//!
//! The xAI remote-announcement banners and upgrade CTAs have been removed. These
//! functions keep their original signatures so existing call sites continue to
//! compile, but they no longer paint anything or surface any content.

use std::collections::BTreeSet;

use ratatui::{buffer::Buffer, layout::Rect};

use crate::theme::Theme;
use xai_grok_shell::util::config::RemoteAnnouncement;

/// Columns the former `[label]` CTA button wanted. Always zero now.
pub(crate) fn upgrade_cta_reserve(label: &str, caption: Option<&str>) -> u16 {
    let _ = (label, caption);
    0
}

/// The former shared CTA button painter. Paints nothing and reports no hit rect.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_cta_button(
    _buf: &mut Buffer,
    _theme: &Theme,
    _x: u16,
    _y: u16,
    _max_width: u16,
    _label: &str,
    _caption: Option<&str>,
    _hovered: bool,
) -> Option<Rect> {
    None
}

/// A removed announcement is never dismissible because it is never shown.
pub fn is_dismissible(_a: &RemoteAnnouncement) -> bool {
    true
}

/// The former CTA caption accessor. Never yields a caption now.
pub(crate) fn usable_cta_caption(_a: &RemoteAnnouncement) -> Option<&str> {
    None
}

/// No session announcement is ever selected.
pub fn first_session_announcement<'a>(
    _announcements: &'a [RemoteAnnouncement],
    _hidden_ids: &BTreeSet<String>,
) -> Option<&'a RemoteAnnouncement> {
    None
}

/// No critical session announcement exists.
pub fn has_critical_session_announcement(
    _announcements: &[RemoteAnnouncement],
    _hidden_ids: &BTreeSet<String>,
) -> bool {
    false
}

/// No session announcement is ever selected.
pub fn first_session_announcement_at<'a>(
    _announcements: &'a [RemoteAnnouncement],
    _hidden_ids: &BTreeSet<String>,
    _now: chrono::DateTime<chrono::Utc>,
) -> Option<&'a RemoteAnnouncement> {
    None
}

/// No promo CTA exists.
pub(crate) fn promo_cta<'a>(
    _announcements: &'a [RemoteAnnouncement],
    _hidden_ids: &BTreeSet<String>,
) -> Option<(&'a RemoteAnnouncement, &'a str, &'a str)> {
    None
}

/// No promo CTA target exists.
pub fn promo_cta_target<'a>(
    _announcements: &'a [RemoteAnnouncement],
    _hidden_ids: &BTreeSet<String>,
) -> Option<(&'a RemoteAnnouncement, &'a str)> {
    None
}

/// There are no session announcement hide keys to clear.
pub fn session_announcement_hide_keys(_announcements: &[RemoteAnnouncement]) -> Vec<String> {
    Vec::new()
}

/// There are no session announcement hide keys to clear.
pub fn session_announcement_hide_keys_at(
    _announcements: &[RemoteAnnouncement],
    _now: chrono::DateTime<chrono::Utc>,
) -> Vec<String> {
    Vec::new()
}

/// No session announcements exist.
pub fn has_session_announcements(_announcements: &[RemoteAnnouncement]) -> bool {
    false
}

/// The session banner is always zero rows tall.
pub fn session_banner_height(
    _announcements: &[RemoteAnnouncement],
    _hidden_ids: &BTreeSet<String>,
) -> u16 {
    0
}

/// Clickable rects painted by [`render_banner`] (`None` means not painted).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BannerHits {
    /// The `[hide]` button.
    pub hide: Option<Rect>,
    /// The promo `[label]` CTA button (critical rows never paint one).
    pub cta: Option<Rect>,
}

/// The former session top banner. Paints nothing and reports no hit rects.
pub fn render_banner(
    _area: Rect,
    _buf: &mut Buffer,
    _announcements: &[RemoteAnnouncement],
    _hidden_ids: &BTreeSet<String>,
    _hide_hovered: bool,
    _cta_hovered: bool,
    _caption_allowed: bool,
) -> BannerHits {
    BannerHits::default()
}
