//! Small turn/session metadata helpers that previously lived in the deleted
//! `upload` module. They are unrelated to uploads and are retained here.

/// Parse `_meta.agentProfile` as a JSON object or string name.
/// Returns `None` if absent or invalid.
pub(super) fn parse_agent_profile_from_meta(
    meta: Option<&agent_client_protocol::Meta>,
) -> Option<xai_grok_agent::AgentDefinition> {
    let value = meta?.get("agentProfile")?;
    if value.is_object() {
        return match xai_grok_agent::AgentDefinition::from_json(value) {
            Ok(def) => {
                tracing::info!(
                    agent_name = %def.name,
                    "Using ACP agent profile from _meta.agentProfile (JSON object)"
                );
                Some(def)
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    "Failed to parse _meta.agentProfile JSON object, falling back to default agent"
                );
                None
            }
        };
    }
    if let Some(name) = value.as_str() {
        tracing::info!(
            agent_name = %name,
            "Resolving agent from _meta.agentProfile (string name)"
        );
        return xai_grok_agent::discovery::by_name(name);
    }
    tracing::warn!(
        "Ignoring _meta.agentProfile: expected a JSON object or string, got {:?}",
        value
    );
    None
}

/// Parse `_meta.askUserQuestion` as a boolean. `Some(false)` means the pager set `--no-ask-user`.
/// The shell passes it to `AgentBuilder::with_ask_user_question_enabled(false)` so the tool is stripped from the model's advertised tool list. `Some(true)` explicitly enables the tool for this session.
/// `None` means the field is absent; the caller falls back to the `ask_user_question` feature (default ON).
pub(super) fn parse_ask_user_question_from_meta(
    meta: Option<&agent_client_protocol::Meta>,
) -> Option<bool> {
    let value = meta?.get("askUserQuestion")?;
    match value.as_bool() {
        Some(b) => Some(b),
        None => {
            tracing::warn!(
                "Ignoring _meta.askUserQuestion: expected a bool, got {:?}",
                value
            );
            None
        }
    }
}

pub(super) fn lookup_session_model(
    session_model: Option<agent_client_protocol::ModelId>,
    default_model_id: &agent_client_protocol::ModelId,
) -> agent_client_protocol::ModelId {
    session_model.unwrap_or_else(|| default_model_id.clone())
}

pub(super) fn apply_yolo_mode_to_matching_sessions<'a>(
    sessions: impl IntoIterator<Item = &'a mut crate::session::SessionHandle>,
    sender_id: Option<&str>,
    yolo_mode: bool,
) -> usize {
    let matches_sender = |h: &crate::session::SessionHandle| -> bool {
        sender_id.is_none() || h.origin_client.as_ref().map(|c| c.product.as_str()) == sender_id
    };
    let mut updated = 0;
    for handle in sessions {
        if matches_sender(handle) {
            handle.yolo_mode = yolo_mode;
            let _ = handle
                .cmd_tx
                .send(crate::session::SessionCommand::SetYoloMode { enabled: yolo_mode });
            updated += 1;
        }
    }
    updated
}
