// Per-test-case module for the `pty_e2e` integration test crate.
//
// PTY regression layer for the feedback modal (plan 01-modal PR 1d): the user-visible
// slash/palette/trace/displacement workflows, driven end-to-end with no hidden input owner.
// The minimal-mode flows live in `minimal/minimal_feedback_*.rs` and the draft send in
// `feedback_draft_send_pty.rs`. Shell-gate request inspection stays in 1c-shell's `mvp_agent`
// tests; here the wire evidence is the mock server's recorded `/v1/feedback` posts. The trace
// upload / `/v1/events` telemetry case was deleted with the telemetry stack (rc5, D6).
#[allow(unused_imports)]
use super::common::*;

use anyhow::Context as _;
use xai_grok_pager_pty_harness::{
    InferenceEndpoint, InferenceExpectation, InferenceRequestMatcher, inference_request_count,
};

const USER_FEEDBACK_EVENT: &str = "deepseek-build-user_feedback";

const EMPTY_SUBMIT_SENTINEL: &str = "Add feedback text";
const THANKS_SENTINEL: &str = "Thanks for the feedback";
const SEND_FAILED_SENTINEL: &str = "Couldn't send feedback";
/// Generic trace-step copy: the slash route supplies no `FeedbackType`.
const TRACE_PROMPT_SENTINEL: &str = "Attach this session's trace to your feedback?";
const TRACE_SEND_CHOICE: &str = "Send this session's trace";
const TRACE_FEEDBACK_ONLY_CHOICE: &str = "No, just the feedback";
const QUESTION_DISPLACED_SENTINEL: &str = "Feedback closed because the agent asked a question";

// ── Blocked tool turns ──────────────────────────────────────────────────

/// Register a foreground tool-call turn on both endpoints, held at its terminal barrier
/// ([`ContentController::expect_agent_turn_blocked`] only scripts text).
fn expect_tool_turn_blocked(
    content: &ContentController,
    call_id: &str,
    name: &str,
    args: &str,
) -> [InferenceExpectation; 2] {
    [
        content.expect_response_blocked(
            format!("blocked tool turn {call_id} (responses)"),
            InferenceRequestMatcher::foreground(InferenceEndpoint::Responses),
            ScriptedResponse::sse(responses_api_tool_call_events(call_id, name, args)),
        ),
        content.expect_response_blocked(
            format!("blocked tool turn {call_id} (chat completions)"),
            InferenceRequestMatcher::foreground(InferenceEndpoint::ChatCompletions),
            ScriptedResponse::sse(chat_completions_tool_call_events_with_id(
                call_id, name, args,
            )),
        ),
    ]
}

/// Wait until the active backend's request reached the held terminal barrier.
async fn wait_tool_turn_blocked(expectations: &mut [InferenceExpectation; 2]) {
    let [responses, chat_completions] = expectations;
    let barrier = async {
        tokio::select! {
            _ = responses.wait_blocked() => {}
            _ = chat_completions.wait_blocked() => {}
        }
    };
    tokio::time::timeout(Duration::from_secs(30), barrier)
        .await
        .expect("blocked tool turn reached its terminal barrier");
}

/// Slash open/type/Enter sends and closes; empty Enter refuses first. Inline `/feedback <text>`
/// thanks on Enter without opening the modal or starting a model turn. The palette entry opens
/// the modal without touching the main composer draft.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "PTY e2e; run the owning pty_e2e_* Cargo test with --ignored (see Cargo.toml)"]
async fn feedback_modal_send_prefill_edit_and_palette_pty() {
    const SENT_REPORT: &str = "pty-modal-send-report-alpha";
    const INLINE_REPORT: &str = "pty-modal-inline-send-beta";
    const MAIN_DRAFT: &str = "pty-main-draft-preserved-xyz";

    let content = ContentController::start().await.expect("start content");
    let binary = pager_binary().expect("resolve pager binary");
    let mut harness = PtyHarness::spawn_with_content_env_ops_in_dir(
        &binary,
        DEFAULT_ROWS,
        DEFAULT_COLS,
        &content,
        &["--yolo", "--trust"],
        // The sandbox baseline disables feedback; the send path needs the shell gate open.
        &[EnvOp::set("DEEPSEEK_BUILD_FEEDBACK_ENABLED", "true")],
        Some(content.home()),
    )
    .expect("spawn pager with content");
    enter_session(&mut harness, &content);

    // Slash open, empty refusal, type, Enter: the success closes the modal and thanks.
    open_feedback_modal(&mut harness);
    harness.inject_keys(b"\r").expect("empty enter");
    harness
        .wait_for_text(EMPTY_SUBMIT_SENTINEL, Duration::from_secs(15))
        .expect("empty Enter must show the validation message");
    assert!(
        harness.contains_text(FEEDBACK_MODAL_PLACEHOLDER_SENTINEL),
        "empty Enter must keep the modal open\nscreen:\n{}",
        harness.screen_contents()
    );
    assert!(
        !harness.contains_text(THANKS_SENTINEL),
        "empty Enter must not thank the user\nscreen:\n{}",
        harness.screen_contents()
    );
    inject_keys_paced(&mut harness, SENT_REPORT.as_bytes());
    harness
        .wait_for_text(SENT_REPORT, Duration::from_secs(15))
        .expect("typed report renders in the modal composer");
    harness.inject_keys(b"\r").expect("submit report");
    harness
        .wait_until("modal closed at send", Duration::from_secs(30), |h| {
            !h.contains_text(SENT_REPORT)
        })
        .expect("the committed report must close the modal immediately");
    harness
        .wait_for_text(THANKS_SENTINEL, Duration::from_secs(15))
        .expect("the send thanks the user at close time");

    // Inline `/feedback <text>` thanks a second time on Enter, clears the composer, and neither
    // opens the modal nor starts a model turn.
    let turns_before_inline = inference_request_count(&content);
    inject_keys_paced(
        &mut harness,
        format!("/feedback {INLINE_REPORT}").as_bytes(),
    );
    harness.inject_keys(b"\r").expect("submit inline /feedback");
    harness
        .wait_until("inline report thanked", Duration::from_secs(15), |h| {
            h.screen_contents().matches(THANKS_SENTINEL).count() >= 2
                && !composer_holds(h, INLINE_REPORT)
        })
        .expect("inline /feedback must thank at send time and clear the composer");
    assert!(
        !harness.contains_text(FEEDBACK_MODAL_PLACEHOLDER_SENTINEL),
        "inline /feedback must not open the modal\nscreen:\n{}",
        harness.screen_contents()
    );
    assert_eq!(
        turns_before_inline,
        inference_request_count(&content),
        "inline /feedback must not start a model turn"
    );

    // Palette open lands on the Write tab: the sent report left no draft behind.
    // The main composer draft stays byte-for-byte where it was.
    inject_keys_paced(&mut harness, MAIN_DRAFT.as_bytes());
    harness
        .wait_until("main draft rendered", Duration::from_secs(15), |h| {
            composer_holds(h, MAIN_DRAFT)
        })
        .expect("main composer holds the draft");
    inject_keys_paced(&mut harness, b"\x10"); // Ctrl+P opens the command palette
    inject_keys_paced(&mut harness, b"send feedback");
    harness
        .wait_for_text("Send Feedback", Duration::from_secs(15))
        .expect("palette filter shows the Send Feedback entry");
    harness.inject_keys(b"\r").expect("pick Send Feedback");
    harness
        .wait_for_text(FEEDBACK_MODAL_PLACEHOLDER_SENTINEL, Duration::from_secs(15))
        .expect("palette must open the feedback modal on the Write tab");
    harness.inject_keys(b"\x1b").expect("esc cancels the modal");
    harness
        .wait_until(
            "palette-opened modal closed",
            Duration::from_secs(15),
            |h| !h.contains_text(FEEDBACK_MODAL_PLACEHOLDER_SENTINEL),
        )
        .expect("Esc should close the palette-opened modal");
    assert!(
        composer_holds(&harness, MAIN_DRAFT),
        "the main composer draft must survive the palette round trip untouched\nscreen:\n{}",
        harness.screen_contents()
    );
    assert!(
        !harness.contains_text("panicked"),
        "pager panicked\nscreen:\n{}",
        harness.screen_contents()
    );
    harness.quit().expect("clean quit");
}

/// An `ask_user_question` ingress visibly displaces the open feedback modal: the eviction notice
/// and the question card render, the unsent draft is discarded (not sent, not moved into the main
/// composer), and the question remains answerable.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "PTY e2e; run the owning pty_e2e_* Cargo test with --ignored (see Cargo.toml)"]
async fn feedback_modal_displaced_by_acp_question_pty() {
    const QUESTION_TEXT: &str = "Which displacement detail matters most?";
    const FIRST_OPTION: &str = "Modal eviction";
    const DRAFT: &str = "pty-question-displaced-draft";
    const DONE_SENTINEL: &str = "QUESTIONDISPLACEDONE";

    let content = ContentController::start().await.expect("start content");
    let args = json!({
        "questions": [{
            "question": QUESTION_TEXT,
            "options": [
                { "label": FIRST_OPTION, "description": "The modal must be gone" },
                { "label": "Draft handling", "description": "The draft must not leak" },
            ],
        }]
    })
    .to_string();
    let mut turn = expect_tool_turn_blocked(
        &content,
        "call_displace_question",
        "ask_user_question",
        &args,
    );
    content.set_response(DONE_SENTINEL);

    let binary = pager_binary().expect("resolve pager binary");
    let mut harness = PtyHarness::spawn_with_content_in_dir(
        &binary,
        DEFAULT_ROWS,
        DEFAULT_COLS,
        &content,
        &["--yolo", "--trust"],
        Some(content.home()),
    )
    .expect("spawn pager with content");
    harness
        .wait_for_text(WELCOME_SCREEN_SENTINEL, WELCOME_TIMEOUT)
        .expect("welcome text");
    harness
        .inject_keys(format!("{PROMPT}\r").as_bytes())
        .expect("submit prompt");
    wait_tool_turn_blocked(&mut turn).await;

    // Open the modal mid-turn, while the question is provably still held.
    open_feedback_modal(&mut harness);
    inject_keys_paced(&mut harness, DRAFT.as_bytes());
    harness
        .wait_for_text(DRAFT, Duration::from_secs(15))
        .expect("draft renders in the modal composer");
    assert!(
        !harness.contains_text(QUESTION_TEXT),
        "the held question must not have landed yet\nscreen:\n{}",
        harness.screen_contents()
    );

    for expectation in &turn {
        expectation.release();
    }
    harness
        .wait_for_text(QUESTION_DISPLACED_SENTINEL, Duration::from_secs(30))
        .expect("the eviction notice must be visible");
    harness
        .wait_for_text(QUESTION_TEXT, Duration::from_secs(15))
        .expect("the question card renders after displacing feedback");
    assert!(
        !harness.contains_text(FEEDBACK_MODAL_PLACEHOLDER_SENTINEL),
        "the feedback modal must be gone\nscreen:\n{}",
        harness.screen_contents()
    );
    assert!(
        !harness.contains_text(DRAFT),
        "the discarded draft must not survive anywhere (composer included)\nscreen:\n{}",
        harness.screen_contents()
    );
    assert!(
        !harness.contains_text(THANKS_SENTINEL),
        "a displaced draft must never send\nscreen:\n{}",
        harness.screen_contents()
    );

    // The question stayed the sole input owner: answer it and let the turn settle.
    harness.inject_keys(b" ").expect("mark the first answer");
    harness.inject_keys(b"\r").expect("submit answers");
    harness
        .wait_for_text(DONE_SENTINEL, Duration::from_secs(30))
        .expect("agent turn resumes after the answers");
    assert!(
        !harness.contains_text("panicked"),
        "pager panicked\nscreen:\n{}",
        harness.screen_contents()
    );
    harness.quit().expect("clean quit");
}
