use super::*;
use tracing::Instrument;

const GIT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

pub(super) enum GitRead {
    Value(String),
    Empty,
    Failed,
}

async fn git_read(cwd: &str, args: &[&str]) -> GitRead {
    let mut cmd = tokio::process::Command::from(xai_tty_utils::git_command());
    cmd.current_dir(cwd)
        .args(args)
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true);
    let Ok(Ok(output)) = tokio::time::timeout(GIT_TIMEOUT, cmd.output()).await else {
        return GitRead::Failed;
    };
    match output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
    {
        Some(value) => GitRead::Value(value),
        None => GitRead::Empty,
    }
}

async fn git_out(cwd: &str, args: &[&str]) -> Option<String> {
    match git_read(cwd, args).await {
        GitRead::Value(value) => Some(value),
        GitRead::Empty | GitRead::Failed => None,
    }
}

async fn sample_git_head(cwd: &str) -> (GitRead, Option<String>) {
    (
        git_read(cwd, &["rev-parse", "HEAD"]).await,
        git_out(cwd, &["branch", "--show-current"]).await,
    )
}

#[must_use = "dropping a TurnEndCapture without finish() skips this turn's git-head persistence"]
pub(super) struct TurnEndCapture {
    git_sample: tokio_util::task::AbortOnDropHandle<(GitRead, Option<String>)>,
}

impl TurnEndCapture {
    pub(super) fn begin(_handle: &SessionHandle, head_cwd: String) -> Self {
        let git_sample = tokio_util::task::AbortOnDropHandle::new(tokio::spawn(async move {
            sample_git_head(&head_cwd).await
        }));
        Self { git_sample }
    }

    pub(super) async fn finish(
        self,
        cmd_tx: &tokio::sync::mpsc::UnboundedSender<SessionCommand>,
    ) {
        let (head, head_branch) = self.git_sample.await.unwrap_or((GitRead::Failed, None));
        if let Some((commit, branch)) = plan_git_head(head, head_branch) {
            let _ = cmd_tx.send(SessionCommand::PersistGitHead { commit, branch });
        }
    }
}

fn plan_git_head(
    commit: GitRead,
    branch: Option<String>,
) -> Option<(Option<String>, Option<String>)> {
    match commit {
        GitRead::Value(sha) => Some((Some(sha), branch)),
        GitRead::Empty => Some((None, branch)),
        GitRead::Failed => None,
    }
}

#[cfg(test)]
mod tests {
    use super::GitRead;
    use super::plan_git_head;

    #[test]
    fn plan_git_head_decides_what_to_store() {
        assert_eq!(
            plan_git_head(GitRead::Value("abc123".into()), Some("main".into())),
            Some((Some("abc123".into()), Some("main".into())))
        );
        assert_eq!(
            plan_git_head(GitRead::Value("abc123".into()), None),
            Some((Some("abc123".into()), None))
        );
        assert_eq!(
            plan_git_head(GitRead::Empty, Some("main".into())),
            Some((None, Some("main".into())))
        );
        assert_eq!(plan_git_head(GitRead::Failed, Some("main".into())), None);
    }
}
