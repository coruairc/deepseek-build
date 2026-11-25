use super::common::*;

/// No blanket `Bash(*)` ask: that rule is `policy_forced_prompt` and skips the
/// session-grant short-circuit, so an identical call would prompt again.
/// Ask mode itself is what makes the first call prompt. `remember_tool_approvals`
/// must be on or the prompter strips the Always allow row this scenario selects.
const SETTINGS: &str = r#"{"permissions":{}}"#;
const CONFIG: &str = "\
[ui]
permission_mode = \"ask\"
remember_tool_approvals = true

[marketplace]
default_skills_installs_purged = true
";

const ALLOW_ONCE: &str = "Yes, proceed";
const ALWAYS_ALLOW: &str = "Always allow:";
const REJECT: &str = "No, reject (type to add feedback)";
const DENIED: &str = "permission was denied";

fn displayed_option_index(screen: &str, label: &str) -> usize {
    let row = screen
        .lines()
        .find(|line| line.contains(label))
        .unwrap_or_else(|| panic!("missing option row {label:?}:\n{screen}"));
    row.split_whitespace()
        .find_map(|word| word.parse::<usize>().ok())
        .unwrap_or_else(|| panic!("option row has no displayed number: {row}"))
}

/// Move to the row whose label is on screen and confirm it. The digit is parsed
/// from that row; cursor stickiness must not be baked into the test.
fn select_labeled_row(harness: &mut PtyHarness, label: &str) {
    let index = displayed_option_index(&harness.screen_contents(), label);
    assert!(
        (1..=9).contains(&index),
        "unexpected displayed index {index} for {label}"
    );
    let keys = format!("{}{}", "\x1b[A".repeat(9), "\x1b[B".repeat(index - 1));
    harness
        .inject_keys(keys.as_bytes())
        .expect("navigate to the displayed option");
    harness
        .wait_until("labeled option selected", Duration::from_secs(10), |h| {
            h.screen_contents()
                .lines()
                .any(|line| line.contains(label) && (line.contains("(●)") || line.contains("(•)")))
        })
        .unwrap_or_else(|_| {
            panic!(
                "row {label:?} never owned the cursor:\n{}",
                harness.screen_contents()
            )
        });
    harness
        .inject_keys(b"\r")
        .expect("confirm the selected option");
}

fn tool_result(content: &ContentController, call_id: &str) -> Option<String> {
    content
        .requests()
        .iter()
        .find_map(|request| request.tool_results().remove(call_id))
}

fn submit(
    harness: &mut PtyHarness,
    content: &ContentController,
    call_id: &str,
    command: &str,
) -> (AgentTurnExpectation, String) {
    let expectation = expect_tool_turn(
        content,
        call_id,
        "run_terminal_command",
        json!({
            "command": command,
            "description": format!("run {call_id}"),
            "timeout": 10000,
        })
        .to_string(),
    );
    let settled = format!("SETTLED_{call_id}");
    content.set_response(&settled);
    harness
        .inject_keys(format!("run {call_id}\r").as_bytes())
        .expect("submit fixture turn");
    (expectation, settled)
}

fn wait_for_prompt(harness: &mut PtyHarness, call_id: &str) {
    harness
        .wait_for_text(ALLOW_ONCE, Duration::from_secs(90))
        .unwrap_or_else(|_| {
            panic!(
                "permission prompt did not appear for {call_id}:\n{}",
                harness.screen_contents()
            )
        });
}

/// Allow-once, allow-for-session (plus the no-reprompt repeat), and reject, on one
/// real session. The warning row must paint for the destructive command.
/// Commands stay a single plain argv: a `&&` chain suppresses `Always allow:` because
/// that row is offered only when the default scope can persist an exact grant.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "PTY e2e; requires PAGER_BINARY"]
async fn permission_prompt_outcomes_and_warning() {
    assert!(
        std::env::var_os("PAGER_BINARY").is_some(),
        "supply PAGER_BINARY; implicit builds are forbidden"
    );
    let binary = pager_binary().expect("supplied pager binary");
    let content = ContentController::start().await.expect("mock inference");
    let cwd = content.sandbox().workspace();
    git2::Repository::init(cwd).expect("fixture repository");
    std::fs::create_dir_all(cwd.join(".claude")).expect("settings directory");
    std::fs::write(cwd.join(".claude/settings.json"), SETTINGS).expect("ask settings");
    std::fs::write(content.sandbox().grok_home().join("config.toml"), CONFIG).expect("ask config");
    // A chained `&&` hides Always allow, because that grant cannot cover the
    // whole script. Each command stays one argv sequence so the row is offered.
    std::fs::create_dir_all(cwd.join("build")).expect("build dir");
    std::fs::create_dir_all(cwd.join("cache")).expect("cache dir");

    let mut harness = PtyHarness::spawn_with_content_in_dir(
        &binary,
        DEFAULT_ROWS,
        DEFAULT_COLS,
        &content,
        &["--trust", "--no-leader"],
        Some(cwd),
    )
    .expect("spawn pager");
    harness
        .wait_for_text(WELCOME_SCREEN_SENTINEL, WELCOME_TIMEOUT)
        .expect("welcome");

    let (once_expect, once_settled) = submit(
        &mut harness,
        &content,
        "allow_once",
        "rm -rf build",
    );
    wait_for_prompt(&mut harness, "allow_once");
    let prompted = harness.screen_contents();
    assert!(
        prompted.contains("Security warning:"),
        "warning row missing:\n{prompted}"
    );
    for label in [ALLOW_ONCE, ALWAYS_ALLOW, REJECT] {
        assert!(
            prompted.contains(label),
            "missing label {label:?}:\n{prompted}"
        );
    }
    select_labeled_row(&mut harness, ALLOW_ONCE);
    harness
        .wait_for_text(&once_settled, Duration::from_secs(90))
        .unwrap_or_else(|_| panic!("allow-once did not settle:\n{}", harness.screen_contents()));
    let once_result = tool_result(&content, "allow_once").unwrap_or_default();
    assert!(
        once_result.contains("exit: 0"),
        "allow-once did not run the command: {once_result}"
    );
    assert!(
        !cwd.join("build").exists(),
        "allow-once left build/ in place"
    );
    once_expect.assert_satisfied();
    harness
        .wait_for_turn_idle(Duration::from_secs(30))
        .expect("allow-once turn idle");

    let (always_expect, always_settled) = submit(
        &mut harness,
        &content,
        "always_allow",
        "rm -rf cache",
    );
    wait_for_prompt(&mut harness, "always_allow");
    select_labeled_row(&mut harness, ALWAYS_ALLOW);
    harness
        .wait_for_text(&always_settled, Duration::from_secs(90))
        .unwrap_or_else(|_| {
            panic!(
                "always-allow did not settle:\n{}",
                harness.screen_contents()
            )
        });
    let always_result = tool_result(&content, "always_allow").unwrap_or_default();
    assert!(
        always_result.contains("exit: 0"),
        "always-allow did not run the command: {always_result}"
    );
    assert!(
        !cwd.join("cache").exists(),
        "always-allow left cache/ in place"
    );
    always_expect.assert_satisfied();
    harness
        .wait_for_turn_idle(Duration::from_secs(30))
        .expect("always-allow turn idle");

    // The persistent grant must swallow the identical call. A blocking prompt
    // would keep SETTLED_repeat from ever arriving; prior prompt text can still
    // sit in scrollback, so absence of the label is not the signal.
    let (repeat_expect, repeat_settled) = submit(
        &mut harness,
        &content,
        "repeat",
        "rm -rf cache",
    );
    harness
        .wait_until(
            "repeat settles without a new decision",
            Duration::from_secs(90),
            |h| h.contains_text(&repeat_settled),
        )
        .unwrap_or_else(|_| {
            panic!(
                "repeat call prompted again or never settled:\n{}",
                harness.screen_contents()
            )
        });
    let repeat_result = tool_result(&content, "repeat").unwrap_or_default();
    assert!(
        repeat_result.contains("exit: 0"),
        "repeat call did not run under the session grant: {repeat_result}"
    );
    repeat_expect.assert_satisfied();
    harness
        .wait_for_turn_idle(Duration::from_secs(30))
        .expect("repeat turn idle");

    let (reject_expect, reject_settled) = submit(
        &mut harness,
        &content,
        "reject",
        "bash -c 'echo hi'",
    );
    wait_for_prompt(&mut harness, "reject");
    select_labeled_row(&mut harness, REJECT);
    harness
        .wait_for_text(DENIED, Duration::from_secs(30))
        .unwrap_or_else(|_| panic!("missing denial banner:\n{}", harness.screen_contents()));
    harness
        .wait_for_turn_idle(Duration::from_secs(30))
        .expect("rejected turn cancelled");
    assert!(
        !harness.contains_text(&reject_settled),
        "rejected turn must not reach the model reply:\n{}",
        harness.screen_contents()
    );
    let reject_result = tool_result(&content, "reject");
    assert!(
        reject_result
            .as_deref()
            .is_none_or(|body| !body.contains("exit: 0")),
        "rejected command ran: {reject_result:?}"
    );
    reject_expect.assert_satisfied();

    write_cast_if_requested(&harness, "permission-prompt.cast");
    harness.quit().expect("quit pager");
}
