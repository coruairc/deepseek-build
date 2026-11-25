//! Headless loop-item proofs for phase 02.
//!
//! 02-04 owns the permission-denial copy (PERM-02). Later plans in this phase add
//! tests to this same target; do not split them into a second binary.

use std::collections::BTreeSet;

use serde_json::Value;
use xai_grok_test_support::{
    Conversation, MockInferenceServer, MockModelEntry, MockToolCall, ScriptViolation, TestSandbox,
    Tool, grok_binary, run_headless_in_sandbox_borrowed,
};

const HINT_PREFIX: &str = "Turn ended: a permission-gated tool was cancelled because headless mode cannot prompt for approval.";
const ERROR_LINE: &str = "Error: permission-gated tool cancelled; re-run with --always-approve, or choose a non-interactive permission mode such as --permission-mode auto.";
const ASK_SETTINGS: &str = r#"{"permissions":{"ask":["Bash(*)"]}}"#;

/// Keys `build_json_result` already emits. A denial must not add one.
const KNOWN_RESULT_KEYS: &[&str] = &[
    "text",
    "stopReason",
    "sessionId",
    "requestId",
    "thought",
    "usage",
    "num_turns",
    "usage_is_incomplete",
    "cost_is_partial",
    "total_cost_usd",
    "estimated_cost_usd",
    "estimate",
    "structuredOutput",
    "structuredOutputError",
    "modelUsage",
    "total_cost_usd_ticks",
];

fn write_ask_settings(sandbox: &TestSandbox) {
    let claude = sandbox.workspace().join(".claude");
    std::fs::create_dir_all(&claude).expect("settings dir");
    std::fs::write(claude.join("settings.json"), ASK_SETTINGS).expect("ask settings");
}

fn headless_cmd(sandbox: &TestSandbox, extra: &[&str]) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(grok_binary());
    cmd.current_dir(sandbox.workspace())
        .arg("-p")
        .arg("clean the build dir")
        .arg("--trust")
        .args(extra);
    cmd
}

fn assert_denial(label: &str, status: &std::process::ExitStatus, stderr: &str) {
    assert_eq!(status.code(), Some(1), "{label} exit: {status:?}\n{stderr}");
    assert!(
        stderr.contains(HINT_PREFIX),
        "{label} missing hint line:\n{stderr}"
    );
    assert!(
        stderr.contains("--always-approve") && stderr.contains("--permission-mode auto"),
        "{label} hint must name both flags:\n{stderr}"
    );
    assert!(
        stderr.contains(ERROR_LINE),
        "{label} missing error line:\n{stderr}"
    );
    assert!(
        !stderr.contains("--permission-mode always-approve"),
        "{label} named an invalid flag:\n{stderr}"
    );
}

fn parse_json_result(stdout: &str) -> Value {
    let start = stdout.find('{').expect("json object on stdout");
    let end = stdout.rfind('}').expect("json object close");
    let slice = stdout.get(start..=end).expect("json slice");
    serde_json::from_str(slice).unwrap_or_else(|err| panic!("parse json result: {err}\n{stdout}"))
}

fn assert_cancelled_result_unchanged(value: &Value) {
    assert_eq!(
        value.get("stopReason").and_then(Value::as_str),
        Some("cancelled")
    );
    let keys: BTreeSet<&str> = value
        .as_object()
        .expect("result object")
        .keys()
        .map(String::as_str)
        .collect();
    let unknown: Vec<&&str> = keys
        .iter()
        .filter(|key| !KNOWN_RESULT_KEYS.contains(key))
        .collect();
    assert!(
        unknown.is_empty(),
        "denial added result keys {unknown:?}; have {keys:?}"
    );
}

#[tokio::test]
async fn headless_permission_denial_names_both_flags() {
    let server = MockInferenceServer::start_with_models(vec![
        MockModelEntry::new("deepseek-v4-pro").with_supports_reasoning_effort(true),
    ])
    .await
    .expect("start mock inference server");
    let shell_turn = |nth: usize| {
        Conversation::nth(nth)
            .calls([MockToolCall::new(
                Tool::Shell,
                serde_json::json!({
                    "command": "rm -rf build",
                    "description": "clean the build dir"
                }),
            )])
            .reply("done")
    };
    // Each headless process opens a new session, so the mock numbers them
    // 1, 2, 3. One scripted shell call per process keeps every path on the gate.
    server.set_conversations(vec![shell_turn(1), shell_turn(2), shell_turn(3)]);

    let mut sandbox = TestSandbox::builder().mock_url(server.url()).build();
    if let Some(ca) = server.ca_pem_path() {
        sandbox.set_env("GROK_EXTRA_CA_BUNDLE", ca);
    }
    write_ask_settings(&sandbox);

    let plain = run_headless_in_sandbox_borrowed(headless_cmd(&sandbox, &[]), &sandbox).await;
    println!("plain stdout:\n{}", plain.stdout);
    println!("plain stderr:\n{}", plain.stderr);
    assert_denial("plain", &plain.status, &plain.stderr);

    let json = run_headless_in_sandbox_borrowed(
        headless_cmd(&sandbox, &["--output-format", "json"]),
        &sandbox,
    )
    .await;
    println!("json stdout:\n{}", json.stdout);
    println!("json stderr:\n{}", json.stderr);
    assert_denial("json", &json.status, &json.stderr);
    let result = parse_json_result(&json.stdout);
    assert_cancelled_result_unchanged(&result);

    let auto = run_headless_in_sandbox_borrowed(
        headless_cmd(
            &sandbox,
            &["--permission-mode", "auto", "--output-format", "json"],
        ),
        &sandbox,
    )
    .await;
    println!("auto stdout:\n{}", auto.stdout);
    println!("auto stderr:\n{}", auto.stderr);
    assert!(
        !auto.timed_out && auto.status.code().is_some(),
        "auto-mode probe did not terminate: timed_out={} status={:?}",
        auto.timed_out,
        auto.status
    );
    let auto_json_ok = auto.stdout.find('{').is_some_and(|start| {
        auto.stdout.rfind('}').is_some_and(|end| {
            auto.stdout
                .get(start..=end)
                .and_then(|slice| serde_json::from_str::<Value>(slice).ok())
                .is_some()
        })
    });
    assert!(
        auto_json_ok,
        "auto-mode probe JSON did not parse:\n{}",
        auto.stdout
    );
    let hint_present = auto.stderr.contains(HINT_PREFIX);
    let tool_ran = auto.stdout.contains("done")
        || auto.stdout.contains("\"stopReason\": \"end_turn\"")
        || auto.stdout.contains("\"stopReason\":\"end_turn\"");
    println!(
        "auto-mode probe: terminated={} json_parsed={} denial_hint={} tool_ran={} exit={:?}",
        !auto.timed_out,
        auto_json_ok,
        hint_present,
        tool_ran,
        auto.status.code()
    );

    // Cancellation returns before the scripted reply, so each conversation stays
    // unfinished. Disarm the drop check and keep only that expected violation.
    let violations = server.finish_scripts();
    assert!(
        violations.iter().all(|violation| {
            matches!(
                violation,
                ScriptViolation::Unfinished {
                    replies_served: 0,
                    ..
                }
            )
        }),
        "script departed for a reason other than the cancelled reply: {violations:?}"
    );
    assert_eq!(violations.len(), 3, "{violations:?}");
}

/// The stable first sentence of the full plan-mode reminder (`plan_mode.rs`).
const PLAN_REMINDER_FULL: &str =
    "Plan mode is active. Do not make any edits or writes to the system.";
/// The alternating-turn variant, injected once the activation reminder has been recorded.
const PLAN_REMINDER_SPARSE: &str =
    "Plan mode is still active. Do not make any edits or writes to the system";

/// Foreground chat-completions request bodies, in arrival order.
/// Auxiliary traffic (titles, summaries) has no conversation and is skipped, so the last body is a turn.
fn chat_completion_bodies(server: &MockInferenceServer) -> Vec<Value> {
    server
        .requests()
        .into_iter()
        .filter(|entry| entry.conversation.is_some() && entry.path.contains("/chat/completions"))
        .filter_map(|entry| entry.body)
        .collect()
}

/// The tail of the conversation as the model received it: the injected reminder precedes the user prompt.
fn recent_message_text(body: &Value) -> String {
    let messages = body
        .get("messages")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let start = messages.len().saturating_sub(3);
    messages
        .get(start..)
        .unwrap_or(&[])
        .iter()
        .filter_map(|message| serde_json::to_string(message).ok())
        .collect::<Vec<_>>()
        .join("\n")
}

fn body_model(body: &Value) -> &str {
    body.get("model")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("no model on the request body: {body}"))
}

fn body_effort(body: &Value) -> &str {
    body.get("reasoning_effort")
        .and_then(Value::as_str)
        .unwrap_or_else(|| {
            panic!("no reasoning_effort on the request body; did the mock model advertise support? {body}")
        })
}

/// SESS-01/SESS-04: one session, three headless invocations, one sandbox.
/// Every invocation goes through `run_headless_in_sandbox_borrowed` so HOME and the session store persist;
/// `run_headless` would build a fresh sandbox per call and make invocation 2 a brand-new session.
#[tokio::test]
async fn headless_resume_restores_model_effort_and_plan_state() {
    let server = MockInferenceServer::start_with_models(vec![
        MockModelEntry::new("deepseek-v4-pro")
            .with_supports_reasoning_effort(true)
            .with_reasoning_efforts(vec![serde_json::json!("high"), serde_json::json!("low")]),
    ])
    .await
    .expect("start mock inference server");
    server.set_response("ok");

    let mut sandbox = TestSandbox::builder().mock_url(server.url()).build();
    if let Some(ca) = server.ca_pem_path() {
        sandbox.set_env("GROK_EXTRA_CA_BUNDLE", ca);
    }
    // The catalog is local (`is_models_fetch_enabled()` is false), and a custom models endpoint skips the
    // built-in entries, so `-m deepseek-v4-pro` only resolves once a `[model.*]` table declares it.
    // `supports_reasoning_effort` is what makes `--effort` reach the wire at all (`apply_supported_effort`).
    std::fs::write(
        sandbox.grok_home().join("config.toml"),
        "[model.\"deepseek-v4-pro\"]\nsupports_reasoning_effort = true\n",
    )
    .expect("write catalog config");

    let run = |args: &[&str]| {
        let mut cmd = tokio::process::Command::new(grok_binary());
        cmd.current_dir(sandbox.workspace()).args(args);
        cmd
    };

    // Invocation 1: a new session in plan mode, pinned to an explicit model and effort.
    let started = run_headless_in_sandbox_borrowed(
        run(&[
            "-p",
            "start the session",
            "--permission-mode",
            "plan",
            "--trust",
            "-m",
            "deepseek-v4-pro",
            "--effort",
            "high",
            "--output-format",
            "json",
        ]),
        &sandbox,
    )
    .await;
    println!(
        "start stdout:\n{}\nstart stderr:\n{}",
        started.stdout, started.stderr
    );
    assert!(
        !started.timed_out && started.status.success(),
        "start invocation failed: timed_out={} status={:?}\n{}",
        started.timed_out,
        started.status,
        started.stderr
    );
    let session_id = parse_json_result(&started.stdout)
        .get("sessionId")
        .and_then(Value::as_str)
        .expect("sessionId on the first result")
        .to_owned();
    let started_body = {
        let bodies = chat_completion_bodies(&server);
        assert!(
            !bodies.is_empty(),
            "the start invocation sent no foreground chat request"
        );
        bodies.into_iter().last().expect("start request body")
    };
    let requests_after_start = chat_completion_bodies(&server).len();

    // Invocation 2: resume by id with neither -m nor --effort; the session record must supply both.
    let resumed = run_headless_in_sandbox_borrowed(
        run(&[
            "-p",
            "continue",
            "--trust",
            "-r",
            &session_id,
            "--output-format",
            "json",
        ]),
        &sandbox,
    )
    .await;
    println!(
        "resume stdout:\n{}\nresume stderr:\n{}",
        resumed.stdout, resumed.stderr
    );
    assert!(
        !resumed.timed_out && resumed.status.success(),
        "resume invocation failed: timed_out={} status={:?}\n{}",
        resumed.timed_out,
        resumed.status,
        resumed.stderr
    );
    let resumed_id = parse_json_result(&resumed.stdout)
        .get("sessionId")
        .and_then(Value::as_str)
        .expect("sessionId on the resumed result")
        .to_owned();
    assert_eq!(
        resumed_id, session_id,
        "resume started a different session: {resumed_id} != {session_id}"
    );
    // The last foreground request of this invocation: a second turn inside it could not go unseen.
    let resumed_body = {
        let bodies = chat_completion_bodies(&server);
        assert!(
            bodies.len() > requests_after_start,
            "the resume invocation sent no foreground chat request"
        );
        bodies
            .into_iter()
            .skip(requests_after_start)
            .last()
            .expect("resumed request body")
    };
    let requests_after_resume = chat_completion_bodies(&server).len();

    // Invocation 3: an explicit --effort on resume wins over the session value.
    let overridden = run_headless_in_sandbox_borrowed(
        run(&[
            "-p",
            "continue",
            "--trust",
            "-r",
            &session_id,
            "--effort",
            "low",
            "--output-format",
            "json",
        ]),
        &sandbox,
    )
    .await;
    println!(
        "override stdout:\n{}\noverride stderr:\n{}",
        overridden.stdout, overridden.stderr
    );
    assert!(
        !overridden.timed_out && overridden.status.success(),
        "override invocation failed: timed_out={} status={:?}\n{}",
        overridden.timed_out,
        overridden.status,
        overridden.stderr
    );

    // The last foreground request of the override invocation; the earlier bodies must stay untouched.
    let overridden_body = {
        let bodies = chat_completion_bodies(&server);
        assert!(
            bodies.len() > requests_after_resume,
            "the override invocation sent no foreground chat request"
        );
        bodies
            .into_iter()
            .skip(requests_after_resume)
            .last()
            .expect("override request body")
    };

    let started_model = body_model(&started_body).to_owned();
    let started_effort = body_effort(&started_body).to_owned();
    let resumed_model = body_model(&resumed_body);
    let resumed_effort = body_effort(&resumed_body).to_owned();
    let overridden_model = body_model(&overridden_body);
    let overridden_effort = body_effort(&overridden_body).to_owned();

    let resumed_tail = recent_message_text(&resumed_body);
    let resumed_full = resumed_tail.contains(PLAN_REMINDER_FULL);
    let resumed_sparse = resumed_tail.contains(PLAN_REMINDER_SPARSE);
    println!(
        "resume facts: started_model={started_model} started_effort={started_effort} \
         resumed_model={resumed_model} resumed_effort={resumed_effort} \
         override_model={overridden_model} override_effort={overridden_effort} \
         plan_reminder_full={resumed_full} plan_reminder_sparse={resumed_sparse}"
    );

    assert_eq!(
        started_model, "deepseek-v4-pro",
        "invocation 1 did not request the pinned model"
    );
    assert_eq!(
        resumed_model, started_model,
        "the resumed turn requested '{resumed_model}', not the started model '{started_model}'"
    );
    assert_eq!(
        resumed_effort, started_effort,
        "the resumed turn sent effort '{resumed_effort}', not the started effort '{started_effort}'"
    );
    assert!(
        resumed_full || resumed_sparse,
        "the resumed turn carries no plan-mode reminder; plan state was not restored\n{resumed_tail}"
    );
    assert_eq!(
        overridden_effort, "low",
        "an explicit --effort low on resume did not reach the wire ({overridden_effort})"
    );
    assert_ne!(
        overridden_effort, resumed_effort,
        "the explicit resume effort did not override the session value"
    );
    assert_eq!(
        overridden_model, started_model,
        "the effort-only resume drifted off the session model"
    );
}

/// Every `plan_mode.json` under the sandbox's session store (a session dir is `<id>/`).
fn plan_mode_files(sandbox: &TestSandbox) -> Vec<std::path::PathBuf> {
    fn walk(dir: &std::path::Path, depth: usize, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && depth < 3 {
                walk(&path, depth + 1, out);
            } else if path
                .file_name()
                .is_some_and(|name| name == "plan_mode.json")
            {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    walk(&sandbox.grok_home().join("sessions"), 0, &mut out);
    out
}

/// PERM-04: `--permission-mode plan` arms plan mode in a headless run.
///
/// The CLI flag alone lands on the agent definition's inert `permission_mode`; `apply_headless_permission_mode`
/// sends one `session/set_mode` before the prompt (`headless.rs`), and the shell then injects the plan-mode
/// reminder into the next request. This asserts the reminder reached the model in the captured request body —
/// the observable a stale `KNOWN-ISSUES.md` bullet claimed never happened.
#[tokio::test]
async fn headless_permission_mode_plan_arms_plan_mode() {
    let server =
        MockInferenceServer::start_with_models(vec![MockModelEntry::new("deepseek-v4-pro")])
            .await
            .expect("start mock inference server");
    server.set_response("I would plan the change before touching anything.");

    let mut sandbox = TestSandbox::builder().mock_url(server.url()).build();
    if let Some(ca) = server.ca_pem_path() {
        sandbox.set_env("GROK_EXTRA_CA_BUNDLE", ca);
    }

    let mut cmd = tokio::process::Command::new(grok_binary());
    cmd.current_dir(sandbox.workspace())
        .args(["-p", "plan the change", "--permission-mode", "plan"])
        .arg("--trust")
        .args(["--output-format", "json"]);
    let run = run_headless_in_sandbox_borrowed(cmd, &sandbox).await;
    println!("plan stdout:\n{}\nplan stderr:\n{}", run.stdout, run.stderr);
    assert!(
        !run.timed_out && run.status.success(),
        "headless --permission-mode plan failed: timed_out={} status={:?}\n{}",
        run.timed_out,
        run.status,
        run.stderr
    );
    let result = parse_json_result(&run.stdout);
    assert_eq!(
        result.get("stopReason").and_then(Value::as_str),
        Some("end_turn"),
        "the planning turn did not run to completion: {result}"
    );

    // The reminder is injected as a message in the turn's own request; search the whole body so a
    // conversation-shape change cannot hide it from the assertion.
    let bodies = chat_completion_bodies(&server);
    let body = bodies
        .last()
        .expect("--permission-mode plan sent no foreground chat request");
    let rendered = serde_json::to_string(body).expect("serialize the request body");
    let matched = rendered
        .find(PLAN_REMINDER_FULL)
        .map(|at| {
            let rest = rendered.get(at..).unwrap_or_default();
            rest.lines()
                .next()
                .unwrap_or_default()
                .chars()
                .take(160)
                .collect::<String>()
        })
        .unwrap_or_else(|| {
            panic!(
                "the captured request body carries no plan-mode reminder, so plan mode was not \
                 armed:\n{rendered}"
            )
        });
    println!("PERM-04: reminder matched in the captured request body: {matched}");

    // The plan edit gate is armed in the same session: assert the persisted state when the shell
    // wrote one, and record its absence (do not fail) when it did not.
    let plan_state_files = plan_mode_files(&sandbox);
    println!("PERM-04: plan_mode.json files: {plan_state_files:?}");
    for path in &plan_state_files {
        let raw = std::fs::read_to_string(path).expect("read plan_mode.json");
        let state: Value = serde_json::from_str(&raw).expect("plan_mode.json is JSON");
        println!(
            "PERM-04: {} state={}",
            path.display(),
            state
                .get("state")
                .and_then(Value::as_str)
                .unwrap_or("<absent>")
        );
        assert_eq!(
            state.get("state").and_then(Value::as_str),
            Some("Active"),
            "a persisted plan_mode.json from this run must be Active: {raw}"
        );
    }
    if plan_state_files.is_empty() {
        println!(
            "PERM-04: no plan_mode.json was written; only the reminder is observed for this run \
             (recorded in the item's notes.md)"
        );
    }
}
