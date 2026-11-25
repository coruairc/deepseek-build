//! SESS-02: plan mode is read-only except the plan file, in every permission mode.
//!
//! One real pager session per permission mode. Each iteration enters plan mode, scripts an edit
//! outside the session plan directory and asserts the shell refuses it on screen (the refusal
//! names the plan file), parses the plan path out of that refusal, then edits that exact path and
//! asserts it is not refused — and that the file on disk actually changed, so "not refused" is
//! proved by an executed edit rather than by the absence of a string.
//!
//! Chip assertions follow the UI-SPEC table: the seeded mode is `ask`, and `Ask` renders **no**
//! permission chip, so the `default` iteration asserts the `plan` chip and the absence of a
//! permission chip; only the `auto` and `always-approve` iterations assert one
//! (`auto-review`, `always-approve`).

use super::common::*;

/// Read-only rules only: nothing here can auto-approve an edit, so a plan-mode refusal cannot be
/// confused with a permission grant.
const SETTINGS: &str = r#"{"permissions":{"allow":["Read(*)","Search(*)"]}}"#;
/// Every iteration seeds `ask`; the permission chips come from `/auto` and `/always-approve`.
const CONFIG: &str = "[ui]\npermission_mode = \"ask\"\n";
/// The shell's refusal for an edit that is not the plan file (`plan_mode.rs` template).
const REFUSAL_PREFIX: &str =
    "Rejected: file edits are not allowed in plan mode - the only editable file is the plan file (";

const PLAN_CHIP: &str = "· plan";

/// Screen text with every run of whitespace collapsed, so a wrapped tool result still matches.
fn collapsed(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Double-click the row holding `needle`, which toggles that scrollback block's fold.
/// A failed edit block finishes collapsed (`EditToolCallBlock::finished_display_mode`), so the
/// refusal is only on screen once the row is expanded.
fn double_click_text(harness: &mut PtyHarness, needle: &str) {
    let screen = harness.screen_contents();
    let (row, col) = locate_screen_text(&screen, needle)
        .unwrap_or_else(|| panic!("locate {needle:?}; screen:\n{screen}"));
    let click = format!(
        "{}{}",
        sgr_mouse(0, row, col, 'M'),
        sgr_mouse(0, row, col, 'm'),
    );
    for _ in 0..2 {
        harness
            .inject_keys(click.as_bytes())
            .expect("inject SGR click");
        harness.update(Duration::from_millis(100));
    }
}

/// Tab focuses the scrollback (the prompt yields keys); the same key returns to the prompt.
fn focus_scrollback(harness: &mut PtyHarness) {
    harness.inject_keys(b"\t").expect("focus scrollback (tab)");
    harness.update(Duration::from_millis(300));
    harness
        .wait_for_text("Space:prompt", Duration::from_secs(10))
        .expect("scrollback owns keys");
}

fn focus_prompt(harness: &mut PtyHarness) {
    harness.inject_keys(b"\t").expect("refocus prompt (tab)");
    harness.update(Duration::from_millis(300));
}

/// Run one scripted tool turn and wait for the follow-up turn to settle.
fn run_tool_turn(
    harness: &mut PtyHarness,
    content: &ContentController,
    call_id: &str,
    args: serde_json::Value,
) {
    let expectation = expect_tool_turn(content, call_id, "search_replace", args.to_string());
    let settled = format!("SETTLED_{call_id}");
    content.set_response(&settled);
    harness
        .inject_keys(format!("apply {call_id}\r").as_bytes())
        .expect("submit scripted tool turn");
    harness
        .wait_for_text(&settled, Duration::from_secs(90))
        .unwrap_or_else(|_| {
            panic!(
                "turn {call_id} never settled:\n{}",
                harness.screen_contents()
            )
        });
    expectation.assert_satisfied();
    harness
        .wait_for_turn_idle(Duration::from_secs(30))
        .expect("turn finalized before the next request");
}

/// One mode's full pass: enter plan mode, arm the mode's permission setting, refuse an out-of-plan
/// edit, then allow the plan file. Returns the observed refusal line and the parsed plan path.
async fn run_one_mode(mode: &str) -> (String, String) {
    let content = ContentController::start().await.expect("mock inference");
    let cwd = content.sandbox().workspace().to_path_buf();
    git2::Repository::init(&cwd).expect("fixture repository");
    std::fs::create_dir_all(cwd.join(".claude")).expect("settings directory");
    std::fs::write(cwd.join(".claude/settings.json"), SETTINGS).expect("read-only settings");
    std::fs::write(content.sandbox().grok_home().join("config.toml"), CONFIG).expect("ask config");
    std::fs::create_dir_all(cwd.join("src")).expect("src directory");
    std::fs::write(cwd.join("src/main.rs"), "fn main() {}\n").expect("fixture source file");

    let binary = pager_binary().expect("supplied pager binary");
    let mut harness = PtyHarness::spawn_with_content_env_ops_in_dir(
        &binary,
        DEFAULT_ROWS,
        DEFAULT_COLS,
        &content,
        &["--trust", "--no-leader"],
        // Pin the auto-permission-mode gate so `/auto` is offered regardless of the runner's shell.
        &[EnvOp::set("GROK_AUTO_PERMISSION_MODE", "1")],
        Some(&cwd),
    )
    .expect("spawn pager");
    harness
        .wait_for_text(WELCOME_SCREEN_SENTINEL, WELCOME_TIMEOUT)
        .expect("welcome frame");

    // Bind a session: mode commands and tool calls are session-scoped.
    content.set_response(format!("{MOCK_RESPONSE_SENTINEL} {mode} ready"));
    harness
        .inject_keys(format!("{PROMPT}\r").as_bytes())
        .expect("bind session");
    harness
        .wait_for_text(MOCK_RESPONSE_SENTINEL, Duration::from_secs(60))
        .expect("session response rendered");
    harness
        .wait_for_turn_idle(Duration::from_secs(30))
        .expect("first turn idle");

    // Plan mode first: every iteration must show the plan chip. The chip is the UI signal (the
    // pager applies the toggle optimistically); the behavioural proof is the refusal below, which
    // cannot appear unless the shell's plan tracker is active.
    harness.inject_keys(b"/plan\r").expect("submit /plan");
    harness
        .wait_for_text(PLAN_CHIP, Duration::from_secs(30))
        .unwrap_or_else(|_| {
            panic!(
                "{mode}: the plan chip never appeared after /plan:\n{}",
                harness.screen_contents()
            )
        });
    assert!(
        harness.contains_text(PLAN_CHIP),
        "{mode}: the plan chip must be on the info row:\n{}",
        harness.screen_contents()
    );

    match mode {
        "auto" => {
            inject_keys_paced(&mut harness, b"/auto");
            harness.inject_keys(b"\r").expect("submit /auto");
            harness
                .wait_for_text("plan · auto-review", Duration::from_secs(20))
                .unwrap_or_else(|_| {
                    panic!(
                        "auto: no auto-review chip after /auto:\n{}",
                        harness.screen_contents()
                    )
                });
        }
        "always-approve" => {
            inject_keys_paced(&mut harness, b"/always-approve");
            harness.inject_keys(b"\r").expect("submit /always-approve");
            // The chip, not the typed draft: a chip is preceded by the info-row separator.
            harness
                .wait_for_text("plan · always-approve", Duration::from_secs(20))
                .unwrap_or_else(|_| {
                    panic!(
                        "always-approve: no always-approve chip after /always-approve:\n{}",
                        harness.screen_contents()
                    )
                });
        }
        "default" => {
            // Seeded `ask` renders no permission chip (UI-SPEC Mode indicator table).
            let screen = harness.screen_contents();
            for chip in ["auto-review", "always-approve"] {
                assert!(
                    !screen.contains(chip),
                    "default (ask): no permission chip may render, found {chip}:\n{screen}"
                );
            }
        }
        other => panic!("unknown mode {other}"),
    }
    // Let the mode change settle on the shell side before the tool turn is submitted.
    harness.update(Duration::from_millis(500));

    // ── An edit outside the plan file must be refused ──────────────────────
    let outside = cwd.join("src/main.rs");
    run_tool_turn(
        &mut harness,
        &content,
        "edit_outside_plan",
        json!({
            "file_path": outside.display().to_string(),
            "old_string": "fn main() {}",
            "new_string": "fn main() { /* edited outside plan mode */ }",
        }),
    );
    // The model-facing tool result is the primary evidence: it is what the shell returned for the
    // call, independent of how the row is folded on screen.
    let outside_result = content
        .requests()
        .iter()
        .find_map(|request| request.tool_results().remove("edit_outside_plan"))
        .unwrap_or_default();
    let outside_body = std::fs::read_to_string(&outside).unwrap_or_default();
    eprintln!(
        "{mode}: out-of-plan tool result: {outside_result}\n{mode}: out-of-plan file now: {outside_body:?}"
    );
    assert!(
        outside_result.contains(REFUSAL_PREFIX),
        "{mode}: the shell refused nothing — the tool result carries no plan-mode refusal: \
         {outside_result}"
    );
    assert!(
        !outside_body.contains("edited outside plan mode"),
        "{mode}: the out-of-plan edit ran despite the refusal: {outside_body:?}"
    );

    // A failed edit block finishes collapsed, so expand its row before asserting the screen shows
    // the refusal. (The tool result above is what the shell returned; this is what the user sees.)
    focus_scrollback(&mut harness);
    double_click_text(&mut harness, "search_replace");
    harness
        .wait_for_text(
            "Rejected: file edits are not allowed in plan mode",
            Duration::from_secs(15),
        )
        .unwrap_or_else(|_| {
            panic!(
                "{mode}: the expanded tool row shows no plan-mode refusal:\n{}",
                harness.screen_contents()
            )
        });
    focus_prompt(&mut harness);

    let screen = collapsed(&harness.full_text());
    assert!(
        screen.contains(REFUSAL_PREFIX),
        "{mode}: the shell did not show the plan-mode refusal on screen; tool result was: \
         {outside_result}\n{}",
        harness.screen_contents()
    );
    // Parse the path from the unwrapped tool result: the screen wraps the 200-char refusal, so a
    // path read back from the collapsed screen would carry a display-only space inside it.
    let refusal_start = outside_result
        .find(REFUSAL_PREFIX)
        .unwrap_or_else(|| panic!("{mode}: no refusal in {outside_result}"));
    let refusal = outside_result.get(refusal_start..).unwrap_or_default();
    let plan_path = refusal
        .get(REFUSAL_PREFIX.len()..)
        .and_then(|rest| rest.split_once(')'))
        .map(|(path, _)| path.trim().to_owned())
        .unwrap_or_else(|| panic!("{mode}: the refusal names no plan path: {refusal}"));
    assert!(
        plan_path.starts_with('/'),
        "{mode}: parsed plan path is not absolute: {plan_path:?}"
    );
    let refusal_line = refusal.chars().take(260).collect::<String>();

    // ── An edit to the plan file itself must not be refused ────────────────
    std::fs::write(&plan_path, "# Plan\n\nSESS02MARKER\n").expect("seed the plan file");
    run_tool_turn(
        &mut harness,
        &content,
        "edit_plan_file",
        json!({
            "file_path": plan_path,
            "old_string": "SESS02MARKER",
            "new_string": "SESS02MARKER edited",
        }),
    );
    let plan_result = content
        .requests()
        .iter()
        .find_map(|request| request.tool_results().remove("edit_plan_file"))
        .unwrap_or_default();
    assert!(
        !plan_result.contains("not allowed in plan mode"),
        "{mode}: the plan file itself must not be refused, got: {plan_result}"
    );
    let plan_body = std::fs::read_to_string(&plan_path).unwrap_or_default();
    assert!(
        plan_body.contains("SESS02MARKER edited"),
        "{mode}: the plan-file edit did not execute; plan file is: {plan_body:?}"
    );

    eprintln!(
        "{mode}: plan chip ok; refusal='{refusal_line}'; plan path={plan_path}; plan-file edit ran"
    );

    write_cast_if_requested(&harness, &format!("plan-read-only-{mode}.cast"));
    harness.quit().expect("quit pager");
    (refusal_line, plan_path)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "PTY e2e; requires PAGER_BINARY"]
async fn plan_mode_read_only_in_every_permission_mode() {
    assert!(
        std::env::var_os("PAGER_BINARY").is_some(),
        "supply PAGER_BINARY; implicit builds are forbidden"
    );
    // One pager process and one sandbox per mode, so plan state never leaks between iterations.
    for mode in ["default", "auto", "always-approve"] {
        let (refusal, plan_path) = run_one_mode(mode).await;
        assert!(
            refusal.contains(REFUSAL_PREFIX),
            "{mode}: refusal line lost its prefix: {refusal}"
        );
        assert!(
            plan_path.ends_with("plan.md"),
            "{mode}: the refusal must name the session plan file, got {plan_path}"
        );
    }
}
