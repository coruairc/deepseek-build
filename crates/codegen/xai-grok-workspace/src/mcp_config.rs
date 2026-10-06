//! MCP server configuration helpers, kept apart from the (removed) hub MCP
//! bridge. Pure validation/ordering of the configured server list.

use std::collections::HashMap;
use std::sync::Arc;

/// Checks one server's tool calls before the server sees them.
/// The host sets one per server with [`crate::config::BindMcpConfig::with_call_gate`].
pub trait McpCallGate: Send + Sync {
    /// `Some` answers the call in the server's place.
    /// The server never sees that call.
    fn before_call<'a>(
        &'a self,
        tool: &'a str,
    ) -> futures::future::BoxFuture<'a, Option<rmcp::model::CallToolResult>>;

    /// Receives the server's answer to every call `before_call` let through.
    fn after_call(&self, result: &rmcp::model::CallToolResult);
}

/// Call gates by server name.
pub type McpCallGates = HashMap<String, Arc<dyn McpCallGate>>;

/// Dedupe MCP server configs by name, LAST definition wins (JSON-object semantics — name-keyed
/// sources can only produce duplicates through list-shaped construction).
pub(crate) fn dedupe_servers_last_wins(servers: &mut Vec<agent_client_protocol::McpServer>) {
    let mut seen = std::collections::HashSet::new();
    let mut dropped = 0usize;
    // Iterate from the back so the LAST occurrence of each name is the one
    // kept, preserving its position.
    for index in (0..servers.len()).rev() {
        let Some(server) = servers.get(index) else {
            continue;
        };
        let name = xai_grok_mcp::servers::mcp_server_name(server).to_owned();
        if !seen.insert(name) {
            servers.remove(index);
            dropped += 1;
        }
    }
    if dropped > 0 {
        tracing::warn!(
            dropped,
            "MCP config has duplicate server names; keeping the last definition of each"
        );
    }
}

/// Compose a host-owned built-in `entry` into `servers`: same-named entries are dropped so none
/// can take its first-party posture, and it goes first so [`cap_servers`] keeps it.
pub fn compose_built_in(
    servers: Vec<agent_client_protocol::McpServer>,
    entry: agent_client_protocol::McpServer,
) -> Vec<agent_client_protocol::McpServer> {
    let name = xai_grok_mcp::servers::mcp_server_name(&entry);
    let (same_name, mut composed): (Vec<_>, Vec<_>) = servers
        .into_iter()
        .partition(|server| xai_grok_mcp::servers::mcp_server_name(server) == name);
    let impersonating = same_name.iter().filter(|server| **server != entry).count();
    if impersonating > 0 {
        tracing::warn!(
            dropped = impersonating,
            server = name,
            "dropping MCP servers that use a reserved built-in server name"
        );
    }
    composed.insert(0, entry);
    composed
}

/// Cap a server-config list at [`crate::config::BindMcpConfig::MAX_SERVERS`], keeping the first
/// entries in config order.
pub(crate) fn cap_servers(servers: &mut Vec<agent_client_protocol::McpServer>) {
    if servers.len() > crate::config::BindMcpConfig::MAX_SERVERS {
        tracing::warn!(
            configured = servers.len(),
            cap = crate::config::BindMcpConfig::MAX_SERVERS,
            "MCP config exceeds the server cap; keeping the first entries in config order"
        );
        servers.truncate(crate::config::BindMcpConfig::MAX_SERVERS);
    }
}
