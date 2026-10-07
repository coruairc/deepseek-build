use super::*;
use crate::capability::CapabilityMode;
use crate::handle::tests::make_handle;
use crate::permission::SandboxPath;
use crate::permission::hub_permission::PermissionHookTransport;
use crate::permission::state::{load_state_from_disk, persist_state};
use crate::permission::types::AccessKind;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::path::Path;
use std::sync::Arc;
use xai_grok_tools::types::tool::{ToolKind, ToolNamespace};
use xai_tool_runtime::{ToolApprovalPolicy, ToolErrorKind};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    Read,
    Grep,
    WebSearch,
    Execute,
    Write,
    Mcp,
    WebFetch,
    AgentMessage,
    Tool,
}
fn class(access: &AccessKind) -> Class {
    match access {
        AccessKind::Read(_) => Class::Read,
        AccessKind::Grep { .. } => Class::Grep,
        AccessKind::WebSearch(_) => Class::WebSearch,
        AccessKind::Bash(_) => Class::Execute,
        AccessKind::Edit(_) => Class::Write,
        AccessKind::MCPTool { .. } => Class::Mcp,
        AccessKind::WebFetch(_) => Class::WebFetch,
        AccessKind::AgentMessage { .. } => Class::AgentMessage,
        AccessKind::Tool(_) => Class::Tool,
    }
}
/// Every tool the daemon advertises, with a representative call and the class the gate must give it.
/// A tool missing here fails the coverage assertion; a tool that decodes into the wrong class fails
/// its row. Reads run unasked; everything else prompts, and only `Write`, `Execute`, `Mcp`, and
/// `WebFetch` have a grant scope an "always" answer can land in.
fn daemon_tool_table() -> Vec<(&'static str, Value, Class)> {
    let mut rows = vec![
        (
            "run_terminal_command",
            json!({"command": "cargo build", "description": "build"}),
            Class::Execute,
        ),
        (
            "monitor",
            json!({"command": "tail -f log", "description": "watch"}),
            Class::Execute,
        ),
        ("read_file", json!({"target_file": "/tmp/a"}), Class::Read),
        ("list_dir", json!({"target_directory": "/tmp"}), Class::Read),
        ("grep", json!({"pattern": "x"}), Class::Grep),
        (
            "search_replace",
            json!({"file_path": "/tmp/a", "old_string": "a", "new_string": "b"}),
            Class::Write,
        ),
        (
            "write",
            json!({"file_path": "/tmp/a", "content": "x"}),
            Class::Write,
        ),
        (
            "kill_command_or_subagent",
            json!({"task_id": "t1"}),
            Class::Read,
        ),
        ("todo_write", json!({"todos": []}), Class::Read),
        (
            "get_command_or_subagent_output",
            json!({"task_ids": ["t1"]}),
            Class::Read,
        ),
        (
            "wait_commands_or_subagents",
            json!({"task_ids": ["t1"], "mode": "wait_all"}),
            Class::Read,
        ),
        (
            "spawn_subagent",
            json!({"prompt": "p", "description": "d", "subagent_type": "general-purpose"}),
            Class::Tool,
        ),
        (
            "scheduler_create",
            json!({"interval": "5m", "prompt": "p"}),
            Class::Tool,
        ),
        ("scheduler_delete", json!({"id": "s1"}), Class::Tool),
        ("scheduler_list", json!({}), Class::Read),
        ("search_tool", json!({"query": "issues"}), Class::Read),
        (
            "use_tool",
            json!({"tool_name": "linear__save_issue", "tool_input": {}}),
            Class::Mcp,
        ),
        ("update_goal", json!({"completed": true}), Class::Read),
        (
            "workflow",
            json!({"source": {"type": "name", "name": "review"}}),
            Class::Tool,
        ),
        ("enter_plan_mode", json!({}), Class::Read),
        ("exit_plan_mode", json!({}), Class::Read),
        ("ask_user_question", json!({"questions": []}), Class::Read),
        (
            "web_fetch",
            json!({"url": "https://example.com"}),
            Class::WebFetch,
        ),
        ("memory_search", json!({"query": "q"}), Class::Read),
        ("memory_get", json!({"path": "notes.md"}), Class::Read),
        (
            "lsp",
            json!({"operation": "hover", "file_path": "/tmp/a.rs", "line": 0, "character": 0}),
            Class::Read,
        ),
    ];
    rows
}
async fn daemon_session(
    handle: &crate::handle::WorkspaceHandle,
) -> Arc<crate::session::WorkspaceSession> {
    daemon_session_at(handle, "gate", None).await
}
/// A daemon-toolset session bound at `cwd`, or at the served root when `None`.
async fn daemon_session_at(
    handle: &crate::handle::WorkspaceHandle,
    session_id: &str,
    cwd: Option<std::path::PathBuf>,
) -> Arc<crate::session::WorkspaceSession> {
    handle
        .create_session_with_config(
            session_id,
            cwd,
            Some(xai_grok_agent::workspace_grok_build_toolset()),
            CapabilityMode::All,
            None,
            false,
        )
        .expect("daemon toolset session")
}
#[tokio::test]
async fn every_daemon_tool_has_the_class_the_table_says() {
    let handle = make_handle();
    let session = daemon_session(&handle).await;
    let toolset = session.toolset();
    let advertised: std::collections::BTreeSet<String> = toolset
        .tool_definitions()
        .into_iter()
        .map(|def| def.function.name)
        .collect();
    let table = daemon_tool_table();
    let covered: std::collections::BTreeSet<String> = table
        .iter()
        .map(|(name, _, _)| (*name).to_owned())
        .collect();
    assert_eq!(
        advertised, covered,
        "every advertised tool needs a classification row"
    );
    for (name, args, expected) in table {
        let input = toolset
            .try_parse(name, &args)
            .await
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        let access = AccessKind::from(&input);
        assert_eq!(expected, class(&access), "{name}");
        assert_eq!(
            !matches!(expected, Class::Read | Class::Grep | Class::WebSearch),
            requires_approval(&access),
            "{name}"
        );
    }
}
#[test]
fn gate_is_off_for_the_daemon_until_sandboxing_lands() {
    use WorkspaceHostKind::*;
    for (host, hitl_opt_in, expected) in [
        (Daemon, false, ToolApprovalGate::Off),
        (Daemon, true, ToolApprovalGate::Off),
        (Sandbox, false, ToolApprovalGate::Off),
        (Sandbox, true, ToolApprovalGate::Enforced),
    ] {
        assert_eq!(
            expected,
            resolve_gate(host, hitl_opt_in),
            "{host:?} {hitl_opt_in}"
        );
    }
}
struct StubTransport {
    reply: Value,
    seen: parking_lot::Mutex<Vec<Value>>,
}
impl StubTransport {
    fn new(reply: Value) -> Self {
        StubTransport {
            reply,
            seen: parking_lot::Mutex::new(Vec::new()),
        }
    }
    fn prompts(&self) -> usize {
        self.seen.lock().len()
    }
}
#[async_trait]
impl PermissionHookTransport for StubTransport {
    async fn request_permission(&self, payload: Value) -> Result<Value, String> {
        self.seen.lock().push(payload);
        Ok(self.reply.clone())
    }
}
const SHELL: &str = "run_terminal_command";
fn shell_args(command: &str, request: Value) -> Value {
    let mut args = json!({ "command": command, "description": "hub gate test" });
    args.as_object_mut()
        .unwrap()
        .extend(request.as_object().unwrap().clone());
    args
}
/// Today's daemon: its gate is `Off` (the pre-release stopgap), nothing is asked before any run,
/// and a run requested in the background keeps the path it had: gated, pinned, decoded as before.
#[tokio::test]
async fn with_the_gate_off_a_run_requested_in_the_background_keeps_its_path() {
    let handle = make_handle();
    let session = daemon_session(&handle).await;
    assert_eq!(
        ToolApprovalGate::Off,
        approval_gate_for(WorkspaceHostKind::Daemon)
    );
    let background = shell_args("python -m http.server", json!({ "is_background": true }));
    let path = SandboxPath::Gated
        .for_call(ToolApprovalGate::Off, &session, SHELL, &background)
        .await;
    assert_eq!(SandboxPath::Gated, path);
    assert_eq!(PromptGate::SandboxCard, path.prompt_gate());
}
