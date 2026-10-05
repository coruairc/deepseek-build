use super::WorkspaceHostKind;
use crate::LockedTestEnv;
use crate::capability::CapabilityMode;
use crate::config::SessionContextFactory;
use crate::handle::tests::{bind_resolver_fixture, handler_names};
use crate::handle::{LocalWorkspaceConnectOptions, WorkspaceHandle, build_local_workspace};
use crate::hub_ids::WORKSPACE_RPC_TOOL_ID;
use crate::session::tool_config::resolve_session_toolset;
use serde_json::json;
use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use xai_computer_hub_sdk::{AuthCredential, SharedAuthProvider};
use xai_grok_tools::registry::types::{FinalizedToolset, ToolServerConfig};
use xai_tool_protocol::SessionId;
const API_BASE_URL: &str = "https://api.invalid/v1";
static TRUNCATION_INSTALL_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Tool ids without their namespace, in catalog order.
fn unqualified_ids(config: &ToolServerConfig) -> Vec<&str> {
    config
        .tools
        .iter()
        .map(|tool| {
            tool.id
                .rsplit_once(':')
                .map_or(tool.id.as_str(), |(_, id)| id)
        })
        .collect()
}
fn bearer() -> SharedAuthProvider {
    Arc::new(AuthCredential::bearer("serve-scoped-token"))
}

#[test]
fn a_caller_that_names_no_host_is_hub_only() {
    assert_eq!(WorkspaceHostKind::Daemon, WorkspaceHostKind::default());
    assert!(WorkspaceHostKind::default().is_hub_only());
    assert!(!WorkspaceHostKind::Sandbox.is_hub_only());
}

/// Only a daemon's root is edited from outside the agent, so only a daemon arms the OS watcher.
#[test]
fn only_a_daemon_streams_fs_changes() {
    assert!(WorkspaceHostKind::Daemon.streams_fs_changes());
    assert!(!WorkspaceHostKind::Sandbox.streams_fs_changes());
}

/// The daemon catalog is the sandbox catalog with the API-backed tools cut. The deepseek-build
/// adapter has no API-backed workspace tools left, so the two catalogs are identical today; this
/// pins that a future API-backed tool cannot slip past `api_backed_tool_ids`.
#[test]
fn daemon_catalog_matches_the_sandbox_catalog() {
    assert!(
        xai_grok_agent::api_backed_tool_ids().is_empty(),
        "no API-backed workspace tools remain"
    );
    let sandbox = WorkspaceHostKind::Sandbox.default_toolset();
    let daemon = WorkspaceHostKind::Daemon.default_toolset();
    let ids = unqualified_ids(&sandbox);
    assert_eq!(ids, unqualified_ids(&daemon));
    assert!(ids.contains(&"read_file"), "{ids:?}");
}

/// Build a session the way `connect_local_workspace` does for `host`, with a bearer credential on hand.
async fn session_for(host: WorkspaceHostKind) -> Arc<FinalizedToolset> {
    let _install = TRUNCATION_INSTALL_TEST_LOCK.lock().await;
    let factory = host.session_context_factory(bearer(), API_BASE_URL.to_owned());
    let (_effective, toolset, _backend) = resolve_session_toolset(
        host.default_toolset(),
        CapabilityMode::All,
        &[],
        &[],
        PathBuf::from("/tmp"),
        Arc::new(HashMap::new()),
        &format!("host-kind-{host:?}"),
        &factory,
        None,
        None,
        None,
        None,
    )
    .expect("the host's catalog must finalize");
    toolset
}

/// A daemon host builds sessions with no credential; the sandbox still hands its credential to the
/// session and into `Resources`, so hub-only tools never see a server-side token.
#[tokio::test]
async fn only_a_sandbox_session_carries_the_credential() {
    for (host, expected) in [
        (WorkspaceHostKind::Daemon, false),
        (WorkspaceHostKind::Sandbox, true),
    ] {
        let factory = host.session_context_factory(bearer(), API_BASE_URL.to_owned());
        let ctx = factory.build_session_context(
            &format!("host-kind-{host:?}-ctx"),
            PathBuf::from("/tmp"),
            Arc::new(HashMap::new()),
            factory.build_terminal_backend().backend().clone(),
        );
        assert_eq!(expected, ctx.auth_provider.is_some(), "{host:?}");
        let toolset = session_for(host).await;
        let resources = toolset.resources.lock().await;
        assert_eq!(
            expected,
            resources.contains::<SharedAuthProvider>(),
            "{host:?}"
        );
    }
}

/// Run `test` against everything `connect_local_workspace` builds for `host` short of the hub
/// connection, under a private workspace home: the production seams decide the catalog and bind
/// policy, not a copy of the rule.
fn with_workspace<F: Future<Output = ()>>(
    host: WorkspaceHostKind,
    test: impl FnOnce(WorkspaceHandle) -> F,
) {
    let root = tempfile::tempdir().expect("workspace root");
    let _env = LockedTestEnv::lock()
        .set("GROK_WORKSPACE_HOME", &root.path().join("home"))
        .set(
            "GROK_WORKSPACE_DATA_COLLECTION_DISABLED",
            Path::new("false"),
        );
    tokio::runtime::Runtime::new()
        .expect("runtime")
        .block_on(async {
            let _install = if host == WorkspaceHostKind::Sandbox {
                Some(TRUNCATION_INSTALL_TEST_LOCK.lock().await)
            } else {
                None
            };
            let handle = build_local_workspace(
                root.path().to_path_buf(),
                url::Url::parse("ws://127.0.0.1:1/").expect("hub url"),
                bearer(),
                LocalWorkspaceConnectOptions {
                    allow_insecure_ws: true,
                    host_kind: host,
                    ..LocalWorkspaceConnectOptions::default()
                },
            )
            .await
            .expect("workspace");
            handle.create_session("main").expect("main session");
            test(handle).await;
        });
}
fn pinned(tool_ids: &[&str]) -> serde_json::Value {
    let tools: Vec<serde_json::Value> = tool_ids.iter().map(|id| json!({ "id": id })).collect();
    json!({ "metadata": { "tools": tools } })
}

/// A daemon host, as `connect_local_workspace` builds it: binds must pin their toolset (one without
/// fails closed with `missing_tool_config`), and the daemon carries no credential of its own.
#[test]
fn a_daemon_host_serves_pinned_toolsets_only() {
    with_workspace(WorkspaceHostKind::Daemon, |handle| async move {
        let shared = handle.shared();
        assert!(shared.require_explicit_toolset);
        assert!(shared.auth_provider().is_none());
        let resolver = bind_resolver_fixture(&handle);
        let served = resolver(
            SessionId::new("daemon-pinned").unwrap(),
            Some(pinned(&["GrokBuild:read_file"])),
        )
        .await
        .expect("a pinned bind is served");
        let names = handler_names(&served);
        assert!(names.iter().any(|n| n == "read_file"), "{names:?}");
        assert!(
            served.unserved_tool_ids.is_empty(),
            "{:?}",
            served.unserved_tool_ids
        );
        assert_eq!(None, served.resolve_error);
        let refused = resolver(SessionId::new("daemon-unpinned").unwrap(), None)
            .await
            .expect("a refused bind still answers");
        assert_eq!(
            vec![WORKSPACE_RPC_TOOL_ID.to_owned()],
            handler_names(&refused),
            "the RPC handler alone"
        );
        let reason = refused
            .resolve_error
            .expect("the bind must say why it failed closed");
        assert!(reason.starts_with("missing_tool_config:"), "{reason}");
    });
}

/// The sandbox is unchanged: the credential is in place, and a bind without a toolset still widens
/// to the full catalog unless the launcher asked for strict mode itself.
#[test]
fn a_sandbox_host_keeps_the_credential_and_lax_binds() {
    with_workspace(WorkspaceHostKind::Sandbox, |handle| async move {
        let shared = handle.shared();
        assert!(!shared.require_explicit_toolset);
        assert!(shared.auth_provider().is_some());
        let resolver = bind_resolver_fixture(&handle);
        let served = resolver(
            SessionId::new("sandbox-pinned").unwrap(),
            Some(pinned(&["GrokBuild:read_file"])),
        )
        .await
        .expect("bind");
        let names = handler_names(&served);
        assert!(names.iter().any(|n| n == "read_file"), "{names:?}");
        assert!(served.unserved_tool_ids.is_empty());
        assert_eq!(None, served.resolve_error);
        let widened = resolver(SessionId::new("sandbox-unpinned").unwrap(), None)
            .await
            .expect("bind");
        let names = handler_names(&widened);
        assert!(names.iter().any(|n| n == "read_file"), "{names:?}");
        assert_eq!(None, widened.resolve_error);
    });
}
