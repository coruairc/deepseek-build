//! Relaunch the pager into the same session under a different agent.
//!
//! This is the **fallback** for `/agent`: the shell normally swaps the harness in place
//! (`deepseek-build/session/switch_agent`). When the swap is unavailable or fails, the event loop
//! quits, the terminal is restored, and this module re-execs the binary with
//! `--resume <session> --agent <name>`; the new process replays the conversation under the
//! requested agent.
//!
//! The argv rebuild mirrors [`super::screen_mode_relaunch`]: session-selection and one-shot
//! creation flags are stripped because the appended `--resume` owns the session, the positional
//! prompt is dropped so it does not re-fire, and every other launch flag is preserved.

use std::ffi::{OsStr, OsString};
use std::io::{self, Write};

use super::screen_mode_relaunch::flag_takes_value;

/// Internal marker set on the re-exec so the new process can show a one-line "switched" notice.
/// Read and removed exactly once at startup by [`take_agent_switch_env_override`]; not a user interface.
pub(crate) const AGENT_SWITCH_ENV: &str = "DEEPSEEK_BUILD_AGENT_SWITCH";

/// Build argv (without the binary name) that resumes `session_id` under `agent_name`.
///
/// Drops prior session-selection flags, the old `--agent`, one-shot creation directives, and the
/// positional prompt; preserves every other flag (and a value token when the flag takes one).
/// Appends exactly `--resume <session_id> --agent <agent_name>`.
pub(crate) fn build_agent_relaunch_args(
    current_args: impl IntoIterator<Item = impl AsRef<OsStr>>,
    session_id: &str,
    agent_name: &str,
) -> Vec<OsString> {
    let mut iter = current_args
        .into_iter()
        .map(|a| a.as_ref().to_os_string())
        .peekable();
    // Drop argv[0] (binary path); the caller supplies the exe separately.
    let _ = iter.next();

    let mut out: Vec<OsString> = Vec::new();
    while let Some(arg) = iter.next() {
        let s = arg.to_string_lossy();

        // `--` ends flag parsing; everything after it is the positional prompt, which must not
        // re-fire on resume. Drop the separator too.
        if s == "--" {
            break;
        }

        // The old agent selection is replaced by the appended `--agent`.
        if s == "--agent" {
            if let Some(next) = iter.peek() {
                let ns = next.to_string_lossy();
                if !ns.starts_with('-') {
                    let _ = iter.next();
                }
            }
            continue;
        }
        if s.starts_with("--agent=") {
            continue;
        }

        // Boolean flags with no surviving meaning on the resume path.
        // `--restore-code` already ran in the old process; `--continue` / `--fork-session` would
        // reselect a different session than the appended `--resume`.
        if matches!(
            s.as_ref(),
            "--continue" | "-c" | "--fork-session" | "--restore-code"
        ) {
            continue;
        }

        // `--flag=value` forms of the session-selection flags.
        if s.starts_with("--resume=")
            || s.starts_with("--load=")
            || s.starts_with("--session-id=")
            || s.starts_with("-s=")
            || s.starts_with("--worktree=")
            || s.starts_with("--worktree-ref=")
            || s.starts_with("--ref=")
        {
            continue;
        }

        // Session-selection / one-shot session-creation flags with an optional following value.
        // `--session-id` would make the appended `--resume` an invalid combo, and a worktree flag
        // would create a second worktree.
        if matches!(
            s.as_ref(),
            "--resume"
                | "-r"
                | "--load"
                | "--session-id"
                | "-s"
                | "--worktree"
                | "-w"
                | "--worktree-ref"
                | "--ref"
        ) {
            if let Some(next) = iter.peek() {
                let ns = next.to_string_lossy();
                if !ns.starts_with('-') {
                    let _ = iter.next();
                }
            }
            continue;
        }

        // Keep flags (and their value tokens when space-separated).
        if s.starts_with('-') {
            let takes_value = !s.contains('=') && flag_takes_value(s.as_ref());
            out.push(arg);
            if takes_value && let Some(next) = iter.peek() {
                let ns = next.to_string_lossy();
                if !ns.starts_with('-') {
                    out.push(iter.next().expect("peeked value present"));
                }
            }
            continue;
        }

        // Bare positional prompt, which must not re-fire on resume.
        continue;
    }

    out.push(OsString::from("--resume"));
    out.push(OsString::from(session_id));
    out.push(OsString::from("--agent"));
    out.push(OsString::from(agent_name));
    out
}

/// Pasteable command when auto re-exec fails.
pub(crate) fn agent_relaunch_resume_hint(session_id: &str, agent_name: &str) -> String {
    format!(
        "{} --resume {session_id} --agent {agent_name}",
        xai_grok_brand::NAME
    )
}

/// Replace the current process with a relaunch into `session_id` under `agent_name`.
/// On success this function never returns.
/// On failure it returns the IO error so the caller can print a hint and exit non-zero.
pub(crate) fn exec_agent_relaunch(session_id: &str, agent_name: &str) -> io::Result<()> {
    let exe = std::env::current_exe()?;
    let args = build_agent_relaunch_args(std::env::args_os(), session_id, agent_name);

    let mut cmd = std::process::Command::new(&exe);
    cmd.args(&args);
    // Marker for the new process's one-line notice; removed on read so tools never see it.
    cmd.env(AGENT_SWITCH_ENV, agent_name);

    eprintln!("Switching to agent '{agent_name}' and resuming the session\u{2026}");
    let _ = io::stdout().flush();
    let _ = io::stderr().flush();

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        // `exec` only returns on failure.
        Err(io::Error::other(format!("failed to exec relaunch: {err}")))
    }

    #[cfg(windows)]
    {
        // Windows has no exec(2): spawn on the inherited console and park the parent in `wait`,
        // exiting with the child's status.
        cmd.stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());
        // A console Ctrl event reaches every attached process; keep this parent alive mid-`wait`.
        // SAFETY: FFI with a null handler pointer, documented to mean "ignore Ctrl+C"; called once,
        // before the child exists, from the only surviving thread of a quiesced event loop.
        unsafe {
            windows_sys::Win32::System::Console::SetConsoleCtrlHandler(None, 1);
        }
        // Let the parent's input reader park before the child attaches to the same console input.
        std::thread::sleep(std::time::Duration::from_millis(150));
        #[allow(clippy::disallowed_methods)] // the parent waits and exits with the child's status
        let mut child = cmd.spawn()?;
        let status = child.wait()?;
        std::process::exit(status.code().unwrap_or(0));
    }

    #[cfg(not(any(unix, windows)))]
    {
        Err(io::Error::other(
            "agent relaunch unsupported on this platform",
        ))
    }
}

/// Consume the one-shot agent-switch marker (see [`AGENT_SWITCH_ENV`]).
/// Every spawned child would otherwise inherit it.
pub(crate) fn take_agent_switch_env_override() -> Option<String> {
    let raw = std::env::var_os(AGENT_SWITCH_ENV);
    if raw.is_some() {
        // SAFETY: called once during pager startup, before the event loop and before this process
        // spawns threads that read the environment. Any set value is removed so children never
        // inherit the override.
        unsafe { std::env::remove_var(AGENT_SWITCH_ENV) };
    }
    raw.and_then(|v| v.into_string().ok())
        .filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<OsString> {
        parts.iter().map(|s| OsString::from(*s)).collect()
    }

    fn as_strs(v: &[OsString]) -> Vec<String> {
        v.iter().map(|s| s.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn minimal_argv_is_resume_then_agent() {
        let out = build_agent_relaunch_args(args(&["deepseek-build"]), "sid-1", "explore");
        assert_eq!(as_strs(&out), ["--resume", "sid-1", "--agent", "explore"]);
    }

    #[test]
    fn preserves_unrelated_flags_and_values() {
        let out = build_agent_relaunch_args(
            args(&[
                "deepseek-build",
                "--model",
                "deepseek-v4-pro",
                "--minimal",
                "--no-alt-screen",
            ]),
            "sid-1",
            "explore",
        );
        assert_eq!(
            as_strs(&out),
            [
                "--model",
                "deepseek-v4-pro",
                "--minimal",
                "--no-alt-screen",
                "--resume",
                "sid-1",
                "--agent",
                "explore",
            ]
        );
    }

    #[test]
    fn replaces_old_agent_in_both_forms() {
        for old in [
            vec!["deepseek-build", "--agent", "plan", "--effort", "high"],
            vec!["deepseek-build", "--agent=plan", "--effort", "high"],
        ] {
            let out = build_agent_relaunch_args(old, "sid-2", "explore");
            assert_eq!(
                as_strs(&out),
                [
                    "--effort", "high", "--resume", "sid-2", "--agent", "explore"
                ]
            );
        }
    }

    #[test]
    fn drops_prior_session_selection_and_prompt() {
        for old in [
            vec!["deepseek-build", "--resume", "old", "--agent", "plan"],
            vec!["deepseek-build", "-r", "old"],
            vec!["deepseek-build", "--session-id", "old"],
            vec!["deepseek-build", "--resume=old"],
            vec!["deepseek-build", "--continue"],
            vec!["deepseek-build", "-c"],
            vec!["deepseek-build", "--fork-session"],
            vec!["deepseek-build", "--restore-code"],
            vec!["deepseek-build", "--worktree", "wt"],
            vec!["deepseek-build", "--", "fix the bug"],
            vec!["deepseek-build", "fix the bug"],
        ] {
            let label = format!("{old:?}");
            let out = build_agent_relaunch_args(old, "sid-3", "explore");
            assert_eq!(
                as_strs(&out),
                ["--resume", "sid-3", "--agent", "explore"],
                "unexpected argv for {label}"
            );
        }
    }
}
