use super::rate_limit_backoff_tests::{SessionKind, actor_under_test};
use super::transient_retry_loop_tests::{on_session_stack, run_paused, sampler_surfaces_5xx};
use super::*;
use std::time::Duration;
use tracing::Instrument;
use xai_grok_test_support::{MockInferenceServer, MockModelEntry};

fn trace_id(traceparent: &str) -> &str {
    traceparent
        .split('-')
        .nth(1)
        .expect("traceparent has a trace id")
}
