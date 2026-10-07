use super::*;
use crate::capability::CapabilityMode;
use crate::config::{
    AgentSessionConfig, BindMcpConfig, DEFAULT_EVENT_BUFFER_CAPACITY, WorkspaceConfig,
};
use crate::error::WorkspaceError;
use crate::session::tool_config::resolve_session_toolset;
use crate::session::tool_config::test_support::{TestSessionContextFactory, baseline_config, tc};
use axum::response::IntoResponse;
use std::sync::Arc;
use xai_grok_tools::registry::types::ToolServerConfig;
use xai_grok_tools::types::tool::ToolKind;
use xai_grok_workspace_types::WorkspaceEvent;
use xai_tool_runtime::ToolCallContext;
/// Create a test workspace handle with a "main" session pre-created.
pub(crate) fn make_handle() -> WorkspaceHandle {
    make_handle_with_rewind_all_outcomes(false)
}
/// [`make_handle`] with `require_explicit_toolset` (strict sandbox mode).
pub(crate) fn make_strict_handle() -> WorkspaceHandle {
    make_handle_with_options(false, true)
}
/// [`make_handle`] with fs confinement on (mirrors a remote-sandbox server).
pub(crate) fn make_confining_handle() -> WorkspaceHandle {
    make_handle_inner(false, false, Default::default(), true)
}
/// [`make_handle`] with the tool-approval gate enforced, as a daemon host has it.
pub(crate) fn make_enforced_handle() -> WorkspaceHandle {
    make_handle_with_factory(
        Arc::new(TestSessionContextFactory::new()),
        false,
        false,
        Default::default(),
        false,
        crate::permission::ToolApprovalGate::Enforced,
    )
}
/// [`make_handle`] with an explicit `workspace_rewind_all_outcomes` value.
pub(crate) fn make_handle_with_rewind_all_outcomes(enabled: bool) -> WorkspaceHandle {
    make_handle_inner(enabled, false, Default::default(), false)
}
pub(crate) fn make_handle_with_options(
    rewind_all_outcomes: bool,
    require_explicit_toolset: bool,
) -> WorkspaceHandle {
    make_handle_inner(
        rewind_all_outcomes,
        require_explicit_toolset,
        Default::default(),
        false,
    )
}
/// [`make_handle`] with an explicit [`crate::StatusConfig`].
pub(crate) fn make_handle_with_status_config(
    status_config: crate::StatusConfig,
) -> WorkspaceHandle {
    make_handle_inner(false, false, status_config, false)
}
/// [`make_handle`], but with the empty `state_path` that real sessions get.
#[allow(dead_code)]
pub(crate) fn make_handle_without_tool_state() -> WorkspaceHandle {
    make_handle_with_factory(
        Arc::new(TestSessionContextFactory::without_tool_state()),
        false,
        false,
        Default::default(),
        false,
        crate::permission::ToolApprovalGate::Off,
    )
}
fn make_handle_inner(
    rewind_all_outcomes: bool,
    require_explicit_toolset: bool,
    status_config: crate::StatusConfig,
    confine_fs_to_workspace_root: bool,
) -> WorkspaceHandle {
    make_handle_with_factory(
        Arc::new(TestSessionContextFactory::new()),
        rewind_all_outcomes,
        require_explicit_toolset,
        status_config,
        confine_fs_to_workspace_root,
        crate::permission::ToolApprovalGate::Off,
    )
}
fn make_handle_with_factory(
    factory: Arc<TestSessionContextFactory>,
    rewind_all_outcomes: bool,
    require_explicit_toolset: bool,
    status_config: crate::StatusConfig,
    confine_fs_to_workspace_root: bool,
    tool_approval: crate::permission::ToolApprovalGate,
) -> WorkspaceHandle {
    let cwd = factory.temp.path().to_path_buf();
    let config = WorkspaceConfig {
        root_cwd: cwd,
        default_tool_config: baseline_config(),
        respect_gitignore: false,
        memory_config: None,
        event_buffer_capacity: DEFAULT_EVENT_BUFFER_CAPACITY,
        session_factory: factory,
        hook_global_sources: vec![],
        hook_project_sources: vec![],
        skills_config: Default::default(),
        plugin_discovery_config: Default::default(),
        server_metadata: None,
        status_config,
        project_lsp_trusted: true,
        require_explicit_toolset,
        confine_fs_to_workspace_root,
        bind_mcp: None,
        tool_approval,
        host_kind: Default::default(),
        sandbox: None,
    };
    let handle = WorkspaceHandle::build(
        config,
        ephemeral_workspace_home(),
        false,
        false,
        rewind_all_outcomes,
        false,
        crate::identity::WorkspaceIdentity::default(),
    )
    .expect("handle construction should succeed");
    handle
        .create_session("main")
        .expect("create main session should succeed");
    handle
}
pub(crate) const BASH_CCO_STUB_NAME: &str = "bash_cco_stub";
pub(crate) const BASH_CCO_STUB_STDOUT: &str = "cco-stdout";
#[derive(Debug)]
pub(crate) struct BashCcoStub;
impl xai_grok_tools::types::tool_metadata::ToolMetadata for BashCcoStub {
    fn kind(&self) -> ToolKind {
        ToolKind::Execute
    }
    fn tool_namespace(&self) -> xai_grok_tools::types::tool::ToolNamespace {
        xai_grok_tools::types::tool::ToolNamespace::MCP
    }
    fn description_template(&self) -> &str {
        "bash cco stub"
    }
}
impl xai_tool_runtime::Tool for BashCcoStub {
    type Args = serde_json::Value;
    type Output = xai_grok_tools::types::output::ToolOutput;
    fn id(&self) -> xai_tool_protocol::ToolId {
        xai_tool_protocol::ToolId::new(BASH_CCO_STUB_NAME).expect("valid tool id")
    }
    fn description(
        &self,
        _ctx: &::xai_tool_runtime::ListToolsContext,
    ) -> xai_tool_types::ToolDescription {
        xai_tool_types::ToolDescription::new(BASH_CCO_STUB_NAME, "bash cco stub")
    }
    async fn run(
        &self,
        _ctx: xai_tool_runtime::ToolCallContext,
        _input: serde_json::Value,
    ) -> Result<xai_grok_tools::types::output::ToolOutput, xai_tool_runtime::ToolError> {
        let output = BASH_CCO_STUB_STDOUT.as_bytes();
        Ok(xai_grok_tools::types::output::ToolOutput::Bash(
            xai_grok_tools::types::output::BashOutput {
                output: output.to_vec(),
                output_for_prompt:
                    xai_grok_tools::types::output::BashOutput::make_output_for_prompt(
                        BASH_CCO_STUB_STDOUT,
                    ),
                exit_code: 0,
                command: format!("echo {BASH_CCO_STUB_STDOUT}"),
                truncated: false,
                signal: None,
                timed_out: false,
                description: None,
                current_dir: "/tmp".into(),
                output_file: String::new(),
                total_bytes: output.len(),
                output_delta: None,
                was_bare_echo: false,
            },
        ))
    }
}
pub(crate) fn register_bash_cco_stub(handle: &WorkspaceHandle) {
    register_bash_cco_stub_on(handle, "main");
}
pub(crate) fn register_bash_cco_stub_on(handle: &WorkspaceHandle, session_id: &str) {
    let session = handle.session(session_id).expect("session present");
    session
        .toolset()
        .register_tool(
            BASH_CCO_STUB_NAME.to_owned(),
            BashCcoStub,
            Some(serde_json::json!({"type": "object", "properties": {}})),
        )
        .expect("register bash_cco_stub");
}
pub(crate) fn assert_bash_cco_terminal(typed: &xai_tool_runtime::TypedToolOutput) {
    use xai_tool_runtime::ToolOutput as _;
    let resp = typed
        .chat_completion_output()
        .expect("bash chat_completion_output must be preserved");
    let cer = resp
        .result
        .as_ref()
        .and_then(|r| r.code_execution_result.as_ref())
        .expect("code_execution_result");
    assert_eq!(cer.stdout, BASH_CCO_STUB_STDOUT);
    assert_eq!(cer.exit_code, 0);
    assert!(!cer.command_timed_out);
}
pub(crate) async fn drain_terminal_ok(
    mut stream: impl futures::Stream<
        Item = xai_tool_runtime::ToolStreamItem<xai_tool_runtime::TypedToolOutput>,
    > + Unpin,
) -> xai_tool_runtime::TypedToolOutput {
    use futures::StreamExt;
    use xai_tool_runtime::ToolStreamItem;
    while let Some(item) = stream.next().await {
        match item {
            ToolStreamItem::Terminal(Ok(t)) => return t,
            ToolStreamItem::Progress(_) => {}
            ToolStreamItem::Terminal(Err(e)) => {
                panic!("expected Terminal(Ok), got Err: {e}")
            }
        }
    }
    panic!("stream ended without terminal")
}
#[tokio::test]
async fn local_harness_preserves_bash_chat_completion_output() {
    use xai_tool_runtime::ToolCallContext;
    let handle = make_handle();
    register_bash_cco_stub(&handle);
    let harness = handle.create_local_harness("main").expect("local harness");
    let tool_id = xai_tool_protocol::ToolId::new(BASH_CCO_STUB_NAME).expect("valid tool id");
    let stream = harness
        .call(tool_id, serde_json::json!({}), ToolCallContext::default())
        .await;
    let typed = drain_terminal_ok(stream).await;
    assert_bash_cco_terminal(&typed);
}
#[test]
fn rewind_outcome_label_maps_each_variant() {
    assert_eq!(
        rewind_outcome_label(TurnHookOutcome::Completed),
        "completed"
    );
    assert_eq!(
        rewind_outcome_label(TurnHookOutcome::Cancelled),
        "cancelled"
    );
    assert_eq!(rewind_outcome_label(TurnHookOutcome::Error), "error");
}
#[test]
fn rewind_domain_and_result_labels_are_stable() {
    assert_eq!(RewindDomain::Fs.as_ref(), "fs");
    assert_eq!(RewindDomain::Hunk.as_ref(), "hunk");
    assert_eq!(RewindDomain::Git.as_ref(), "git");
    assert_eq!(rewind_result_label(true), "success");
    assert_eq!(rewind_result_label(false), "failure");
}
/// Client names advertised by a session's current toolset.
fn session_tool_names(session: &Arc<crate::session::WorkspaceSession>) -> Vec<String> {
    session
        .toolset()
        .tool_definitions()
        .iter()
        .map(|d| d.function.name.clone())
        .collect()
}
/// The sandbox-resume regression (`workspace_tool_coverage_incomplete`): a session created by a metadata-less bind resolves the workspace default.
/// A later rebind that carries the client's explicit toolset must re-resolve and swap it in rather than silently reuse the default.
#[tokio::test]
async fn rebind_with_changed_explicit_toolset_reresolves_and_swaps() {
    let handle = make_handle();
    let session = handle
        .create_session_with_config("resumed", None, None, CapabilityMode::All, None, false)
        .expect("create default-resolved session");
    session.set_bind_tool_config_fingerprint(None);
    assert!(
        session_tool_names(&session)
            .iter()
            .all(|n| n != "renamed_read"),
        "precondition: the default toolset must not carry the override name"
    );
    let mut renamed = tc("GrokBuild:read_file", Some(ToolKind::Read));
    renamed.name_override = Some("renamed_read".to_owned());
    let cfg = ToolServerConfig {
        tools: vec![renamed],
        behavior_preset: None,
    };
    let fingerprint = serde_json::to_value(&cfg).ok();
    let (rebound, outcome) = handle
        .rebind_existing_hub_session("resumed", Some(cfg.clone()), fingerprint.clone())
        .await
        .expect("session exists");
    assert_eq!(outcome, RebindOutcome::Reresolved);
    assert_eq!(
        session_tool_names(&rebound),
        vec!["renamed_read".to_owned()],
        "the rebind must swap in the explicit toolset's resolution"
    );
    let (_, outcome) = handle
        .rebind_existing_hub_session("resumed", Some(cfg), fingerprint)
        .await
        .expect("session exists");
    assert_eq!(outcome, RebindOutcome::Reused);
}
/// A rebind without an explicit toolset must never downgrade an explicitly-configured session to the default toolset.
/// "Without" covers default resolution and the fail-closed placeholders the caller maps to `None`.
#[tokio::test]
async fn rebind_without_explicit_toolset_reuses_existing() {
    let handle = make_handle();
    let mut renamed = tc("GrokBuild:read_file", Some(ToolKind::Read));
    renamed.name_override = Some("renamed_read".to_owned());
    let cfg = ToolServerConfig {
        tools: vec![renamed],
        behavior_preset: None,
    };
    let session = handle
        .create_session_with_config(
            "configured",
            None,
            Some(cfg.clone()),
            CapabilityMode::All,
            None,
            false,
        )
        .expect("create configured session");
    session.set_bind_tool_config_fingerprint(serde_json::to_value(&cfg).ok());
    let (rebound, outcome) = handle
        .rebind_existing_hub_session("configured", None, None)
        .await
        .expect("session exists");
    assert_eq!(outcome, RebindOutcome::Reused);
    assert_eq!(
        session_tool_names(&rebound),
        vec!["renamed_read".to_owned()],
        "a metadata-less rebind must not clobber the configured toolset"
    );
}
/// The create arm's fingerprint write is set-if-unset. The create task's deferred write must not clobber that fingerprint.
/// Otherwise a later identical rebind would `Reused`-skip against a fingerprint that no longer describes the live toolset.
#[tokio::test]
async fn create_fingerprint_write_does_not_clobber_concurrent_rebind() {
    let handle = make_handle();
    let session = handle
        .create_session_with_config("racy", None, None, CapabilityMode::All, None, false)
        .expect("create session");
    let mut renamed = tc("GrokBuild:read_file", Some(ToolKind::Read));
    renamed.name_override = Some("renamed_read".to_owned());
    let cfg_b = ToolServerConfig {
        tools: vec![renamed],
        behavior_preset: None,
    };
    let fp_b = serde_json::to_value(&cfg_b).ok();
    let (_, outcome) = handle
        .rebind_existing_hub_session("racy", Some(cfg_b.clone()), fp_b.clone())
        .await
        .expect("session exists");
    assert_eq!(outcome, RebindOutcome::Reresolved);
    let fp_a = serde_json::to_value(&ToolServerConfig {
        tools: vec![tc("GrokBuild:list_dir", Some(ToolKind::ListDir))],
        behavior_preset: None,
    })
    .ok();
    session.set_bind_tool_config_fingerprint_if_unset(fp_a);
    let (rebound, outcome) = handle
        .rebind_existing_hub_session("racy", Some(cfg_b), fp_b)
        .await
        .expect("session exists");
    assert_eq!(outcome, RebindOutcome::Reused);
    assert_eq!(
        session_tool_names(&rebound),
        vec!["renamed_read".to_owned()]
    );
}
/// A vanished session yields `None` (the caller falls back to RPC-only).
#[tokio::test]
async fn rebind_missing_session_returns_none() {
    let handle = make_handle();
    assert!(
        handle
            .rebind_existing_hub_session("no-such-session", None, None)
            .await
            .is_none()
    );
}
fn swap_rejected_count(reason: &str, trigger: &str) -> u64 {
    crate::session::swap_policy::WORKSPACE_TOOLSET_SWAP_REJECTED_TOTAL
        .with_label_values(&[reason, trigger])
        .get()
}
/// The lazy-bind and resume-correction regression lock.
/// A default-resolved session (stored fingerprint `None`) must accept the owner's explicit-config rebind even mid-turn with a call in flight.
/// The owner bind is designed to land mid-turn; deferring it would serve a toolset that contradicts the config-built prompt.
#[tokio::test]
async fn rebind_none_to_explicit_swaps_mid_turn() {
    let handle = make_handle();
    let session = handle
        .create_session_with_config("lazy", None, None, CapabilityMode::All, None, false)
        .expect("create default-resolved session");
    session.set_bind_tool_config_fingerprint(None);
    let tracker = handle.activity_tracker().clone();
    tracker.turn_started("lazy", 1);
    tracker.tool_call_started("lazy-c1", "read_file", Some("lazy"));
    let cfg = explicit_cfg("renamed_read");
    let fingerprint = serde_json::to_value(&cfg).ok();
    let (rebound, outcome) = handle
        .rebind_existing_hub_session("lazy", Some(cfg), fingerprint)
        .await
        .expect("session exists");
    assert_eq!(
        outcome,
        RebindOutcome::Reresolved,
        "a None → explicit correction must swap even mid-turn with calls in flight"
    );
    assert_eq!(
        session_tool_names(&rebound),
        vec!["renamed_read".to_owned()]
    );
}
/// An explicit-to-different-explicit rebind under dispatch keeps the existing toolset (`ReresolveDeferredInFlight`, counted).
/// Once the call completes, a later rebind applies the correction.
#[tokio::test]
async fn rebind_explicit_to_explicit_with_in_flight_call_defers_then_corrects() {
    use xai_grok_session_events::ToolOutcome;
    let rejected_before = swap_rejected_count("in_flight", "owner_rebind");
    let handle = make_handle();
    let cfg_a = explicit_cfg("read_a");
    let session = handle
        .create_session_with_config(
            "busy",
            None,
            Some(cfg_a.clone()),
            CapabilityMode::All,
            None,
            false,
        )
        .expect("create session with cfg A");
    session.set_bind_tool_config_fingerprint(serde_json::to_value(&cfg_a).ok());
    let tracker = handle.activity_tracker().clone();
    tracker.tool_call_started("busy-c1", "read_a", Some("busy"));
    let cfg_b = explicit_cfg("read_b");
    let fp_b = serde_json::to_value(&cfg_b).ok();
    let (kept, outcome) = handle
        .rebind_existing_hub_session("busy", Some(cfg_b.clone()), fp_b.clone())
        .await
        .expect("session exists");
    assert_eq!(outcome, RebindOutcome::ReresolveDeferredInFlight);
    assert_eq!(
        session_tool_names(&kept),
        vec!["read_a".to_owned()],
        "the existing toolset must be kept while a call is in flight"
    );
    assert!(
        swap_rejected_count("in_flight", "owner_rebind") > rejected_before,
        "the deferred swap must be counted"
    );
    tracker.tool_call_completed("busy-c1", Some("busy"), ToolOutcome::Success);
    let (rebound, outcome) = handle
        .rebind_existing_hub_session("busy", Some(cfg_b), fp_b)
        .await
        .expect("session exists");
    assert_eq!(
        outcome,
        RebindOutcome::Reresolved,
        "the correction must apply once no calls are in flight"
    );
    assert_eq!(session_tool_names(&rebound), vec!["read_b".to_owned()]);
}
/// A reconnect's identical `session.bind` heals a stale session: reuse without the marker, defer in-flight, rebuild and clear once idle.
#[tokio::test]
async fn rebind_identical_reapply_repairs_stale_resolve() {
    use xai_grok_session_events::ToolOutcome;
    let handle = make_handle();
    let cfg = explicit_cfg("renamed_read");
    let fingerprint = serde_json::to_value(&cfg).ok();
    let session = handle
        .create_session_with_config(
            "stale-rebind",
            None,
            Some(cfg.clone()),
            CapabilityMode::All,
            None,
            false,
        )
        .expect("create session");
    session.set_bind_tool_config_fingerprint(fingerprint.clone());
    let toolset_before = session.toolset();
    let (_, outcome) = handle
        .rebind_existing_hub_session("stale-rebind", Some(cfg.clone()), fingerprint.clone())
        .await
        .expect("session exists");
    assert_eq!(outcome, RebindOutcome::Reused);
    assert!(
        Arc::ptr_eq(&session.toolset(), &toolset_before),
        "without the stale marker the identical rebind must not rebuild"
    );
    session.mark_stale_resolve();
    let tracker = handle.activity_tracker().clone();
    tracker.tool_call_started("stale-c1", "read_file", Some("stale-rebind"));
    let rejected_before = swap_rejected_count("in_flight", "owner_rebind");
    let (kept, outcome) = handle
        .rebind_existing_hub_session("stale-rebind", Some(cfg.clone()), fingerprint.clone())
        .await
        .expect("session exists");
    assert_eq!(
        outcome,
        RebindOutcome::ReresolveDeferredInFlight,
        "the heal must defer while a call is in flight"
    );
    assert!(
        Arc::ptr_eq(&kept.toolset(), &toolset_before),
        "the deferred heal must keep the existing toolset"
    );
    assert!(kept.stale_resolve(), "the deferred heal keeps the marker");
    assert!(
        swap_rejected_count("in_flight", "owner_rebind") > rejected_before,
        "the deferred heal must be counted"
    );
    tracker.tool_call_completed("stale-c1", Some("stale-rebind"), ToolOutcome::Success);
    let (healed, outcome) = handle
        .rebind_existing_hub_session("stale-rebind", Some(cfg), fingerprint)
        .await
        .expect("session exists");
    assert_eq!(
        outcome,
        RebindOutcome::Reresolved,
        "the idle reconnect must repair the stale toolset"
    );
    assert!(
        !Arc::ptr_eq(&healed.toolset(), &toolset_before),
        "the heal must install a freshly resolved toolset"
    );
    assert!(
        !healed.stale_resolve(),
        "a successful install must clear the stale marker"
    );
}
/// The RPC path rejects a mid-turn config change with the retryable `TurnActive` error (counted); the retry at the turn boundary succeeds.
#[tokio::test]
async fn update_tool_config_rejects_mid_turn_then_succeeds_at_boundary() {
    let rejected_before = swap_rejected_count("turn_active", "update_tool_config");
    let handle = make_handle();
    handle.activity_tracker().turn_started("main", 1);
    let cfg = explicit_cfg("renamed_read");
    let err = handle
        .update_tool_config("main", "main", cfg.clone())
        .await
        .expect_err("a mid-turn config change must be rejected");
    assert!(
        matches!(err, WorkspaceError::TurnActive(ref s) if s == "main"),
        "got {err:?}"
    );
    assert!(
        swap_rejected_count("turn_active", "update_tool_config") > rejected_before,
        "the rejection must be counted"
    );
    let session = handle.session("main").expect("main session exists");
    assert!(
        session_tool_names(&session)
            .iter()
            .all(|n| n != "renamed_read"),
        "the rejected config must not take effect"
    );
    handle.activity_tracker().turn_completed("main", 1, 0);
    handle
        .update_tool_config("main", "main", cfg)
        .await
        .expect("the retry at the turn boundary must succeed");
    let session = handle.session("main").expect("main session exists");
    assert_eq!(
        session_tool_names(&session),
        vec!["renamed_read".to_owned()]
    );
}
/// TOCTOU lock: a turn that starts DURING the re-resolve (after the entry check passed) must still abort the install.
/// The resolved toolset is discarded, the fingerprint stays unchanged, and the rejection is counted under `reason="turn_active_late"`.
/// The retry at the turn boundary then succeeds.
#[tokio::test]
async fn update_tool_config_rejects_turn_started_during_resolve() {
    let late_rejected_before = swap_rejected_count("turn_active_late", "update_tool_config");
    let handle = make_handle();
    let session = handle.session("main").expect("main session exists");
    let toolset_before = session.toolset();
    let hook_handle = handle.clone();
    *handle.shared.post_resolve_test_hook.lock() = Some(Box::new(move || {
        hook_handle.activity_tracker().turn_started("main", 7);
    }));
    let cfg = explicit_cfg("late_read");
    let err = handle
        .update_tool_config("main", "main", cfg.clone())
        .await
        .expect_err("a turn starting mid-resolve must abort the install");
    assert!(
        matches!(err, WorkspaceError::TurnActive(ref s) if s == "main"),
        "got {err:?}"
    );
    assert!(
        swap_rejected_count("turn_active_late", "update_tool_config") > late_rejected_before,
        "the post-resolve rejection must be counted distinctly"
    );
    let session = handle.session("main").expect("main session exists");
    assert!(
        Arc::ptr_eq(&session.toolset(), &toolset_before),
        "the resolved toolset must be discarded, not installed"
    );
    assert!(
        session.bind_tool_config_matches(None),
        "the unapplied config's fingerprint must NOT be recorded"
    );
    *handle.shared.post_resolve_test_hook.lock() = None;
    handle.activity_tracker().turn_completed("main", 7, 0);
    handle
        .update_tool_config("main", "main", cfg)
        .await
        .expect("the retry at the turn boundary must succeed");
    let session = handle.session("main").expect("main session exists");
    assert_eq!(session_tool_names(&session), vec!["late_read".to_owned()]);
}
/// Re-applying the session's current config mid-turn stays allowed (matching fingerprint), so hot-reload re-applies keep working during turns.
#[tokio::test]
async fn update_tool_config_reapply_of_current_config_allowed_mid_turn() {
    let handle = make_handle();
    let cfg = explicit_cfg("renamed_read");
    let session = handle
        .create_session_with_config(
            "hot",
            None,
            Some(cfg.clone()),
            CapabilityMode::All,
            None,
            false,
        )
        .expect("create session");
    session.set_bind_tool_config_fingerprint(serde_json::to_value(&cfg).ok());
    handle.activity_tracker().turn_started("hot", 1);
    handle
        .update_tool_config("hot", "hot", cfg)
        .await
        .expect("an identical-config re-apply must not be turn_active-rejected");
}
#[tokio::test]
async fn update_tool_config_identical_reapply_repairs_stale_resolve() {
    let handle = make_handle();
    let cfg = explicit_cfg("renamed_read");
    let session = handle
        .create_session_with_config(
            "stale",
            None,
            Some(cfg.clone()),
            CapabilityMode::All,
            None,
            false,
        )
        .expect("create session");
    session.set_bind_tool_config_fingerprint(serde_json::to_value(&cfg).ok());
    let toolset_before = session.toolset();
    handle
        .update_tool_config("stale", "stale", cfg.clone())
        .await
        .expect("an identical re-apply must succeed");
    assert!(
        Arc::ptr_eq(&session.toolset(), &toolset_before),
        "without the stale marker the identical re-apply must not rebuild"
    );
    session.mark_stale_resolve();
    let rejected_before = swap_rejected_count("turn_active", "update_tool_config");
    handle.activity_tracker().turn_started("stale", 1);
    let err = handle
        .update_tool_config("stale", "stale", cfg.clone())
        .await
        .expect_err("a mid-turn recovery re-apply must be rejected");
    assert!(
        matches!(err, WorkspaceError::TurnActive(ref s) if s == "stale"),
        "got {err:?}"
    );
    assert!(
        swap_rejected_count("turn_active", "update_tool_config") > rejected_before,
        "the rejected recovery must be counted"
    );
    assert!(
        session.stale_resolve(),
        "the rejected recovery must keep the stale marker"
    );
    assert!(
        Arc::ptr_eq(&session.toolset(), &toolset_before),
        "the rejected recovery must not install"
    );
    handle.activity_tracker().turn_completed("stale", 1, 0);
    handle
        .update_tool_config("stale", "stale", cfg.clone())
        .await
        .expect("the boundary retry must repair the stale toolset");
    let session = handle.session("stale").expect("session exists");
    assert!(
        !Arc::ptr_eq(&session.toolset(), &toolset_before),
        "the recovery re-apply must install a freshly resolved toolset"
    );
    assert!(
        !session.stale_resolve(),
        "a successful install must clear the stale marker"
    );
    assert!(
        session.bind_tool_config_matches(serde_json::to_value(&cfg).ok().as_ref()),
        "the stored fingerprint must be unchanged by the identical recovery"
    );
}
/// The `Terminal` resource of a session's current toolset.
async fn toolset_terminal(
    toolset: &Arc<xai_grok_tools::registry::types::FinalizedToolset>,
) -> Arc<dyn xai_grok_tools::computer::types::TerminalBackend> {
    let res = toolset.resources.lock().await;
    res.get::<xai_grok_tools::types::resources::Terminal>()
        .map(|t| t.0.clone())
        .expect("toolset must carry a Terminal resource")
}
fn orphaned_swap_count() -> u64 {
    WORKSPACE_TERMINAL_BACKEND_ORPHANED_TOTAL
        .with_label_values(&["swap"])
        .get()
}
fn explicit_cfg(name_override: &str) -> ToolServerConfig {
    let mut renamed = tc("GrokBuild:read_file", Some(ToolKind::Read));
    renamed.name_override = Some(name_override.to_owned());
    ToolServerConfig {
        tools: vec![renamed],
        behavior_preset: None,
    }
}
/// The background-capable toolset (execute, task-output, kill) that the restart-recovery and RPC-survival tests resolve.
pub(crate) fn background_capable_cfg() -> ToolServerConfig {
    ToolServerConfig {
        tools: vec![
            tc("GrokBuild:read_file", Some(ToolKind::Read)),
            tc("GrokBuild:run_terminal_cmd", Some(ToolKind::Execute)),
            tc(
                "GrokBuild:get_task_output",
                Some(ToolKind::BackgroundTaskAction),
            ),
            tc("GrokBuild:kill_task", Some(ToolKind::KillTaskAction)),
        ],
        behavior_preset: None,
    }
}
/// A minimal bash-kind [`TerminalRunRequest`] for `command`, writing output under `out_dir`.
///
/// [`TerminalRunRequest`]: xai_grok_tools::computer::types::TerminalRunRequest
pub(crate) fn terminal_run_request(
    command: &str,
    out_dir: &std::path::Path,
    tool_call_id: &str,
) -> xai_grok_tools::computer::types::TerminalRunRequest {
    xai_grok_tools::computer::types::TerminalRunRequest {
        command: command.to_string(),
        working_directory: out_dir.to_path_buf(),
        env: std::collections::HashMap::new(),
        timeout: std::time::Duration::from_secs(60),
        output_byte_limit: 4096,
        output_file: out_dir.join(format!("{tool_call_id}.out")),
        notification_handle: xai_grok_tools::notification::ToolNotificationHandle::noop(),
        tool_call_id: tool_call_id.to_string(),
        display_command: None,
        auto_background_on_timeout: false,
        foreground_block_budget: None,
        kind: xai_grok_tools::computer::types::TaskKind::Bash,
        owner_session_id: None,
        description: None,
    }
}
/// Start a `sleep 30` background task on `session`'s owned backend and return its handle.
/// Shared by the swap-survival, rebind-survival, and restart tests.
pub(crate) async fn start_background_sleep(
    session: &Arc<crate::session::WorkspaceSession>,
    out_dir: &std::path::Path,
    tool_call_id: &str,
) -> xai_grok_tools::computer::types::BackgroundHandle {
    session
        .terminal_backend()
        .run_background(terminal_run_request("sleep 30", out_dir, tool_call_id))
        .await
        .expect("start background task")
}
/// A rebind that swaps in a different explicit toolset must rebuild the toolset AROUND the session-owned terminal backend, not a fresh one.
/// That identity is what keeps background tasks alive across the swap.
#[tokio::test]
async fn rebind_swap_preserves_session_terminal_backend() {
    let orphaned_before = orphaned_swap_count();
    let handle = make_handle();
    let cfg_a = explicit_cfg("read_a");
    let session = handle
        .create_session_with_config(
            "owned",
            None,
            Some(cfg_a.clone()),
            CapabilityMode::All,
            None,
            false,
        )
        .expect("create session with cfg A");
    session.set_bind_tool_config_fingerprint(serde_json::to_value(&cfg_a).ok());
    let backend = session.terminal_backend().clone();
    assert!(
        Arc::ptr_eq(&backend, &toolset_terminal(&session.toolset()).await),
        "create must wire the session-owned backend into the toolset"
    );
    let cfg_b = explicit_cfg("read_b");
    let fingerprint_b = serde_json::to_value(&cfg_b).ok();
    let (rebound, outcome) = handle
        .rebind_existing_hub_session("owned", Some(cfg_b), fingerprint_b)
        .await
        .expect("session exists");
    assert_eq!(outcome, RebindOutcome::Reresolved);
    assert_eq!(session_tool_names(&rebound), vec!["read_b".to_owned()]);
    assert!(
        Arc::ptr_eq(&backend, rebound.terminal_backend()),
        "the session-owned backend must not be replaced by a swap"
    );
    assert!(
        Arc::ptr_eq(&backend, &toolset_terminal(&rebound.toolset()).await),
        "the swapped-in toolset must reference the session-owned backend"
    );
    assert_eq!(
        orphaned_swap_count(),
        orphaned_before,
        "the orphaned-backend tripwire must stay 0"
    );
}
/// A snapshot-driven `re_resolve_all_sessions` rebuild (MCP snapshot change) must also rebuild around the session-owned backend.
/// The test keeps a LIVE background task running through the rebuild.
/// This locks the regression where snapshot-triggered swaps killed background tasks by building a fresh backend per session.
#[tokio::test]
async fn re_resolve_all_sessions_preserves_session_terminal_backend() {
    let orphaned_before = orphaned_swap_count();
    let handle = make_handle();
    let session = handle.session("main").expect("main session exists");
    let backend = session.terminal_backend().clone();
    let out_dir = tempfile::tempdir().expect("temp dir");
    let bg = start_background_sleep(&session, out_dir.path(), "snapshot-bg").await;
    handle.shared.mcp_tools_snapshot.store(Arc::new(vec![tc(
        "GrokBuild:read_file",
        Some(ToolKind::Read),
    )]));
    let rebuilt = handle
        .shared
        .re_resolve_all_sessions("mcp_snapshot_changed", true)
        .await;
    assert!(rebuilt >= 1, "the main session must be rebuilt");
    let session = handle.session("main").expect("main session still exists");
    assert!(
        Arc::ptr_eq(&backend, session.terminal_backend()),
        "the session-owned backend must survive a snapshot rebuild"
    );
    let new_terminal = toolset_terminal(&session.toolset()).await;
    assert!(
        Arc::ptr_eq(&backend, &new_terminal),
        "the rebuilt toolset must reference the session-owned backend"
    );
    assert!(
        !new_terminal
            .get_task(&bg.task_id)
            .await
            .expect("the task table must survive the snapshot rebuild")
            .completed,
        "the task's process must still be running after the rebuild"
    );
    assert_eq!(
        orphaned_swap_count(),
        orphaned_before,
        "the orphaned-backend tripwire must stay 0"
    );
    new_terminal.kill_task(&bg.task_id).await;
}
/// A local-bound session gets an external toolset via `bind_local_session`; that toolset keeps the shell's backend while the session-owned one idles.
/// Snapshot-driven rebuilds must SKIP it: rebuilding around the idle backend would detach tools from the shell's live task table.
/// The skip must not fire the orphan tripwire; the mismatch is the local-bind contract.
#[tokio::test]
async fn local_bound_session_skips_snapshot_rebuild() {
    let orphaned_before = orphaned_swap_count();
    let handle = make_handle();
    let donor = handle
        .create_session_with_config(
            "donor",
            None,
            Some(explicit_cfg("read_donor")),
            CapabilityMode::All,
            None,
            false,
        )
        .expect("create donor session");
    let local = handle
        .create_session_with_config(
            "local",
            None,
            Some(explicit_cfg("read_local")),
            CapabilityMode::All,
            None,
            false,
        )
        .expect("create local session");
    let external_toolset = donor.toolset();
    local.replace(local.effective_tool_config(), external_toolset.clone());
    assert!(
        !local.toolset_terminal_is_session_owned().await,
        "precondition: the installed toolset's Terminal must be external"
    );
    handle.shared.mcp_tools_snapshot.store(Arc::new(vec![tc(
        "GrokBuild:read_file",
        Some(ToolKind::Read),
    )]));
    handle
        .shared
        .re_resolve_all_sessions("mcp_snapshot_changed", true)
        .await;
    let local = handle.session("local").expect("local session still exists");
    assert!(
        Arc::ptr_eq(&local.toolset(), &external_toolset),
        "the local-bound session's toolset must be untouched by the rebuild"
    );
    assert!(
        Arc::ptr_eq(
            &toolset_terminal(&local.toolset()).await,
            donor.terminal_backend()
        ),
        "the external (shell) backend must still ride the toolset"
    );
    assert_eq!(
        orphaned_swap_count(),
        orphaned_before,
        "the skip must not fire the orphaned-backend tripwire"
    );
    let outcome = handle
        .resolve_and_swap_session_toolset(&local, explicit_cfg("read_new"), SwapTrigger::UpdateRpc)
        .await
        .expect("the skip is not an internal error at the choke point");
    assert_eq!(outcome, SwapOutcome::SkippedExternallyOwned);
    assert!(
        Arc::ptr_eq(&local.toolset(), &external_toolset),
        "the choke point must not swap an externally-owned toolset"
    );
    assert_eq!(orphaned_swap_count(), orphaned_before);
    let err = handle
        .update_tool_config("local", "local", explicit_cfg("read_new"))
        .await
        .expect_err("update_tool_config must refuse an externally-owned toolset");
    assert!(
        matches!(err, crate::error::WorkspaceError::ToolsetExternallyOwned(ref s) if s == "local"),
        "expected ToolsetExternallyOwned, got: {err:?}"
    );
    assert!(
        Arc::ptr_eq(&local.toolset(), &external_toolset),
        "the refused update must leave the toolset untouched"
    );
    let fp_local = serde_json::to_value(explicit_cfg("read_local")).ok();
    local.set_bind_tool_config_fingerprint(fp_local.clone());
    let cfg_new = explicit_cfg("read_new2");
    let fp_new = serde_json::to_value(&cfg_new).ok();
    let (rebound, outcome) = handle
        .rebind_existing_hub_session("local", Some(cfg_new), fp_new.clone())
        .await
        .expect("session exists");
    assert_eq!(outcome, RebindOutcome::KeptExternallyOwned);
    assert!(
        Arc::ptr_eq(&rebound.toolset(), &external_toolset),
        "the rebind must keep the externally-owned toolset"
    );
    assert!(
        rebound.bind_tool_config_matches(fp_local.as_ref()),
        "the stored fingerprint must be unchanged by the skipped swap"
    );
    assert!(
        !rebound.bind_tool_config_matches(fp_new.as_ref()),
        "the unapplied config's fingerprint must NOT be recorded"
    );
    assert_eq!(orphaned_swap_count(), orphaned_before);
    handle
        .update_tool_config("local", "local", explicit_cfg("read_local"))
        .await
        .expect("an identical config on an externally-owned toolset is a no-op success");
    assert!(
        Arc::ptr_eq(&local.toolset(), &external_toolset),
        "the identical no-op must leave the externally-owned toolset untouched"
    );
    assert!(
        local.bind_tool_config_matches(fp_local.as_ref()),
        "the identical no-op must leave the stored fingerprint untouched"
    );
    assert_eq!(orphaned_swap_count(), orphaned_before);
}
/// A background task started before a toolset swap must still be queryable through the NEW toolset's `Terminal` resource.
/// This locks the incident where a swap left an empty task table and SIGKILLed running tasks.
#[tokio::test]
async fn background_task_survives_toolset_swap() {
    let orphaned_before = orphaned_swap_count();
    let handle = make_handle();
    let cfg_a = explicit_cfg("read_a");
    let session = handle
        .create_session_with_config(
            "bg",
            None,
            Some(cfg_a.clone()),
            CapabilityMode::All,
            None,
            false,
        )
        .expect("create session");
    session.set_bind_tool_config_fingerprint(serde_json::to_value(&cfg_a).ok());
    let out_dir = tempfile::tempdir().expect("temp dir");
    let bg = start_background_sleep(&session, out_dir.path(), "bg-task").await;
    let cfg_b = explicit_cfg("read_b");
    let fingerprint_b = serde_json::to_value(&cfg_b).ok();
    let (rebound, outcome) = handle
        .rebind_existing_hub_session("bg", Some(cfg_b), fingerprint_b)
        .await
        .expect("session exists");
    assert_eq!(outcome, RebindOutcome::Reresolved);
    let new_terminal = toolset_terminal(&rebound.toolset()).await;
    let task = new_terminal
        .get_task(&bg.task_id)
        .await
        .expect("the task table must survive the toolset swap");
    assert!(
        !task.completed,
        "the task's process must still be running after the swap"
    );
    assert_eq!(
        orphaned_swap_count(),
        orphaned_before,
        "the orphaned-backend tripwire must stay 0"
    );
    new_terminal.kill_task(&bg.task_id).await;
}
/// Test factory whose sessions own a PERSISTENT-shell backend (the production factory shape).
/// The plain [`TestSessionContextFactory`] builds a non-persistent backend, which tracks no shell cwd.
/// The shell-state-survival test uses this wrapper instead.
struct PersistentShellFactory {
    inner: TestSessionContextFactory,
}
impl crate::config::SessionContextFactory for PersistentShellFactory {
    fn build_session_context(
        &self,
        session_id: &str,
        cwd: std::path::PathBuf,
        session_env: Arc<std::collections::HashMap<String, String>>,
        backend: Arc<dyn xai_grok_tools::computer::types::TerminalBackend>,
    ) -> xai_grok_tools::registry::types::SessionContext {
        self.inner
            .build_session_context(session_id, cwd, session_env, backend)
    }
    fn build_terminal_backend(&self) -> crate::config::SessionTerminalBackend {
        crate::config::SessionTerminalBackend::local(
            xai_grok_tools::computer::local::LocalTerminalBackend::with_persistent_shell(),
        )
    }
    fn registry_builder(&self) -> xai_grok_tools::registry::types::ToolRegistryBuilder {
        self.inner.registry_builder()
    }
}
/// [`make_handle`] shape around a [`PersistentShellFactory`]; no pre-created session.
fn make_persistent_shell_handle() -> WorkspaceHandle {
    let factory = Arc::new(PersistentShellFactory {
        inner: TestSessionContextFactory::new(),
    });
    let root_cwd = factory.inner.temp.path().to_path_buf();
    let config = WorkspaceConfig {
        root_cwd,
        default_tool_config: baseline_config(),
        respect_gitignore: false,
        memory_config: None,
        event_buffer_capacity: DEFAULT_EVENT_BUFFER_CAPACITY,
        session_factory: factory,
        hook_global_sources: vec![],
        hook_project_sources: vec![],
        skills_config: Default::default(),
        plugin_discovery_config: Default::default(),
        server_metadata: None,
        status_config: Default::default(),
        project_lsp_trusted: true,
        require_explicit_toolset: false,
        confine_fs_to_workspace_root: false,
        host_kind: Default::default(),
        bind_mcp: None,
        tool_approval: crate::permission::ToolApprovalGate::Off,
        sandbox: None,
    };
    WorkspaceHandle::build(
        config,
        ephemeral_workspace_home(),
        false,
        false,
        false,
        false,
        crate::identity::WorkspaceIdentity::default(),
    )
    .expect("handle construction should succeed")
}
/// The persistent shell's state (a model-issued `cd`) survives a `Reresolved` swap because the shell lives inside the session-owned backend.
/// This is the isolation-matrix #3 "persistent-shell cwd preserved" sub-assert, on the production backend shape (`with_persistent_shell`).
/// Unix-only, like the persistent shell.
#[cfg(unix)]
#[tokio::test]
async fn reresolved_swap_preserves_persistent_shell_cwd() {
    let handle = make_persistent_shell_handle();
    let root = handle.root_cwd().expect("root cwd");
    let cfg_a = explicit_cfg("read_a");
    let session = handle
        .create_session_with_config(
            "shell-swap",
            None,
            Some(cfg_a.clone()),
            CapabilityMode::All,
            None,
            false,
        )
        .expect("create session");
    session.set_bind_tool_config_fingerprint(serde_json::to_value(&cfg_a).ok());
    std::fs::create_dir_all(root.join("swap_kept_dir")).expect("create subdir");
    let result = session
        .terminal_backend()
        .run(terminal_run_request("cd swap_kept_dir", &root, "shell-cd"))
        .await
        .expect("cd through the persistent shell");
    assert_eq!(
        result.exit_code,
        Some(0),
        "cd must succeed: {}",
        result.combined_output
    );
    let cwd_before = session
        .terminal_backend()
        .get_shell_cwd()
        .await
        .expect("the persistent shell must track a cwd after a command");
    assert_eq!(
        cwd_before.file_name().and_then(|n| n.to_str()),
        Some("swap_kept_dir"),
        "the shell must have entered the subdir: {}",
        cwd_before.display()
    );
    let cfg_b = explicit_cfg("read_b");
    let (rebound, outcome) = handle
        .rebind_existing_hub_session(
            "shell-swap",
            Some(cfg_b.clone()),
            serde_json::to_value(&cfg_b).ok(),
        )
        .await
        .expect("session exists");
    assert_eq!(outcome, RebindOutcome::Reresolved);
    let cwd_after = toolset_terminal(&rebound.toolset())
        .await
        .get_shell_cwd()
        .await
        .expect("the swapped-in toolset's terminal must still track the shell cwd");
    assert_eq!(
        cwd_after, cwd_before,
        "the persistent shell's cwd must survive the toolset swap"
    );
}
/// Each fork owns its own fresh backend: fork teardown kills only the fork's tasks, never the parent's.
#[tokio::test]
async fn fork_session_owns_distinct_terminal_backend() {
    let handle = make_handle();
    let parent = handle.session("main").expect("main session exists");
    let fork = handle
        .fork_session(fork_cfg_with(
            "fork-backend",
            CapabilityMode::ReadWrite,
            None,
            Some("main"),
        ))
        .await
        .expect("fork succeeds");
    assert!(
        !Arc::ptr_eq(parent.terminal_backend(), fork.terminal_backend()),
        "a fork must own its own backend, not share the parent's"
    );
    assert!(
        Arc::ptr_eq(
            fork.terminal_backend(),
            &toolset_terminal(&fork.toolset()).await
        ),
        "the fork's toolset must reference the fork-owned backend"
    );
}
/// Poll `backend` with a trivial command until its actor refuses it, proving an explicit shutdown since callers still hold live `Arc`s.
/// Shared by the `drop_session` and hub-evict teardown tests.
pub(crate) async fn assert_backend_stops(
    backend: &Arc<dyn xai_grok_tools::computer::types::TerminalBackend>,
) {
    let out_dir = tempfile::tempdir().expect("temp dir");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let request = terminal_run_request("true", out_dir.path(), "probe");
        if backend.run(request).await.is_err() {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "backend actor must stop after an explicit shutdown even with live Arcs"
        );
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}
/// `drop_session` shuts the backend down explicitly: the actor stops even while other `Arc`s to the backend are still alive.
/// Teardown must not depend on the last toolset `Arc` dropping.
#[tokio::test]
async fn drop_session_shuts_down_terminal_backend_explicitly() {
    let handle = make_handle();
    let session = handle
        .create_session_with_config("doomed", None, None, CapabilityMode::All, None, false)
        .expect("create session");
    let retained_backend = session.terminal_backend().clone();
    let retained_toolset = session.toolset();
    drop(session);
    handle.drop_session("doomed", "doomed").expect("drop");
    assert_backend_stops(&retained_backend).await;
    drop(retained_toolset);
}
async fn assert_hunk_tracker_stops(tracker: &xai_hunk_tracker::HunkTrackerHandle) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !tracker.is_closed() {
        assert!(
            std::time::Instant::now() < deadline,
            "hunk-tracker actor must stop within the deadline despite live \
             handle clones"
        );
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}
/// `drop_session` cancels the workspace-spawned hunk-tracker actor even while a leaked `HunkTrackerHandle` clone keeps its command channel open.
/// Rationale on `cancel_hunk_tracker`.
#[tokio::test]
async fn drop_session_cancels_workspace_spawned_hunk_tracker() {
    let handle = make_handle();
    let session = handle
        .create_session_with_config("doomed-ht", None, None, CapabilityMode::All, None, false)
        .expect("create session");
    let leaked_tracker = session.hunk_tracker().clone();
    assert!(
        !leaked_tracker.is_closed(),
        "precondition: the actor is alive while the session exists"
    );
    drop(session);
    handle.drop_session("doomed-ht", "doomed-ht").expect("drop");
    assert_hunk_tracker_stops(&leaked_tracker).await;
}
/// Same guarantee for the fork spawn site.
#[tokio::test]
async fn drop_session_cancels_forked_session_hunk_tracker() {
    let handle = make_handle();
    let child = handle
        .fork_session(fork_cfg_with(
            "child-ht",
            CapabilityMode::ReadWrite,
            None,
            Some("main"),
        ))
        .await
        .expect("fork should succeed");
    let leaked_tracker = child.hunk_tracker().clone();
    assert!(
        !leaked_tracker.is_closed(),
        "precondition: the actor is alive while the session exists"
    );
    drop(child);
    handle.drop_session("child-ht", "child-ht").expect("drop");
    assert_hunk_tracker_stops(&leaked_tracker).await;
}
/// The inverse guarantee: a tracker bound via `create_session_with_tracker` is externally owned, so `drop_session` must NOT cancel it.
/// The agent shares such trackers with the workspace session.
#[tokio::test]
async fn drop_session_leaves_externally_owned_hunk_tracker_alive() {
    let handle = make_handle();
    let cwd = handle.shared.root_cwd.clone();
    let (hunk_event_tx, _hunk_event_rx) = tokio::sync::mpsc::unbounded_channel();
    let owner_cancel = tokio_util::sync::CancellationToken::new();
    let tracker = HunkTrackerActor::spawn(
        "external-ht".to_string(),
        cwd.clone(),
        hunk_event_tx,
        TrackingMode::AllDirty,
        owner_cancel.clone(),
    );
    let session = handle
        .create_session_with_tracker(
            "external-ht",
            cwd,
            tracker.clone(),
            None,
            CapabilityMode::All,
        )
        .expect("create session");
    assert!(
        !tracker.is_closed(),
        "precondition: the actor is alive while the session exists"
    );
    drop(session);
    handle
        .drop_session("external-ht", "external-ht")
        .expect("drop");
    let _ = tracker.get_all_hunks().await;
    assert!(
        !tracker.is_closed(),
        "drop_session must not cancel an externally owned hunk tracker"
    );
    owner_cancel.cancel();
    assert_hunk_tracker_stops(&tracker).await;
}
/// Isolation matrix #5: a workspace process restart loses tasks (they are process state), and what's pinned here is the recovery UX.
/// The same session id recreates cleanly on the fresh process and the task table starts empty (loss is visible, not silent).
/// `get_task_output` for the lost id returns the informative not-found message.
#[tokio::test]
async fn restarted_workspace_recreates_session_and_reports_lost_task() {
    let handle_a = make_handle();
    let session_a = handle_a
        .create_session_with_config(
            "reborn",
            None,
            Some(background_capable_cfg()),
            CapabilityMode::All,
            None,
            false,
        )
        .expect("create session");
    let out_dir = tempfile::tempdir().expect("temp dir");
    let bg = start_background_sleep(&session_a, out_dir.path(), "restart-bg").await;
    assert!(
        session_a
            .terminal_backend()
            .get_task(&bg.task_id)
            .await
            .is_some(),
        "precondition: the task exists in the first process"
    );
    let handle_b = make_handle();
    let session_b = handle_b
        .create_session_with_config(
            "reborn",
            None,
            Some(background_capable_cfg()),
            CapabilityMode::All,
            None,
            false,
        )
        .expect("the session must recreate cleanly after a restart");
    assert!(
        session_b.terminal_backend().list_tasks().await.is_empty(),
        "precondition: a fresh handle must start with an empty task table"
    );
    let result = session_b
        .toolset()
        .call(
            "get_task_output",
            serde_json::json!({"task_ids": [bg.task_id.clone()]}),
            "restart-probe",
            None,
        )
        .await
        .expect("get_task_output must answer, not error");
    let xai_grok_tools::types::output::ToolOutput::TaskOutput(
        xai_tool_types::TaskOutputOutput::TaskNotFound(msg),
    ) = &result.output
    else {
        panic!("expected TaskNotFound, got: {:?}", result.output);
    };
    assert!(
        msg.contains(&format!("Task {} not found", bg.task_id)),
        "the message must name the lost task id: {msg}"
    );
    assert!(
        msg.contains("No background tasks or subagents exist in this session"),
        "the message must say the restarted session has no tasks: {msg}"
    );
    session_a.terminal_backend().kill_task(&bg.task_id).await;
}
/// The typed helpers feed the registry and the targeted counters advance.
/// Counters are monotonic, so `after > before` is robust despite the process-global registry and parallel tests (capture, restore, canary).
#[test]
fn rewind_metric_helpers_record_observable_effects() {
    let capture_labels = [
        RewindDomain::Git.as_ref(),
        rewind_outcome_label(TurnHookOutcome::Cancelled),
    ];
    let restore_labels = [RewindDomain::Fs.as_ref(), rewind_result_label(true)];
    let canary_label = [rewind_outcome_label(TurnHookOutcome::Error)];
    let capture_before = REWIND_CHECKPOINT_CAPTURE_TOTAL
        .with_label_values(&capture_labels)
        .get();
    let restore_before = REWIND_RESTORE_TOTAL
        .with_label_values(&restore_labels)
        .get();
    let canary_before = REWIND_NON_COMPLETED_FINALIZE_TOTAL
        .with_label_values(&canary_label)
        .get();
    record_rewind_capture(RewindDomain::Git, TurnHookOutcome::Cancelled);
    observe_rewind_capture_duration(RewindDomain::Hunk, 0.002);
    record_rewind_restore(RewindDomain::Fs, true);
    record_rewind_restore(RewindDomain::Git, false);
    record_fs_finalize(TurnHookOutcome::Completed, 0.001);
    record_non_completed_finalize_canary(TurnHookOutcome::Error);
    assert!(
        REWIND_CHECKPOINT_CAPTURE_TOTAL
            .with_label_values(&capture_labels)
            .get()
            > capture_before,
        "capture counter must advance"
    );
    assert!(
        REWIND_RESTORE_TOTAL
            .with_label_values(&restore_labels)
            .get()
            > restore_before,
        "restore counter must advance"
    );
    assert!(
        REWIND_NON_COMPLETED_FINALIZE_TOTAL
            .with_label_values(&canary_label)
            .get()
            > canary_before,
        "canary counter must advance"
    );
}
/// The client ext-notification sink is invoked with the emitted method and params, and is no-op until installed.
#[tokio::test]
async fn client_ext_sink_receives_emitted_notification() {
    let handle = make_handle();
    assert!(!handle.has_client_ext_sink());
    handle.emit_client_ext("deepseek-build/noop".to_string(), serde_json::json!({}));
    let captured = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let sink_captured = captured.clone();
    handle.set_client_ext_sink(Arc::new(move |method, params| {
        sink_captured.lock().push((method, params));
    }));
    assert!(handle.has_client_ext_sink());
    handle.emit_client_ext(
        "deepseek-build/search/fuzzy/status".to_string(),
        serde_json::json!({"a": 1}),
    );
    let got = captured.lock();
    assert_eq!(got.len(), 1);
    let Some(first) = got.first() else {
        panic!("expected one captured emit: {got:?}");
    };
    assert_eq!(first.0, "deepseek-build/search/fuzzy/status");
    assert_eq!(first.1, serde_json::json!({"a": 1}));
}
/// End-to-end local streaming: open and change a fuzzy search over real files, then run the notification driver.
/// A correctly-shaped `deepseek-build/search/fuzzy/status` must be delivered through the sink with the match.
#[tokio::test]
async fn fuzzy_change_streams_status_through_sink() {
    use crate::file_system::TargetClientId;
    let handle = make_handle();
    let cwd = handle.root_cwd().unwrap();
    std::fs::write(cwd.join("alpha_widget.rs"), b"").unwrap();
    std::fs::write(cwd.join("beta_gadget.rs"), b"").unwrap();
    let captured = Arc::new(parking_lot::Mutex::new(Vec::<serde_json::Value>::new()));
    let sink_captured = captured.clone();
    handle.set_client_ext_sink(Arc::new(move |method, params| {
        if method == "deepseek-build/search/fuzzy/status" {
            sink_captured.lock().push(params);
        }
    }));
    let search_id = handle
        .fuzzy_open(
            Some(cwd.as_path()),
            None,
            false,
            Some("sess-1".into()),
            TargetClientId::None,
        )
        .await;
    let (min_gen, has_query, query_version) = handle
        .fuzzy_change(&search_id, "alpha_widget", false)
        .await
        .expect("search should exist");
    handle
        .run_fuzzy_notifications(search_id.clone(), min_gen, has_query, query_version, 50)
        .await;
    let got = captured.lock();
    assert!(
        !got.is_empty(),
        "expected at least one fuzzy status notification"
    );
    let last = got.last().unwrap();
    assert_eq!(
        last.get("sessionId").unwrap_or(&serde_json::Value::Null),
        "sess-1"
    );
    assert_eq!(
        last.get("searchId").unwrap_or(&serde_json::Value::Null),
        &serde_json::json!(search_id)
    );
    let matches = last
        .get("matches")
        .unwrap_or(&serde_json::Value::Null)
        .as_array()
        .expect("matches array");
    assert!(
        matches.iter().any(|m| m
            .get("path")
            .unwrap_or(&serde_json::Value::Null)
            .as_str()
            .is_some_and(|p| p.contains("alpha_widget"))),
        "expected alpha_widget in matches, got: {last}"
    );
}
/// Like [`make_handle`] but with `events_enabled = true` and a known `workspace_home` (the returned `TempDir`).
/// Tests can then read the per-session `events.jsonl`.
/// The flag goes through the private `build` path, not the env var, so the assertion never races a sibling test's process environment.
pub(crate) fn make_handle_with_events() -> (WorkspaceHandle, tempfile::TempDir) {
    let factory = Arc::new(TestSessionContextFactory::new());
    let cwd = factory.temp.path().to_path_buf();
    let config = WorkspaceConfig {
        root_cwd: cwd,
        default_tool_config: baseline_config(),
        respect_gitignore: false,
        memory_config: None,
        event_buffer_capacity: DEFAULT_EVENT_BUFFER_CAPACITY,
        session_factory: factory,
        hook_global_sources: vec![],
        hook_project_sources: vec![],
        skills_config: Default::default(),
        plugin_discovery_config: Default::default(),
        server_metadata: None,
        status_config: Default::default(),
        project_lsp_trusted: true,
        require_explicit_toolset: false,
        confine_fs_to_workspace_root: false,
        host_kind: Default::default(),
        bind_mcp: None,
        tool_approval: crate::permission::ToolApprovalGate::Off,
        sandbox: None,
    };
    let home = tempfile::tempdir().unwrap();
    let handle = WorkspaceHandle::build(
        config,
        home.path().to_path_buf(),
        false,
        true,
        false,
        false,
        crate::identity::WorkspaceIdentity::default(),
    )
    .expect("handle construction should succeed");
    (handle, home)
}
/// Full wiring: a turn with a tool call, the volatile-config toggles, and a representative `Mcp*` event all land in the per-session `events.jsonl`.
/// Their field content is truthful.
#[tokio::test]
async fn events_jsonl_captures_turn_tool_toggle_and_mcp_variants() {
    use xai_grok_session_events::ToolOutcome;
    use xai_tool_protocol::turn_hook::{AfterTurnPayload, BeforeTurnPayload, TurnHookOutcome};
    let (handle, home) = make_handle_with_events();
    let sid = "sess-int";
    handle
        .on_before_turn(
            sid,
            &BeforeTurnPayload {
                turn_number: 7,
                model_id: "deepseek-4".to_owned(),
                yolo_mode: false,
                conversation_message_count: 5,
                session_relationship: "subagent".to_owned(),
                schema_version: "1.0".to_owned(),
            },
        )
        .await;
    let tracker = handle.activity_tracker();
    tracker.tool_call_started("c1", "read_file", Some(sid));
    tracker.tool_call_completed("c1", Some(sid), ToolOutcome::Success);
    handle.on_yolo_toggled(sid, true);
    handle.on_mcp_server_toggled(sid, "linear", false);
    handle.shared().session_event_writer(sid).emit(
        xai_grok_session_events::Event::McpToolCallStarted {
            server_name: "linear".into(),
            tool_name: "list_issues".into(),
            call_id: "mcp-1".into(),
            timeout_sec: 30,
        },
    );
    handle
        .on_after_turn(
            sid,
            &AfterTurnPayload {
                turn_number: 7,
                outcome: TurnHookOutcome::Completed,
                duration_ms: 1234,
                tool_call_count: 1,
                model_id: "deepseek-4".to_owned(),
                written_repo_paths: Vec::new(),
                cancellation_category: None,
                cancellation_context: None,
            },
        )
        .await;
    let path = home.path().join("sessions").join(sid).join("events.jsonl");
    let text = std::fs::read_to_string(&path).expect("events.jsonl must exist");
    let events: Vec<serde_json::Value> = text
        .trim()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let by_type = |t: &str| {
        events
            .iter()
            .find(|e| e.get("type").unwrap_or(&serde_json::Value::Null) == t)
            .unwrap_or_else(|| panic!("{t} event missing from events.jsonl"))
    };
    let ts = by_type("turn_started");
    assert_eq!(
        ts.get("session_id").unwrap_or(&serde_json::Value::Null),
        sid
    );
    assert_eq!(ts.get("turn_number").unwrap_or(&serde_json::Value::Null), 7);
    assert_eq!(
        ts.get("model_id").unwrap_or(&serde_json::Value::Null),
        "deepseek-4"
    );
    assert_eq!(
        ts.get("yolo_mode").unwrap_or(&serde_json::Value::Null),
        false
    );
    assert_eq!(
        ts.get("conversation_message_count")
            .unwrap_or(&serde_json::Value::Null),
        5
    );
    assert_eq!(
        ts.get("session_relationship")
            .unwrap_or(&serde_json::Value::Null),
        "subagent"
    );
    assert_eq!(
        ts.get("schema_version").unwrap_or(&serde_json::Value::Null),
        "1.0"
    );
    assert_eq!(
        by_type("tool_started")
            .get("tool_name")
            .unwrap_or(&serde_json::Value::Null),
        "read_file"
    );
    let tc = by_type("tool_completed");
    assert_eq!(
        tc.get("tool_name").unwrap_or(&serde_json::Value::Null),
        "read_file"
    );
    assert_eq!(
        tc.get("outcome").unwrap_or(&serde_json::Value::Null),
        "success"
    );
    assert_eq!(
        by_type("yolo_toggled")
            .get("enabled")
            .unwrap_or(&serde_json::Value::Null),
        true
    );
    let mcp_toggle = by_type("mcp_server_toggled");
    assert_eq!(
        mcp_toggle
            .get("server_name")
            .unwrap_or(&serde_json::Value::Null),
        "linear"
    );
    assert_eq!(
        mcp_toggle
            .get("enabled")
            .unwrap_or(&serde_json::Value::Null),
        false
    );
    let mcp_call = by_type("mcp_tool_call_started");
    assert_eq!(
        mcp_call
            .get("server_name")
            .unwrap_or(&serde_json::Value::Null),
        "linear"
    );
    assert_eq!(
        mcp_call
            .get("tool_name")
            .unwrap_or(&serde_json::Value::Null),
        "list_issues"
    );
    assert_eq!(
        by_type("turn_ended")
            .get("outcome")
            .unwrap_or(&serde_json::Value::Null),
        "completed"
    );
    let pos = |t: &str| {
        events
            .iter()
            .position(|e| e.get("type").unwrap_or(&serde_json::Value::Null) == t)
            .unwrap()
    };
    assert!(
        pos("turn_started") < pos("tool_started"),
        "turn_started must precede tool_started"
    );
    assert!(
        pos("tool_completed") < pos("turn_ended"),
        "tool_completed must precede turn_ended"
    );
}
/// Both before-turn hook delivery styles sync YOLO state into the session.
#[tokio::test]
async fn before_turn_hooks_sync_session_yolo_mode() {
    use xai_tool_protocol::turn_hook::{BeforeTurnPayload, TurnHookRequest};
    let handle = make_handle();
    let session = handle.session("main").expect("main session");
    assert!(!session.yolo_mode(), "fail-closed default");
    handle
        .on_before_turn(
            "main",
            &BeforeTurnPayload {
                turn_number: 1,
                model_id: "deepseek-4".to_owned(),
                yolo_mode: true,
                ..Default::default()
            },
        )
        .await;
    assert!(session.yolo_mode(), "on_before_turn must sync yolo on");
    let reply = handle
        .compute_turn_injections(
            "main",
            &TurnHookRequest::Before(BeforeTurnPayload {
                turn_number: 2,
                model_id: "deepseek-4".to_owned(),
                yolo_mode: false,
                ..Default::default()
            }),
        )
        .await;
    assert_eq!(
        reply,
        xai_tool_protocol::turn_hook::HookReply::default(),
        "reply stays a behavior-neutral no-op"
    );
    assert!(
        !session.yolo_mode(),
        "compute_turn_injections must sync yolo off"
    );
    handle
        .compute_turn_injections(
            "never-bound",
            &TurnHookRequest::Before(BeforeTurnPayload {
                turn_number: 1,
                model_id: "deepseek-4".to_owned(),
                yolo_mode: true,
                ..Default::default()
            }),
        )
        .await;
}
/// YOLO transitions emit `yolo_toggled` in events.jsonl; repeats don't.
#[tokio::test]
async fn before_turn_yolo_transition_emits_yolo_toggled_event() {
    use xai_tool_protocol::turn_hook::BeforeTurnPayload;
    let (handle, home) = make_handle_with_events();
    let sid = "sess-yolo";
    let _session = handle
        .create_session_with_config(sid, None, None, CapabilityMode::All, None, false)
        .expect("create session");
    for (turn, yolo) in [(1, true), (2, true), (3, false)] {
        handle
            .on_before_turn(
                sid,
                &BeforeTurnPayload {
                    turn_number: turn,
                    model_id: "deepseek-4".to_owned(),
                    yolo_mode: yolo,
                    ..Default::default()
                },
            )
            .await;
    }
    let path = home.path().join("sessions").join(sid).join("events.jsonl");
    let text = std::fs::read_to_string(&path).expect("events.jsonl must exist");
    let toggles: Vec<bool> = text
        .trim()
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
        .filter(|e| e.get("type").unwrap_or(&serde_json::Value::Null) == "yolo_toggled")
        .map(|e| {
            e.get("enabled")
                .unwrap_or(&serde_json::Value::Null)
                .as_bool()
                .unwrap()
        })
        .collect();
    assert_eq!(
        toggles,
        vec![true, false],
        "exactly one toggle per transition (turn 2 repeats true → no re-emit)"
    );
    let turn_yolo: Vec<bool> = text
        .trim()
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
        .filter(|e| e.get("type").unwrap_or(&serde_json::Value::Null) == "turn_started")
        .map(|e| {
            e.get("yolo_mode")
                .unwrap_or(&serde_json::Value::Null)
                .as_bool()
                .unwrap()
        })
        .collect();
    assert_eq!(
        turn_yolo,
        vec![true, true, false],
        "turn_started must carry the per-turn yolo state"
    );
}
/// Flag-off preservation: `WorkspaceHandle::new` resolves `events_enabled` from the (unset) env var, so the whole emission path must stay a noop.
/// It caches no session writers and creates no `sessions/` dir.
#[tokio::test]
async fn events_disabled_keeps_noop_and_writes_nothing() {
    use xai_grok_session_events::ToolOutcome;
    use xai_tool_protocol::turn_hook::{AfterTurnPayload, BeforeTurnPayload, TurnHookOutcome};
    let handle = make_handle();
    assert!(
        !handle.shared().events_enabled,
        "test precondition: events must be disabled"
    );
    let sid = "main";
    handle
        .on_before_turn(
            sid,
            &BeforeTurnPayload {
                turn_number: 1,
                model_id: "deepseek-4".to_owned(),
                yolo_mode: false,
                conversation_message_count: 0,
                session_relationship: "primary".to_owned(),
                schema_version: "1.0".to_owned(),
            },
        )
        .await;
    let tracker = handle.activity_tracker();
    tracker.tool_call_started("c1", "read_file", Some(sid));
    tracker.tool_call_completed("c1", Some(sid), ToolOutcome::Success);
    handle.on_yolo_toggled(sid, true);
    handle.on_mcp_server_toggled(sid, "linear", true);
    handle
        .on_after_turn(
            sid,
            &AfterTurnPayload {
                turn_number: 1,
                outcome: TurnHookOutcome::Completed,
                duration_ms: 1,
                tool_call_count: 1,
                model_id: "deepseek-4".to_owned(),
                written_repo_paths: Vec::new(),
                cancellation_category: None,
                cancellation_context: None,
            },
        )
        .await;
    assert!(
        handle.shared().session_event_writers.is_empty(),
        "flag-off must not cache any session writer (EventWriter::noop preserved)"
    );
    let sessions_dir = handle.shared().workspace_home().join("sessions");
    assert!(
        !sessions_dir.exists(),
        "flag-off must not create the sessions dir or any events.jsonl"
    );
}
/// `on_session_ended` must evict the session's `events.jsonl` writer from the shared map (releasing the open file descriptor).
/// Events already written to disk must survive.
#[tokio::test]
async fn session_end_evicts_event_writer_without_data_loss() {
    use xai_tool_protocol::turn_hook::BeforeTurnPayload;
    let (handle, home) = make_handle_with_events();
    let sid = "sess-evict";
    handle
        .on_before_turn(
            sid,
            &BeforeTurnPayload {
                turn_number: 1,
                model_id: "deepseek-4".to_owned(),
                yolo_mode: false,
                conversation_message_count: 0,
                session_relationship: "primary".to_owned(),
                schema_version: "1.0".to_owned(),
            },
        )
        .await;
    assert!(
        handle.shared().session_event_writers.contains_key(sid),
        "writer must be cached after the turn opens it"
    );
    let path = home.path().join("sessions").join(sid).join("events.jsonl");
    let before = std::fs::read_to_string(&path).unwrap();
    assert!(
        before.contains("turn_started"),
        "TurnStarted must be persisted before eviction"
    );
    handle.on_session_ended(sid);
    assert!(
        !handle.shared().session_event_writers.contains_key(sid),
        "writer must be evicted from the map on session end (fd released)"
    );
    let after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        before, after,
        "evicting the writer must not lose already-written events"
    );
}
/// `on_session_ended` evicts the session's tool-defs debounce entry (no per-session leak in a long-lived hub server).
#[tokio::test]
async fn session_end_evicts_tool_defs_debounce_entry() {
    let handle = make_handle();
    let sid = "sess-tool-defs-evict";
    assert!(tool_defs_reemit_gate(
        true,
        &handle.shared().tool_defs_last_emit,
        sid,
        std::time::Instant::now(),
        TOOL_DEFS_DEBOUNCE,
    ));
    assert!(
        handle.shared().tool_defs_last_emit.contains_key(sid),
        "debounce entry must be recorded after a gated re-emit"
    );
    handle.on_session_ended(sid);
    assert!(
        !handle.shared().tool_defs_last_emit.contains_key(sid),
        "debounce entry must be evicted on session end (no per-session leak)"
    );
}
/// The RPC `drop_session` path evicts the debounce entry like `on_session_ended` does.
#[tokio::test]
async fn drop_session_evicts_tool_defs_debounce_entry() {
    let handle = make_handle();
    let sid = "main";
    assert!(tool_defs_reemit_gate(
        true,
        &handle.shared().tool_defs_last_emit,
        sid,
        std::time::Instant::now(),
        TOOL_DEFS_DEBOUNCE,
    ));
    handle.drop_session(sid, sid).expect("drop main session");
    assert!(
        !handle.shared().tool_defs_last_emit.contains_key(sid),
        "drop_session must evict the debounce entry"
    );
}
/// Object-key segment safety: separators, traversal, and NUL are refused.
#[test]
fn is_safe_object_segment_rejects_traversal() {
    assert!(is_safe_object_segment("sess-1_a"));
    assert!(!is_safe_object_segment(""));
    assert!(!is_safe_object_segment("a/b"));
    assert!(!is_safe_object_segment("a\\b"));
    assert!(!is_safe_object_segment("../etc"));
    assert!(!is_safe_object_segment("a\0b"));
}
/// The single mapping from `TurnHookOutcome` to `TurnOutcomeLabel` used by `on_after_turn` must be exhaustive and stable.
#[test]
fn turn_outcome_label_maps_every_variant() {
    use xai_grok_session_events::TurnOutcomeLabel;
    use xai_tool_protocol::turn_hook::TurnHookOutcome;
    assert!(matches!(
        turn_outcome_label(TurnHookOutcome::Completed),
        TurnOutcomeLabel::Completed
    ));
    assert!(matches!(
        turn_outcome_label(TurnHookOutcome::Cancelled),
        TurnOutcomeLabel::Cancelled
    ));
    assert!(matches!(
        turn_outcome_label(TurnHookOutcome::Error),
        TurnOutcomeLabel::Error
    ));
}
pub(crate) fn fork_cfg_with(
    agent_id: &str,
    capability: CapabilityMode,
    tool_config: Option<ToolServerConfig>,
    parent: Option<&str>,
) -> AgentSessionConfig {
    let mut c = AgentSessionConfig::new(agent_id);
    c.capability_mode = capability;
    c.tool_config = tool_config;
    c.parent_session_id = parent.map(|p| p.to_owned());
    c
}
/// `WorkspaceHandle::new` (the test/default path, not `connect_local_workspace`) must use an ephemeral temp `workspace_home`.
/// It must never use the real `$GROK_WORKSPACE_HOME`, so tests stay hermetic.
#[tokio::test]
async fn new_defaults_to_ephemeral_home() {
    let handle = make_handle();
    let shared = handle.shared();
    let home = shared.workspace_home();
    assert!(
        home.starts_with(std::env::temp_dir()),
        "default workspace_home must live under the temp dir, got {}",
        home.display()
    );
    assert_ne!(
        home,
        resolve_workspace_home(),
        "default construction must NOT use the real $GROK_WORKSPACE_HOME"
    );
}
#[tokio::test]
async fn fork_session_inherits_parent_tool_config_when_none() {
    let handle = make_handle();
    let parent = handle.session("main").expect("main session present");
    let parent_baseline = parent.effective_tool_config();
    let parent_ids: Vec<String> = parent_baseline.tools.iter().map(|t| t.id.clone()).collect();
    let child = handle
        .fork_session(fork_cfg_with(
            "child",
            CapabilityMode::ReadWrite,
            None,
            Some("main"),
        ))
        .await
        .expect("fork should succeed");
    let child_baseline = child.effective_tool_config();
    let child_ids: Vec<String> = child_baseline.tools.iter().map(|t| t.id.clone()).collect();
    assert_eq!(child_ids, parent_ids);
    let new_parent_baseline = ToolServerConfig {
        tools: vec![tc("GrokBuild:read_file", Some(ToolKind::Read))],
        behavior_preset: None,
    };
    let factory = handle.shared.session_factory.clone();
    let mcp_snapshot = handle.shared.mcp_tools_snapshot.load_full();
    let (eff, ts, _backend) = resolve_session_toolset(
        new_parent_baseline,
        parent.capability_mode(),
        &mcp_snapshot,
        &[],
        parent.cwd().to_path_buf(),
        parent.session_env().clone(),
        "main",
        factory.as_ref(),
        None,
        None,
        None,
        None,
    )
    .expect("re-resolve should succeed");
    parent.replace(Arc::new(eff), ts);
    let child_after: Vec<String> = child
        .effective_tool_config()
        .tools
        .iter()
        .map(|t| t.id.clone())
        .collect();
    assert_eq!(
        child_after, child_ids,
        "child baseline must not change when parent is mutated"
    );
}
#[tokio::test]
async fn fork_session_uses_explicit_tool_config_when_provided() {
    let handle = make_handle();
    let custom = ToolServerConfig {
        tools: vec![
            tc("GrokBuild:read_file", Some(ToolKind::Read)),
            tc("GrokBuild:list_dir", Some(ToolKind::ListDir)),
        ],
        behavior_preset: None,
    };
    let child = handle
        .fork_session(fork_cfg_with(
            "explicit",
            CapabilityMode::ReadWrite,
            Some(custom.clone()),
            Some("main"),
        ))
        .await
        .expect("fork should succeed");
    let baseline_ids: Vec<String> = child
        .effective_tool_config()
        .tools
        .iter()
        .map(|t| t.id.clone())
        .collect();
    let custom_ids: Vec<String> = custom.tools.iter().map(|t| t.id.clone()).collect();
    assert_eq!(baseline_ids, custom_ids);
}
#[tokio::test]
async fn fork_session_uses_main_session_when_parent_session_id_is_none() {
    let handle = make_handle();
    let marker_config = ToolServerConfig {
        tools: vec![tc("GrokBuild:read_file", Some(ToolKind::Read))],
        behavior_preset: None,
    };
    let main = handle.session("main").expect("main present");
    let factory = handle.shared.session_factory.clone();
    let mcp_snapshot = handle.shared.mcp_tools_snapshot.load_full();
    let (eff, ts, _backend) = resolve_session_toolset(
        marker_config,
        main.capability_mode(),
        &mcp_snapshot,
        &[],
        main.cwd().to_path_buf(),
        main.session_env().clone(),
        "main",
        factory.as_ref(),
        None,
        None,
        None,
        None,
    )
    .expect("re-resolve should succeed");
    main.replace(Arc::new(eff), ts);
    let child = handle
        .fork_session(fork_cfg_with(
            "child",
            CapabilityMode::ReadWrite,
            None,
            Some("main"),
        ))
        .await
        .expect("fork should succeed");
    let baseline_ids: Vec<String> = child
        .effective_tool_config()
        .tools
        .iter()
        .map(|t| t.id.clone())
        .collect();
    assert_eq!(baseline_ids, vec!["GrokBuild:read_file".to_string()]);
}
#[tokio::test]
async fn fork_session_all_child_drops_root_only_tools() {
    let handle = make_handle();
    let mut custom = tc("GrokBuild:grep", None);
    custom.name_override = Some("custom_kindless".to_owned());
    let config = ToolServerConfig {
        tools: vec![
            tc("GrokBuild:read_file", Some(ToolKind::Read)),
            custom,
            tc("GrokBuild:list_dir", Some(ToolKind::ActiveAgentMessage)),
            tc("GrokBuild:send_subagent_message", None),
        ],
        behavior_preset: None,
    };
    let child = handle
        .fork_session(fork_cfg_with(
            "all-child",
            CapabilityMode::All,
            Some(config),
            Some("main"),
        ))
        .await
        .expect("All child fork should succeed");
    let effective_config = child.effective_tool_config();
    let ids: Vec<&str> = effective_config
        .tools
        .iter()
        .map(|tool| tool.id.as_str())
        .collect();
    assert_eq!(ids, ["GrokBuild:read_file", "GrokBuild:grep"]);
    let Some(tool1) = effective_config.tools.get(1) else {
        panic!("expected second tool: {:?}", effective_config.tools);
    };
    assert_eq!(
        (tool1.name_override.as_deref(), tool1.kind),
        (Some("custom_kindless"), None)
    );
}
#[tokio::test]
async fn fork_session_uses_named_parent_when_parent_session_id_is_set() {
    let handle = make_handle();
    let custom = ToolServerConfig {
        tools: vec![tc("GrokBuild:read_file", Some(ToolKind::Read))],
        behavior_preset: None,
    };
    handle
        .fork_session(fork_cfg_with(
            "intermediate",
            CapabilityMode::ReadWrite,
            Some(custom.clone()),
            Some("main"),
        ))
        .await
        .expect("intermediate fork should succeed");
    let leaf = handle
        .fork_session(fork_cfg_with(
            "leaf",
            CapabilityMode::ReadWrite,
            None,
            Some("intermediate"),
        ))
        .await
        .expect("leaf fork should succeed");
    let baseline_ids: Vec<String> = leaf
        .effective_tool_config()
        .tools
        .iter()
        .map(|t| t.id.clone())
        .collect();
    let custom_ids: Vec<String> = custom.tools.iter().map(|t| t.id.clone()).collect();
    assert_eq!(baseline_ids, custom_ids);
}
#[test]
fn fork_session_concurrent_same_id_only_one_winner() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .enable_all()
        .build()
        .expect("runtime");
    let _g = rt.enter();
    let handle = Arc::new(make_handle());
    let mut handles = vec![];
    for _ in 0..16 {
        let h = handle.clone();
        let g = rt.handle().clone();
        handles.push(std::thread::spawn(move || {
            g.block_on(h.fork_session({
                let mut c = AgentSessionConfig::new("racer");
                c.parent_session_id = Some("main".into());
                c
            }))
        }));
    }
    let mut wins = 0;
    let mut losses = 0;
    for jh in handles {
        let res = jh.join().expect("thread panic");
        match res {
            Ok(_) => wins += 1,
            Err(WorkspaceError::SessionAlreadyExists(id)) => {
                assert_eq!(id, "racer");
                losses += 1;
            }
            Err(other) => panic!("unexpected error: {other:?}"),
        }
    }
    assert_eq!(wins, 1, "exactly one fork must succeed");
    assert_eq!(losses, 15, "the other 15 must see SessionAlreadyExists");
}
#[tokio::test]
async fn fork_session_empty_agent_id_rejected() {
    let handle = make_handle();
    let err = handle
        .fork_session({
            let mut c = AgentSessionConfig::new("");
            c.parent_session_id = Some("main".into());
            c
        })
        .await
        .expect_err("empty agent_id must error");
    assert!(matches!(err, WorkspaceError::EmptyAgentId), "got {err:?}");
}
#[tokio::test]
async fn fork_session_capability_widening_rejected() {
    let handle = make_handle();
    handle
        .fork_session(fork_cfg_with(
            "ro",
            CapabilityMode::ReadOnly,
            None,
            Some("main"),
        ))
        .await
        .expect("readonly fork ok");
    let err = handle
        .fork_session(fork_cfg_with(
            "widen",
            CapabilityMode::All,
            None,
            Some("ro"),
        ))
        .await
        .expect_err("widening must error");
    assert!(
        matches!(
            err,
            WorkspaceError::CapabilityWidening {
                parent: CapabilityMode::ReadOnly,
                child: CapabilityMode::All
            }
        ),
        "got {err:?}"
    );
}
/// A fork that races a terminal drain must be rejected by the same shutdown gate as `create_session`.
/// Otherwise it could repopulate the session map while the shared upload queue is being flushed/closed.
#[tokio::test]
async fn fork_session_rejected_while_draining() {
    let handle = make_handle();
    handle.activity_tracker().set_draining();
    let err = handle
        .fork_session(fork_cfg_with(
            "child",
            CapabilityMode::ReadWrite,
            None,
            Some("main"),
        ))
        .await
        .expect_err("fork must be rejected while draining");
    assert!(matches!(err, WorkspaceError::ShuttingDown), "got {err:?}");
}
#[tokio::test]
async fn fork_session_capability_widening_readwrite_to_execute_rejected() {
    let handle = make_handle();
    handle
        .fork_session(fork_cfg_with(
            "rw",
            CapabilityMode::ReadWrite,
            None,
            Some("main"),
        ))
        .await
        .expect("rw fork ok");
    let err = handle
        .fork_session(fork_cfg_with(
            "exe",
            CapabilityMode::Execute,
            None,
            Some("rw"),
        ))
        .await
        .expect_err("incomparable widen must error");
    assert!(matches!(err, WorkspaceError::CapabilityWidening { .. }));
}
#[tokio::test]
async fn fork_session_max_depth_rejected_when_budget_zero() {
    let handle = make_handle();
    let mut cfg = AgentSessionConfig::new("budgeted");
    cfg.parent_session_id = Some("main".into());
    cfg.max_depth = 0;
    let child = handle.fork_session(cfg).await.expect("budgeted fork ok");
    assert_eq!(child.fork_budget(), 0);
    let err = handle
        .fork_session(fork_cfg_with(
            "grandchild",
            CapabilityMode::ReadWrite,
            None,
            Some("budgeted"),
        ))
        .await
        .expect_err("further fork must error");
    assert!(matches!(err, WorkspaceError::MaxDepthExceeded { .. }));
}
#[tokio::test]
async fn fork_session_parent_session_not_found_errors() {
    let handle = make_handle();
    let mut cfg = AgentSessionConfig::new("orphan");
    cfg.parent_session_id = Some("ghost".into());
    let err = handle
        .fork_session(cfg)
        .await
        .expect_err("missing parent must error");
    match err {
        WorkspaceError::ParentSessionNotFound(id) => assert_eq!(id, "ghost"),
        other => panic!("unexpected: {other:?}"),
    }
}
#[tokio::test]
async fn fork_session_finalize_error_propagated() {
    let handle = make_handle();
    let bad = ToolServerConfig {
        tools: vec![tc("DoesNotExist:nope", Some(ToolKind::Read))],
        behavior_preset: None,
    };
    let cfg = fork_cfg_with("bogus", CapabilityMode::ReadOnly, Some(bad), Some("main"));
    let err = handle
        .fork_session(cfg)
        .await
        .expect_err("bogus id must error");
    assert!(matches!(err, WorkspaceError::Finalize(_)), "got {err:?}");
}
#[tokio::test]
async fn fork_session_extra_env_layered_on_parent() {
    let handle = make_handle();
    let mut intermediate_cfg = AgentSessionConfig::new("parent_env");
    intermediate_cfg
        .extra_env
        .insert("INHERITED".into(), "from_parent".into());
    intermediate_cfg
        .extra_env
        .insert("OVERRIDDEN".into(), "old_value".into());
    intermediate_cfg.parent_session_id = Some("main".into());
    let parent = handle
        .fork_session(intermediate_cfg)
        .await
        .expect("parent ok");
    assert_eq!(
        parent.session_env().get("INHERITED").map(String::as_str),
        Some("from_parent")
    );
    let mut child_cfg = AgentSessionConfig::new("child_env");
    child_cfg.parent_session_id = Some("parent_env".into());
    child_cfg
        .extra_env
        .insert("OVERRIDDEN".into(), "new_value".into());
    child_cfg
        .extra_env
        .insert("CHILD_ONLY".into(), "yes".into());
    let child = handle.fork_session(child_cfg).await.expect("child ok");
    assert_eq!(
        child.session_env().get("INHERITED").map(String::as_str),
        Some("from_parent"),
        "parent var must be inherited"
    );
    assert_eq!(
        child.session_env().get("OVERRIDDEN").map(String::as_str),
        Some("new_value"),
        "extra_env must override parent var"
    );
    assert_eq!(
        child.session_env().get("CHILD_ONLY").map(String::as_str),
        Some("yes"),
        "extra_env must add new var"
    );
}
#[tokio::test]
async fn fork_session_cwd_override_used_when_set() {
    let handle = make_handle();
    let alt = std::env::temp_dir().join("xai-grok-workspace-test-cwd-override");
    std::fs::create_dir_all(&alt).expect("create alt cwd");
    let mut cfg = AgentSessionConfig::new("cwdchild");
    cfg.cwd_override = Some(alt.clone());
    cfg.parent_session_id = Some("main".into());
    let child = handle.fork_session(cfg).await.expect("ok");
    assert_eq!(child.cwd(), alt);
}
#[tokio::test]
async fn fork_session_inheritance_arc_distinct() {
    let handle = make_handle();
    let main = handle.session("main").expect("main");
    let child = handle
        .fork_session({
            let mut c = AgentSessionConfig::new("kid");
            c.parent_session_id = Some("main".into());
            c
        })
        .await
        .expect("ok");
    assert!(
        !Arc::ptr_eq(
            &main.effective_tool_config(),
            &child.effective_tool_config()
        ),
        "child must hold its own Arc<ToolServerConfig>"
    );
    assert!(
        !Arc::ptr_eq(&main.toolset(), &child.toolset()),
        "child must hold its own Arc<FinalizedToolset>"
    );
}
#[tokio::test]
async fn fork_session_empty_baseline_tools_succeeds() {
    let handle = make_handle();
    let empty = ToolServerConfig {
        tools: vec![],
        behavior_preset: None,
    };
    let child = handle
        .fork_session(fork_cfg_with(
            "empty",
            CapabilityMode::ReadOnly,
            Some(empty),
            Some("main"),
        ))
        .await
        .expect("empty tool set is valid");
    assert!(child.toolset().tool_definitions().is_empty());
}
#[tokio::test]
async fn hook_registry_empty_when_no_sources() {
    let handle = make_handle();
    let registry = handle.hook_registry();
    assert!(registry.is_empty(), "no sources => empty registry");
    assert!(
        handle.hook_load_errors().is_empty(),
        "no sources => no errors"
    );
}
#[tokio::test]
async fn hook_registry_loads_from_settings_file() {
    let factory = Arc::new(TestSessionContextFactory::new());
    let cwd = factory.temp.path().to_path_buf();
    let settings_path = cwd.join("claude_settings.json");
    std::fs::write(
        &settings_path,
        r#"{"hooks":{"PreToolUse":[{"hooks":[{"type":"command","command":"echo ok"}]}]}}"#,
    )
    .expect("write settings");
    let config = WorkspaceConfig {
        root_cwd: cwd,
        default_tool_config: baseline_config(),
        respect_gitignore: false,
        memory_config: None,
        event_buffer_capacity: DEFAULT_EVENT_BUFFER_CAPACITY,
        session_factory: factory,
        hook_global_sources: vec![HookSourceConfig::SettingsFile(settings_path)],
        hook_project_sources: vec![],
        skills_config: Default::default(),
        plugin_discovery_config: Default::default(),
        server_metadata: None,
        status_config: Default::default(),
        project_lsp_trusted: true,
        require_explicit_toolset: false,
        confine_fs_to_workspace_root: false,
        host_kind: Default::default(),
        bind_mcp: None,
        tool_approval: crate::permission::ToolApprovalGate::Off,
        sandbox: None,
    };
    let handle = WorkspaceHandle::new(config).expect("ok");
    let registry = handle.hook_registry();
    assert!(!registry.is_empty(), "settings file should yield hooks");
    assert!(handle.hook_load_errors().is_empty());
}
#[tokio::test]
async fn hook_registry_loads_from_directory() {
    let factory = Arc::new(TestSessionContextFactory::new());
    let cwd = factory.temp.path().to_path_buf();
    let hooks_dir = cwd.join("hooks");
    std::fs::create_dir_all(&hooks_dir).expect("mkdir");
    std::fs::write(
        hooks_dir.join("my_hook.json"),
        r#"{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"echo hi"}]}]}}"#,
    )
    .expect("write hook file");
    let config = WorkspaceConfig {
        root_cwd: cwd,
        default_tool_config: baseline_config(),
        respect_gitignore: false,
        memory_config: None,
        event_buffer_capacity: DEFAULT_EVENT_BUFFER_CAPACITY,
        session_factory: factory,
        hook_global_sources: vec![],
        hook_project_sources: vec![HookSourceConfig::Directory(hooks_dir)],
        skills_config: Default::default(),
        plugin_discovery_config: Default::default(),
        server_metadata: None,
        status_config: Default::default(),
        project_lsp_trusted: true,
        require_explicit_toolset: false,
        confine_fs_to_workspace_root: false,
        host_kind: Default::default(),
        bind_mcp: None,
        tool_approval: crate::permission::ToolApprovalGate::Off,
        sandbox: None,
    };
    let handle = WorkspaceHandle::new(config).expect("ok");
    let registry = handle.hook_registry();
    assert!(!registry.is_empty(), "directory source should yield hooks");
}
#[tokio::test]
async fn hook_registry_snapshot_is_disconnected() {
    let handle = make_handle();
    let snap1 = handle.hook_registry();
    assert!(snap1.is_empty());
    {
        let spec = xai_grok_hooks::config::HookSpec {
            name: "injected".into(),
            event: xai_grok_hooks::event::HookEventName::SessionStart,
            handler_type: xai_grok_hooks::config::HandlerType::Command,
            configured_matcher: None,
            matcher: None,
            enabled: true,
            command: Some("echo injected".into()),
            command_raw: Some("echo injected".into()),
            url: None,
            url_raw: None,
            timeout_ms: 10_000,
            source_dir: std::path::PathBuf::from("/tmp"),
            extra_env: std::collections::HashMap::new(),
            layer: xai_grok_hooks::config::HookProvenance::File,
        };
        handle.shared.hook_registry.write().append_specs(vec![spec]);
    }
    assert!(snap1.is_empty(), "snapshot must not see live mutations");
    let snap2 = handle.hook_registry();
    assert!(!snap2.is_empty(), "fresh snapshot must see mutation");
}
#[tokio::test]
async fn hook_load_errors_reported_for_bad_file() {
    let factory = Arc::new(TestSessionContextFactory::new());
    let cwd = factory.temp.path().to_path_buf();
    let bad_path = cwd.join("bad_settings.json");
    std::fs::write(&bad_path, "NOT VALID JSON").expect("write bad file");
    let config = WorkspaceConfig {
        root_cwd: cwd,
        default_tool_config: baseline_config(),
        respect_gitignore: false,
        memory_config: None,
        event_buffer_capacity: DEFAULT_EVENT_BUFFER_CAPACITY,
        session_factory: factory,
        hook_global_sources: vec![HookSourceConfig::SettingsFile(bad_path)],
        hook_project_sources: vec![],
        skills_config: Default::default(),
        plugin_discovery_config: Default::default(),
        server_metadata: None,
        status_config: Default::default(),
        project_lsp_trusted: true,
        require_explicit_toolset: false,
        confine_fs_to_workspace_root: false,
        host_kind: Default::default(),
        bind_mcp: None,
        tool_approval: crate::permission::ToolApprovalGate::Off,
        sandbox: None,
    };
    let handle = WorkspaceHandle::new(config).expect("construction must still succeed");
    assert!(
        !handle.hook_load_errors().is_empty(),
        "bad JSON must produce load errors"
    );
}
#[tokio::test]
async fn hook_registry_global_and_project_sources_merge() {
    let factory = Arc::new(TestSessionContextFactory::new());
    let cwd = factory.temp.path().to_path_buf();
    let global_settings = cwd.join("global.json");
    std::fs::write(
        &global_settings,
        r#"{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"echo global"}]}]}}"#,
    )
    .expect("write");
    let project_settings = cwd.join("project.json");
    std::fs::write(
        &project_settings,
        r#"{"hooks":{"PreToolUse":[{"hooks":[{"type":"command","command":"echo project"}]}]}}"#,
    )
    .expect("write");
    let config = WorkspaceConfig {
        root_cwd: cwd,
        default_tool_config: baseline_config(),
        respect_gitignore: false,
        memory_config: None,
        event_buffer_capacity: DEFAULT_EVENT_BUFFER_CAPACITY,
        session_factory: factory,
        hook_global_sources: vec![HookSourceConfig::SettingsFile(global_settings)],
        hook_project_sources: vec![HookSourceConfig::SettingsFile(project_settings)],
        skills_config: Default::default(),
        plugin_discovery_config: Default::default(),
        server_metadata: None,
        status_config: Default::default(),
        project_lsp_trusted: true,
        require_explicit_toolset: false,
        confine_fs_to_workspace_root: false,
        host_kind: Default::default(),
        bind_mcp: None,
        tool_approval: crate::permission::ToolApprovalGate::Off,
        sandbox: None,
    };
    let handle = WorkspaceHandle::new(config).expect("ok");
    let registry = handle.hook_registry();
    assert_eq!(registry.len(), 2, "both sources must contribute hooks");
}
#[tokio::test]
async fn hook_registry_missing_source_is_non_fatal() {
    let factory = Arc::new(TestSessionContextFactory::new());
    let cwd = factory.temp.path().to_path_buf();
    let missing = cwd.join("does_not_exist.json");
    let config = WorkspaceConfig {
        root_cwd: cwd,
        default_tool_config: baseline_config(),
        respect_gitignore: false,
        memory_config: None,
        event_buffer_capacity: DEFAULT_EVENT_BUFFER_CAPACITY,
        session_factory: factory,
        hook_global_sources: vec![HookSourceConfig::SettingsFile(missing)],
        hook_project_sources: vec![],
        skills_config: Default::default(),
        plugin_discovery_config: Default::default(),
        server_metadata: None,
        status_config: Default::default(),
        project_lsp_trusted: true,
        require_explicit_toolset: false,
        confine_fs_to_workspace_root: false,
        host_kind: Default::default(),
        bind_mcp: None,
        tool_approval: crate::permission::ToolApprovalGate::Off,
        sandbox: None,
    };
    let handle = WorkspaceHandle::new(config).expect("must not panic on missing source");
    assert!(handle.hook_registry().is_empty());
    assert!(
        handle.hook_load_errors().is_empty(),
        "missing file should not produce errors"
    );
}
#[tokio::test]
async fn hook_registry_empty_directory_yields_empty_registry() {
    let factory = Arc::new(TestSessionContextFactory::new());
    let cwd = factory.temp.path().to_path_buf();
    let empty_dir = cwd.join("empty_hooks");
    std::fs::create_dir_all(&empty_dir).expect("mkdir");
    let config = WorkspaceConfig {
        root_cwd: cwd,
        default_tool_config: baseline_config(),
        respect_gitignore: false,
        memory_config: None,
        event_buffer_capacity: DEFAULT_EVENT_BUFFER_CAPACITY,
        session_factory: factory,
        hook_global_sources: vec![],
        hook_project_sources: vec![HookSourceConfig::Directory(empty_dir)],
        skills_config: Default::default(),
        plugin_discovery_config: Default::default(),
        server_metadata: None,
        status_config: Default::default(),
        project_lsp_trusted: true,
        require_explicit_toolset: false,
        confine_fs_to_workspace_root: false,
        host_kind: Default::default(),
        bind_mcp: None,
        tool_approval: crate::permission::ToolApprovalGate::Off,
        sandbox: None,
    };
    let handle = WorkspaceHandle::new(config).expect("ok");
    assert!(handle.hook_registry().is_empty());
    assert!(handle.hook_load_errors().is_empty());
}
#[test]
fn startup_stage_observe_records_independent_samples() {
    let recovery_before = super::STARTUP_STAGE_DURATION_SECONDS
        .with_label_values(&[
            super::STARTUP_STAGE_STARTUP_RECOVERY,
            super::STARTUP_OUTCOME_OK,
        ])
        .get_sample_count();
    let catalog_before = super::STARTUP_STAGE_DURATION_SECONDS
        .with_label_values(&[super::STARTUP_STAGE_TOOL_CATALOG, super::STARTUP_OUTCOME_OK])
        .get_sample_count();
    let hub_ok_before = super::STARTUP_STAGE_DURATION_SECONDS
        .with_label_values(&[
            super::STARTUP_STAGE_HUB_WS_CONNECT,
            super::STARTUP_OUTCOME_OK,
        ])
        .get_sample_count();
    let hub_err_before = super::STARTUP_STAGE_DURATION_SECONDS
        .with_label_values(&[
            super::STARTUP_STAGE_HUB_WS_CONNECT,
            super::STARTUP_OUTCOME_ERROR,
        ])
        .get_sample_count();
    super::observe_startup_stage(
        super::STARTUP_STAGE_STARTUP_RECOVERY,
        super::STARTUP_OUTCOME_OK,
        0.42,
    );
    super::observe_startup_stage(
        super::STARTUP_STAGE_HUB_WS_CONNECT,
        super::STARTUP_OUTCOME_ERROR,
        12.5,
    );
    assert_eq!(
        super::STARTUP_STAGE_DURATION_SECONDS
            .with_label_values(&[
                super::STARTUP_STAGE_STARTUP_RECOVERY,
                super::STARTUP_OUTCOME_OK
            ])
            .get_sample_count(),
        recovery_before + 1
    );
    assert_eq!(
        super::STARTUP_STAGE_DURATION_SECONDS
            .with_label_values(&[
                super::STARTUP_STAGE_HUB_WS_CONNECT,
                super::STARTUP_OUTCOME_ERROR
            ])
            .get_sample_count(),
        hub_err_before + 1
    );
    assert_eq!(
        super::STARTUP_STAGE_DURATION_SECONDS
            .with_label_values(&[
                super::STARTUP_STAGE_HUB_WS_CONNECT,
                super::STARTUP_OUTCOME_OK
            ])
            .get_sample_count(),
        hub_ok_before,
        "error sample must not advance ok hub_ws_connect"
    );
    assert_eq!(
        super::STARTUP_STAGE_DURATION_SECONDS
            .with_label_values(&[super::STARTUP_STAGE_TOOL_CATALOG, super::STARTUP_OUTCOME_OK])
            .get_sample_count(),
        catalog_before,
        "observing recovery/hub must not sample tool_catalog"
    );
}
#[tokio::test]
async fn codebase_index_forwarder_abort_releases_shared() {
    let handle = make_handle();
    tokio::task::yield_now().await;
    let before = Arc::strong_count(handle.shared());
    let task = handle.spawn_codebase_index_event_forwarder();
    tokio::task::yield_now().await;
    assert!(!task.is_finished());
    assert!(Arc::strong_count(handle.shared()) > before);
    task.abort();
    let _ = task.await;
    assert_eq!(
        Arc::strong_count(handle.shared()),
        before,
        "abort must drop the forwarder's WorkspaceShared ref"
    );
}
#[tokio::test]
async fn resolve_service_path_normal() {
    let handle = make_handle();
    let root = handle.root_cwd().unwrap();
    let canonical_root = handle.canonical_root().await.unwrap();
    let resolved = handle
        .resolve_service_path("src/main.rs", &canonical_root)
        .await
        .expect("normal path should resolve");
    assert_eq!(resolved, root.join("src/main.rs"));
}
#[tokio::test]
async fn resolve_service_path_rejects_empty() {
    let handle = make_handle();
    let canonical_root = handle.canonical_root().await.unwrap();
    let err = handle
        .resolve_service_path("", &canonical_root)
        .await
        .expect_err("empty path must be rejected");
    let msg = format!("{err}");
    assert!(
        msg.contains("empty path"),
        "error should mention empty path: {msg}"
    );
}
#[tokio::test]
async fn resolve_service_path_rejects_absolute_outside_root() {
    let handle = make_handle();
    let canonical_root = handle.canonical_root().await.unwrap();
    let err = handle
        .resolve_service_path("/etc/passwd", &canonical_root)
        .await
        .expect_err("absolute path outside root must be rejected");
    let msg = format!("{err}");
    assert!(
        msg.contains("escapes workspace root"),
        "error should mention escape: {msg}"
    );
}
#[tokio::test]
async fn resolve_service_path_accepts_absolute_within_root() {
    let handle = make_handle();
    let root = handle.root_cwd().unwrap();
    let canonical_root = handle.canonical_root().await.unwrap();
    let rel = handle
        .resolve_service_path("src/main.rs", &canonical_root)
        .await
        .expect("relative path should resolve");
    let abs_input = root.join("src/main.rs");
    let abs = handle
        .resolve_service_path(abs_input.to_str().expect("utf-8 path"), &canonical_root)
        .await
        .expect("absolute path within root should resolve");
    assert_eq!(abs, rel);
}
#[tokio::test]
async fn resolve_service_path_rejects_escape() {
    let handle = make_handle();
    let canonical_root = handle.canonical_root().await.unwrap();
    let err = handle
        .resolve_service_path("../../etc/passwd", &canonical_root)
        .await
        .expect_err("escape path must be rejected");
    let msg = format!("{err}");
    assert!(
        msg.contains("path escapes workspace root"),
        "error should mention escape: {msg}"
    );
}
#[tokio::test]
async fn resolve_service_path_allows_dotdot_within_root() {
    let handle = make_handle();
    let root = handle.root_cwd().unwrap();
    let canonical_root = handle.canonical_root().await.unwrap();
    let resolved = handle
        .resolve_service_path("src/../lib.rs", &canonical_root)
        .await
        .expect("dotdot within root should resolve");
    assert_eq!(resolved, root.join("lib.rs"));
}
#[tokio::test]
async fn resolve_service_path_rejects_symlink_escape() {
    let handle = make_handle();
    let root = handle.root_cwd().unwrap();
    let canonical_root = handle.canonical_root().await.unwrap();
    let outside = tempfile::tempdir().expect("create outside dir");
    let secret = outside.path().join("secret.txt");
    std::fs::write(&secret, "top secret").expect("write secret");
    let link_path = root.join("escape_link");
    #[cfg(unix)]
    std::os::unix::fs::symlink(outside.path(), &link_path).expect("create symlink");
    #[cfg(not(unix))]
    {
        return;
    }
    let err = handle
        .resolve_service_path("escape_link/secret.txt", &canonical_root)
        .await
        .expect_err("symlink escape must be rejected");
    let msg = format!("{err}");
    assert!(
        msg.contains("symlink escape"),
        "error should mention symlink escape: {msg}"
    );
}
/// A *dangling* leaf symlink (target missing, outside root) must be rejected.
/// `canonicalize` fails NotFound, so the leaf is resolved via `read_link`.
#[tokio::test]
#[cfg(unix)]
async fn resolve_service_path_rejects_dangling_symlink_escape() {
    let handle = make_handle();
    let root = handle.root_cwd().unwrap();
    let canonical_root = handle.canonical_root().await.unwrap();
    let outside = tempfile::tempdir().expect("create outside dir");
    std::os::unix::fs::symlink(outside.path().join("new.txt"), root.join("lnk"))
        .expect("create symlink");
    let err = handle
        .resolve_service_path("lnk", &canonical_root)
        .await
        .expect_err("dangling symlink escape must be rejected");
    assert!(
        format!("{err}").contains("symlink escape"),
        "error should mention symlink escape: {err}"
    );
}
/// A multi-hop chain of dangling in-root links ending outside the root must be followed and rejected (not fall through the ancestor walk).
#[tokio::test]
#[cfg(unix)]
async fn resolve_service_path_rejects_dangling_symlink_chain() {
    let handle = make_handle();
    let root = handle.root_cwd().unwrap();
    let canonical_root = handle.canonical_root().await.unwrap();
    let outside = tempfile::tempdir().expect("outside");
    for i in 0..3 {
        std::os::unix::fs::symlink(
            root.join(format!("lnk{}", i + 1)),
            root.join(format!("lnk{i}")),
        )
        .expect("chain link");
    }
    std::os::unix::fs::symlink(outside.path().join("x"), root.join("lnk3")).expect("tail link");
    let err = handle
        .resolve_service_path("lnk0", &canonical_root)
        .await
        .expect_err("dangling symlink chain escaping root must be rejected");
    assert!(
        format!("{err}").contains("symlink escape")
            || format!("{err}").contains("unresolved symlink chain"),
        "unexpected error: {err}"
    );
}
#[tokio::test]
async fn resolve_service_path_nested_subdir() {
    let handle = make_handle();
    let root = handle.root_cwd().unwrap();
    let canonical_root = handle.canonical_root().await.unwrap();
    let resolved = handle
        .resolve_service_path("a/b/c/d.txt", &canonical_root)
        .await
        .expect("deeply nested path should resolve");
    assert_eq!(resolved, root.join("a/b/c/d.txt"));
}
#[tokio::test]
async fn resolve_service_path_dot_current_dir() {
    let handle = make_handle();
    let root = handle.root_cwd().unwrap();
    let canonical_root = handle.canonical_root().await.unwrap();
    let resolved = handle
        .resolve_service_path("./src/./main.rs", &canonical_root)
        .await
        .expect("dot segments should be stripped");
    assert_eq!(resolved, root.join("src/main.rs"));
}
#[tokio::test]
async fn confine_to_root_accepts_path_within_alternative_root() {
    let handle = make_confining_handle();
    let alt = tempfile::tempdir().expect("create alt root");
    let alt_root = alt.path().to_path_buf();
    let target = alt_root.join("src/foo.rs");
    let (confined, _canonical) = handle
        .confine_to_root(&target, &alt_root)
        .await
        .expect("path within the alternative root should resolve");
    assert_eq!(confined, target);
    handle
        .confine_to_workspace_root(&target)
        .await
        .expect_err("path outside the workspace root must be rejected");
}
#[tokio::test]
async fn confine_to_root_rejects_dotdot_escape() {
    let handle = make_confining_handle();
    let alt = tempfile::tempdir().expect("create alt root");
    let err = handle
        .confine_to_root(std::path::Path::new("../../etc/passwd"), alt.path())
        .await
        .expect_err("dotdot escape from the alternative root must be rejected");
    assert!(
        format!("{err}").contains("path escapes workspace root"),
        "error should mention escape: {err}"
    );
}
#[tokio::test]
async fn confine_to_root_rejects_absolute_path_outside_root() {
    let handle = make_confining_handle();
    let alt = tempfile::tempdir().expect("create alt root");
    let err = handle
        .confine_to_root(std::path::Path::new("/etc/passwd"), alt.path())
        .await
        .expect_err("absolute path outside the alternative root must be rejected");
    assert!(
        format!("{err}").contains("escapes workspace root"),
        "error should mention escape: {err}"
    );
}
#[tokio::test]
#[cfg(unix)]
async fn confine_to_root_rejects_symlink_escape() {
    let handle = make_confining_handle();
    let alt = tempfile::tempdir().expect("create alt root");
    let outside = tempfile::tempdir().expect("create outside dir");
    std::fs::write(outside.path().join("secret.txt"), "top secret").expect("write secret");
    std::os::unix::fs::symlink(outside.path(), alt.path().join("escape_link"))
        .expect("create symlink");
    let err = handle
        .confine_to_root(&alt.path().join("escape_link/secret.txt"), alt.path())
        .await
        .expect_err("symlink escaping the alternative root must be rejected");
    assert!(
        format!("{err}").contains("symlink escape"),
        "error should mention symlink escape: {err}"
    );
}
/// Off by default: an out-of-root absolute path is passed through, not rejected.
#[tokio::test]
async fn confine_to_workspace_root_unconfined_by_default_allows_escape() {
    let handle = make_handle();
    let outside = tempfile::tempdir().expect("create outside dir");
    let target = outside.path().join("secret.txt");
    let (resolved, walk_root) = handle
        .confine_to_workspace_root(&target)
        .await
        .expect("unconfined resolution must not reject an outside path");
    assert_eq!(resolved, target, "path is passed through unchanged");
    assert!(
        walk_root.is_none(),
        "no confining walk root when confinement is off"
    );
}
/// Off by default: a symlink escaping the root is followed, not rejected.
#[tokio::test]
#[cfg(unix)]
async fn confine_to_workspace_root_unconfined_by_default_follows_symlink() {
    let handle = make_handle();
    let root = handle.root_cwd().unwrap();
    let outside = tempfile::tempdir().expect("create outside dir");
    std::fs::write(outside.path().join("secret.txt"), "ok").expect("write secret");
    std::os::unix::fs::symlink(outside.path(), root.join("escape_link")).expect("create symlink");
    let link_path = root.join("escape_link/secret.txt");
    let (resolved, walk_root) = handle
        .confine_to_workspace_root(&link_path)
        .await
        .expect("unconfined resolution must follow a symlink out of the root");
    assert_eq!(resolved, link_path);
    assert!(walk_root.is_none());
}
#[tokio::test]
async fn per_session_hunk_tracker_isolation() {
    let handle = make_handle();
    let child = handle
        .fork_session(fork_cfg_with(
            "child",
            CapabilityMode::ReadWrite,
            None,
            Some("main"),
        ))
        .await
        .expect("fork should succeed");
    child.hunk_tracker().record_agent_write(
        std::path::PathBuf::from("/tmp/test-file.rs"),
        "fn main() {}".to_string(),
        0,
        None,
    );
    let child_hunks = child.hunk_tracker().get_all_hunks().await;
    assert!(
        !child_hunks.is_empty(),
        "child session should have tracked hunks"
    );
    let main = handle.session("main").expect("main session present");
    let main_hunks = main.hunk_tracker().get_all_hunks().await;
    assert!(
        main_hunks.is_empty(),
        "main session hunk tracker must be isolated from child: got {} hunks",
        main_hunks.len()
    );
}
#[tokio::test]
async fn cancel_tool_call_marks_call_completed() {
    let handle = make_handle();
    let tracker = handle.activity_tracker();
    tracker.tool_call_started("call-1", "read_file", Some("main"));
    assert_eq!(tracker.snapshot().active_tool_calls, 1);
    handle.cancel_tool_call("main", "call-1");
    assert_eq!(
        tracker.snapshot().active_tool_calls,
        0,
        "cancel_tool_call should mark the call as completed"
    );
}
#[tokio::test]
async fn cancel_tool_call_unknown_id_is_noop() {
    let handle = make_handle();
    handle.cancel_tool_call("main", "never-started");
    assert_eq!(handle.activity_tracker().snapshot().active_tool_calls, 0);
}
#[tokio::test]
async fn on_session_ended_clears_turn_active() {
    let handle = make_handle();
    let tracker = handle.activity_tracker();
    tracker.turn_started("main", 1);
    assert!(tracker.is_turn_active("main"));
    handle.on_session_ended("main");
    assert!(
        !tracker.is_turn_active("main"),
        "on_session_ended should clear turn_active"
    );
}
#[tokio::test]
async fn on_session_ended_unknown_session_is_noop() {
    let handle = make_handle();
    let tracker = handle.activity_tracker();
    let sessions_before = tracker.known_sessions();
    handle.on_session_ended("nonexistent");
    assert_eq!(
        tracker.known_sessions(),
        sessions_before,
        "on_session_ended must not create a new session entry"
    );
}
#[tokio::test]
async fn fork_session_inherits_viewer_ctx_from_parent() {
    let handle = make_handle();
    handle.drop_session("main", "main").expect("drop main");
    let parent = handle
        .create_session_with_tracker_and_viewer_ctx(
            "main",
            handle.root_cwd().unwrap(),
            xai_hunk_tracker::HunkTrackerHandle::noop(),
            None,
            CapabilityMode::All,
            Some(xai_tool_runtime::WorkspaceViewerContext {
                stream_tool_progress: true,
            }),
            false,
        )
        .expect("create parent");
    assert!(parent.viewer_ctx().is_some());
    let child = handle
        .fork_session(fork_cfg_with(
            "child",
            CapabilityMode::ReadWrite,
            None,
            Some("main"),
        ))
        .await
        .expect("fork should succeed");
    let inherited = child.viewer_ctx().expect("child inherits viewer_ctx");
    assert!(
        inherited.stream_tool_progress,
        "child must inherit the parent's stream_tool_progress flag"
    );
}
#[derive(Clone, Default)]
struct BindMcpTestState {
    session_ids: Arc<parking_lot::Mutex<Vec<String>>>,
    tool_calls: Arc<parking_lot::Mutex<Vec<serde_json::Value>>>,
    hang_tools_list: bool,
    zero_tools: bool,
    tool_name: Option<String>,
    /// When set, `tools/list` signals the first notify and waits on the
    /// second before answering, so a test can interleave work mid-discovery.
    tools_list_gate: Option<(Arc<tokio::sync::Notify>, Arc<tokio::sync::Notify>)>,
    /// When set, `tools/list` returns this many tools (`tool_000`, ...)
    /// instead of the single default.
    tool_count: Option<usize>,
}
async fn bind_mcp_post(
    axum::extract::State(state): axum::extract::State<BindMcpTestState>,
    headers: axum::http::HeaderMap,
    axum::Json(request): axum::Json<serde_json::Value>,
) -> axum::response::Response {
    if let Some(session_id) = headers
        .get(xai_grok_mcp::servers::GROK_AGENT_ID_HEADER)
        .and_then(|value| value.to_str().ok())
    {
        state.session_ids.lock().push(session_id.to_owned());
    }
    let id = request
        .get("id")
        .unwrap_or(&serde_json::Value::Null)
        .clone();
    match request
        .get("method")
        .unwrap_or(&serde_json::Value::Null)
        .as_str()
    {
        Some("initialize") => (
            [("mcp-session-id", "local-test-session")],
            axum::Json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": request
                        .get("params")
                        .and_then(|v| v.get("protocolVersion"))
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                    "capabilities": {},
                    "serverInfo": {"name": "local-test", "version": "1"}
                }
            })),
        )
            .into_response(),
        Some("tools/list") => {
            if state.hang_tools_list {
                std::future::pending::<()>().await;
            }
            if let Some((reached, release)) = &state.tools_list_gate {
                reached.notify_one();
                release.notified().await;
            }
            let tools = if state.zero_tools {
                Vec::new()
            } else if let Some(count) = state.tool_count {
                (0..count)
                    .map(|index| {
                        serde_json::json!({
                            "name": format!("tool_{index:03}"),
                            "description": "generated",
                            "inputSchema": {"type": "object"}
                        })
                    })
                    .collect()
            } else {
                vec![serde_json::json!({
                    "name": state.tool_name.as_deref().unwrap_or("echo"),
                    "description": "Echo a value",
                    "inputSchema": {"type": "object"}
                })]
            };
            axum::Json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {"tools": tools}
            }))
            .into_response()
        }
        Some("tools/call") => {
            state.tool_calls.lock().push(
                request
                    .get("params")
                    .unwrap_or(&serde_json::Value::Null)
                    .clone(),
            );
            axum::Json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{"type": "text", "text": "MCP_CALL_OK"}],
                    "isError": false
                }
            }))
            .into_response()
        }
        Some("server/discover") => axum::Json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {"code": -32601, "message": "Method not found"}
        }))
        .into_response(),
        _ => axum::http::StatusCode::ACCEPTED.into_response(),
    }
}
async fn bind_mcp_get() -> axum::response::Response {
    let body =
        axum::body::Body::from_stream(futures::stream::pending::<Result<String, std::io::Error>>());
    (
        [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
        body,
    )
        .into_response()
}
async fn spawn_bind_mcp_server(state: BindMcpTestState) -> (String, tokio::task::JoinHandle<()>) {
    let app = axum::Router::new()
        .route("/mcp", axum::routing::get(bind_mcp_get).post(bind_mcp_post))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{address}/mcp"), task)
}
fn configured_test_mcp(name: &str, url: String) -> agent_client_protocol::McpServer {
    agent_client_protocol::McpServer::Http(
        agent_client_protocol::McpServerHttp::new(name, url).headers(vec![]),
    )
}
async fn drain_terminal_err(
    mut stream: impl futures::Stream<
        Item = xai_tool_runtime::ToolStreamItem<xai_tool_runtime::TypedToolOutput>,
    > + Unpin,
) -> xai_tool_runtime::ToolError {
    use futures::StreamExt;
    use xai_tool_runtime::ToolStreamItem;
    while let Some(item) = stream.next().await {
        match item {
            ToolStreamItem::Terminal(Err(e)) => return e,
            ToolStreamItem::Progress(_) => {}
            ToolStreamItem::Terminal(Ok(t)) => {
                panic!("expected Terminal(Err), got Ok: {t:?}")
            }
        }
    }
    panic!("stream ended without terminal")
}
/// Minimal handler standing in for a resolver-installed native tool.
struct StaticHandler(ToolId);
impl StaticHandler {
    fn new(id: &str) -> Self {
        Self(ToolId::new(id).unwrap())
    }
}
/// Owner bind: capability `all` and an explicit toolset (strict servers fail closed otherwise).
fn owner_full_bind_metadata() -> serde_json::Value {
    serde_json::json!({
        "metadata": {
            "capability_mode": "all",
            "tools": [
                {"id": "GrokBuild:read_file"},
                {"id": "GrokBuild:search_replace"},
                {"id": "GrokBuild:grep"},
                {"id": "GrokBuild:list_dir"},
            ],
        },
    })
}
const OWNER_TOOLS: [&str; 4] = ["read_file", "search_replace", "grep", "list_dir"];
#[track_caller]
fn assert_advertises_owner_tools(names: &[String], context: &str) {
    for tool in OWNER_TOOLS {
        assert!(
            names.iter().any(|n| n == tool),
            "{context}: owner tool `{tool}` missing from advertised set {names:?}"
        );
    }
}
/// Dropping and rebinding a session with the same ID picks up the new `viewer_ctx`, the kill switch for a stale value mid-session.
#[tokio::test]
async fn drop_then_rebind_session_replaces_viewer_ctx_value() {
    let handle = make_handle();
    handle.drop_session("main", "main").expect("drop main");
    let s1 = handle
        .create_session_with_tracker_and_viewer_ctx(
            "main",
            handle.root_cwd().unwrap(),
            xai_hunk_tracker::HunkTrackerHandle::noop(),
            None,
            CapabilityMode::All,
            Some(xai_tool_runtime::WorkspaceViewerContext {
                stream_tool_progress: true,
            }),
            false,
        )
        .expect("first bind");
    assert_eq!(s1.viewer_ctx().map(|c| c.stream_tool_progress), Some(true));
    handle.drop_session("main", "main").expect("drop");
    let s2 = handle
        .create_session_with_tracker_and_viewer_ctx(
            "main",
            handle.root_cwd().unwrap(),
            xai_hunk_tracker::HunkTrackerHandle::noop(),
            None,
            CapabilityMode::All,
            Some(xai_tool_runtime::WorkspaceViewerContext {
                stream_tool_progress: false,
            }),
            false,
        )
        .expect("second bind");
    assert_eq!(
        s2.viewer_ctx().map(|c| c.stream_tool_progress),
        Some(false),
        "rebind must surface the new viewer_ctx value"
    );
}
/// The hand-written decode `match` must not drift from the enum's serde snake_case forms.
#[test]
fn session_relationship_wire_forms_round_trip() {
    for variant in [SessionRelationship::Primary, SessionRelationship::Subagent] {
        let wire = serde_json::to_value(variant).unwrap();
        let wire = wire.as_str().unwrap();
        let decoded = decode_session_relationship(wire);
        assert_eq!(
            serde_json::to_value(decoded).unwrap().as_str(),
            Some(wire),
            "{variant:?} must round-trip through decode_session_relationship"
        );
    }
    assert!(matches!(
        decode_session_relationship("nonsense"),
        SessionRelationship::Primary
    ));
}
/// The workspace decodes the bare snake_case `cancellation_category` string back into the enum; unknown / absent values decode to `None`.
#[test]
fn cancellation_category_decode_round_trips() {
    assert_eq!(
        decode_cancellation_category(Some("hook_denied")),
        Some(CancellationCategory::HookDenied),
    );
    assert_eq!(
        decode_cancellation_category(Some("permission_rejected")),
        Some(CancellationCategory::PermissionRejected),
    );
    assert_eq!(decode_cancellation_category(Some("not_a_category")), None);
    assert_eq!(decode_cancellation_category(None), None);
}
/// A `Before` request answers with a no-op reply (no ack) while driving the same turn-start work as the fire-and-forget hook.
/// The request channel is the only turn signal the server-side sampler sends.
#[tokio::test]
async fn compute_turn_injections_before_runs_turn_start_and_replies_noop() {
    use xai_tool_protocol::turn_hook::{BeforeTurnPayload, HookReply, TurnHookRequest};
    let handle = make_handle();
    let reply = handle
        .compute_turn_injections(
            "main",
            &TurnHookRequest::Before(BeforeTurnPayload {
                turn_number: 9,
                ..BeforeTurnPayload::default()
            }),
        )
        .await;
    assert_eq!(reply, HookReply::default());
    assert!(
        handle
            .activity_tracker()
            .known_sessions()
            .iter()
            .any(|s| s == "main"),
        "Before request must drive on_before_turn (activity tracking)"
    );
}
/// The extended after-turn cancellation pair is decoded into the `TurnEnded` line.
/// The category string becomes the enum's snake_case form and the context object passes through verbatim.
#[tokio::test]
async fn after_turn_decodes_cancellation_fields_into_events_jsonl() {
    use xai_tool_protocol::turn_hook::{AfterTurnPayload, BeforeTurnPayload, TurnHookOutcome};
    let (handle, home) = make_handle_with_events();
    let sid = "sess-cancel";
    handle
        .on_before_turn(
            sid,
            &BeforeTurnPayload {
                turn_number: 2,
                model_id: "deepseek-4".to_owned(),
                yolo_mode: false,
                conversation_message_count: 0,
                session_relationship: "primary".to_owned(),
                schema_version: "1.0".to_owned(),
            },
        )
        .await;
    handle
        .on_after_turn(
            sid,
            &AfterTurnPayload {
                turn_number: 2,
                outcome: TurnHookOutcome::Cancelled,
                duration_ms: 10,
                tool_call_count: 0,
                model_id: "deepseek-4".to_owned(),
                written_repo_paths: Vec::new(),
                cancellation_category: Some("permission_rejected".to_owned()),
                cancellation_context: Some(serde_json::json!({ "recovery": false })),
            },
        )
        .await;
    let path = home.path().join("sessions").join(sid).join("events.jsonl");
    let text = std::fs::read_to_string(&path).expect("events.jsonl must exist");
    let ended = text
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
        .find(|e| e.get("type").unwrap_or(&serde_json::Value::Null) == "turn_ended")
        .expect("turn_ended must be present");
    assert_eq!(
        ended.get("outcome").unwrap_or(&serde_json::Value::Null),
        "cancelled"
    );
    assert_eq!(
        ended
            .get("cancellation_category")
            .unwrap_or(&serde_json::Value::Null),
        "permission_rejected"
    );
    assert_eq!(
        ended
            .get("cancellation_context")
            .unwrap_or(&serde_json::Value::Null),
        &serde_json::json!({ "recovery": false })
    );
}
/// The default watchdog must undercut the requester's 10s hook timeout.
#[test]
fn after_turn_watchdog_default_is_8s() {
    assert_eq!(after_turn_watchdog(), std::time::Duration::from_secs(8));
}
fn bundled_dir_fixture(subdirs: &[&str]) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    for name in subdirs {
        std::fs::create_dir(tmp.path().join(name)).expect("create subdir");
    }
    std::fs::write(tmp.path().join("BUILD.bazel"), b"").expect("create file");
    tmp
}
#[test]
fn bundled_allowlist_unset_ignores_nothing() {
    let tmp = bundled_dir_fixture(&["bundled__pdf", "bundled__xlsx"]);
    let dir = tmp.path().to_string_lossy().into_owned();
    assert_eq!(
        bundled_allowlist_ignore_dirs(&dir, None),
        Vec::<String>::new(),
        "an unset allow-list must produce no ignore entries"
    );
}
#[test]
fn bundled_allowlist_empty_ignores_everything() {
    let tmp = bundled_dir_fixture(&["bundled__pdf", "bundled__xlsx"]);
    let dir = tmp.path().to_string_lossy().into_owned();
    let want = vec![
        tmp.path()
            .join("bundled__pdf")
            .to_string_lossy()
            .into_owned(),
        tmp.path()
            .join("bundled__xlsx")
            .to_string_lossy()
            .into_owned(),
    ];
    for allowlist in ["", "  ", " , ,"] {
        assert_eq!(
            bundled_allowlist_ignore_dirs(&dir, Some(allowlist)),
            want,
            "allow-list {allowlist:?} must ignore every bundled skill"
        );
    }
}
#[test]
fn workspace_tool_definitions_path_is_session_root() {
    assert_eq!(
        workspace_tool_definitions_path("sess-1"),
        "sess-1/workspace_tool_definitions.json"
    );
}
#[test]
fn tool_defs_reemit_gate_flag_off_never_emits_and_records_nothing() {
    let map = dashmap::DashMap::new();
    let now = std::time::Instant::now();
    assert!(!tool_defs_reemit_gate(
        false,
        &map,
        "s",
        now,
        TOOL_DEFS_DEBOUNCE
    ));
    assert!(
        map.is_empty(),
        "flag-off must not record any debounce state (legacy path stays inert)"
    );
    assert!(tool_defs_reemit_gate(
        true,
        &map,
        "s",
        now,
        TOOL_DEFS_DEBOUNCE
    ));
}
#[test]
fn tool_defs_reemit_gate_debounces_within_5s_window() {
    let map = dashmap::DashMap::new();
    let window = std::time::Duration::from_secs(5);
    let t0 = std::time::Instant::now();
    assert!(tool_defs_reemit_gate(true, &map, "s", t0, window));
    assert!(!tool_defs_reemit_gate(
        true,
        &map,
        "s",
        t0 + std::time::Duration::from_secs(1),
        window
    ));
    assert!(!tool_defs_reemit_gate(
        true,
        &map,
        "s",
        t0 + std::time::Duration::from_millis(4_999),
        window
    ));
    assert!(tool_defs_reemit_gate(
        true,
        &map,
        "s",
        t0 + std::time::Duration::from_secs(5),
        window
    ));
    assert!(!tool_defs_reemit_gate(
        true,
        &map,
        "s",
        t0 + std::time::Duration::from_secs(6),
        window
    ));
}
#[test]
fn tool_defs_reemit_gate_is_per_session() {
    let map = dashmap::DashMap::new();
    let now = std::time::Instant::now();
    assert!(tool_defs_reemit_gate(
        true,
        &map,
        "a",
        now,
        TOOL_DEFS_DEBOUNCE
    ));
    assert!(tool_defs_reemit_gate(
        true,
        &map,
        "b",
        now,
        TOOL_DEFS_DEBOUNCE
    ));
    assert!(!tool_defs_reemit_gate(
        true,
        &map,
        "a",
        now,
        TOOL_DEFS_DEBOUNCE
    ));
}
#[tokio::test]
async fn workspace_tool_definitions_payload_matches_chat_completions_shape() {
    let handle = make_handle();
    let (path, bytes) = handle
        .workspace_tool_definitions_payload("main")
        .expect("payload for an existing session");
    assert_eq!(path, "main/workspace_tool_definitions.json");
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).expect("valid JSON");
    let arr = parsed.as_array().expect("a JSON array of tool definitions");
    assert!(!arr.is_empty(), "baseline session must expose tools");
    for def in arr {
        assert_eq!(
            def.get("type").unwrap_or(&serde_json::Value::Null),
            "function",
            "tool def must be type=function: {def}"
        );
        let function = &def.get("function").unwrap_or(&serde_json::Value::Null);
        assert!(
            function
                .get("name")
                .unwrap_or(&serde_json::Value::Null)
                .as_str()
                .is_some_and(|n| !n.is_empty()),
            "function.name must be a non-empty string: {def}"
        );
        assert!(
            function
                .get("parameters")
                .unwrap_or(&serde_json::Value::Null)
                .is_object(),
            "function.parameters must be a JSON object: {def}"
        );
        let keys: std::collections::BTreeSet<&str> = function
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert!(
            keys.is_subset(&["name", "description", "parameters"].into_iter().collect()),
            "unexpected function keys {keys:?}"
        );
    }
    let names: std::collections::BTreeSet<&str> = arr
        .iter()
        .filter_map(|d| {
            d.get("function")
                .and_then(|f| f.get("name"))
                .and_then(|v| v.as_str())
        })
        .collect();
    for expected in ["read_file", "search_replace", "grep", "list_dir"] {
        assert!(
            names.contains(expected),
            "missing baseline tool {expected}: {names:?}"
        );
    }
}
#[test]
fn bundled_allowlist_ignores_complement() {
    let tmp = bundled_dir_fixture(&["bundled__pdf", "bundled__xlsx", "bundled__docx"]);
    let dir = tmp.path().to_string_lossy().into_owned();
    let got = bundled_allowlist_ignore_dirs(&dir, Some("xlsx, pdf"));
    let want = vec![
        tmp.path()
            .join("bundled__docx")
            .to_string_lossy()
            .into_owned(),
    ];
    assert_eq!(got, want);
}
#[test]
fn bundled_allowlist_strips_bundled_prefix() {
    let tmp = bundled_dir_fixture(&["bundled__pdf", "xlsx", "bundled__skip"]);
    let dir = tmp.path().to_string_lossy().into_owned();
    let got = bundled_allowlist_ignore_dirs(&dir, Some("bundled__pdf,bundled__xlsx"));
    let want = vec![
        tmp.path()
            .join("bundled__skip")
            .to_string_lossy()
            .into_owned(),
    ];
    assert_eq!(got, want);
}
#[test]
fn bundled_allowlist_unreadable_dir_fails_closed() {
    let got = bundled_allowlist_ignore_dirs("/nonexistent/bundled-skills", Some("pdf"));
    assert_eq!(got, vec!["/nonexistent/bundled-skills".to_string()]);
}
/// Unique skill names: discovery also reads the dev machine's `~/.grok`.
#[tokio::test]
async fn bundled_allowlist_filters_discovery() {
    let tmp = tempfile::tempdir().expect("tempdir");
    for name in ["allowlist-e2e-kept", "allowlist-e2e-blocked"] {
        let skill_dir = tmp.path().join(format!("bundled__{name}"));
        std::fs::create_dir(&skill_dir).expect("create subdir");
        std::fs::write(
            skill_dir.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: test\n---\nbody"),
        )
        .expect("write SKILL.md");
    }
    let dir = tmp.path().to_string_lossy().into_owned();
    let cwd = tempfile::tempdir().expect("tempdir");
    let mut config = crate::discovery::SkillsConfig {
        bundled_skill_dirs: vec![dir.clone()],
        ..Default::default()
    };
    config.ignore.extend(bundled_allowlist_ignore_dirs(
        &dir,
        Some("allowlist-e2e-kept"),
    ));
    let skills = crate::discovery::discover_skills(cwd.path(), &config, true).await;
    let names: Vec<&str> = skills
        .iter()
        .filter_map(|s| s.get("name").unwrap_or(&serde_json::Value::Null).as_str())
        .filter(|n| n.starts_with("allowlist-e2e-"))
        .collect();
    assert_eq!(
        names,
        vec!["allowlist-e2e-kept"],
        "only the allowlisted skill survives"
    );
}
#[tokio::test]
async fn workspace_tool_definitions_payload_none_for_unknown_session() {
    let handle = make_handle();
    assert!(
        handle.workspace_tool_definitions_payload("ghost").is_none(),
        "unknown session yields no payload"
    );
}
#[test]
fn phase1_budget_is_one_third_of_grace() {
    assert_eq!(
        phase1_budget(std::time::Duration::from_secs(45)),
        std::time::Duration::from_secs(15)
    );
    assert_eq!(
        phase1_budget(std::time::Duration::from_secs(120)),
        std::time::Duration::from_secs(40)
    );
}
#[test]
fn phase15_budget_is_half_of_remaining() {
    assert_eq!(
        phase15_budget(std::time::Duration::from_secs(30)),
        std::time::Duration::from_secs(15)
    );
    assert_eq!(
        phase15_budget(std::time::Duration::ZERO),
        std::time::Duration::ZERO
    );
}
#[test]
fn classify_drain_outcome_covers_all_arms() {
    assert_eq!(
        classify_drain_outcome(false, false, 0, 1),
        DrainOutcome::Partial
    );
    assert_eq!(
        classify_drain_outcome(false, true, 0, 0),
        DrainOutcome::Partial
    );
    assert_eq!(
        classify_drain_outcome(true, false, 0, 2),
        DrainOutcome::ProducersTimeout
    );
    assert_eq!(
        classify_drain_outcome(true, false, 0, 0),
        DrainOutcome::ProducersTimeout
    );
    assert_eq!(
        classify_drain_outcome(true, true, 1, 0),
        DrainOutcome::ProducersTimeout
    );
    assert_eq!(
        classify_drain_outcome(true, true, 0, 3),
        DrainOutcome::Timeout
    );
    assert_eq!(classify_drain_outcome(true, true, 0, 0), DrainOutcome::Full);
}
#[test]
fn drain_reason_and_outcome_labels_are_stable() {
    assert_eq!(DrainReason::Sigterm.as_ref(), "sigterm");
    assert_eq!(DrainReason::Evict.as_ref(), "evict");
    assert_eq!(DrainOutcome::Full.as_ref(), "full");
    assert_eq!(DrainOutcome::Partial.as_ref(), "partial");
    assert_eq!(DrainOutcome::ProducersTimeout.as_ref(), "producers_timeout");
    assert_eq!(DrainOutcome::Timeout.as_ref(), "timeout");
}
#[test]
fn grace_budget_from_raw_parses_and_falls_back() {
    let d = |ms| std::time::Duration::from_millis(ms);
    assert_eq!(grace_budget_from_raw(None), d(DEFAULT_TERMINATION_GRACE_MS));
    assert_eq!(grace_budget_from_raw(Some("120000".into())), d(120_000));
    assert_eq!(grace_budget_from_raw(Some("  90000 ".into())), d(90_000));
    assert_eq!(
        grace_budget_from_raw(Some("0".into())),
        d(DEFAULT_TERMINATION_GRACE_MS)
    );
    assert_eq!(
        grace_budget_from_raw(Some("nonsense".into())),
        d(DEFAULT_TERMINATION_GRACE_MS)
    );
}
#[test]
fn write_draining_marker_writes_count_atomically() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace-server.draining");
    write_draining_marker(&path, 5);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "5");
    let leftover_tmp = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .any(|e| e.file_name().to_string_lossy().ends_with(".draining.tmp"));
    assert!(!leftover_tmp, "temp file must be renamed away");
    write_draining_marker(&path, 0);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "0");
}
#[tokio::test]
async fn two_phase_drain_no_queue_marks_draining_and_returns_zero() {
    let handle = make_handle();
    let tracker = handle.activity_tracker().clone();
    assert!(!tracker.is_draining());
    let unfinished = handle
        .two_phase_drain(std::time::Duration::from_millis(300), DrainReason::Sigterm)
        .await;
    assert_eq!(unfinished, 0, "no queue → nothing pending to lose");
    assert!(
        tracker.is_draining(),
        "drain must mark the tracker draining"
    );
    let snap = tracker.snapshot();
    assert_eq!(
        snap.status,
        xai_tool_protocol::ToolServerLifecycleStatus::Draining
    );
    assert!(
        snap.drain_started_ms.is_some(),
        "drain_started_ms must be stamped at drain start"
    );
}
#[tokio::test]
async fn spawn_producer_is_counted_and_withholds_idle() {
    let handle = make_handle();
    let tracker = handle.activity_tracker().clone();
    assert_eq!(tracker.snapshot().artifact_producers_inflight, 0);
    let gate = Arc::new(tokio::sync::Notify::new());
    let gate2 = gate.clone();
    let join = handle.spawn_producer(async move { gate2.notified().await });
    let snap = tracker.snapshot();
    assert_eq!(snap.artifact_producers_inflight, 1);
    assert!(
        snap.idle_since_ms.is_none(),
        "an in-flight producer must report the workspace busy"
    );
    gate.notify_one();
    join.await.expect("producer must finish");
    let snap = tracker.snapshot();
    assert_eq!(snap.artifact_producers_inflight, 0);
    assert!(
        snap.idle_since_ms.is_some(),
        "idle must be restored after the producer completes"
    );
}
/// A producer spawned after a drain has started stays TRACKED (the idle gate must keep seeing it) and is counted as at-risk.
#[tokio::test]
async fn spawn_producer_after_drain_start_stays_tracked() {
    let handle = make_handle();
    handle.shared.activity_tracker.set_draining();
    let before = PRODUCER_SPAWNED_AFTER_DRAIN_TOTAL.get();
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let join = handle.spawn_producer(async move {
        let _ = rx.await;
        42
    });
    assert_eq!(
        handle.shared.producer_tasks.len(),
        1,
        "a late producer must remain visible to the durability idle gate"
    );
    assert_eq!(
        PRODUCER_SPAWNED_AFTER_DRAIN_TOTAL.get(),
        before + 1,
        "the at-risk late spawn must be counted"
    );
    let _ = tx.send(());
    assert_eq!(join.await.expect("task must run"), 42);
}
/// The producer tracker survives a completed drain: a workspace that keeps running after a hub evict still tracks (and idle-gates) new producers.
#[tokio::test]
async fn producer_tracker_usable_after_drain() {
    let handle = make_handle();
    handle
        .two_phase_drain(std::time::Duration::from_millis(200), DrainReason::Evict)
        .await;
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let join = handle.spawn_producer(async move {
        let _ = rx.await;
        7
    });
    assert_eq!(
        handle.shared.producer_tasks.len(),
        1,
        "post-drain spawns must still be tracked (TaskTracker never closed)"
    );
    let _ = tx.send(());
    assert_eq!(join.await.expect("task must run"), 7);
}
/// Phase 1.5 is capped at half the post-phase-1 remainder.
/// A producer that would finish within the total budget (at 400ms of 600ms) but past the cap (300ms) is cut off there.
/// That preserves the phase-2 floor.
#[tokio::test(start_paused = true)]
async fn drain_phase15_is_capped_at_half_the_remaining_budget() {
    let handle = make_handle();
    let _join = handle.spawn_producer(async {
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    });
    let t0 = tokio::time::Instant::now();
    let unfinished = handle
        .two_phase_drain(std::time::Duration::from_millis(600), DrainReason::Sigterm)
        .await;
    let elapsed = t0.elapsed();
    assert_eq!(
        unfinished, 1,
        "the producer cut off at the phase-1.5 cap is still in flight, so it \
         counts as outstanding work in the returned total"
    );
    assert!(
        elapsed < std::time::Duration::from_millis(400),
        "phase 1.5 must give up at the cap, not wait for the \
         400ms producer; drained in {elapsed:?}"
    );
}
/// A producer that outlives the whole grace budget classifies as `producers_timeout` and must not wedge the drain.
#[tokio::test(start_paused = true)]
async fn two_phase_drain_producer_exceeding_budget_times_out() {
    let handle = make_handle();
    let _join = handle.spawn_producer(std::future::pending::<()>());
    let before = DRAIN_COMPLETED_TOTAL
        .with_label_values(&[DrainOutcome::ProducersTimeout.as_ref()])
        .get();
    let unfinished = handle
        .two_phase_drain(std::time::Duration::from_millis(300), DrainReason::Sigterm)
        .await;
    assert_eq!(
        unfinished, 1,
        "no queue, but the wedged producer is outstanding work, so the returned \
         total is 1 (it was 0 when the return value ignored producers)"
    );
    assert!(
        DRAIN_COMPLETED_TOTAL
            .with_label_values(&[DrainOutcome::ProducersTimeout.as_ref()])
            .get()
            > before,
        "the drain must classify as producers_timeout"
    );
}
/// Runs `command` through the bound session's `run_terminal_cmd` and returns its stdout.
async fn bound_shell_stdout(handle: &WorkspaceHandle, session_id: &str, command: &str) -> String {
    let harness = handle
        .create_local_harness(session_id)
        .expect("local harness");
    let stream = harness
        .call(
            xai_tool_protocol::ToolId::new("run_terminal_cmd").expect("valid tool id"),
            serde_json::json!({ "command": command, "description": "probe the shell env" }),
            xai_tool_runtime::ToolCallContext::default(),
        )
        .await;
    let typed = drain_terminal_ok(stream).await;
    typed
        .chat_completion_output
        .and_then(|cco| cco.result)
        .and_then(|r| r.code_execution_result)
        .map(|cer| cer.stdout)
        .expect("run_terminal_cmd stdout")
}
/// `run_terminal_cmd` refuses to finalize without its background companions.
fn shell_tools() -> serde_json::Value {
    serde_json::json!([
        {"id": "GrokBuild:run_terminal_cmd"},
        {"id": "GrokBuild:get_task_output"},
        {"id": "GrokBuild:kill_task"},
    ])
}
const BIND_ENV_PROBE: &str =
    "echo \"$TERMINAL_JWT_FILE|$TMPDIR|$XDG_CACHE_HOME|$XDG_STATE_HOME|$npm_config_cache|$PORT\"";
#[tokio::test]
async fn local_harness_virtualizes_inbound_and_outbound() {
    use xai_tool_runtime::ToolCallContext;
    let handle = make_handle();
    let session = handle
        .create_session_with_cwd("virt-local", None)
        .expect("create");
    session.set_path_virtualization(
        crate::path_virtualization::PathVirtualization::try_from_session_root(
            "/workspace/conv-abc",
        )
        .expect("valid"),
    );
    let received = Arc::new(std::sync::Mutex::new(None));
    let received_c = received.clone();
    #[derive(Debug)]
    struct LocalPathEcho(Arc<std::sync::Mutex<Option<serde_json::Value>>>);
    impl xai_grok_tools::types::tool_metadata::ToolMetadata for LocalPathEcho {
        fn kind(&self) -> ToolKind {
            ToolKind::Other
        }
        fn tool_namespace(&self) -> xai_grok_tools::types::tool::ToolNamespace {
            xai_grok_tools::types::tool::ToolNamespace::MCP
        }
        fn description_template(&self) -> &str {
            "local path echo"
        }
    }
    impl xai_tool_runtime::Tool for LocalPathEcho {
        type Args = serde_json::Value;
        type Output = serde_json::Value;
        fn id(&self) -> xai_tool_protocol::ToolId {
            xai_tool_protocol::ToolId::new("local_path_echo").expect("valid")
        }
        fn description(
            &self,
            _ctx: &::xai_tool_runtime::ListToolsContext,
        ) -> xai_tool_types::ToolDescription {
            xai_tool_types::ToolDescription::new("local_path_echo", "local path echo")
        }
        async fn run(
            &self,
            _ctx: xai_tool_runtime::ToolCallContext,
            input: serde_json::Value,
        ) -> Result<serde_json::Value, xai_tool_runtime::ToolError> {
            *self.0.lock().expect("lock") = Some(input.clone());
            Ok(serde_json::json!({
                "guest": "/workspace/conv-abc/out.txt",
            }))
        }
    }
    session
        .toolset()
        .register_tool(
            "local_path_echo".to_owned(),
            LocalPathEcho(received_c),
            Some(serde_json::json!({"type": "object", "properties": {"path": {"type": "string"}}})),
        )
        .expect("register");
    let harness = handle
        .create_local_harness("virt-local")
        .expect("local harness");
    let stream = harness
        .call(
            xai_tool_protocol::ToolId::new("local_path_echo").expect("valid"),
            serde_json::json!({ "path": "/workspace/foo.txt" }),
            ToolCallContext::default(),
        )
        .await;
    let typed = drain_terminal_ok(stream).await;
    assert_eq!(
        received
            .lock()
            .expect("lock")
            .as_ref()
            .and_then(|v| v.get("path")),
        Some(&serde_json::json!("/workspace/conv-abc/foo.txt")),
        "local harness must rewrite inbound /workspace"
    );
    let dumped = typed.value.to_string();
    assert!(
        dumped.contains("/workspace/out.txt"),
        "local harness must rewrite outbound: {dumped}"
    );
    assert!(
        !dumped.contains("/workspace/conv-abc/"),
        "local harness must not leak the real root: {dumped}"
    );
}
#[tokio::test]
async fn fork_inherits_path_virtualization() {
    let handle = make_handle();
    handle
        .session("main")
        .expect("main")
        .set_path_virtualization(
            crate::path_virtualization::PathVirtualization::try_from_session_root(
                "/workspace/conv-abc",
            )
            .expect("valid"),
        );
    let child = handle
        .fork_session(crate::config::AgentSessionConfig {
            parent_session_id: Some("main".into()),
            ..crate::config::AgentSessionConfig::new("child-virt")
        })
        .await
        .expect("fork");
    let virt = child
        .path_virtualization()
        .expect("fork must inherit mapping");
    assert_eq!(virt.real_root(), "/workspace/conv-abc");
}
