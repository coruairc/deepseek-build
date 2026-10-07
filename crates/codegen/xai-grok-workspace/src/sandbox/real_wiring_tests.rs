//! The command sandbox through the real tool path (`hub.rs` → `result_path.rs` →
//! `sandbox_gate.rs`) with real children; where an OS backend is needed, a stub kernel refuses one
//! specific write the way Seatbelt's deny rule would.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::StreamExt;
use serde_json::{Value, json};
use xai_grok_paths::AbsPathBuf;
use xai_grok_sandbox::command::backend::{
    BackendCapabilities, CommandTag, OriginalArgv, RenderedPolicy, SandboxBackend,
    SandboxCommandError, WrapReceipt,
};
use xai_grok_sandbox::command::grants::FixedClock;
use xai_grok_sandbox::command::{
    BackendName, Blocked, CallId, GitConfigEnv, SandboxMode, SandboxPolicy,
};
use xai_grok_telemetry::events::SandboxCommandOutcome;
use xai_grok_tools::registry::types::ToolServerConfig;
use xai_grok_tools::types::tool::ToolKind;
use xai_tool_runtime::{
    ContentBlock, SessionContext, ToolApprovalPolicy, ToolCallContext, ToolCallId, ToolError,
    ToolErrorKind, ToolStream, ToolStreamItem, TypedToolOutput,
};
use xai_tool_types::ToolDescription;

use super::metrics;
use super::result_path::PIN_LOST_TEXT;
use super::{
    BackendSource, CALL_TABLE_FULL_TEXT, CallOwner, ENFORCE_UNAVAILABLE_TEXT, MAX_OPEN_CALLS,
    WorkspaceSandbox, WorkspaceSandboxConfig,
};
use crate::capability::CapabilityMode;
use crate::handle::WorkspaceHandle;
use crate::handle::tests::{BASH_CCO_STUB_NAME, BASH_CCO_STUB_STDOUT};
use crate::host_kind::WorkspaceHostKind;
use crate::permission::{
    StateFileAccess, ToolApprovalGate, approval_gate_for, grant_store_access, load_state_from_disk,
    persist_state,
};
use crate::session::tool_config::test_support::tc;

/// Test-only replacement for the deleted `crate::hub::SessionRoutedToolHandler`.
///
/// The old hub handler dispatched a tool call to the workspace session's `FinalizedToolset`
/// over the hub RPC transport. The hub is deleted, so this dispatches through the in-process
/// local harness, which resolves the same live `FinalizedToolset` and runs the tool in-process
/// (`ToolHarness::call` -> `SessionToolHandle::execute` -> `FinalizedToolset::call_streaming`).
/// Sandbox and permission wiring are unchanged; only the hub RPC transport is gone.
struct SessionRoutedToolHandler {
    handle: WorkspaceHandle,
    tool: String,
}
impl SessionRoutedToolHandler {
    fn new(
        tool: String,
        _desc: ToolDescription,
        _unused: Option<()>,
        handle: WorkspaceHandle,
    ) -> Result<Self, xai_tool_protocol::IdError> {
        Ok(Self { handle, tool })
    }
    async fn handle_call(&self, ctx: ToolCallContext, args: Value) -> ToolStream<TypedToolOutput> {
        let harness = self
            .handle
            .create_local_harness("main")
            .expect("local harness for the main session");
        let tool_id = xai_tool_protocol::ToolId::new(self.tool.clone()).expect("tool id");
        harness.call(tool_id, args, ctx).await
    }
}

/// Everything the stub kernel refuses lives under this tree: outside the workspace, `/tmp` and
/// every build cache.
const REFUSED_TREE: &str = "/srv/grok-w0-real";
/// A refused write whose parent is too shallow to propose (two components): the card offers the
/// one file, confirmed.
const REFUSED_TARGET: &str = "/srv/grok-w0-real/out.txt";
/// A refused write the card can offer a directory for: the highest missing ancestor, `scratch/`.
const GRANTABLE_TARGET: &str = "/srv/grok-w0-real/scratch/out.txt";

/// Stands in for the OS layer. Every spawn is "wrapped" (the receipt says so); a command that
/// names a path under [`REFUSED_TREE`] the policy does not allow is replaced by what the kernel
/// would have made it print.
struct StubKernel {
    wraps: AtomicUsize,
    /// Spawns of a command naming a refused path: the runs the sandbox let through to the OS.
    refused: AtomicUsize,
    /// The tag of every spawn wrapped, in order.
    wrapped: parking_lot::Mutex<Vec<CommandTag>>,
}

impl StubKernel {
    fn new() -> Arc<StubKernel> {
        Arc::new(StubKernel {
            wraps: AtomicUsize::new(0),
            refused: AtomicUsize::new(0),
            wrapped: parking_lot::Mutex::new(Vec::new()),
        })
    }

    /// Whether `call`'s spawn was wrapped.
    fn wrapped(&self, call: &CallId) -> bool {
        self.wrapped.lock().contains(&CommandTag::for_call(call))
    }
}

/// The `Box<dyn SandboxBackend>` the sandbox owns, sharing the counters with the test.
struct KernelHandle(Arc<StubKernel>);

impl SandboxBackend for KernelHandle {
    fn name(&self) -> BackendName {
        BackendName::Seatbelt
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::default()
    }

    fn wrap(
        &self,
        cmd: &mut tokio::process::Command,
        original: &OriginalArgv,
        policy: &SandboxPolicy,
        tag: &CommandTag,
    ) -> Result<WrapReceipt, SandboxCommandError> {
        self.0.wraps.fetch_add(1, Ordering::SeqCst);
        self.0.wrapped.lock().push(tag.clone());
        // Like the kernel, refuse the first write under the tree the policy does not allow; a
        // path a grant widened the policy to is let through
        let refused_path = original.args.iter().find_map(|arg| {
            arg.to_string_lossy()
                .split_whitespace()
                .filter(|token| token.starts_with(REFUSED_TREE))
                .find(|token| {
                    !policy.would_allow(&Blocked::FsWrite {
                        path: PathBuf::from(token),
                    })
                })
                .map(str::to_owned)
        });
        if let Some(path) = refused_path {
            self.0.refused.fetch_add(1, Ordering::SeqCst);
            let script =
                format!("printf '%s\\n' \"touch: {path}: Operation not permitted\" >&2; exit 1");
            let mut replacement = tokio::process::Command::new("/bin/sh");
            replacement.arg("-c").arg(&script);
            replacement.current_dir(&original.cwd);
            *cmd = replacement;
        }
        let mut argv: Vec<OsString> = vec![original.program.clone().into_os_string()];
        argv.extend(original.args.iter().cloned());
        Ok(WrapReceipt {
            backend: BackendName::Seatbelt,
            rendered: RenderedPolicy::Sbpl {
                profile: "(version 1)".to_owned(),
                params: argv
                    .iter()
                    .map(|arg| ("ARG".to_owned(), arg.to_string_lossy().into_owned()))
                    .collect(),
            },
        })
    }
}

struct Fixture {
    _tmp: tempfile::TempDir,
    root: PathBuf,
    grok_home: PathBuf,
}

impl Fixture {
    fn new(mode: &str) -> Fixture {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("ws");
        let grok_home = tmp.path().join("grok-home");
        std::fs::create_dir_all(root.join(".grok")).unwrap();
        std::fs::create_dir_all(&grok_home).unwrap();
        std::fs::write(
            crate::sandbox_mode::workspace_config_path(&root),
            format!("[sandbox]\nmode = \"{mode}\"\n"),
        )
        .unwrap();
        Fixture {
            _tmp: tmp,
            root,
            grok_home,
        }
    }

    fn config(&self, backend: BackendSource) -> WorkspaceSandboxConfig {
        WorkspaceSandboxConfig {
            workspace_root: self.root.clone(),
            grok_home: self.grok_home.clone(),
            user_home: Some(self._tmp.path().join("home")),
            git_env: GitConfigEnv::default(),
            control_socket_dir: self.grok_home.join("workspaced"),
            remote: None,
            backend,
            clock: Arc::new(FixedClock::at(1_800_000_000)),
        }
    }

    async fn sandbox(&self, backend: BackendSource) -> Arc<WorkspaceSandbox> {
        Arc::new(WorkspaceSandbox::open(self.config(backend)).await)
    }

    /// The folder as the daemon serves it: `WorkspaceSandbox::serve`, the one call
    /// `exposures.rs` makes, so the proxy is started or not exactly as it would be in the daemon.
    async fn serve(&self, backend: BackendSource) -> Arc<WorkspaceSandbox> {
        WorkspaceSandbox::serve(self.config(backend)).await
    }

    /// The handle as the daemon builds it: its pre-run approval gate is the one `HOST_KIND_DAEMON`
    /// resolves to, so every test here runs under the real daemon gate.
    fn handle(&self, sandbox: Arc<WorkspaceSandbox>) -> WorkspaceHandle {
        self.handle_with_gate(sandbox, approval_gate_for(WorkspaceHostKind::Daemon))
    }

    fn handle_with_gate(
        &self,
        sandbox: Arc<WorkspaceSandbox>,
        tool_approval: ToolApprovalGate,
    ) -> WorkspaceHandle {
        let handle = WorkspaceHandle::for_test_in_with_sandbox(&self.root, sandbox, tool_approval);
        handle
            .create_session_with_config(
                "main",
                None,
                Some(ToolServerConfig {
                    tools: vec![
                        tc("GrokBuild:run_terminal_cmd", Some(ToolKind::Execute)),
                        tc(
                            "GrokBuild:get_task_output",
                            Some(ToolKind::BackgroundTaskAction),
                        ),
                        tc("GrokBuild:kill_task", Some(ToolKind::KillTaskAction)),
                        tc("GrokBuild:read_file", Some(ToolKind::Read)),
                        tc("GrokBuild:search_replace", Some(ToolKind::Edit)),
                        tc("OpenCode:write", Some(ToolKind::Write)),
                        tc("Codex:apply_patch", Some(ToolKind::Edit)),
                    ],
                    behavior_preset: None,
                }),
                CapabilityMode::All,
                None,
                false,
            )
            .expect("create the session");
        handle
    }
}

/// What one real `run_terminal_cmd` call produced, as the model would see it.
#[derive(Debug)]
struct CallResult {
    /// `Ok(value)` is the typed Bash output; `Err(text)` the tool error.
    outcome: Result<Value, String>,
    /// The tool error's kind, `None` for a result.
    error_kind: Option<ToolErrorKind>,
    model_text: String,
}

async fn run_bash(handle: &WorkspaceHandle, command: &str) -> CallResult {
    run_tool(
        handle,
        "run_terminal_cmd",
        json!({ "command": command, "description": "real wiring test" }),
    )
    .await
}

async fn run_tool(handle: &WorkspaceHandle, tool: &str, args: Value) -> CallResult {
    run_tool_as(handle, tool, args, ToolCallId::new_v7()).await
}

async fn run_tool_as(
    handle: &WorkspaceHandle,
    tool: &str,
    args: Value,
    call_id: ToolCallId,
) -> CallResult {
    let handler = SessionRoutedToolHandler::new(
        tool.to_owned(),
        ToolDescription::new(tool.to_owned(), String::new()),
        None,
        handle.clone(),
    )
    .expect("tool id");
    let mut ctx = ToolCallContext::new(call_id);
    ctx.insert(SessionContext("main".to_owned()));
    terminal_of(handler.handle_call(ctx, args).await).await
}

/// The call's terminal item, as the model would see it.
async fn terminal_of(mut stream: ToolStream<TypedToolOutput>) -> CallResult {
    while let Some(item) = stream.next().await {
        match item {
            ToolStreamItem::Progress(_) => {}
            ToolStreamItem::Terminal(Ok(output)) => {
                let model_text = output
                    .model_output
                    .iter()
                    .filter_map(|block| match block {
                        ContentBlock::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                return CallResult {
                    outcome: Ok(output.value),
                    error_kind: None,
                    model_text,
                };
            }
            ToolStreamItem::Terminal(Err(error)) => {
                return CallResult {
                    outcome: Err(error.to_string()),
                    error_kind: Some(error.kind),
                    model_text: error.to_string(),
                };
            }
        }
    }
    panic!("the tool stream ended without a terminal item");
}

fn exit_code(value: &Value) -> i64 {
    value
        .get("output")
        .and_then(|output| output.get("exit_code"))
        .and_then(Value::as_i64)
        .unwrap_or_else(|| panic!("bash output has exit_code: {value}"))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn enforce_without_a_backend_refuses_the_real_command_with_the_documented_text() {
    let fx = Fixture::new("enforce");
    let sandbox = fx.sandbox(BackendSource::Fixed(None)).await;
    let handle = fx.handle(sandbox.clone());

    let marker = fx.root.join("enforce-must-not-write");
    let result = run_bash(&handle, &format!("touch {}", marker.display())).await;
    assert!(
        !marker.exists(),
        "nothing may run: the refusal happens before the spawn"
    );
    assert!(
        result.model_text.contains(ENFORCE_UNAVAILABLE_TEXT),
        "the model sees the documented text: {result:?}"
    );
    assert_eq!(0, sandbox.open_calls());
    let status = sandbox.status_json();
    assert_eq!(Some(&json!("none")), status.get("backend"));
    assert_eq!(Some(&json!("workspace_config")), status.get("mode_source"));
}

/// A shell tool of a namespace the decoder does not read (the pre-run-prompt path under every
/// mode) that spawns `command` through the sandbox's launch hook, as the terminal does.
struct SpawningShellTool {
    sandbox: Arc<WorkspaceSandbox>,
    cwd: PathBuf,
}

const SPAWNING_SHELL: &str = "spawning_shell";

impl std::fmt::Debug for SpawningShellTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpawningShellTool")
            .field("cwd", &self.cwd)
            .finish()
    }
}

impl xai_grok_tools::types::tool_metadata::ToolMetadata for SpawningShellTool {
    fn kind(&self) -> ToolKind {
        ToolKind::Execute
    }
    fn tool_namespace(&self) -> xai_grok_tools::types::tool::ToolNamespace {
        xai_grok_tools::types::tool::ToolNamespace::MCP
    }
    fn description_template(&self) -> &str {
        "spawning shell"
    }
}

impl xai_tool_runtime::Tool for SpawningShellTool {
    type Args = Value;
    type Output = String;

    fn id(&self) -> xai_tool_protocol::ToolId {
        xai_tool_protocol::ToolId::new(SPAWNING_SHELL).expect("valid id")
    }
    fn description(&self, _ctx: &xai_tool_runtime::ListToolsContext) -> ToolDescription {
        ToolDescription::new(SPAWNING_SHELL, "spawning shell")
    }
    async fn run(&self, ctx: ToolCallContext, input: Value) -> Result<String, ToolError> {
        let command = input
            .get("command")
            .and_then(Value::as_str)
            .expect("a command")
            .to_owned();
        let mut cmd = tokio::process::Command::new("/bin/sh");
        cmd.arg("-c").arg(&command).current_dir(&self.cwd);
        let call = CallId::tool(ctx.call_id.to_string());
        let terminal = |error: String| ToolError::new(ToolErrorKind::TerminalError, error);
        let bound = match self.sandbox.calls.lock().session_of(&call) {
            Some(_) => "bound",
            None => "unbound",
        };
        xai_grok_tools::sandbox_launch::prepare(Some(&*self.sandbox), &mut cmd, &call)
            .map_err(|error| terminal(error.to_string()))?;
        let status = cmd
            .status()
            .await
            .map_err(|error| terminal(error.to_string()))?;
        Ok(format!("spawning shell exited {status}, {bound}"))
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn off_leaves_the_real_command_alone() {
    let fx = Fixture::new("off");
    let kernel = StubKernel::new();
    let sandbox = fx
        .sandbox(BackendSource::Fixed(Some(Box::new(KernelHandle(
            kernel.clone(),
        )))))
        .await;
    let handle = fx.handle(sandbox.clone());
    let result = run_bash(&handle, "echo off-mode").await;
    let value = result.outcome.clone().expect("off runs everything");
    assert_eq!(0, exit_code(&value));
    assert!(result.model_text.contains("off-mode"));
    assert_eq!(0, kernel.wraps.load(Ordering::SeqCst));
    assert_eq!(0, sandbox.open_calls());
}

/// A `HOST_KIND_DAEMON` handle with `[sandbox] mode = off` runs a shell call with zero permission
/// hooks: the daemon's pre-run gate is `Off` and under `off` the sandbox gate has nothing to
/// settle. Were the daemon arm `Enforced`, this handle (no hub) would refuse the call.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn daemon_host_runs_a_shell_call_under_off_with_zero_permission_hooks() {
    let fx = Fixture::new("off");
    let sandbox = fx.sandbox(BackendSource::Fixed(None)).await;
    let owner = Arc::new(ScriptedOwner::silent());
    sandbox.set_card_transport(owner.clone());
    assert_eq!(
        ToolApprovalGate::Off,
        approval_gate_for(WorkspaceHostKind::Daemon)
    );
    let handle = fx.handle(sandbox.clone());

    let result = run_bash(&handle, "echo unasked").await;
    let value = result.outcome.clone().expect("the call ran unasked");
    assert_eq!(0, exit_code(&value), "{result:?}");
    assert!(result.model_text.contains("unasked"), "{result:?}");
    assert!(
        !result.model_text.contains("permission"),
        "no pre-run denial text: {result:?}"
    );
    assert_eq!(
        Vec::<Value>::new(),
        owner.seen.lock().clone(),
        "no card of any kind was posted"
    );
}

/// A tool that holds its run until the test lets it go, so the call table can be read mid-call.
#[derive(Debug)]
struct HeldTool {
    kind: ToolKind,
    id: &'static str,
    started: Arc<tokio::sync::Notify>,
    release: Arc<tokio::sync::Notify>,
}

impl xai_grok_tools::types::tool_metadata::ToolMetadata for HeldTool {
    fn kind(&self) -> ToolKind {
        self.kind
    }
    fn tool_namespace(&self) -> xai_grok_tools::types::tool::ToolNamespace {
        xai_grok_tools::types::tool::ToolNamespace::MCP
    }
    fn description_template(&self) -> &str {
        "held tool"
    }
}

impl xai_tool_runtime::Tool for HeldTool {
    type Args = Value;
    type Output = String;

    fn id(&self) -> xai_tool_protocol::ToolId {
        xai_tool_protocol::ToolId::new(self.id).expect("valid id")
    }
    fn description(&self, _ctx: &xai_tool_runtime::ListToolsContext) -> ToolDescription {
        ToolDescription::new(self.id, "held tool")
    }
    async fn run(
        &self,
        _ctx: ToolCallContext,
        _input: Value,
    ) -> Result<String, xai_tool_runtime::ToolError> {
        self.started.notify_one();
        self.release.notified().await;
        Ok("held tool ran".to_owned())
    }
}

/// The hub gate's grant store is the daemon's own file only while the folder's sandbox is on;
/// under `off`, and with no sandbox wired at all, it stays the CLI's plain file.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_hub_gate_reads_grants_as_the_daemons_own_file_only_while_the_sandbox_is_on() {
    let daemon_owned = StateFileAccess::DaemonOwned {
        grok_home: xai_grok_config::grok_home(),
    };
    for (mode, expected) in [
        ("off", StateFileAccess::Plain),
        ("observe", daemon_owned.clone()),
        ("enforce", daemon_owned),
    ] {
        let fx = Fixture::new(mode);
        let sandbox = fx.sandbox(BackendSource::Fixed(None)).await;
        let handle = fx.handle(sandbox);
        assert_eq!(expected, grant_store_access(&handle), "{mode}");
    }
    let unwired = WorkspaceHandle::for_test();
    assert_eq!(StateFileAccess::Plain, grant_store_access(&unwired));
}

/// The `off` pin at daemon level: the folder is served as the daemon serves it
/// (`WorkspaceSandbox::serve`) and the real command runs with nothing
/// around it — no proxy process or listener for the folder, no proxy or sandbox variables in the
/// child's environment, and (on Linux, read from `/proc`) the shell's argv is exactly what the
/// tool built and its parent is this process, so no wrapper stands between the two. Its
/// `enforce` twin is [`enforce_serves_the_folder_with_a_proxy_the_real_child_is_pointed_at`].
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn off_starts_no_proxy_and_the_real_child_sees_no_sandbox() {
    let fx = Fixture::new("off");
    let kernel = StubKernel::new();
    let sandbox = fx
        .serve(BackendSource::Fixed(Some(Box::new(KernelHandle(
            kernel.clone(),
        )))))
        .await;
    assert!(
        sandbox.network().is_none(),
        "serving an off folder starts no proxy"
    );
    let status = sandbox.status_json();
    assert_eq!(Some(&json!("off")), status.get("network"));
    assert_eq!(Some(&Value::Null), status.get("proxy"));
    let handle = fx.handle(sandbox.clone());

    let command = "echo PPID=$PPID; tr '\\0' ' ' </proc/$$/cmdline; echo; env";
    let result = run_bash(&handle, command).await;
    let value = result.outcome.clone().expect("off runs everything");
    assert_eq!(0, exit_code(&value), "{result:?}");
    let text = &result.model_text;
    for var in [
        "HTTP_PROXY=",
        "HTTPS_PROXY=",
        "http_proxy=",
        "https_proxy=",
        "ALL_PROXY=",
        "GROK_SANDBOX",
    ] {
        assert!(
            !text.contains(var),
            "{var} reached the child under off: {text}"
        );
    }
    #[cfg(target_os = "linux")]
    {
        assert!(
            text.contains(&format!("PPID={}", std::process::id())),
            "the shell's parent is this process, no wrapper in between: {text}"
        );
        let cmdline = text
            .lines()
            .find(|line| line.contains("-O extglob -c "))
            .unwrap_or_else(|| panic!("the shell's own argv is in the output: {text}"));
        let argv0 = cmdline.split(' ').next().unwrap_or_default();
        assert!(
            argv0.ends_with("bash"),
            "argv[0] is the shell itself, not a wrapper: {cmdline}"
        );
        assert!(
            cmdline.trim_end().ends_with(&format!("-- {command}")),
            "the tool's `-- <command>` closes the argv: {cmdline}"
        );
    }
    assert_eq!(0, kernel.wraps.load(Ordering::SeqCst));
    assert_eq!(0, sandbox.open_calls());
    assert!(sandbox.observe_summary().would_block.is_empty());
    assert!(sandbox.network().is_none(), "still no proxy after the run");
}

/// Serving an `off` folder as the daemon does (`WorkspaceSandbox::serve`) builds its shell only:
/// no probe, no protected-path scan, no grant store (the session directory is not created) and
/// no proxy; a real shell call through the daemon gate binds nothing and pins nothing, so nothing
/// re-reads the grant files either. The mode verb's steps (`set_workspace_mode`, then
/// `sync_network`, as `sandbox.mode.set` runs them) engage the folder exactly once on the first
/// mode that is not `off` — `observe`, which still starts no proxy — and the step on to
/// `enforce` starts the proxy without engaging again.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn off_serves_a_shell_and_the_first_mode_verb_that_is_not_off_engages_it_once() {
    let fx = Fixture::new("off");
    let kernel = StubKernel::new();
    let sandbox = fx
        .serve(BackendSource::Fixed(Some(Box::new(KernelHandle(
            kernel.clone(),
        )))))
        .await;
    let own = xai_grok_config::sessions_cwd_dir_in(&fx.grok_home, &fx.root.to_string_lossy());
    assert_eq!(
        0,
        sandbox.engagements(),
        "serving an off folder engages nothing"
    );
    assert!(!own.exists(), "no grant store was opened");
    assert!(sandbox.network().is_none(), "and no proxy started");
    assert_eq!(Some(&json!("none")), sandbox.status_json().get("backend"));

    let handle = fx.handle(sandbox.clone());
    // The call is looked at while its child runs: a binding or a pin is released with the call
    let started = fx.root.join("off-call-started");
    let hold = fx.root.join("off-call-hold");
    std::fs::write(&hold, "").unwrap();
    let command = format!(
        "touch {} && while [ -e {} ]; do sleep 0.05; done",
        started.display(),
        hold.display()
    );
    let running = tokio::spawn({
        let handle = handle.clone();
        async move { run_bash(&handle, &command).await }
    });
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while !started.exists() {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("the child started");
    assert_eq!(
        0,
        sandbox.engagements(),
        "a shell call under off engages nothing"
    );
    assert_eq!(
        0,
        sandbox.calls.lock().len(),
        "and is neither bound nor pinned while it runs"
    );
    assert!(!own.exists());
    std::fs::remove_file(&hold).unwrap();
    let result = running.await.unwrap();
    assert_eq!(0, exit_code(&result.outcome.clone().unwrap()), "{result:?}");
    assert_eq!(0, kernel.wraps.load(Ordering::SeqCst));

    sandbox.set_workspace_mode(SandboxMode::Observe).unwrap();
    assert!(matches!(sandbox.sync_network().await, Ok(None)));
    assert_eq!(
        1,
        sandbox.engagements(),
        "the first mode that is not off engaged it"
    );
    assert!(own.is_dir(), "the grant store is open");
    assert!(sandbox.network().is_none(), "observe starts no proxy");
    assert_eq!(
        Some(&json!(<&str>::from(BackendName::Seatbelt))),
        sandbox.status_json().get("backend"),
        "the backend shows once the folder engaged"
    );

    sandbox.set_workspace_mode(SandboxMode::Enforce).unwrap();
    let info = sandbox
        .sync_network()
        .await
        .expect("the proxy binds")
        .expect("enforce starts one");
    assert!(info.address.ip().is_loopback());
    assert_eq!(
        1,
        sandbox.engagements(),
        "engaged once, whatever the mode does next"
    );
    let result = run_bash(&handle, "echo hi").await;
    assert_eq!(0, exit_code(&result.outcome.clone().unwrap()), "{result:?}");
    assert_eq!(
        1,
        kernel.wraps.load(Ordering::SeqCst),
        "the call ran wrapped"
    );
    assert_eq!(1, sandbox.engagements());
    sandbox.stop_network().await;
}

/// The session owner, scripted through the sandbox's own card transport (no hub on the VM).
struct ScriptedOwner {
    replies: parking_lot::Mutex<std::collections::VecDeque<Value>>,
    seen: parking_lot::Mutex<Vec<Value>>,
}

impl ScriptedOwner {
    /// An owner with no answers: any card posted to it is recorded and fails the settle.
    fn silent() -> ScriptedOwner {
        ScriptedOwner {
            replies: parking_lot::Mutex::new(std::collections::VecDeque::new()),
            seen: parking_lot::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl crate::permission::PermissionHookTransport for ScriptedOwner {
    async fn request_permission(&self, payload: Value) -> Result<Value, String> {
        self.seen.lock().push(payload);
        self.replies
            .lock()
            .pop_front()
            .ok_or_else(|| "no scripted answer left".to_owned())
    }
}
