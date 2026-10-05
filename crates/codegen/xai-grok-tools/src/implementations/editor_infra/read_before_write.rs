//! Runtime read-before-write enforcement.
//!
//! `SearchReplaceParams::skip_read_before_edit` and the `requires_expr` Read-tool
//! check are only config-time invariants. This module adds the runtime half: a
//! per-session tracker of files the model has actually read, consulted before a
//! mutating edit tool writes to an existing file.
//!
//! The tracker is an ephemeral `Resources` value (never serialized), so it is
//! scoped to the live toolset. `WorkspaceSession::replace_carrying_browser_service`
//! carries it across toolset rebuilds so a reload does not force spurious re-reads.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::types::resources::{Resources, SharedResources};

/// Files read by the model during this session, keyed by canonical path.
#[derive(Debug, Default, Clone)]
pub struct ReadBeforeWriteTracker {
    read: HashSet<PathBuf>,
}

impl ReadBeforeWriteTracker {
    /// Record a canonical path as read.
    pub fn record(&mut self, canonical: PathBuf) {
        self.read.insert(canonical);
    }

    /// Whether `canonical` has been read this session.
    pub fn was_read(&self, canonical: &Path) -> bool {
        self.read.contains(canonical)
    }

    /// Number of tracked files (test/diagnostic helper).
    pub fn len(&self) -> usize {
        self.read.len()
    }

    /// Whether no files have been tracked.
    pub fn is_empty(&self) -> bool {
        self.read.is_empty()
    }
}

/// Canonicalize `path` for tracker keys, falling back to the raw path when it
/// does not exist yet or cannot be resolved.
pub(crate) async fn canonicalize_for_tracking(path: &Path) -> PathBuf {
    crate::util::fs::try_canonicalize(path)
        .await
        .unwrap_or_else(|_| path.to_path_buf())
}

/// Record that the model read `path` via a read tool.
pub async fn record_read(resources: &SharedResources, path: &Path) {
    let canonical = canonicalize_for_tracking(path).await;
    let mut res = resources.lock().await;
    res.get_or_default::<ReadBeforeWriteTracker>()
        .record(canonical);
}

/// Why a mutating write is permitted without a prior read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bypass {
    /// The target does not exist yet, so there is nothing to read first.
    NewFile,
    /// The target is the session plan file, written through the plan-file
    /// auto-approve path in plan mode.
    PlanFile,
}

/// Decide whether `path` is exempt from read-before-write, independent of the
/// tracker. `is_new_file` must reflect the on-disk state at check time.
pub async fn bypass_reason(
    resources: &SharedResources,
    path: &Path,
    is_new_file: bool,
) -> Option<Bypass> {
    if is_new_file {
        return Some(Bypass::NewFile);
    }
    let plan_target = {
        let res = resources.lock().await;
        crate::types::resources::resolve_plan_file_path(&res).0
    };
    if let Some(plan) = plan_target {
        let plan_canonical = canonicalize_for_tracking(&plan).await;
        let path_canonical = canonicalize_for_tracking(path).await;
        if plan_canonical == path_canonical {
            return Some(Bypass::PlanFile);
        }
    }
    None
}

/// Enforce read-before-write for `path`. Returns the actionable denial message
/// when the model must read the file first, or `None` when the write is allowed.
pub async fn read_before_write_denial(
    resources: &SharedResources,
    path: &Path,
    display_path: &str,
    is_new_file: bool,
) -> Option<String> {
    if bypass_reason(resources, path, is_new_file).await.is_some() {
        return None;
    }
    // Memory-v2 files have their own optimistic-concurrency guard and are not
    // read through the ordinary read tool, so keep them exempt here.
    if crate::types::memory_v2::validate_memory_v2_read(resources, path)
        .await
        .unwrap_or(false)
    {
        return None;
    }
    let canonical = canonicalize_for_tracking(path).await;
    let was_read = {
        let res = resources.lock().await;
        res.get::<ReadBeforeWriteTracker>()
            .is_some_and(|tracker| tracker.was_read(&canonical))
    };
    if was_read {
        return None;
    }
    Some(format!(
        "Error: you must read {display_path} before editing it. \
         Use the read tool to view the current contents, then retry this edit."
    ))
}

/// Carry the tracker from an outgoing toolset's resources into a rebuilt one.
/// A no-op when there is nothing to carry.
pub async fn carry_tracker(from: &SharedResources, to: &SharedResources) {
    let tracker = {
        let from = from.lock().await;
        from.get::<ReadBeforeWriteTracker>().cloned()
    };
    if let Some(tracker) = tracker {
        let mut to = to.lock().await;
        to.insert(tracker);
    }
}

/// Seed a tracker directly (tests / session restore).
pub fn seed(resources: &mut Resources, paths: impl IntoIterator<Item = PathBuf>) {
    let mut tracker = ReadBeforeWriteTracker::default();
    for path in paths {
        tracker.record(path);
    }
    resources.insert(tracker);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::resources::{Cwd, PlanFilePath};

    fn shared(cwd: &Path) -> SharedResources {
        let mut resources = Resources::new();
        resources.insert(Cwd(cwd.to_path_buf()));
        resources.into_shared()
    }

    #[tokio::test]
    async fn new_file_is_always_allowed() {
        let tmp = tempfile::tempdir().unwrap();
        let resources = shared(tmp.path());
        let path = tmp.path().join("missing.txt");
        let denial = read_before_write_denial(&resources, &path, "missing.txt", true).await;
        assert!(denial.is_none());
    }

    #[tokio::test]
    async fn existing_unread_file_is_denied() {
        let tmp = tempfile::tempdir().unwrap();
        let resources = shared(tmp.path());
        let path = tmp.path().join("existing.txt");
        std::fs::write(&path, "hello\n").unwrap();
        let denial = read_before_write_denial(&resources, &path, "existing.txt", false).await;
        let msg = denial.expect("unread existing file must be denied");
        assert!(msg.contains("existing.txt"), "{msg}");
        assert!(msg.contains("read tool"), "{msg}");
    }

    #[tokio::test]
    async fn read_then_write_is_allowed() {
        let tmp = tempfile::tempdir().unwrap();
        let resources = shared(tmp.path());
        let path = tmp.path().join("existing.txt");
        std::fs::write(&path, "hello\n").unwrap();
        record_read(&resources, &path).await;
        let denial = read_before_write_denial(&resources, &path, "existing.txt", false).await;
        assert!(denial.is_none());
    }

    #[tokio::test]
    async fn plan_file_is_exempt() {
        let tmp = tempfile::tempdir().unwrap();
        let mut resources = Resources::new();
        resources.insert(Cwd(tmp.path().to_path_buf()));
        let plan = tmp.path().join(".grok/plan.md");
        resources.insert(PlanFilePath(plan.clone()));
        let resources = resources.into_shared();
        std::fs::create_dir_all(plan.parent().unwrap()).unwrap();
        std::fs::write(&plan, "# plan\n").unwrap();
        let denial = read_before_write_denial(&resources, &plan, ".grok/plan.md", false).await;
        assert!(denial.is_none());
    }

    #[tokio::test]
    async fn carry_tracker_preserves_reads() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("existing.txt");
        std::fs::write(&path, "hello\n").unwrap();
        let from = shared(tmp.path());
        record_read(&from, &path).await;
        let to = shared(tmp.path());
        carry_tracker(&from, &to).await;
        let denial = read_before_write_denial(&to, &path, "existing.txt", false).await;
        assert!(denial.is_none());
    }
}
