//! `/agent` switch and picker dispatch tests.

use super::{test_agent_mut, test_app_with_agent};
use crate::app::actions::{Action, Effect, TaskResult};
use crate::app::agent::AgentState;
use crate::app::app_view::ActiveView;
use crate::app::dispatch::dispatch;
use crate::scrollback::block::RenderBlock;
use crate::views::agents_modal::AgentsModalMode;

fn isolated_agent(active: Option<&str>) -> crate::app::app_view::AppView {
    let mut app = test_app_with_agent();
    let cwd = tempfile::tempdir().expect("tempdir");
    let agent = test_agent_mut(&mut app, crate::app::agent::AgentId(0));
    // Keep discovery hermetic: no project agents from the real cwd.
    agent.session.cwd = cwd.path().to_path_buf();
    agent.session_agent_name = active.map(str::to_owned);
    app
}

fn scrollback_has_system_text(app: &crate::app::app_view::AppView, needle: &str) -> bool {
    let Some(agent) = app.agents.get(&crate::app::agent::AgentId(0)) else {
        return false;
    };
    (0..agent.scrollback.len()).any(|i| {
        matches!(
            &agent.scrollback.get(i).unwrap().block,
            RenderBlock::System(block) if block.text.contains(needle)
        )
    })
}

#[test]
fn picker_opens_on_the_active_agent() {
    let mut app = isolated_agent(Some("explore"));
    let effects = dispatch(Action::OpenAgentPicker, &mut app);
    assert!(
        effects
            .iter()
            .all(|effect| matches!(effect, Effect::FetchSessionAgentName { .. })),
        "opening the picker only refreshes the agent name, got {effects:?}"
    );
    let agent = app.agents.get(&crate::app::agent::AgentId(0)).unwrap();
    let modal = agent.agents_modal.as_ref().expect("picker opened");
    assert_eq!(modal.mode, AgentsModalMode::Switch);
    assert_eq!(modal.active_agent.as_deref(), Some("explore"));
    assert_eq!(
        modal.agents.get(modal.selected).map(|e| e.name.as_str()),
        Some("explore"),
        "cursor starts on the active agent"
    );
}

#[test]
fn switch_to_a_named_agent_requests_an_in_process_swap() {
    let mut app = isolated_agent(Some("default"));
    let effects = dispatch(Action::SwitchAgent("explore".to_string()), &mut app);
    assert_eq!(effects.len(), 1, "one request, got {effects:?}");
    match &effects[0] {
        Effect::SwitchSessionAgent {
            agent_id,
            session_id,
            name,
        } => {
            assert_eq!(*agent_id, crate::app::agent::AgentId(0));
            assert_eq!(session_id.0.to_string(), "test-session");
            assert_eq!(name, "explore");
        }
        other => panic!("expected SwitchSessionAgent, got {other:?}"),
    }
    assert!(
        app.pending_agent_switch.is_none(),
        "the relaunch is only the fallback"
    );
    assert!(app.agent_relaunch.is_none(), "event loop arms the relaunch");
}

#[test]
fn successful_in_place_swap_updates_identity_and_says_so() {
    let mut app = isolated_agent(Some("default"));
    let effects = dispatch(
        Action::TaskComplete(TaskResult::SessionAgentSwitched {
            agent_id: crate::app::agent::AgentId(0),
            name: "explore".to_string(),
        }),
        &mut app,
    );
    assert!(effects.is_empty());
    let agent = app.agents.get(&crate::app::agent::AgentId(0)).unwrap();
    assert_eq!(agent.session_agent_name.as_deref(), Some("explore"));
    assert!(
        scrollback_has_system_text(&app, "in place"),
        "the notice records the in-process path"
    );
    assert_eq!(
        app.agent_override,
        Some(serde_json::json!("explore")),
        "a later load keeps the chosen agent, like --agent does"
    );
    assert!(app.pending_agent_switch.is_none());
}

#[test]
fn failed_in_place_swap_falls_back_to_the_relaunch() {
    let mut app = isolated_agent(Some("default"));
    let effects = dispatch(
        Action::TaskComplete(TaskResult::SessionAgentSwitchFailed {
            agent_id: crate::app::agent::AgentId(0),
            session_id: agent_client_protocol::SessionId::new("test-session"),
            name: "explore".to_string(),
            error: "this shell cannot switch agents in place".to_string(),
            fallback: true,
        }),
        &mut app,
    );
    assert!(effects.is_empty());
    let request = app
        .pending_agent_switch
        .as_ref()
        .expect("fallback relaunch queued");
    assert_eq!(request.agent, "explore");
    assert_eq!(request.session_id, "test-session");
    assert!(
        scrollback_has_system_text(&app, "Couldn't switch in place"),
        "the notice records the fallback path"
    );
}

#[test]
fn deliberate_refusal_does_not_fall_back_to_the_relaunch() {
    let mut app = isolated_agent(Some("default"));
    let effects = dispatch(
        Action::TaskComplete(TaskResult::SessionAgentSwitchFailed {
            agent_id: crate::app::agent::AgentId(0),
            session_id: agent_client_protocol::SessionId::new("test-session"),
            name: "explore".to_string(),
            error: "Finish or cancel the running turn before switching agents.".to_string(),
            fallback: false,
        }),
        &mut app,
    );
    assert!(effects.is_empty());
    assert!(
        app.pending_agent_switch.is_none(),
        "a refusal must not race the turn with a relaunch"
    );
    assert!(scrollback_has_system_text(
        &app,
        "Finish or cancel the running turn"
    ));
}

#[test]
fn switch_refused_while_a_turn_runs() {
    let mut app = isolated_agent(Some("default"));
    test_agent_mut(&mut app, crate::app::agent::AgentId(0))
        .session
        .state = AgentState::TurnRunning;
    let effects = dispatch(Action::SwitchAgent("explore".to_string()), &mut app);
    assert!(effects.is_empty());
    assert!(app.pending_agent_switch.is_none(), "no relaunch requested");
    assert!(app.agent_relaunch.is_none());
    assert!(scrollback_has_system_text(
        &app,
        "Finish or cancel the running turn"
    ));
}

#[test]
fn picker_refused_while_a_turn_cancels() {
    let mut app = isolated_agent(Some("default"));
    test_agent_mut(&mut app, crate::app::agent::AgentId(0))
        .session
        .state = AgentState::TurnCancelling;
    let effects = dispatch(Action::OpenAgentPicker, &mut app);
    assert!(effects.is_empty());
    assert!(
        app.agents
            .get(&crate::app::agent::AgentId(0))
            .is_some_and(|agent| agent.agents_modal.is_none()),
        "the picker must not open while a cancel is in flight"
    );
    assert!(scrollback_has_system_text(
        &app,
        "Finish or cancel the running turn"
    ));
}

#[test]
fn unknown_agent_lists_valid_names() {
    let mut app = isolated_agent(Some("default"));
    let effects = dispatch(Action::SwitchAgent("nope".to_string()), &mut app);
    assert!(effects.is_empty());
    assert!(app.pending_agent_switch.is_none());
    assert!(scrollback_has_system_text(
        &app,
        "Unknown agent 'nope'. Available:"
    ));
    assert!(
        scrollback_has_system_text(&app, "explore"),
        "the error lists the discovered agents"
    );
}

#[test]
fn switch_requires_an_agent_session() {
    let mut app = test_app_with_agent();
    app.active_view = ActiveView::Welcome;
    let effects = dispatch(Action::SwitchAgent("explore".to_string()), &mut app);
    assert!(effects.is_empty());
    assert!(app.pending_agent_switch.is_none());
}
