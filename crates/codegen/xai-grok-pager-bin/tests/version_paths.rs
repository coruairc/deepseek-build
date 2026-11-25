//! Both `--version` paths, asserted on one host.
//!
//! `--version` prints `deepseek-build <version> (<commit>)`, and `<version>`
//! comes from one of two independent paths:
//!
//! - **source path**: `xai-grok-pager-bin`'s `build.rs` falls back to
//!   `CARGO_PKG_VERSION` when `GROK_VERSION` is unset, so a build from this
//!   tree reports the manifest version (inherited from
//!   `[workspace.package] version`);
//! - **CI stamp path**: the release workflow exports `GROK_VERSION` (derived
//!   from the tag) and `build.rs` stamps it instead of the manifest value.
//!
//! Commands that exercise each path:
//!
//! ```text
//! env -u GROK_VERSION cargo test -p xai-grok-pager-bin --test version_paths
//! GROK_VERSION=9.9.9-test cargo test -p xai-grok-pager-bin --test version_paths
//! ```
//!
//! The second command's sentinel differs from the manifest value, so the stamp
//! path cannot pass by inheriting the source path's value. Both `build.rs`
//! files declare `cargo:rerun-if-env-changed=GROK_VERSION`, so toggling the
//! variable rebuilds the binary instead of reusing the previous stamp.

use std::process::{Command, Stdio};

/// Resolve the built binary.
///
/// `CARGO_BIN_EXE_<name>` is exported for integration tests, but `cargo
/// clippy --all-targets` can omit it, so fall back to the binary sitting next
/// to the test executable in the same profile directory (`target/<profile>/deps/…`
/// → `target/<profile>/deepseek-build`).
fn bin() -> std::path::PathBuf {
    if let Some(path) = option_env!("CARGO_BIN_EXE_deepseek-build") {
        let path = std::path::PathBuf::from(path);
        if path.exists() {
            return path;
        }
    }
    let exe = std::env::current_exe().expect("test executable path");
    let sibling = exe
        .parent()
        .and_then(std::path::Path::parent)
        .map(|dir| dir.join(format!("deepseek-build{}", std::env::consts::EXE_SUFFIX)));
    if let Some(path) = sibling.filter(|p| p.exists()) {
        return path;
    }
    panic!(
        "could not locate the deepseek-build binary next to {}; build it with \
         `cargo build -p xai-grok-pager-bin`",
        exe.display()
    );
}

/// The workspace version pin.
///
/// This names the value on purpose: a test comparing `VERSION` to
/// `env!("CARGO_PKG_VERSION")` would be tautological, while this one fails if
/// the inheritance is reverted to a literal or the workspace version drifts.
#[test]
fn workspace_version_is_0_1_0() {
    assert_eq!(
        env!("CARGO_PKG_VERSION"),
        "0.1.0",
        "xai-grok-pager-bin must inherit `[workspace.package] version = \"0.1.0\"`"
    );
}

/// `--version` must name the `GROK_VERSION` stamp when set, and the workspace
/// manifest version when not.
///
/// The expectation is read at *runtime* so a stale compile-time constant in
/// the test cannot satisfy the assertion. The child runs keyless and with a
/// scratch `DEEPSEEK_BUILD_HOME`: `--version` exits before any network, and
/// removing the keys keeps that a structural fact rather than a convention.
#[test]
fn version_flag_reports_the_stamp_or_the_manifest() {
    let expected = std::env::var("GROK_VERSION").unwrap_or_else(|_| "0.1.0".to_string());
    let scratch = std::env::temp_dir().join(format!(
        "deepseek-build-version-paths-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&scratch).expect("create scratch DEEPSEEK_BUILD_HOME");

    let output = Command::new(bin())
        .arg("--version")
        .env_remove("DEEPSEEK_API_KEY")
        .env_remove("DEEPSEEK_BUILD_API_KEY")
        .env_remove("GROK_API_KEY")
        .env("DEEPSEEK_BUILD_HOME", &scratch)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run deepseek-build --version");
    let _ = std::fs::remove_dir_all(&scratch);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "`deepseek-build --version` failed ({}):\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
        output.status
    );

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "`--version` must print exactly one line:\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
    let tokens: Vec<&str> = lines[0].split_whitespace().collect();
    assert_eq!(
        tokens.len(),
        3,
        "`--version` must print `deepseek-build <version> (<commit>)`:\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
    assert_eq!(
        tokens[0], "deepseek-build",
        "binary name token:\n--- stdout ---\n{stdout}"
    );
    assert_eq!(
        tokens[1], expected,
        "version token must be the `GROK_VERSION` stamp when set, else the workspace \
         manifest version:\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );

    let commit_token = tokens[2];
    assert!(
        commit_token.starts_with('(') && commit_token.ends_with(')'),
        "commit token must be parenthesized, got {commit_token:?}:\n--- stdout ---\n{stdout}"
    );
    let commit = &commit_token[1..commit_token.len() - 1];
    assert!(
        (7..=40).contains(&commit.len())
            && commit.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')),
        "commit inside the parentheses must be 7-40 lowercase hex characters \
         (`build.rs` truncates the git revision to 12), got {commit:?}:\n--- stdout ---\n{stdout}"
    );
}
