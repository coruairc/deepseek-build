//! Local session listing, enriched with repository/worktree metadata.
//!
//! Used by both the ACP `deepseek-build/session/list` handler and the `grok sessions` CLI command.
//! Filters local results by query.
//! Sorts by the same key the picker UI displays (`last_active_at` falling back to `updated_at`) descending.

use serde::Serialize;
use std::cmp::Reverse;

use crate::session::persistence::{Summary, list_summaries};
use xai_grok_workspace::session::git::normalize_repo_url;

/// Over-fetch factor: headroom for local session scans before pagination.
pub(crate) fn over_fetch(limit: usize) -> usize {
    (limit * 3).max(100)
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MergedSession {
    pub session_id: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_prompt: Option<String>,
    pub updated_at: String,
    pub created_at: String,
    pub cwd: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    /// Session data source; currently always "local".
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(default)]
    pub num_messages: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_active_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_root_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub git_remotes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_workspace_dir: Option<String>,
    /// Per-turn dashboard summary from `summary.json` (local sessions only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_turn_summary: Option<String>,
    /// Latest session recap from `summary.json` (local sessions only).
    /// Distinct from `last_turn_summary`; shown on `/resume` / `/session-info`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_recap: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_kind: Option<String>,
}

use crate::session::visibility::HeadlessPolicy;

/// Local session data used by the merged listing.
pub(crate) struct SessionLanes {
    pub local: Vec<Summary>,
    pub repo_urls: Vec<String>,
    /// The visibility policy dropped a local row proven relevant to this cwd/repo, so an empty page must not widen past it.
    pub rows_dropped_by_policy: bool,
}

/// Which directories a cwd-scoped listing draws from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CwdScope {
    /// The requested directory only.
    Only,
    /// Sibling worktrees of the same repo.
    #[default]
    WithSiblings,
    /// `WithSiblings`, widening past the cwd when it holds no messaged session.
    RelaxIfEmpty,
}

/// Spellings a session may be stored under, since clients supply their own path: as given and canonicalized, each without a trailing separator.
pub(crate) fn cwd_match_keys(cwd: &str) -> Vec<String> {
    let trimmed = cwd.trim_end_matches('/');
    let mut keys = vec![trimmed.to_owned()];
    if let Ok(real) = dunce::canonicalize(trimmed) {
        let real = real.to_string_lossy().trim_end_matches('/').to_owned();
        if keys.first().is_none_or(|k| k != &real) {
            keys.push(real);
        }
    }
    keys
}

/// Fetch local sessions and return a sorted list.
pub async fn fetch_merged(
    cwd: Option<&str>,
    scope: CwdScope,
    query: Option<&str>,
    limit: usize,
    headless: HeadlessPolicy,
) -> Vec<MergedSession> {
    let SessionLanes { local, .. } = fetch_lanes(cwd, scope, query, headless).await;
    merge(local, query, limit)
}

/// Retain summaries matching `repo_urls`; empty `repo_urls` leaves them unfiltered.
pub(crate) fn filter_summaries_by_repo(
    summaries: Vec<Summary>,
    repo_urls: &[String],
) -> Vec<Summary> {
    if repo_urls.is_empty() {
        return summaries;
    }
    summaries
        .into_iter()
        .filter(|s| {
            s.git_remotes
                .iter()
                .any(|u| normalize_repo_url(u).is_some_and(|n| repo_urls.contains(&n)))
        })
        .collect()
}

/// Fetch local summaries and repository context concurrently for `cwd`.
pub(crate) async fn fetch_lanes(
    cwd: Option<&str>,
    scope: CwdScope,
    query: Option<&str>,
    headless: HeadlessPolicy,
) -> SessionLanes {
    let cwd_owned = cwd.map(String::from);
    // `merge` truncates before any caller-side filter runs.
    let exact_keys = match (scope, cwd_owned.as_deref()) {
        (CwdScope::Only, Some(c)) => cwd_match_keys(c),
        _ => Vec::new(),
    };

    let local_fut = async {
        // Aggregate sessions from worktree sibling CWDs when possible
        let cwds = if let Some(ref c) = cwd_owned {
            match scope {
                CwdScope::Only => exact_keys.clone(),
                CwdScope::WithSiblings | CwdScope::RelaxIfEmpty => {
                    crate::session::worktree::candidate_worktree_cwds_for_same_repo(
                        std::path::Path::new(c),
                    )
                    .unwrap_or_else(|_| vec![c.clone()])
                }
            }
        } else {
            vec![]
        };
        let mut all = Vec::new();
        if cwds.is_empty() {
            // No CWD or worktree lookup failed: list all
            if let Ok(v) = list_summaries(cwd_owned.as_deref()).await {
                all.extend(v);
            }
        } else {
            for c in &cwds {
                if let Ok(v) = list_summaries(Some(c)).await {
                    all.extend(v);
                }
            }
        }
        all
    };

    let repo_urls_fut = async {
        if matches!(scope, CwdScope::Only) {
            return Vec::new();
        }
        cwd.map(|c| {
            xai_grok_workspace::session::git::resolve_normalized_remote_urls(std::path::Path::new(
                c,
            ))
        })
        .unwrap_or_default()
    };

    let (mut local, repo_urls) = tokio::join!(local_fut, repo_urls_fut);
    // Every narrowing happens before `merge`, which truncates, and before the caller paginates
    // A row dropped later would leave a hole in a sized page
    if matches!(scope, CwdScope::Only) {
        local.retain(|s| std::path::Path::new(&s.info.cwd).is_absolute());
    }
    // `grok --resume <uuid>` resolves across every cwd
    // Promote an exact UUID hit from any local directory into the lane before merge filters
    if let Some(id) = query
        .map(str::trim)
        .filter(|q| uuid::Uuid::try_parse(q).is_ok())
        && !local.iter().any(|s| s.info.id.0.as_ref() == id)
    {
        let id = id.to_string();
        if let Ok(Some(summary)) = tokio::task::spawn_blocking(move || {
            crate::session::persistence::find_summary_by_session_id(&id)
        })
        .await
        {
            local.push(summary);
        }
    }
    // This runs after the uuid promotion so a pasted headless id still obeys the page's policy
    let rows_dropped_by_policy =
        crate::session::visibility::retain_local_sessions(&mut local, headless);
    SessionLanes {
        local,
        repo_urls,
        rows_dropped_by_policy,
    }
}

/// Local results are optionally filtered by `query` (case-insensitive substring on summary, display title, and session ID).
/// Results are sorted by [`effective_sort_time`] descending and truncated to `limit`.
pub fn merge(local: Vec<Summary>, query: Option<&str>, limit: usize) -> Vec<MergedSession> {
    let mut merged = Vec::with_capacity(local.len());
    let query_lower = query.map(|q| q.to_lowercase());
    for s in local {
        if let Some(ref q) = query_lower
            && !s.session_summary.to_lowercase().contains(q.as_str())
            && !s.info.id.to_string().to_lowercase().contains(q.as_str())
            && !s.display_title().to_lowercase().contains(q.as_str())
        {
            continue;
        }
        let id = s.info.id.to_string();
        let display_summary = s.display_title().to_owned();
        let repo_name = s
            .git_root_dir
            .as_deref()
            .and_then(|p| std::path::Path::new(p).file_name())
            .and_then(|n| n.to_str())
            .map(String::from);
        merged.push(MergedSession {
            session_id: id,
            summary: display_summary,
            first_prompt: None,
            updated_at: s.updated_at.to_rfc3339(),
            created_at: s.created_at.to_rfc3339(),
            cwd: s.info.cwd,
            hostname: None,
            source: "local".to_string(),
            model_id: Some(s.current_model_id.to_string()),
            num_messages: s.num_messages,
            last_active_at: s.last_active_at.map(|t| t.to_rfc3339()),
            branch: s.head_branch,
            repo_name,
            worktree_label: s.worktree_label,
            git_root_dir: s.git_root_dir,
            git_remotes: s.git_remotes,
            source_workspace_dir: s.source_workspace_dir,
            last_turn_summary: s.last_turn_summary,
            last_recap: s.last_recap,
            session_kind: s.session_kind,
        });
    }

    // Sort newest-first by the same key the picker UI shows, so the visible "time ago" column is monotonic with the list order
    // `sort_by_cached_key` parses each timestamp once instead of on every comparison
    // Sessions with an unparseable timestamp sort to the bottom; equal times tie-break on `session_id` ascending
    merged.sort_by_cached_key(|s| (Reverse(effective_sort_time(s)), s.session_id.clone()));
    // Dedup empty sessions BEFORE truncating so the final list has `limit` entries.
    dedup_empty_sessions(&mut merged);
    merged.truncate(limit);
    merged
}

/// Mirrors the key the session picker UI displays (`last_active_at`, falling back to `updated_at`) so the "time ago" column matches the sort order.
/// Like the UI (`session_picker.rs`), an unparseable `last_active_at` counts as absent.
/// `None` means neither timestamp parses; that entry sorts to the bottom.
fn effective_sort_time(s: &MergedSession) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    s.last_active_at
        .as_deref()
        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
        .or_else(|| chrono::DateTime::parse_from_rfc3339(&s.updated_at).ok())
}

/// Drop unused optimistic-home husks (TUI open, never sent). Named empties
/// (`/rename` before send) and explicit worktree/fork sessions stay visible.
fn dedup_empty_sessions(sessions: &mut Vec<MergedSession>) {
    sessions.retain(|s| !is_unnamed_empty_session(s));
}

fn is_unnamed_empty_session(s: &MergedSession) -> bool {
    if matches!(s.session_kind.as_deref(), Some("worktree" | "fork"))
        || s.worktree_label
            .as_deref()
            .is_some_and(|label| !label.is_empty())
    {
        return false;
    }
    s.num_messages == 0 && s.summary.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::info::Info;
    use agent_client_protocol as acp;
    use chrono::{TimeZone, Utc};

    fn first<T: std::fmt::Debug>(xs: &[T]) -> &T {
        let Some(x) = xs.first() else {
            panic!("expected non-empty: {xs:?}");
        };
        x
    }

    fn at<T: std::fmt::Debug>(xs: &[T], i: usize) -> &T {
        let Some(x) = xs.get(i) else {
            panic!("expected index {i}: {xs:?}");
        };
        x
    }

    fn make_summary(id: &str, title: &str, updated: &str) -> Summary {
        Summary {
            info: Info {
                id: acp::SessionId::new(id),
                cwd: "/test".into(),
            },
            agent_id: None,
            attempt_id: None,
            cwd_generation: 0,
            previous_cwd: None,
            pending_cwd_switch_reminder: None,
            cwd_switch_bookkeeping_generation: 0,
            session_summary: title.into(),
            created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            updated_at: updated.parse().unwrap(),
            num_messages: 10,
            num_chat_messages: 5,
            current_model_id: acp::ModelId::new("test-model"),
            parent_session_id: None,
            forked_at: None,
            collection_id: None,
            next_trace_turn: 0,
            chat_format_version: 1,
            prompt_display_cwd: None,
            session_kind: None,
            fork_context_source: None,
            fork_parent_prompt_id: None,
            inherited_prefix_len: None,
            hidden: None,
            source_workspace_dir: None,
            git_root_dir: None,
            git_remotes: Vec::new(),
            head_commit: None,
            head_branch: None,
            request_id: None,
            grok_home: None,
            last_active_at: None,
            generated_title: None,
            title_is_manual: false,
            worktree_label: None,
            agent: Default::default(),
            sandbox_profile: None,
            reasoning_effort: None,
            context_window: None,
            last_turn_summary: None,
            last_turn_summary_prompt_id: None,
            last_recap: None,
        }
    }

    #[test]
    fn cwd_keys_ignore_a_trailing_separator() {
        assert_eq!(cwd_match_keys("/Users/me/xai/"), ["/Users/me/xai"]);
    }

    #[test]
    fn local_metadata_is_preserved_in_listing() {
        let local = vec![Summary {
            git_remotes: vec!["git@github.com:example/repo.git".into()],
            source_workspace_dir: Some("/home/user/src".into()),
            session_kind: Some("worktree".into()),
            ..make_summary_with_metadata(
                "s1",
                "local",
                "2026-03-01T00:00:00Z",
                None,
                Some("feature/branch"),
                Some("/home/user/repo"),
                Some("my-label"),
            )
        }];
        let merged = merge(local, None, 20);
        assert_eq!(merged.len(), 1);
        assert_eq!(first(&merged).source, "local");
        assert_eq!(first(&merged).summary, "local");
        assert_eq!(first(&merged).branch.as_deref(), Some("feature/branch"));
        assert_eq!(first(&merged).repo_name.as_deref(), Some("repo"));
        assert_eq!(first(&merged).worktree_label.as_deref(), Some("my-label"));
        // Local git metadata drives repo grouping and worktree labels.
        assert_eq!(
            first(&merged).git_root_dir.as_deref(),
            Some("/home/user/repo")
        );
        assert_eq!(
            first(&merged).git_remotes,
            vec!["git@github.com:example/repo.git"]
        );
        assert_eq!(
            first(&merged).source_workspace_dir.as_deref(),
            Some("/home/user/src")
        );
        assert_eq!(first(&merged).session_kind.as_deref(), Some("worktree"));
    }

    #[test]
    fn local_only_sessions_included() {
        let local = vec![make_summary("s1", "only on disk", "2026-03-01T00:00:00Z")];
        let merged = merge(local, None, 20);
        assert_eq!(merged.len(), 1);
        assert_eq!(first(&merged).source, "local");
    }

    /// `last_turn_summary` is carried into the session list from local `summary.json`.
    #[test]
    fn last_turn_summary_carried_from_local_summary() {
        let mut s = make_summary("s1", "title", "2026-03-01T00:00:00Z");
        s.last_turn_summary = Some("Fixed the parser".into());
        let merged = merge(vec![s], None, 20);
        assert_eq!(
            first(&merged).last_turn_summary.as_deref(),
            Some("Fixed the parser")
        );
    }

    #[test]
    fn last_recap_carried_from_local_summary() {
        let mut s = make_summary("s1", "title", "2026-03-01T00:00:00Z");
        s.last_recap = Some("Where we left off: auth refactor".into());
        let merged = merge(vec![s], None, 20);
        assert_eq!(
            first(&merged).last_recap.as_deref(),
            Some("Where we left off: auth refactor")
        );
    }

    #[test]
    fn sorted_by_updated_at_descending() {
        let local = vec![
            make_summary("old", "old", "2026-01-01T00:00:00Z"),
            make_summary("new", "new", "2026-04-01T00:00:00Z"),
            make_summary("mid", "mid", "2026-02-01T00:00:00Z"),
        ];
        let merged = merge(local, None, 20);
        assert_eq!(first(&merged).session_id, "new");
        assert_eq!(at(&merged, 1).session_id, "mid");
        assert_eq!(at(&merged, 2).session_id, "old");
    }

    #[test]
    fn sorted_by_last_active_at_over_updated_at() {
        // The picker displays `last_active_at` (falling back to `updated_at`), and the sort must match that key
        // This guards against the regression where `updated_at`-only sorting made the visible "time ago" column look unordered
        let local = vec![
            make_summary_with_last_active(
                "stale_activity",
                "stale",
                "2026-04-01T00:00:00Z", // newer updated_at (e.g. metadata bump)
                Some("2026-01-01T00:00:00Z"), // older real activity
            ),
            make_summary_with_last_active(
                "recent_activity",
                "recent",
                "2026-02-01T00:00:00Z",       // older updated_at
                Some("2026-05-01T00:00:00Z"), // newer real activity
            ),
        ];
        let merged = merge(local, None, 20);
        assert_eq!(first(&merged).session_id, "recent_activity");
        assert_eq!(at(&merged, 1).session_id, "stale_activity");
    }

    #[test]
    fn sort_falls_back_to_updated_at_when_last_active_absent() {
        // When `last_active_at` is None, ordering uses `updated_at`, matching the UI fallback
        let local = vec![
            make_summary_with_last_active("a", "a", "2026-01-01T00:00:00Z", None),
            make_summary_with_last_active(
                "b",
                "b",
                "2026-01-01T00:00:00Z",
                Some("2026-06-01T00:00:00Z"),
            ),
            make_summary_with_last_active("c", "c", "2026-03-01T00:00:00Z", None),
        ];
        let merged = merge(local, None, 20);
        // b: last_active 2026-06 (newest), c: updated 2026-03, a: updated 2026-01.
        assert_eq!(first(&merged).session_id, "b");
        assert_eq!(at(&merged, 1).session_id, "c");
        assert_eq!(at(&merged, 2).session_id, "a");
    }

    #[test]
    fn truncated_to_limit() {
        let local: Vec<Summary> = (0..10)
            .map(|i| make_summary(&format!("s{i}"), "title", "2026-01-01T00:00:00Z"))
            .collect();
        let merged = merge(local, None, 3);
        assert_eq!(merged.len(), 3);
    }

    #[test]
    fn search_matches_session_id_substring() {
        let id = "019f870d-6976-7d73-a12a-52e9d4aebcd4";
        let local = vec![
            make_summary(id, "unrelated title", "2026-03-01T00:00:00Z"),
            make_summary(
                "019f9999-0000-7000-8000-000000000001",
                "other",
                "2026-03-01T00:00:00Z",
            ),
        ];
        let merged = merge(local, Some(id), 20);
        assert_eq!(merged.len(), 1);
        assert_eq!(first(&merged).session_id, id);

        let prefix = merge(
            vec![make_summary(id, "unrelated title", "2026-03-01T00:00:00Z")],
            Some("019f870d"),
            20,
        );
        assert_eq!(prefix.len(), 1);
        assert_eq!(first(&prefix).session_id, id);
    }

    #[test]
    fn search_filters_local_case_insensitive() {
        let local = vec![
            make_summary("s1", "Fix Kubernetes deployment", "2026-03-01T00:00:00Z"),
            make_summary("s2", "Unrelated session", "2026-03-01T00:00:00Z"),
            make_summary("s3", "KUBERNETES cluster issue", "2026-03-01T00:00:00Z"),
        ];
        let merged = merge(local, Some("kubernetes"), 20);
        assert_eq!(merged.len(), 2);
        assert!(merged.iter().all(|r| r.session_id != "s2"));
    }

    // ── local last_active_at ordering tests ──────────────────────────────

    fn make_summary_with_last_active(
        id: &str,
        title: &str,
        updated: &str,
        last_active: Option<&str>,
    ) -> Summary {
        Summary {
            last_active_at: last_active.map(|s| s.parse().unwrap()),
            ..make_summary(id, title, updated)
        }
    }

    #[test]
    fn last_active_at_is_preserved_from_local_summary() {
        let local = vec![make_summary_with_last_active(
            "s1",
            "local",
            "2026-03-01T00:00:00Z",
            Some("2026-04-10T12:00:00Z"),
        )];
        let merged = merge(local, None, 20);
        assert_eq!(merged.len(), 1);
        assert_eq!(
            first(&merged).last_active_at.as_deref(),
            Some("2026-04-10T12:00:00+00:00")
        );
    }

    // ── generated_title / branch / repo_name / worktree_label merge tests ──

    fn make_summary_with_metadata(
        id: &str,
        session_summary: &str,
        updated: &str,
        generated_title: Option<&str>,
        head_branch: Option<&str>,
        git_root_dir: Option<&str>,
        worktree_label: Option<&str>,
    ) -> Summary {
        Summary {
            generated_title: generated_title.map(String::from),
            head_branch: head_branch.map(String::from),
            git_root_dir: git_root_dir.map(String::from),
            worktree_label: worktree_label.map(String::from),
            ..make_summary(id, session_summary, updated)
        }
    }

    #[test]
    fn generated_title_preferred_over_session_summary() {
        let local = vec![make_summary_with_metadata(
            "s1",
            "hi",
            "2026-03-01T00:00:00Z",
            Some("Refactor auth middleware"),
            None,
            None,
            None,
        )];
        let merged = merge(local, None, 20);
        assert_eq!(merged.len(), 1);
        assert_eq!(first(&merged).summary, "Refactor auth middleware");
    }

    #[test]
    fn session_summary_used_when_generated_title_absent() {
        let local = vec![make_summary_with_metadata(
            "s1",
            "Fix deployment bug",
            "2026-03-01T00:00:00Z",
            None,
            None,
            None,
            None,
        )];
        let merged = merge(local, None, 20);
        assert_eq!(first(&merged).summary, "Fix deployment bug");
    }

    #[test]
    fn empty_generated_title_falls_back_to_session_summary() {
        let local = vec![make_summary_with_metadata(
            "s1",
            "fallback summary",
            "2026-03-01T00:00:00Z",
            Some(""),
            None,
            None,
            None,
        )];
        let merged = merge(local, None, 20);
        assert_eq!(first(&merged).summary, "fallback summary");
    }

    #[test]
    fn branch_populated_from_head_branch() {
        let local = vec![make_summary_with_metadata(
            "s1",
            "summary",
            "2026-03-01T00:00:00Z",
            None,
            Some("feature/auth-refactor"),
            None,
            None,
        )];
        let merged = merge(local, None, 20);
        assert_eq!(
            first(&merged).branch.as_deref(),
            Some("feature/auth-refactor")
        );
    }

    #[test]
    fn repo_name_extracted_from_git_root_dir() {
        let local = vec![make_summary_with_metadata(
            "s1",
            "summary",
            "2026-03-01T00:00:00Z",
            None,
            None,
            Some("/home/user/projects/myrepo"),
            None,
        )];
        let merged = merge(local, None, 20);
        assert_eq!(first(&merged).repo_name.as_deref(), Some("myrepo"));
    }

    #[test]
    fn repo_name_handles_trailing_slash() {
        let local = vec![make_summary_with_metadata(
            "s1",
            "summary",
            "2026-03-01T00:00:00Z",
            None,
            None,
            Some("/home/user/repo/"),
            None,
        )];
        let merged = merge(local, None, 20);
        // On Unix, Path::file_name("/x/y/") returns Some("y")
        assert_eq!(first(&merged).repo_name.as_deref(), Some("repo"));
    }

    #[test]
    fn repo_name_none_when_git_root_not_set() {
        let local = vec![make_summary_with_metadata(
            "s1",
            "summary",
            "2026-03-01T00:00:00Z",
            None,
            None,
            None,
            None,
        )];
        let merged = merge(local, None, 20);
        assert!(first(&merged).repo_name.is_none());
    }

    #[test]
    fn worktree_label_surfaced_in_merged_session() {
        let local = vec![make_summary_with_metadata(
            "s1",
            "summary",
            "2026-03-01T00:00:00Z",
            None,
            None,
            None,
            Some("nuke-v-tables"),
        )];
        let merged = merge(local, None, 20);
        assert_eq!(
            first(&merged).worktree_label.as_deref(),
            Some("nuke-v-tables")
        );
    }

    #[test]
    fn all_metadata_fields_populated_together() {
        let local = vec![make_summary_with_metadata(
            "s1",
            "old summary",
            "2026-03-01T00:00:00Z",
            Some("Implement retry logic"),
            Some("feature/retry"),
            Some("/home/dev/xai"),
            Some("retry-feature"),
        )];
        let merged = merge(local, None, 20);
        assert_eq!(first(&merged).summary, "Implement retry logic");
        assert_eq!(first(&merged).branch.as_deref(), Some("feature/retry"));
        assert_eq!(first(&merged).repo_name.as_deref(), Some("xai"));
        assert_eq!(
            first(&merged).worktree_label.as_deref(),
            Some("retry-feature")
        );
    }

    #[test]
    fn query_filters_on_session_summary_display_uses_generated_title() {
        let local = vec![
            make_summary_with_metadata(
                "s1",
                "hi",
                "2026-03-01T00:00:00Z",
                Some("Kubernetes deployment fix"),
                None,
                None,
                None,
            ),
            make_summary_with_metadata(
                "s2",
                "unrelated session",
                "2026-03-01T00:00:00Z",
                Some("Database migration"),
                None,
                None,
                None,
            ),
        ];
        // Query filters on display_title (which prefers generated_title), session_summary, and session_id
        let merged = merge(local, Some("hi"), 20);
        assert_eq!(merged.len(), 1);
        assert_eq!(first(&merged).session_id, "s1");
        assert_eq!(first(&merged).summary, "Kubernetes deployment fix");
    }

    #[test]
    fn query_matches_generated_title() {
        let local = vec![make_summary_with_metadata(
            "s1",
            "plain summary",
            "2026-03-01T00:00:00Z",
            Some("Kubernetes deployment fix"),
            None,
            None,
            None,
        )];
        // "kubernetes" appears in generated_title, so the query matches via display_title()
        let merged = merge(local, Some("kubernetes"), 20);
        assert_eq!(merged.len(), 1);
        assert_eq!(first(&merged).summary, "Kubernetes deployment fix");
    }

    // ── dedup_empty_sessions tests ──────────────────────────────────────

    fn make_merged(id: &str, cwd: &str, updated: &str, num_messages: usize) -> MergedSession {
        MergedSession {
            session_id: id.into(),
            summary: String::new(),
            first_prompt: None,
            updated_at: updated.into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            cwd: cwd.into(),
            hostname: None,
            source: "local".into(),
            model_id: None,
            num_messages,
            last_active_at: None,
            branch: None,
            repo_name: None,
            worktree_label: None,
            git_root_dir: None,
            git_remotes: Vec::new(),
            source_workspace_dir: None,
            last_turn_summary: None,
            last_recap: None,
            session_kind: None,
        }
    }

    #[test]
    fn dedup_empty_drops_unnamed_empty_sessions() {
        let mut sessions = vec![
            make_merged("newest", "/repo", "2026-04-01T00:00:00Z", 0),
            make_merged("middle", "/repo", "2026-03-01T00:00:00Z", 0),
            make_merged("oldest", "/repo", "2026-02-01T00:00:00Z", 0),
        ];
        dedup_empty_sessions(&mut sessions);
        assert!(sessions.is_empty());
    }

    #[test]
    fn dedup_empty_preserves_nonempty_and_drops_unnamed_empty() {
        let mut sessions = vec![
            make_merged("nonempty", "/repo", "2026-04-01T00:00:00Z", 5),
            make_merged("empty", "/repo", "2026-03-01T00:00:00Z", 0),
        ];
        dedup_empty_sessions(&mut sessions);
        assert_eq!(sessions.len(), 1);
        assert_eq!(first(&sessions).session_id, "nonempty");
    }

    #[test]
    fn dedup_empty_keeps_unnamed_empty_worktree() {
        let mut wt = make_merged("wt", "/repo", "2026-04-01T00:00:00Z", 0);
        wt.session_kind = Some("worktree".into());
        wt.worktree_label = Some("fix-bug".into());
        let mut sessions = vec![wt, make_merged("home", "/repo", "2026-03-01T00:00:00Z", 0)];
        dedup_empty_sessions(&mut sessions);
        assert_eq!(sessions.len(), 1);
        assert_eq!(first(&sessions).session_id, "wt");
    }

    #[test]
    fn dedup_empty_keeps_named_empty_session() {
        let mut named = make_merged("renamed", "/repo", "2026-04-01T00:00:00Z", 0);
        named.summary = "my title".into();
        let mut sessions = vec![
            named,
            make_merged("blank", "/repo", "2026-03-01T00:00:00Z", 0),
        ];
        dedup_empty_sessions(&mut sessions);
        assert_eq!(sessions.len(), 1);
        assert_eq!(first(&sessions).session_id, "renamed");
    }

    #[test]
    fn dedup_empty_different_cwds_drops_unnamed() {
        let mut sessions = vec![
            make_merged("e1", "/repo-a", "2026-04-01T00:00:00Z", 0),
            make_merged("e2", "/repo-b", "2026-03-01T00:00:00Z", 0),
        ];
        dedup_empty_sessions(&mut sessions);
        assert!(sessions.is_empty());
    }

    #[test]
    fn dedup_empty_noop_on_empty_input() {
        let mut v = vec![];
        dedup_empty_sessions(&mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn dedup_empty_multi_cwd_keeps_only_nonempty() {
        let mut sessions = vec![
            make_merged("a-nonempty", "/repo-a", "2026-04-03T00:00:00Z", 3),
            make_merged("a-empty1", "/repo-a", "2026-04-02T00:00:00Z", 0),
            make_merged("a-empty2", "/repo-a", "2026-04-01T00:00:00Z", 0),
            make_merged("b-empty1", "/repo-b", "2026-03-03T00:00:00Z", 0),
            make_merged("b-nonempty", "/repo-b", "2026-03-02T00:00:00Z", 7),
            make_merged("b-empty2", "/repo-b", "2026-03-01T00:00:00Z", 0),
        ];
        dedup_empty_sessions(&mut sessions);
        assert!(sessions.iter().any(|s| s.session_id == "a-nonempty"));
        assert!(sessions.iter().any(|s| s.session_id == "b-nonempty"));
        assert!(
            !sessions
                .iter()
                .any(|s| s.session_id.ends_with("empty1") || s.session_id.ends_with("empty2"))
        );
        assert_eq!(sessions.len(), 2);
    }

    // ── limit applied after merge tests ─────────────────────────────────

    #[test]
    fn limit_applied_after_local_scan() {
        // Apply the user-facing limit after collecting local sessions.
        let local: Vec<Summary> = (0..5)
            .map(|i| {
                make_summary(
                    &format!("local-{i}"),
                    &format!("local {i}"),
                    &format!("2026-01-{:02}T00:00:00Z", i + 1),
                )
            })
            .collect();
        let merged = merge(local, None, 4);
        assert_eq!(merged.len(), 4);
    }

    #[test]
    fn limit_preserves_sessions_from_multiple_cwds() {
        // Even though each cwd has fewer sessions than the limit, the total across cwds may exceed it; the limit is applied to the merged set
        let mut local = Vec::new();
        for cwd_idx in 0..3 {
            for sess_idx in 0..3 {
                let mut s = make_summary(
                    &format!("s-{cwd_idx}-{sess_idx}"),
                    &format!("session {cwd_idx}-{sess_idx}"),
                    &format!("2026-03-{:02}T{:02}:00:00Z", cwd_idx + 1, sess_idx + 1),
                );
                s.info.cwd = format!("/repo/cwd-{cwd_idx}");
                local.push(s);
            }
        }
        // 9 local sessions across 3 cwds, limit 5
        let merged = merge(local, None, 5);
        assert_eq!(merged.len(), 5);
    }
}
