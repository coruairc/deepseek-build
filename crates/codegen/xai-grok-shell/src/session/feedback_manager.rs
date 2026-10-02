//! Inert replacement for the deleted feedback manager.
//!
//! The feedback/analytics backend client, heuristic sampling, and upload-queue
//! coupling were removed with the exfiltration stack. This module keeps only
//! the session-signal handle and a no-op submission surface so the session
//! actor can be unwound without a flag day. No feedback is sent.

use std::sync::Arc;
use std::time::Duration;

use prod_mc_cli_chat_proxy_types::feedback_types::{
    ClientType, FeedbackContent, FeedbackMode, FeedbackSubmission,
};

use crate::session::feedback::{FeedbackRequest, FeedbackTier};
use crate::session::signals::{SessionSignalsActor, SessionSignalsHandle};

pub(crate) enum SubmitOutcome {
    Submitted,
    LocalOnly,
    Failed(anyhow::Error),
}

/// Build a feedback submission envelope; local only.
pub(crate) fn new_submission(
    session_id: String,
    client_type: ClientType,
    content: FeedbackContent,
) -> FeedbackSubmission {
    let mut s = FeedbackSubmission::with_content(session_id, client_type, content);
    s.shell_version = Some(xai_grok_version::VERSION.to_string());
    s
}

#[derive(Debug)]
pub(crate) struct SubmitFeedbackOptions {
    pub solicited: bool,
    pub telemetry_enabled: bool,
    pub author_identity: Option<crate::util::user_identity::ResolvedUserIdentity>,
}

/// Chat-state fields the session actor passes to `submit_text_feedback`.
pub(crate) struct SessionFeedbackData {
    pub model_id: Option<String>,
    pub resolved_model_id: Option<String>,
    pub reasoning_effort: Option<String>,
    pub client_version: Option<String>,
    pub session_cwd: String,
}

/// Feedback feature flags threaded through session spawn.
#[derive(Debug, Clone, Default)]
pub(crate) struct FeedbackFlags {
    pub enabled: bool,
    pub user: Option<crate::agent::config::FeedbackUserConfig>,
}

#[derive(Clone)]
pub struct FeedbackManagerConfig {
    pub sync_interval: Duration,
    pub feedback_enabled: bool,
    pub telemetry_enabled: bool,
    pub client_type: ClientType,
    pub loc_tracking_enabled: bool,
    pub drain_timeout: Duration,
    pub user: Option<crate::agent::config::FeedbackUserConfig>,
}

impl Default for FeedbackManagerConfig {
    fn default() -> Self {
        Self {
            sync_interval: Duration::from_secs(60),
            feedback_enabled: false,
            telemetry_enabled: false,
            client_type: ClientType::Agent,
            loc_tracking_enabled: false,
            drain_timeout: Duration::from_secs(30),
            user: None,
        }
    }
}

/// Inert per-session feedback manager: owns only the signal handle.
pub struct FeedbackManager {
    session_id: String,
    signals_handle: SessionSignalsHandle,
    config: FeedbackManagerConfig,
}

impl FeedbackManager {
    pub fn new(
        session_id: impl Into<String>,
        _feedback_client: Option<()>,
        config: FeedbackManagerConfig,
    ) -> Self {
        let (signals_handle, actor) = SessionSignalsActor::with_sync_interval(config.sync_interval);
        tokio::spawn(actor.run());
        Self {
            session_id: session_id.into(),
            signals_handle,
            config,
        }
    }

    pub fn local_only(session_id: impl Into<String>) -> Self {
        Self::new(session_id, None, FeedbackManagerConfig::default())
    }

    pub(crate) fn set_upload_queue_stats<T>(&self, _stats: T) {}

    pub fn signals_handle(&self) -> SessionSignalsHandle {
        self.signals_handle.clone()
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn is_enabled(&self) -> bool {
        self.config.feedback_enabled
    }

    pub fn client_type(&self) -> ClientType {
        self.config.client_type
    }

    pub(crate) async fn submit_text_feedback(
        &self,
        _text: String,
        _session_data: SessionFeedbackData,
        _persistence_tx: Option<
            &tokio::sync::mpsc::UnboundedSender<crate::session::persistence::PersistenceMsg>,
        >,
        _telemetry_enabled: bool,
    ) -> SubmitOutcome {
        SubmitOutcome::LocalOnly
    }

    pub(crate) async fn maybe_request_feedback(
        &self,
        _request_id: Option<String>,
    ) -> Option<FeedbackRequest> {
        None
    }

    pub(crate) async fn force_feedback_request(
        &self,
        tier: FeedbackTier,
        mode: FeedbackMode,
    ) -> FeedbackRequest {
        use crate::session::feedback::{TriggerCondition, TriggerSignalSnapshot};
        let condition = TriggerCondition {
            tier,
            condition: "feedback requests removed".to_string(),
            signal_snapshot: TriggerSignalSnapshot {
                turn_count: 0,
                tool_calls_count: 0,
                compactions_count: 0,
                errors_count: 0,
                cancellations_count: 0,
                has_reverted: false,
            },
        };
        FeedbackRequest::with_mode(self.session_id.clone(), condition, mode, true, None)
    }

    pub(crate) async fn send_turn_delta_with_snapshot(
        &self,
        _snapshot: Option<&crate::session::signals::TurnDeltaSnapshot>,
        _request_id: Option<String>,
        _duration_ms: Option<i64>,
        _outcome: prod_mc_cli_chat_proxy_types::feedback_types::TurnOutcome,
    ) {
    }

    pub(crate) async fn run_sync_loop(&self, _cancel: tokio_util::sync::CancellationToken) {}

    pub async fn shutdown<T>(&self, _queue: Option<T>) {}

    pub(crate) fn shared(self: &Arc<Self>) -> Arc<Self> {
        self.clone()
    }
}
