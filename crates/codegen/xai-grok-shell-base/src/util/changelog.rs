//! Changelog loading from the local disk cache.
//!
//! Both markdown (`*.external.md`) and JSON (`*.external.json`) changelogs are read from `$GROK_HOME`.
//! Remote CDN fetching was removed from this build; the manager only reads the on-disk copies.
//!
//! `ChangelogManager::fetch()` returns a `Changelog` with optional markdown and structured entries.
//! Consumers pick the format they need:
//! - `/release-notes` uses `changelog.markdown` for rich scrollback display
//! - The welcome screen uses `changelog.entries` for bullet rendering

use std::path::PathBuf;

/// Retained for callers/tests that pass an explicit base; this build performs no remote fetch.
const CHANGELOG_BASE: &str = "";

/// A single structured changelog entry from the published JSON changelog. Shape must match the output of `render_external_json` in `changelog.sh`: `{category, description, breaking_change}`
/// If you change fields here, update `changelog.sh:render_external_json` too. All fields use `#[serde(default)]` so a single malformed entry doesn't kill the entire array parse.
/// Entries with an empty description are filtered out by `bullets_from_entries`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ChangelogEntry {
    /// Category label (e.g. "features", "fixes", "breaking", "performance").
    #[serde(default)]
    pub category: String,
    /// Human-readable description (may contain `**bold**` or backticks).
    #[serde(default)]
    pub description: String,
    /// Whether this entry represents a breaking change.
    #[serde(default)]
    pub breaking_change: bool,
}

/// Both formats of a version's changelog, fetched together.
pub struct Changelog {
    /// Rendered markdown (for `/release-notes` display).
    pub markdown: Option<String>,
    /// Structured entries (for welcome screen bullets).
    pub entries: Option<Vec<ChangelogEntry>>,
}

/// Manages changelog retrieval from CDN with local disk caching.
/// Single entry point: `fetch()` returns both markdown and JSON in one `Changelog` struct.
/// Each format is fetched independently with its own cache file, so a failure in one doesn't block the other.
pub struct ChangelogManager {
    md_cache: PathBuf,
    json_cache: PathBuf,
}

impl Default for ChangelogManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ChangelogManager {
    pub fn new() -> Self {
        // Prefer the live `$GROK_HOME` over the `grok_home()` OnceLock
        // A home injected by the PTY e2e harness must beat a path some earlier init cached in the same process
        Self::from_env_home()
    }

    /// Resolve cache paths from the live process environment (not the `grok_home()` OnceLock).
    /// A seeded `$GROK_HOME` set on the pager process is always honoured even if some earlier init path cached a different home.
    fn from_env_home() -> Self {
        let home = std::env::var_os("GROK_HOME")
            .map(std::path::PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(crate::util::grok_home::grok_home);
        Self {
            md_cache: home.join("CHANGELOG.md"),
            json_cache: home.join("CHANGELOG.json"),
        }
    }

    /// Read both markdown and JSON changelogs for the current version from the local disk cache.
    /// Either field may be `None` when no cache is present. Remote CDN fetching was removed from this build.
    pub fn fetch(&self) -> Changelog {
        // Always re-resolve from env so a caller holding an older manager (or a stale OnceLock) still reads the live harness home
        Self::from_env_home().fetch_with(changelog_offline(), CHANGELOG_BASE)
    }

    /// Read from this manager's already-resolved cache paths. The `offline` flag and `base` are retained for call-site compatibility; this build performs no remote fetch.
    /// Split out of [`fetch`] so unit tests can drive it against a temp home without touching process-global env.
    fn fetch_with(&self, _offline: bool, _base: &str) -> Changelog {
        Changelog {
            markdown: read_cache(&self.md_cache),
            entries: self.read_json_cache(),
        }
    }

    fn read_json_cache(&self) -> Option<Vec<ChangelogEntry>> {
        let cached = read_cache(&self.json_cache)?;
        match serde_json::from_str(&cached) {
            Ok(entries) => Some(entries),
            Err(e) => {
                tracing::debug!(error = %e, "failed to parse cached JSON changelog");
                None
            }
        }
    }
}

/// Retained for callers that set it; this build always reads only the disk cache.
fn changelog_offline() -> bool {
    std::env::var_os("GROK_CHANGELOG_OFFLINE").is_some_and(|v| !v.is_empty() && v != "0")
}

fn read_cache(path: &std::path::Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .filter(|c| !c.trim().is_empty())
}

/// Strip `**bold**` markers and backticks from a description string.
fn strip_markdown_inline(s: &str) -> String {
    s.replace("**", "").replace('`', "")
}

/// Convert changelog entries to plain-text bullet strings.
/// Strips `**bold**` and backtick formatting from each description and returns at most `max` entries.
/// Entries with an empty description (from tolerant deserialization) are skipped.
pub fn bullets_from_entries(entries: &[ChangelogEntry], max: usize) -> Vec<String> {
    entries
        .iter()
        .filter(|e| !e.description.is_empty())
        .take(max)
        .map(|e| strip_markdown_inline(&e.description))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a manager pointing at `home` directly, bypassing the global `$GROK_HOME` env so tests never race the parallel harness.
    fn manager_for(home: &std::path::Path) -> ChangelogManager {
        ChangelogManager {
            md_cache: home.join("CHANGELOG.md"),
            json_cache: home.join("CHANGELOG.json"),
        }
    }

    #[test]
    fn offline_mode_reads_seeded_disk_cache_only() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path().join("grok-home");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(home.join("CHANGELOG.md"), "# seeded offline md\n").unwrap();
        std::fs::write(
            home.join("CHANGELOG.json"),
            r#"[{"category":"features","description":"seeded entry","breaking_change":false}]"#,
        )
        .unwrap();

        // Offline path: read only the seeded disk cache, no network.
        let changelog = manager_for(&home).fetch_with(true, CHANGELOG_BASE);
        assert_eq!(
            changelog.markdown.as_deref(),
            Some("# seeded offline md\n"),
            "offline mode must return seeded markdown"
        );
        let entries = changelog.entries.expect("seeded json entries");
        assert_eq!(entries.len(), 1);
        let [entry] = entries.as_slice() else {
            panic!("expected exactly one entry, got {}", entries.len());
        };
        assert_eq!(entry.description, "seeded entry");
    }

    #[test]
    fn cdn_miss_falls_back_to_env_home_disk_cache() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path().join("grok-home-fallback");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(home.join("CHANGELOG.md"), "# fallback md\n").unwrap();

        // Non-offline path with an unreachable CDN base: the remote fetch fails deterministically, so the on-disk cache must win
        // The failure does not depend on whether the sandbox blocks network
        let changelog = manager_for(&home).fetch_with(false, "http://127.0.0.1:1");
        assert_eq!(
            changelog.markdown.as_deref(),
            Some("# fallback md\n"),
            "CDN miss must fall back to the seeded CHANGELOG.md"
        );
    }

    #[test]
    fn bullets_strips_markdown_and_respects_max() {
        let entries = vec![
            ChangelogEntry {
                category: "features".into(),
                description: "Added **dark mode** support".into(),
                breaking_change: false,
            },
            ChangelogEntry {
                category: "fixes".into(),
                description: "Fixed `crash` on startup".into(),
                breaking_change: false,
            },
            ChangelogEntry {
                category: "performance".into(),
                description: "Faster **rendering** of `code` blocks".into(),
                breaking_change: false,
            },
        ];

        let bullets = bullets_from_entries(&entries, 2);
        assert_eq!(
            bullets.as_slice(),
            ["Added dark mode support", "Fixed crash on startup"]
        );
    }

    #[test]
    fn bullets_skips_empty_descriptions() {
        let entries = vec![
            ChangelogEntry {
                category: "features".into(),
                description: "Good entry".into(),
                breaking_change: false,
            },
            ChangelogEntry {
                category: String::new(),
                description: String::new(), // bad entry from tolerant deserialization
                breaking_change: false,
            },
            ChangelogEntry {
                category: "fixes".into(),
                description: "Another good one".into(),
                breaking_change: false,
            },
        ];
        let bullets = bullets_from_entries(&entries, 10);
        assert_eq!(bullets, vec!["Good entry", "Another good one"]);
    }

    #[test]
    fn tolerant_deserialization_partial_entry() {
        // A missing description field defaults to an empty string, not a parse error
        let json = r#"[{"category":"features"},{"description":"ok"}]"#;
        let entries: Vec<ChangelogEntry> = serde_json::from_str(json).unwrap();
        let [first, second] = entries.as_slice() else {
            panic!("expected two entries: {entries:?}");
        };
        assert_eq!(first.description, "");
        assert_eq!(second.category, "");
        assert_eq!(second.description, "ok");
    }
}
