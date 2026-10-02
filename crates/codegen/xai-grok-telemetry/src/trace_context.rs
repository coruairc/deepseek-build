//! W3C trace-context propagation helpers.
//!
//! The former OpenTelemetry-backed implementation lived in the deleted
//! `xai-grok-otel` crate. These are local, dependency-free no-ops that keep the
//! historical call surface compiling after the OTLP stack was removed; no span
//! context is minted or propagated.

use reqwest::header::HeaderMap;

pub fn link_span_to_current(_span: &tracing::Span) {}

pub fn current_traceparent() -> Option<String> {
    None
}

pub fn span_traceparent(_span: &tracing::Span) -> Option<String> {
    None
}

pub fn traceparent_of_span(_span: &tracing::Span) -> Option<String> {
    None
}

pub fn inject_trace_context_into_request(
    builder: reqwest::RequestBuilder,
) -> reqwest::RequestBuilder {
    builder
}

pub fn trace_context_headers() -> HeaderMap {
    HeaderMap::new()
}

pub fn inject_trace_context(_headers: &mut HeaderMap) {}

pub fn span_from_meta_traceparent(
    _meta: &serde_json::Map<String, serde_json::Value>,
) -> tracing::Span {
    tracing::info_span!("acp_dispatch")
}

pub fn link_span_to_meta(_span: &tracing::Span, _meta: &serde_json::Value) -> bool {
    false
}

pub fn link_current_span_to_meta(_meta: &serde_json::Value) {}

/// OTel-parent `span` under `traceparent` with no tracing parent link.
/// Retained as a no-op after the OTLP stack removal; always returns `false`.
pub fn set_parent_from_traceparent(_span: &tracing::Span, _traceparent: &str) -> bool {
    false
}

/// A held local trace setup. Retained as a no-op guard so callers can keep
/// installing it scoped to a thread.
#[must_use]
pub struct LocalTraceGuard;

/// Install a [`LocalTraceGuard`]. No-op after the OTLP stack removal.
pub fn set_local_trace_subscriber() -> LocalTraceGuard {
    LocalTraceGuard
}
