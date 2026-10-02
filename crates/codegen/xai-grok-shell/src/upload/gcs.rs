//! Inert shell-side adapter left after the object-storage upload stack was
//! deleted. It no longer constructs a `StorageClient` or performs any network
//! I/O; it only carries the config wrapper the remaining types name.
use std::sync::Arc;

use xai_grok_login::AuthManager;

use crate::file_utils_compat::{TraceExportConfig, UploadMethod};

/// Wrapper pairing a trace-export config with an optional auth manager.
/// Retained for source compatibility; it performs no uploads.
#[derive(Clone)]
pub(crate) struct TraceExportConfigWithAuth {
    inner: TraceExportConfig,
    #[allow(dead_code)]
    auth_manager: Option<Arc<AuthManager>>,
}

impl TraceExportConfigWithAuth {
    pub(crate) fn new(inner: TraceExportConfig, auth_manager: Option<Arc<AuthManager>>) -> Self {
        Self {
            inner,
            auth_manager,
        }
    }
}

/// Convenience trait for wrapping a `TraceExportConfig` at (former) upload call sites.
pub(crate) trait WithAuth {
    fn with_auth(&self, auth_manager: Option<Arc<AuthManager>>) -> TraceExportConfigWithAuth;
}

impl WithAuth for TraceExportConfig {
    fn with_auth(&self, auth_manager: Option<Arc<AuthManager>>) -> TraceExportConfigWithAuth {
        TraceExportConfigWithAuth::new(self.clone(), auth_manager)
    }
}

/// Override at runtime with `GROK_TELEMETRY_GCS_BUCKET`; no uploads occur regardless.
pub(crate) const SESSION_TRACES_BUCKET: Option<&str> =
    option_env!("GROK_SESSION_TRACES_BUCKET_DEFAULT");

/// Formerly uploaded `auth-diagnostics/...`; now a no-op.
pub(crate) async fn upload_to_auth_diagnostics(
    _log_bytes: &[u8],
    _user_id: &str,
    _upload_method: &UploadMethod,
    _auth_manager: Arc<xai_grok_login::AuthManager>,
) {
}
