//! PERM-03: the permission chip is resolved from config on every launch and matches after a resume.
//!
//! Both launches read `[ui] permission_mode = "always-approve"` from the sandbox's own
//! `config.toml` and **neither** passes `--permission-mode`, so a chip that reappears cannot be a
//! repeated flag (research Pitfall 4). The resume target is the session id read from the shell's
//! session directory, not `--continue`, so the resumed process is provably the session this test
//! started rather than whatever happens to be most recent.
//!
//! `/plan` is sent before the quit as an **observation only**: the pager's plan toggle is
//! per-session and not persisted (`settings/registry.rs`), so only the permission chip is PERM-03's
//! claim. What the plan chip did is written to the item's `notes.md`.

use super::common::*;

/// Permission mode owned by the sandbox config, never by a CLI flag on either launch.
const CONFIG: &str = "[ui]\npermission_mode = \"always-approve\"\n";
/// The chip text equals the permission id for `always-approve` (`PermissionLabel::display_name`).
const CHIP: &str = "always-approve";
/// The info row joins its flags with ` · `, so the pair is what a two-chip frame reads as.
const PLAN_AND_PERMISSION: &str = "plan · always-approve";

/// Every session directory name under `<grok home>/sessions/*/*`. A directory name **is** the
/// session id (`storage/relocation/mod.rs` keys candidates by the directory name).
fn session_ids(content: &ContentController) -> Vec<String> {
    let sessions = content.sandbox().grok_home().join("sessions");
    let mut ids = Vec::new();
    let Ok(cwd_dirs) = std::fs::read_dir(&sessions) else {
        return ids;
    };
    for cwd_dir in cwd_dirs.flatten() {
        if !cwd_dir.path().is_dir() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(cwd_dir.path()) else {
            continue;
        };
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                ids.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
    }
    ids.sort();
    ids
}

/// Wait for the shell to flush the session record, then return the single session id.
/// More than one candidate is a hard failure: guessing would break the "same session" claim.
fn only_session_id(content: &ContentController, harness: &mut PtyHarness) -> String {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let ids = session_ids(content);
        match ids.len() {
            1 => return ids[0].clone(),
            _ if Instant::now() >= deadline => {
                panic!(
                    "expected exactly one session directory under {} to resume; found {ids:?}",
                    content.sandbox().grok_home().join("sessions").display()
                );
            }
            _ => harness.update(Duration::from_millis(100)),
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "PTY e2e; requires PAGER_BINARY"]
async fn permission_mode_chip_matches_after_resume() {
    assert!(
        std::env::var_os("PAGER_BINARY").is_some(),
        "supply PAGER_BINARY; implicit builds are forbidden"
    );
    let binary = pager_binary().expect("supplied pager binary");
    let content = ContentController::start().await.expect("mock inference");
    let cwd = content.sandbox().workspace().to_path_buf();
    git2::Repository::init(&cwd).expect("fixture repository");
    std::fs::write(content.sandbox().grok_home().join("config.toml"), CONFIG)
        .expect("always-approve config");

    // ── Launch 1: no permission flag anywhere on the command line ──────────
    let mut first = PtyHarness::spawn_with_content_in_dir(
        &binary,
        DEFAULT_ROWS,
        DEFAULT_COLS,
        &content,
        &["--trust", "--no-leader"],
        Some(&cwd),
    )
    .expect("spawn first pager");
    first
        .wait_for_text(WELCOME_SCREEN_SENTINEL, WELCOME_TIMEOUT)
        .expect("welcome frame");
    let welcome = first.screen_contents();
    assert!(
        welcome.contains(CHIP),
        "the configured permission chip must be on the welcome frame (no flag was passed):\n{welcome}"
    );

    // Bind a session so the shell persists a session directory the resume can name.
    content.set_response(format!("{MOCK_RESPONSE_SENTINEL} permission mode resume"));
    first
        .inject_keys(format!("{PROMPT}\r").as_bytes())
        .expect("submit prompt");
    first
        .wait_for_text(MOCK_RESPONSE_SENTINEL, Duration::from_secs(60))
        .expect("session response rendered");
    first
        .wait_for_turn_idle(Duration::from_secs(30))
        .expect("first turn idle");
    let in_session = first.screen_contents();
    assert!(
        in_session.contains(CHIP),
        "the permission chip must survive into the agent frame:\n{in_session}"
    );

    // Observation only: enter plan mode and record whether the pair rendered.
    first.inject_keys(b"/plan\r").expect("submit /plan");
    let plan_banner_seen = first
        .wait_for_text("Switched to mode: Plan", Duration::from_secs(20))
        .is_ok();
    let plan_chip_before_quit = first.contains_text(PLAN_AND_PERMISSION);
    eprintln!(
        "PERM-03 observation: plan banner={plan_banner_seen} plan+permission pair={plan_chip_before_quit}"
    );

    // The reference frame PERM-03 compares against.
    let before_quit = first.screen_contents();
    assert!(
        before_quit.contains(CHIP),
        "the last pre-quit frame must still carry the permission chip:\n{before_quit}"
    );
    write_cast_if_requested(&first, "permission-mode-before-quit.cast");
    first.inject_keys(b"\x11").expect("ctrl-q once");
    first.update(Duration::from_millis(200));
    first.inject_keys(b"\x11").expect("ctrl-q confirm");
    first.quit().expect("reap first pager");

    let session_id = only_session_id(&content, &mut first);
    eprintln!("PERM-03: resuming session by id {session_id} with no permission flag");

    // ── Launch 2: the same session, still no permission flag ──────────────
    let mut resumed = PtyHarness::spawn_with_content_in_dir(
        &binary,
        DEFAULT_ROWS,
        DEFAULT_COLS,
        &content,
        &["--trust", "--no-leader", "-r", &session_id],
        Some(&cwd),
    )
    .expect("spawn resumed pager");
    // The replay of the first turn is the proof this is the named session, not a fresh one.
    resumed
        .wait_for_text(MOCK_RESPONSE_SENTINEL, RESUME_TIMEOUT)
        .unwrap_or_else(|_| {
            panic!(
                "the resumed pager never replayed the session's first turn:\n{}",
                resumed.screen_contents()
            )
        });
    let after_resume = resumed.screen_contents();
    assert!(
        after_resume.contains(CHIP),
        "the resumed frame must carry the same permission chip as the pre-quit frame:\n{after_resume}"
    );
    assert!(
        !resumed.contains_text("panicked"),
        "pager panicked on resume:\n{after_resume}"
    );
    // Same session, not a fresh one: the first turn's transcript came back.
    let transcript_restored = resumed.full_text().contains(MOCK_RESPONSE_SENTINEL);
    let plan_chip_after_resume = resumed.contains_text("plan · always-approve");
    eprintln!(
        "PERM-03: chip before quit = '{CHIP}', after -r {session_id} = '{CHIP}'; \
         transcript_restored={transcript_restored}; plan chip survived={plan_chip_after_resume}"
    );
    assert!(
        transcript_restored,
        "the resumed pager did not restore the first turn's transcript, so this is not the named \
         session:\n{after_resume}"
    );

    write_cast_if_requested(&resumed, "permission-mode.cast");
    resumed.quit().expect("quit resumed pager");
}
