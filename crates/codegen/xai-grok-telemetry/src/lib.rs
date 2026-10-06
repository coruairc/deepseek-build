#![allow(
    unused_imports,
    unused_variables,
    unused_mut,
    unreachable_code,
    dead_code
)]
//! Telemetry engine for deepseek-build sessions.
//! Covers local structured logging and the typed event surface.
//!
//! Extracted from `xai-file-utils` so telemetry has its own ownership boundary (see CODEOWNERS).
//! Consumers that only want event tracking and inference metrics no longer pull in Mixpanel/HTTP/identity dependencies.
#![deny(clippy::indexing_slicing)]
pub mod client;
pub mod config;
pub mod context;
pub mod enums;
pub mod events;
pub mod http;
pub mod id;
mod logs;
mod process;
mod session;
mod spans;
pub use client::{
    Metadata, TelemetryClient, UserContext, init, init_if_needed, is_enabled,
    is_session_metrics_enabled,
};
pub use events::TelemetryEvent;
pub(crate) use logs::appender;
pub use logs::{debug_log, hooks_log, memory_log, sampling_log, unified_log};
pub use process::{memory_telemetry, process_info, process_metrics};
pub use session::session_ctx::{
    EmitterOrigin, TelemetryCtx, emit_event, emit_event_with_origin, log_event, log_session_event,
    log_session_event_with_origin, spawn_local_in_session_ctx, with_session_ctx,
};
pub use session::{activity, session_ctx, session_end, session_metrics, subagent_spawn};
pub use spans::{instrumentation, prompt_timing, region, span_profile, startup, turn_phases};
pub mod redact_common;
pub use redact_common::redact_error_detail;
