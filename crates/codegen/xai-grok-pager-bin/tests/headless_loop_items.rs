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
