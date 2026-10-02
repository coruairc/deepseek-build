//! No-op remnant of the opt-in external OTEL stream.
//!
//! The real stream (customer-collector OTLP exporters, redaction pipeline,
//! schema mapping, and provider lifecycle) was deleted. This module preserves
//! the public call surface as inert stubs so startup/login call sites still
//! compile. No exporter, provider, socket, or thread is ever constructed.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub mod truncate;

/// Identity *attributes* (plain id strings, never tokens).
#[derive(Debug, Clone, Default)]
pub struct IdentityAttrs {
    pub user_id: Option<String>,
    pub email: Option<String>,
    pub organization_id: Option<String>,
    pub team_id: Option<String>,
    pub deployment_id: Option<String>,
}

impl IdentityAttrs {
    pub fn from_snapshot(snapshot: &xai_grok_auth::CredentialSnapshot) -> Self {
        Self {
            user_id: snapshot.user_id.clone(),
            email: None,
            organization_id: snapshot.organization_id.clone(),
            team_id: snapshot.team_id.clone(),
            deployment_id: snapshot.deployment_id.clone(),
        }
    }
}

/// Remote-settings policy for the (now inert) external stream.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExternalOtelRemotePolicy {
    pub force_disable: bool,
    pub lock_content_gates: bool,
}

/// Snapshot of external-stream export-health counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExportHealthSnapshot {
    pub records_dropped: u64,
    pub metric_exports_dropped: u64,
    pub export_failures: u64,
    pub export_successes: u64,
}

static SETTINGS_RESOLVED: AtomicBool = AtomicBool::new(true);
static GATE_MAX_WAIT_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(30_000);

/// No-op: the external stream is gone.
pub fn init() {}

/// Set the bound on the fail-closed window.
pub fn set_settings_gate_max_wait(max_wait: Duration) {
    GATE_MAX_WAIT_MS.store(
        u64::try_from(max_wait.as_millis()).unwrap_or(u64::MAX),
        Ordering::Relaxed,
    );
}

/// The current bound on the fail-closed window.
pub fn settings_gate_max_wait() -> Duration {
    Duration::from_millis(GATE_MAX_WAIT_MS.load(Ordering::Relaxed))
}

/// Close the gate.
pub fn suppress_external_otel_until_settings() {
    SETTINGS_RESOLVED.store(false, Ordering::Release);
}

/// Open the gate.
pub fn mark_external_otel_settings_resolved() {
    SETTINGS_RESOLVED.store(true, Ordering::Release);
}

#[inline]
pub fn is_settings_gate_open() -> bool {
    SETTINGS_RESOLVED.load(Ordering::Acquire)
}

/// The external stream is always inactive.
pub fn is_active() -> bool {
    false
}

/// No-op fan-out.
pub fn emit<T: crate::events::TelemetryEvent>(_data: &T) {}

/// No-op: identity no longer reaches any exporter.
pub fn set_identity(_attrs: IdentityAttrs) {}

/// No-op.
pub fn apply_remote_policy(_policy: ExternalOtelRemotePolicy) {}

/// No-op.
pub fn flush() {}

/// No-op.
pub fn shutdown() {}

/// The stream was never activated.
pub fn export_health() -> Option<ExportHealthSnapshot> {
    None
}
