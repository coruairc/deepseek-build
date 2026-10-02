//! No-op remnant of the tracing-to-OTLP span layer.
//!
//! The real layer/provider/exporter stack lived in the deleted `xai-grok-otel`
//! crate. The types below keep the historical call surface compiling; the
//! layer installs nothing and no spans are exported.

use std::sync::Arc;
use std::time::Duration;

use tracing_subscriber::registry::LookupSpan;

pub struct OtelLayerConfig {
    pub credentials: Arc<dyn xai_grok_auth::AuthCredentialProvider>,
    pub token_header_value: String,
    pub alpha_test_key: Option<String>,
    pub exporter: OtelExporterConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct OtelClientInfo {
    pub client_name: &'static str,
    pub client_version: &'static str,
    pub service_version: &'static str,
    pub app_entrypoint: &'static str,
}

#[derive(Debug, Default, Clone)]
pub struct OtelExporterConfig {
    pub traces_url: String,
    pub extra_headers: Vec<(String, String)>,
    pub export_interval: Option<Duration>,
    pub timeout: Option<Duration>,
    pub enabled: bool,
}

/// An inert layer: installed so the tracing registry shape is unchanged.
pub struct NoopOtelLayer;

impl<S> tracing_subscriber::Layer<S> for NoopOtelLayer where S: tracing::Subscriber {}

pub fn build_otel_layer<S>(
    _client: OtelClientInfo,
    _config: OtelLayerConfig,
) -> impl tracing_subscriber::Layer<S>
where
    S: tracing::Subscriber + for<'span> LookupSpan<'span>,
{
    NoopOtelLayer
}

pub fn shutdown_otel() {
    crate::external::shutdown();
}

pub struct OtelGuard;

impl Drop for OtelGuard {
    fn drop(&mut self) {
        shutdown_otel();
    }
}

pub fn otel_guard() -> OtelGuard {
    OtelGuard
}
