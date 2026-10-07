use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use super::{AgentConfig, BootstrapError, bootstrap_with_cancel, hold_bootstrap_gate_for_tests};
use tokio_util::sync::CancellationToken;
use xai_grok_login::{AuthManager, GrokComConfig};

fn file_len_and_mtime(path: &std::path::Path) -> Option<(u64, Option<std::time::SystemTime>)> {
    std::fs::metadata(path)
        .ok()
        .map(|meta| (meta.len(), meta.modified().ok()))
}

#[test]
fn unit_test_bootstrap_does_not_touch_the_managed_config_lock() {
    let dir = tempfile::tempdir().expect("tempdir");
    let home = dir.path().to_str().expect("utf8 temp home");
    let _env = crate::env::EnvVarGuard::set("GROK_HOME", home)
        .and_set("GROK_DEPLOYMENT_KEY", "unit-test-deployment-key");
    // `grok_home()` is a process-wide OnceLock. Read the env path directly so this
    // test neither observes a home cached by an earlier test nor pins one for later tests.
    let grok_home = xai_dirs::resolve_grok_home().expect("GROK_HOME is set");
    let lock = grok_home.join("managed_config.lock");
    let lock_before = file_len_and_mtime(&lock);
    let mut cfg = AgentConfig {
        remote_settings: Some(Default::default()),
        ..AgentConfig::default()
    };
    cfg.models.allowed_models = Some(vec!["[".to_string()]);
    let auth = Arc::new(AuthManager::new(dir.path(), GrokComConfig::default()));

    let err = match bootstrap_with_cancel(&cfg, &auth, None, &CancellationToken::new(), None) {
        Err(err) => err,
        Ok(_) => panic!("an invalid model filter must stop bootstrap before init"),
    };

    assert!(
        matches!(err, BootstrapError::Config(ref message) if message.contains("allowed_models")),
        "bootstrap must pass the gate and fail on the filter, got {err}"
    );
    assert_eq!(
        lock_before,
        file_len_and_mtime(&lock),
        "the unit-test build must not create or touch the managed-config lock under {}",
        grok_home.display()
    );
}

#[test]
fn cancelled_bootstrap_returns_before_side_effects() {
    let dir = tempfile::tempdir().expect("tempdir");
    let auth = Arc::new(AuthManager::new(dir.path(), GrokComConfig::default()));
    let cancel = CancellationToken::new();
    cancel.cancel();
    let err = match bootstrap_with_cancel(&AgentConfig::default(), &auth, None, &cancel, None) {
        Err(err) => err,
        Ok(_) => panic!("a pre-cancelled token must not run bootstrap"),
    };
    assert!(matches!(err, BootstrapError::Cancelled), "got {err}");
}

#[test]
fn second_bootstrap_bails_when_cancelled_while_the_gate_is_held() {
    let _held = hold_bootstrap_gate_for_tests();
    let entered = Arc::new(AtomicBool::new(false));
    let entered_worker = entered.clone();
    let cancel = CancellationToken::new();
    let worker_cancel = cancel.clone();
    let handle = std::thread::spawn(move || {
        let dir = tempfile::tempdir().expect("tempdir");
        let auth = Arc::new(AuthManager::new(dir.path(), GrokComConfig::default()));
        entered_worker.store(true, Ordering::SeqCst);
        bootstrap_with_cancel(&AgentConfig::default(), &auth, None, &worker_cancel, None)
    });
    let started = std::time::Instant::now();
    while !entered.load(Ordering::SeqCst) && started.elapsed() < Duration::from_secs(2) {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(entered.load(Ordering::SeqCst), "waiter never started");
    std::thread::sleep(Duration::from_millis(40));
    cancel.cancel();
    let err = match handle.join().expect("waiter thread") {
        Err(err) => err,
        Ok(_) => panic!("cancelled waiter must not run bootstrap beside the holder"),
    };
    assert!(matches!(err, BootstrapError::Cancelled), "got {err}");
}
