use super::WorkspaceHostKind;
use xai_grok_tools::registry::types::ToolServerConfig;

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
