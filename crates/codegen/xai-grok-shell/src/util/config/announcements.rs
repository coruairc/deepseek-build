use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;
use toml::Value as TomlValue;

/// Announcement entry received from cli-chat-proxy `/v1/settings`.
pub use xai_grok_config::{AnnouncementCta, RemoteAnnouncement};

// ---------------------------------------------------------------------------
// Announcement types (moved from the deleted xai-grok-announcements crate)
// ---------------------------------------------------------------------------

/// Payload for `x.ai/announcements/update` ACP notification.
#[derive(Debug, Clone, Deserialize)]
pub struct AnnouncementsRefreshed {
    // The wire value is a plain JSON number.
    #[serde(rename = "gen")]
    pub r#gen: u64,
    #[serde(default)]
    pub announcements: Vec<RemoteAnnouncement>,
}

/// Stable per-announcement hide key: the trimmed non-empty `id`, else a content-derived fallback so id-less items are still hideable.
/// The fallback joins title/message with the unprintable unit separator (\x1f), so distinct title/message splits cannot collide.
pub fn announcement_hide_key(a: &RemoteAnnouncement) -> String {
    match a.id.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(id) => id.to_string(),
        None => format!(
            "content:{}\u{1f}{}",
            a.title.as_deref().unwrap_or_default(),
            a.message.as_deref().unwrap_or_default()
        ),
    }
}

/// Parse persisted hidden state into a set of hidden announcement ids.
pub fn parse_hidden_announcement_ids(s: &str) -> BTreeSet<String> {
    #[derive(Deserialize)]
    struct State {
        #[serde(default)]
        hidden_ids: BTreeSet<String>,
    }
    serde_json::from_str::<State>(s)
        .map(|s| s.hidden_ids)
        .unwrap_or_default()
}

/// Serialize hidden announcement ids (writes only the `hidden_ids` shape).
pub fn serialize_hidden_announcement_ids(ids: &BTreeSet<String>) -> Option<String> {
    #[derive(Serialize)]
    struct State<'a> {
        hidden_ids: &'a BTreeSet<String>,
    }
    serde_json::to_string(&State { hidden_ids: ids }).ok()
}

/// Drop hidden ids whose announcement is no longer active; returns whether the set changed.
pub fn prune_hidden_announcement_ids(
    ids: &mut BTreeSet<String>,
    active: &[RemoteAnnouncement],
) -> bool {
    let live: BTreeSet<String> = active.iter().map(announcement_hide_key).collect();
    let before = ids.len();
    ids.retain(|id| live.contains(id));
    ids.len() != before
}

/// Read hidden announcement ids from `~/.grok/announcements.json`.
pub async fn read_hidden_announcement_ids() -> BTreeSet<String> {
    let path = announcements_state_path();
    match tokio::fs::read_to_string(&path).await {
        Ok(s) => parse_hidden_announcement_ids(&s),
        Err(_) => BTreeSet::new(),
    }
}

/// Write hidden announcement ids to `~/.grok/announcements.json`.
pub async fn write_hidden_announcement_ids(ids: &BTreeSet<String>) {
    let path = announcements_state_path();
    if let Some(s) = serialize_hidden_announcement_ids(ids) {
        let _ = tokio::fs::write(&path, s).await;
    }
}

fn announcements_state_path() -> PathBuf {
    xai_grok_tools::util::grok_home::grok_home().join("announcements.json")
}

/// Return only announcements with non-empty (trimmed) messages.
pub fn visible_announcements(announcements: &[RemoteAnnouncement]) -> Vec<&RemoteAnnouncement> {
    announcements
        .iter()
        .filter(|a| {
            a.message
                .as_ref()
                .map(|m| !m.trim().is_empty())
                .unwrap_or(false)
        })
        .collect()
}

/// Filter out announcements whose `expires_at` is in the past.
pub fn filter_expired(
    announcements: impl IntoIterator<Item = RemoteAnnouncement>,
) -> Vec<RemoteAnnouncement> {
    filter_expired_at(announcements, Utc::now())
}

/// [`filter_expired`] with an injectable clock.
pub fn filter_expired_at(
    announcements: impl IntoIterator<Item = RemoteAnnouncement>,
    now: DateTime<Utc>,
) -> Vec<RemoteAnnouncement> {
    announcements
        .into_iter()
        .filter(|a| !is_expired_at(a, now))
        .collect()
}

/// Whether `expires_at` parses and is at/behind `now`; missing/unparseable never expires.
pub fn is_expired_at(a: &RemoteAnnouncement, now: DateTime<Utc>) -> bool {
    if let Some(exp) = &a.expires_at
        && let Ok(dt) = DateTime::parse_from_rfc3339(exp)
    {
        return dt <= now;
    }
    false
}

// ---------------------------------------------------------------------------
// Announcements & tips from TOML
// ---------------------------------------------------------------------------

/// Parse `announcements` from a TOML value (inline tables or array-of-tables).
pub(crate) fn announcements_from_toml(root: &TomlValue) -> Vec<RemoteAnnouncement> {
    root.get("announcements")
        .and_then(|v| v.clone().try_into::<Vec<RemoteAnnouncement>>().ok())
        .unwrap_or_default()
}

/// Merge announcement slices in priority order. Dedup by `id`; first wins.
pub(crate) fn merge_announcements(sources: &[&[RemoteAnnouncement]]) -> Vec<RemoteAnnouncement> {
    let mut seen = std::collections::HashSet::<String>::new();
    let mut out = Vec::new();
    for source in sources {
        for a in *source {
            if let Some(ref id) = a.id
                && !seen.insert(id.clone())
            {
                continue;
            }
            out.push(a.clone());
        }
    }
    out
}

/// Dev/test override for announcements via `GROK_ANNOUNCEMENTS_OVERRIDE` (a JSON array of announcements).
pub(crate) fn announcements_override() -> Option<Vec<RemoteAnnouncement>> {
    let raw = std::env::var("GROK_ANNOUNCEMENTS_OVERRIDE").ok()?;
    match serde_json::from_str::<Vec<RemoteAnnouncement>>(&raw) {
        Ok(list) => Some(list),
        Err(_) => {
            tracing::warn!("invalid GROK_ANNOUNCEMENTS_OVERRIDE JSON; ignoring");
            None
        }
    }
}

/// Priority order: requirements, then remote, then user config, then managed config.
/// The `GROK_ANNOUNCEMENTS_OVERRIDE` env var overrides everything (dev only).
pub fn resolve_announcements(
    requirements: Option<&TomlValue>,
    user: Option<&TomlValue>,
    managed: Option<&TomlValue>,
    remote: Option<&[RemoteAnnouncement]>,
) -> Vec<RemoteAnnouncement> {
    if let Some(list) = announcements_override() {
        return list;
    }

    let req = requirements
        .map(announcements_from_toml)
        .unwrap_or_default();
    let usr = user.map(announcements_from_toml).unwrap_or_default();
    let mgd = managed.map(announcements_from_toml).unwrap_or_default();
    let remote_slice = remote.unwrap_or_default();

    merge_announcements(&[&req, remote_slice, &usr, &mgd])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_expired_removes_past() {
        let past = RemoteAnnouncement {
            expires_at: Some("2000-01-01T00:00:00Z".to_string()),
            ..Default::default()
        };
        let future = RemoteAnnouncement {
            expires_at: Some("2100-01-01T00:00:00Z".to_string()),
            ..Default::default()
        };
        let none = RemoteAnnouncement {
            expires_at: None,
            ..Default::default()
        };

        let filtered = filter_expired(vec![past, future, none]);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn hidden_ids_round_trip() {
        let ids: BTreeSet<String> = ["outage-a".to_string(), "outage-b".to_string()]
            .into_iter()
            .collect();
        let s = serialize_hidden_announcement_ids(&ids).expect("serialize");
        assert_eq!(parse_hidden_announcement_ids(&s), ids);
    }

    #[test]
    fn announcement_hide_key_prefers_id_with_content_fallback() {
        let with_id = RemoteAnnouncement {
            id: Some("  spaced-id  ".into()),
            title: Some("T".into()),
            message: Some("M".into()),
            ..Default::default()
        };
        assert_eq!(announcement_hide_key(&with_id), "spaced-id");

        let no_id = RemoteAnnouncement::default();
        assert_eq!(announcement_hide_key(&no_id), "content:\u{1f}");
    }
}
