//! Workspace session-directory janitor (formerly the upload-queue restart-recovery scan).

use std::path::Path;
use std::time::Duration;

/// Zero-init this module's metric families. See [`crate::init_metrics`].
pub(crate) fn init_metrics() {}

pub const DEFAULT_SESSION_MAX_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// Remove `<workspace_home>/sessions/` dirs whose mtime is older than `max_age`, bounding growth of a long-lived workspace.
/// Mtime advances on atomic-rename persistence, a close-enough last-activity signal. Errors are swallowed so the janitor never fails boot; stray files and future mtimes stay.
pub async fn cleanup_stale_sessions(workspace_home: &Path, max_age: Duration) {
    let sessions_dir = workspace_home.join("sessions");
    let Ok(mut entries) = tokio::fs::read_dir(&sessions_dir).await else {
        return; // No sessions dir yet (first boot).
    };

    let mut removed = 0u32;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let Ok(ft) = entry.file_type().await else {
            continue;
        };
        if !ft.is_dir() {
            continue;
        }
        let path = entry.path();
        if let Ok(metadata) = tokio::fs::metadata(&path).await
            && let Ok(modified) = metadata.modified()
            && let Ok(age) = modified.elapsed()
            && age > max_age
        {
            match tokio::fs::remove_dir_all(&path).await {
                Ok(()) => {
                    removed += 1;
                    tracing::info!(
                        path = %path.display(),
                        age_secs = age.as_secs(),
                        "cleanup_stale_sessions: removed stale session dir"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        path = %path.display(),
                        error = %e,
                        "cleanup_stale_sessions: failed to remove stale session dir"
                    );
                }
            }
        }
    }

    if removed > 0 {
        tracing::info!(
            removed,
            "cleanup_stale_sessions: stale session-dir sweep complete"
        );
    }
}
