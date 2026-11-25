//! Plan-approval chrome restored by the shell after quit and resume.
//!
//! When `exit_plan_mode` is parked and the user quits, the shell persists `awaiting_plan_approval = true` in `plan_mode.json`.
//! On `--continue` the shell re-issues the `deepseek-build/exit_plan_mode` reverse-request, a real live ACP waiter.
//! The pager then re-shows approval chrome through its normal path with no pager-side disk logic.
//! Approving then leaves plan mode and starts the implement turn.
//!
//! This FAILS without the shell re-park: no reverse-request reaches the resumed pager, so no approval chrome appears.

use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result, bail};

use super::wait_for_welcome;
use crate::{ContentController, PtyHarness, pager_binary};

const DEFAULT_ROWS: u16 = 50;
const DEFAULT_COLS: u16 = 120;
const WELCOME_TIMEOUT: Duration = Duration::from_secs(20);
/// Turn 1 seeds the session before quit; turn 2 is the implement turn the shell injects after the resumed approval is approved.
const SETUP_SENTINEL: &str = "GBT3703SETUP";
const IMPLEMENT_SENTINEL: &str = "GBT3703IMPLEMENTED";

const PLAN_BODY: &str = "\
# Plan GBT3703Repro

## Steps
1. Seed plan file on disk
2. Quit pager with the approval parked
3. Resume and expect restored approval chrome
";

/// Regression: the shell re-parks `exit_plan_mode` on resume; pressing approve leaves plan mode and starts the implement turn.
pub async fn assert_plan_approval_restored_after_resume() -> Result<()> {
    let content = ContentController::start()
        .await
        .context("start ContentController")?;
    let mut setup_turn = content.expect_agent_turn(
        "initial plan-drafting turn",
        format!("{SETUP_SENTINEL}: drafted a plan for the user to review."),
    );
    let mut implement_turn = content.expect_agent_turn(
        "implementation after approval",
        format!("{IMPLEMENT_SENTINEL}: implementing the approved plan."),
    );

    let project = tempfile::tempdir().context("project dir")?;
    std::fs::create_dir_all(project.path().join(".git")).context("create .git")?;

    let binary = pager_binary().context("resolve pager binary")?;
    let mut first = PtyHarness::spawn_with_content_in_dir(
        &binary,
        DEFAULT_ROWS,
        DEFAULT_COLS,
        &content,
        &[],
        Some(project.path()),
    )
    .context("spawn first pager")?;

    wait_for_welcome(&mut first).await?;

    first.inject_keys(b"go\r").context("submit setup turn")?;
    first
        .wait_for_text(SETUP_SENTINEL, Duration::from_secs(30))
        .context("setup turn rendered")?;
    tokio::time::timeout(Duration::from_secs(10), setup_turn.wait_satisfied())
        .await
        .context("setup turn expectation timeout")?;

    // Quit and reap BEFORE seeding so the still-live shell cannot re-persist and clobber the seeded state
    first.inject_keys(b"\x11").context("ctrl-q once")?;
    first.update(Duration::from_millis(200));
    first.inject_keys(b"\x11").context("ctrl-q confirm")?;
    first.quit().context("reap first pager")?;

    // The shell has flushed the session record; capture the model it will have to restore.
    let model_before_resume = session_model_id(content.home()).context("model before resume")?;
    let setup_turn_model =
        last_request_model(&content).context("model of the setup turn's request")?;

    let seeded = seed_parked_approval(content.home()).context("seed parked approval")?;
    assert!(seeded > 0, "no session dir seeded");

    let mut resumed = PtyHarness::spawn_with_content_in_dir(
        &binary,
        DEFAULT_ROWS,
        DEFAULT_COLS,
        &content,
        &["--continue"],
        Some(project.path()),
    )
    .context("spawn resumed pager")?;

    // The shell re-parks `exit_plan_mode` on resume, so approval chrome can open immediately and cover chat history
    // Prefer the chrome markers (product signal) over SETUP_SENTINEL, which may not be visible under the plan viewer
    // Without the shell re-park this times out.
    resumed
        .wait_for_text("request changes", WELCOME_TIMEOUT)
        .context("SESS-03: restored approval 'request changes' after --continue")?;
    resumed
        .wait_for_text("quit plan", Duration::from_secs(5))
        .context("SESS-03: restored approval 'quit plan' after resume")?;
    let screen = resumed.screen_contents();
    if !screen.contains("approve") {
        bail!("SESS-03: expected approval primary action after resume\n{screen}");
    }
    // History was seeded before quit; plan body from disk is a stronger signal that the session was restored when chrome already covers the transcript
    if !screen.contains("GBT3703Repro")
        && !screen.contains(SETUP_SENTINEL)
        && !screen.contains("Seed plan file on disk")
    {
        bail!("SESS-03: expected resumed session content (plan body or setup sentinel)\n{screen}");
    }
    if resumed.contains_text("panicked") {
        bail!("SESS-03: pager panicked\n{screen}");
    }

    // Approve: the shell leaves plan mode and injects the implement turn.
    resumed.inject_keys(b"a").context("press 'a' to approve")?;
    resumed
        .wait_for_text(IMPLEMENT_SENTINEL, Duration::from_secs(30))
        .context("approve must leave plan mode and start the implement turn")?;
    tokio::time::timeout(Duration::from_secs(10), implement_turn.wait_satisfied())
        .await
        .context("implement turn expectation timeout")?;

    // The model after the quit: the session record must still name it, and the resumed process must send it.
    // The prompt info row renders `unknown` in this mock (no model catalog), so it cannot distinguish models;
    // the shell's `summary.json` and the resumed request body are the observables this scenario relies on.
    let model_after_resume = session_model_id(content.home()).context("model after resume")?;
    let resumed_turn_model =
        last_request_model(&content).context("model of the resumed turn's request")?;
    if model_after_resume != model_before_resume {
        bail!(
            "SESS-03: the resumed session record carries model '{model_after_resume}', not \
             '{model_before_resume}'"
        );
    }
    if resumed_turn_model != setup_turn_model {
        bail!(
            "SESS-03: the resumed turn asked for model '{resumed_turn_model}', not \
             '{setup_turn_model}'"
        );
    }
    eprintln!(
        "SESS-03: model {setup_turn_model} before and after the quit (summary and request body); \
         plan body shown after resume"
    );

    write_cast_if_requested(&resumed, "resume-fidelity.cast");
    resumed.quit().context("quit resumed pager")?;
    Ok(())
}

/// Dump the resumed pager's asciinema cast into `$GROK_PTY_CAST_DIR/resume-fidelity.cast` when set.
/// Same opt-in and non-fatal contract as `write_cast_if_requested` in the pty_e2e tests, implemented
/// here because this scenario lives in `src/`, not in that test module.
fn write_cast_if_requested(harness: &PtyHarness, file_name: &str) {
    let Ok(dir) = std::env::var("GROK_PTY_CAST_DIR") else {
        return;
    };
    if dir.is_empty() {
        return;
    }
    let path = std::path::PathBuf::from(dir).join(file_name);
    match harness.write_cast(&path) {
        Ok(()) => eprintln!("wrote cast: {}", path.display()),
        Err(e) => eprintln!("failed to write cast {}: {e}", path.display()),
    }
}

/// The model id recorded in the shell's session summary (`summary.json`), the value `restore_persisted_model` reads.
fn session_model_id(home: &Path) -> Result<String> {
    let sessions_root = home.join(".grok").join("sessions");
    let mut model = None;
    for cwd_ent in std::fs::read_dir(&sessions_root).context("read sessions root")? {
        let cwd_ent = cwd_ent.context("cwd entry")?;
        if !cwd_ent.file_type().context("ft")?.is_dir() {
            continue;
        }
        for sess_ent in std::fs::read_dir(cwd_ent.path()).context("read cwd sessions")? {
            let sess_ent = sess_ent.context("session entry")?;
            if !sess_ent.file_type().context("ft")?.is_dir() {
                continue;
            }
            let summary = sess_ent.path().join("summary.json");
            let Ok(raw) = std::fs::read_to_string(&summary) else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
                continue;
            };
            if let Some(id) = value
                .get("current_model_id")
                .and_then(serde_json::Value::as_str)
            {
                model = Some(id.to_owned());
            }
        }
    }
    model.context("no session summary carried current_model_id")
}

/// The model the pager last asked the provider for, from the foreground chat/completions bodies.
/// Auxiliary traffic (titles, summaries) takes no conversation and is skipped.
fn last_request_model(content: &ContentController) -> Result<String> {
    content
        .server()
        .requests()
        .into_iter()
        .filter(|entry| entry.conversation.is_some() && entry.path.contains("/chat/completions"))
        .filter_map(|entry| entry.body)
        .filter_map(|body| {
            body.get("model")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .next_back()
        .context("no chat/completions request body carried a model")
}

/// For every session dir under the sandbox home, write `plan.md` and flip `awaiting_plan_approval` to `true` in `plan_mode.json`.
fn seed_parked_approval(home: &Path) -> Result<usize> {
    let sessions_root = home.join(".grok").join("sessions");
    if !sessions_root.is_dir() {
        bail!(
            "expected sessions under {} after first turn",
            sessions_root.display()
        );
    }
    let mut seeded = 0usize;
    for cwd_ent in std::fs::read_dir(&sessions_root).context("read sessions root")? {
        let cwd_ent = cwd_ent.context("cwd entry")?;
        if !cwd_ent.file_type().context("ft")?.is_dir() {
            continue;
        }
        for sess_ent in std::fs::read_dir(cwd_ent.path()).context("read cwd sessions")? {
            let sess_ent = sess_ent.context("session entry")?;
            if !sess_ent.file_type().context("ft")?.is_dir() {
                continue;
            }
            let dir = sess_ent.path();
            std::fs::write(dir.join("plan.md"), PLAN_BODY).context("write plan.md")?;
            write_awaiting_plan_mode(&dir.join("plan_mode.json"))?;
            seeded += 1;
        }
    }
    if seeded == 0 {
        bail!(
            "expected at least one session dir under {}",
            sessions_root.display()
        );
    }
    Ok(seeded)
}

/// Flip one field and preserve the rest, so schema growth does not require the shell crate. Fresh Active snapshot if the shell wrote nothing.
fn write_awaiting_plan_mode(path: &Path) -> Result<()> {
    let mut value: serde_json::Value = std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_else(|| {
            serde_json::json!({
                "state": "Active",
                "was_previously_active": true,
                "reminder_count": 0,
                "pending_exit_reminder": false,
            })
        });
    let obj = value
        .as_object_mut()
        .context("plan_mode.json must be a JSON object")?;
    // Must be Active for the re-park; awaiting flag is the trigger.
    obj.insert("state".into(), serde_json::Value::String("Active".into()));
    obj.insert(
        "awaiting_plan_approval".into(),
        serde_json::Value::Bool(true),
    );
    std::fs::write(path, serde_json::to_vec_pretty(&value)?).context("write plan_mode.json")?;
    Ok(())
}
