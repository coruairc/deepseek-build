//! Network-free compatibility shim for the former `xai-file-utils` crate.
//!
//! The real object-storage / upload-queue implementation (GCS, S3, storage
//! HTTP, spill-to-disk worker) has been deleted. This module keeps the small
//! surface the shell still names so the upload plumbing can be unwound without
//! a flag day. Every method here is inert: nothing is written to disk and no
//! network request is ever made.
#![allow(dead_code)]

use std::path::Path;

pub use crate::session::repo_changes::{
    ARCHIVE_SCHEMA_VERSION, ARCHIVE_SCHEMA_VERSION_V3, BlobCompression, DEDUP_BLOB_SUBDIR,
    DEDUP_GCS_PREFIX, DEDUP_PATCH_SUBDIR, DedupMetadata, ExcludedContent, FileReference,
    PatchReference, SKIP_DIR_NAMES, TraceExportConfig, UploadMethod, skip_dir_set,
};

/// Compute SHA256 of content as a lowercase hex string.
pub fn sha256_hex(content: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

/// Compute SHA256 of a file, streaming; mirrors the deleted helper.
pub fn sha256_hex_from_file(path: &Path, max_bytes: Option<u64>) -> std::io::Result<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let file = std::fs::File::open(path)?;
    let mut reader: Box<dyn Read> = if let Some(limit) = max_bytes {
        Box::new(file.take(limit))
    } else {
        Box::new(file)
    };
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        if let Some(chunk) = buffer.get(..n) {
            hasher.update(chunk);
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub mod workspace_classifier {
    use std::path::Path;

    const EXCLUDED_DIR_NAMES: &[&str] = &[
        ".grok", ".cache", ".daemon", ".config", ".npm", ".cargo", ".rustup", ".vscode", ".gemini",
        ".hermes", ".claude",
    ];

    pub fn is_project_dir(cwd: &Path) -> bool {
        if cwd.as_os_str().is_empty() || cwd.parent().is_none() {
            return false;
        }
        if cwd.ancestors().any(|p| p.join(".git").exists()) {
            return true;
        }
        for component in cwd.components() {
            if let std::path::Component::Normal(name) = component {
                let name_lower = name.to_string_lossy().to_lowercase();
                if EXCLUDED_DIR_NAMES.contains(&name_lower.as_str())
                    || name_lower.starts_with(".grok-")
                {
                    return false;
                }
            }
        }
        if let Some(home) = xai_dirs::home_dir()
            && cwd == home
        {
            return false;
        }
        true
    }
}

pub mod storage_client {
    use std::sync::Arc;

    /// Inert stand-in for the deleted object-storage HTTP client.
    #[derive(Clone, Debug, Default)]
    pub struct StorageClient;

    impl StorageClient {
        pub fn with_provider<T, P>(_proxy_base_url: &str, _http: T, _provider: P) -> Self {
            Self
        }
        pub fn with_client_identity<T, U>(self, _version: T, _identifier: U) -> Self {
            self
        }
        pub fn with_client_mode<T>(self, _mode: T) -> Self {
            self
        }
        pub fn with_attribution<T>(self, _attribution: T) -> Self {
            self
        }
    }

    /// 401-attribution callback boundary; kept so call sites still name it.
    pub trait Auth401AttributionCallback: Send + Sync + std::fmt::Debug {
        fn record_401(&self, operation: &str, sent_bearer_prefix: Option<&str>);
    }

    /// Error type named by the upload pipeline's error downcasts.
    #[derive(Debug)]
    pub struct HttpUploadError {
        pub status_code: Option<u16>,
    }

    impl std::fmt::Display for HttpUploadError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "storage upload error")
        }
    }

    impl std::error::Error for HttpUploadError {}

    /// Retry configuration; inert.
    #[derive(Clone, Debug, Default)]
    pub struct RetryConfig;
}

pub mod gcs {
    /// Always-empty: no storage destination is configured.
    pub async fn upload_bytes<T>(
        _config: &T,
        _object_path: &str,
        _content: &[u8],
        _content_type: &str,
    ) -> anyhow::Result<String> {
        anyhow::bail!("object storage upload removed")
    }

    pub async fn upload_file<T, P>(
        _config: &T,
        _object_name: &str,
        _path: P,
        _content_type: &str,
    ) -> anyhow::Result<String> {
        anyhow::bail!("object storage upload removed")
    }

    pub async fn upload_bytes_signed<T>(
        _config: &T,
        _object_path: &str,
        _content: &[u8],
        _content_type: &str,
    ) -> anyhow::Result<String> {
        anyhow::bail!("object storage upload removed")
    }

    pub const MULTIPART_UPLOAD_THRESHOLD: u64 = u64::MAX;
}

pub mod queue {
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::time::Duration;

    use super::TraceExportConfig;

    pub const DEFAULT_MAX_AGE: Duration = Duration::from_secs(2 * 60 * 60);

    #[derive(Clone, Debug, Default)]
    pub struct UploadRetryPolicy;

    #[derive(Debug)]
    pub struct QueueClosed;

    impl std::fmt::Display for QueueClosed {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "upload queue closed")
        }
    }

    impl std::error::Error for QueueClosed {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum EnqueueOutcome {
        Enqueued,
        FellBackToInline,
        Failed { reason: String },
        Deduplicated,
        Skipped { reason: String },
    }

    #[derive(Debug)]
    pub struct UploadQueueStats {
        pub enqueued: AtomicU64,
        pub uploaded: AtomicU64,
        pub failed: AtomicU64,
        pub enqueue_fallbacks: AtomicU64,
        pub circuit_breaker_trips: AtomicU64,
        pub pending: AtomicU64,
        pub pending_bytes: AtomicU64,
        pub inflight: AtomicU64,
        pub circuit_breaker_active: AtomicBool,
    }

    impl Default for UploadQueueStats {
        fn default() -> Self {
            Self {
                enqueued: AtomicU64::new(0),
                uploaded: AtomicU64::new(0),
                failed: AtomicU64::new(0),
                enqueue_fallbacks: AtomicU64::new(0),
                circuit_breaker_trips: AtomicU64::new(0),
                pending: AtomicU64::new(0),
                pending_bytes: AtomicU64::new(0),
                inflight: AtomicU64::new(0),
                circuit_breaker_active: AtomicBool::new(false),
            }
        }
    }

    impl UploadQueueStats {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn set_transition_notify<T>(&self, _notify: T) {}

        pub fn pending(&self) -> u64 {
            self.pending.load(Ordering::Relaxed)
        }
    }

    pub trait TraceExportSource: Send + Sync {
        fn resolve(&self) -> TraceExportConfig;
        fn resolve_async(
            &self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = TraceExportConfig> + Send + '_>>
        {
            Box::pin(std::future::ready(self.resolve()))
        }
        fn proxy_attribution(
            &self,
        ) -> Option<Arc<dyn super::storage_client::Auth401AttributionCallback>> {
            None
        }
        fn proxy_credentials(&self) -> Option<Arc<dyn xai_grok_auth::AuthCredentialProvider>> {
            None
        }
        fn proxy_http_client(&self) -> Option<reqwest::Client> {
            None
        }
        fn wait_for_auth_recovery(
            &self,
            _failed_bearer: Option<&str>,
            _timeout: Duration,
        ) -> Option<std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>>>
        {
            None
        }
        fn has_usable_credential(&self) -> bool {
            true
        }
    }

    /// Inert upload queue: accepts nothing and writes nothing.
    #[derive(Clone)]
    pub struct UploadQueue {
        stats: Arc<UploadQueueStats>,
    }

    impl UploadQueue {
        pub fn spawn(
            _grok_home: &Path,
            _resolver: Arc<dyn TraceExportSource>,
            _retry_policy: UploadRetryPolicy,
        ) -> Self {
            Self {
                stats: Arc::new(UploadQueueStats::new()),
            }
        }

        pub fn with_client_version(self, _version: impl Into<String>) -> Self {
            self
        }

        pub fn with_max_queue_bytes(self, _max_bytes: u64) -> Self {
            self
        }

        pub async fn enqueue_bytes_blocking(
            &self,
            _content: &[u8],
            _gcs_path: &str,
            _content_type: &str,
            _artifact_name: &str,
            _session_id: &str,
            _turn_number: u64,
        ) -> EnqueueOutcome {
            EnqueueOutcome::Failed {
                reason: "object storage upload removed".to_string(),
            }
        }

        pub async fn enqueue_blocking(
            &self,
            _content: &[u8],
            _gcs_path: &str,
            _content_type: &str,
            _artifact_name: &str,
            _session_id: &str,
            _turn_number: u64,
            _diverted_inline: Option<&std::sync::atomic::AtomicBool>,
        ) -> anyhow::Result<String> {
            anyhow::bail!("object storage upload removed")
        }

        pub async fn enqueue(
            &self,
            _content: &[u8],
            _gcs_path: &str,
            _content_type: &str,
            _artifact_name: &str,
            _session_id: &str,
            _turn_number: u64,
        ) -> anyhow::Result<()> {
            anyhow::bail!("object storage upload removed")
        }

        pub async fn wait_idle(&self, _timeout: Duration) -> usize {
            0
        }

        pub async fn drain(&self, _deadline: Duration) -> usize {
            0
        }

        pub fn stats(&self) -> &UploadQueueStats {
            &self.stats
        }

        pub fn stats_arc(&self) -> Arc<UploadQueueStats> {
            self.stats.clone()
        }

        pub fn cleanup_orphans(&self, _max_age: Duration) {}
    }

    pub fn last_orphans_cleaned() -> u64 {
        0
    }

    pub fn cleanup_orphaned_uploads(_grok_home: &Path, _max_age: Duration) -> u64 {
        0
    }
}
