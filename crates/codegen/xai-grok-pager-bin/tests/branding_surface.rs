//! Branding gate for the installed binary's user-visible surface.
//!
//! Runs the real `deepseek-build` binary and fails if its help, version,
//! completions, or keyless headless output prints the legacy product names
//! (`grok`, `xAI`, `x.ai`) outside the small allowlist of deliberate legacy
//! compatibility mentions. Also pins the binary artifact name to the
//! centralized brand constant so a rename cannot silently diverge.

use std::io::Read;
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// Resolve the built binary.
///
/// `CARGO_BIN_EXE_<name>` is only exported when the binary target is part of
/// the same build unit graph; `cargo test --test <name>` and `cargo clippy
/// --all-targets` may omit it, so fall back to the binary sitting next to the
/// test executable in the same profile directory (`target/<profile>/deps/…`
/// → `target/<profile>/deepseek-build`).
fn bin() -> std::path::PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_deepseek-build") {
        let path = std::path::PathBuf::from(path);
        if path.exists() {
            return path;
        }
    }
    let exe = std::env::current_exe().expect("test executable path");
    let sibling = exe.parent().and_then(std::path::Path::parent).map(|dir| {
        dir.join(format!(
            "{}{}",
            xai_grok_brand::NAME,
            std::env::consts::EXE_SUFFIX
        ))
    });
    if let Some(path) = sibling.filter(|p| p.exists()) {
        return path;
    }
    panic!(
        "could not locate the deepseek-build binary next to {}; build it with \
         `cargo build -p xai-grok-pager-bin`",
        exe.display()
    );
}

/// Case-insensitive legacy branding tokens that must not reach users.
const LEGACY_TOKENS: &[&str] = &["grok", "xai", "x.ai"];

/// Captured child result; [`std::process::Output`] cannot be built by hand and
/// its `wait_with_output` has no timeout, so the runner owns the pieces.
struct RunResult {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl RunResult {
    fn combined(&self) -> String {
        format!(
            "{}{}",
            String::from_utf8_lossy(&self.stdout),
            String::from_utf8_lossy(&self.stderr)
        )
    }
}

/// Run the binary with a wall-clock guard so a hung child fails the test
/// instead of hanging the suite. Stdout/stderr are drained on reader threads,
/// because a verbose child (e.g. `completions bash`) would otherwise block on a
/// full pipe while we wait for it to exit.
fn run(args: &[&str]) -> RunResult {
    let mut cmd = Command::new(bin());
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    run_child(cmd)
}

fn run_child(mut cmd: Command) -> RunResult {
    #[allow(clippy::disallowed_methods)] // test-only child, waited on with a kill-on-timeout guard
    let mut child = cmd.spawn().expect("spawn deepseek-build");
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let drain = |mut pipe: Box<dyn Read + Send>| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = pipe.read_to_end(&mut buf);
            buf
        })
    };
    let out_thread = drain(Box::new(stdout));
    let err_thread = drain(Box::new(stderr));

    let deadline = Instant::now() + Duration::from_secs(120);
    let status = loop {
        match child.try_wait().expect("wait for deepseek-build") {
            Some(status) => break status,
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("deepseek-build did not exit within 120s: {cmd:?}");
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    };
    RunResult {
        status,
        stdout: out_thread.join().expect("stdout reader"),
        stderr: err_thread.join().expect("stderr reader"),
    }
}

/// Legacy tokens found in `text`, reported as `token: line` pairs for the
/// failure message.
fn legacy_hits(text: &str) -> Vec<String> {
    let lowered = text.to_lowercase();
    let mut hits = Vec::new();
    for token in LEGACY_TOKENS {
        if let Some(pos) = lowered.find(token) {
            let line = lowered[..pos].matches('\n').count() + 1;
            let snippet = text.lines().nth(line - 1).unwrap_or("");
            hits.push(format!("{token:?} at line {line}: {snippet}"));
        }
    }
    hits
}

/// The binary artifact must be named after the centralized brand constant, so
/// the installed name and the printed resume hints cannot drift apart.
#[test]
fn binary_artifact_is_brand_named() {
    let bin = bin();
    let file_name = bin
        .file_name()
        .and_then(|n| n.to_str())
        .expect("binary path has a file name");
    let stem = file_name.strip_suffix(".exe").unwrap_or(file_name);
    assert_eq!(
        stem,
        xai_grok_brand::NAME,
        "built binary {} does not match the brand constant",
        bin.display()
    );
}

#[test]
fn help_version_completions_and_headless_output_avoid_old_branding() {
    let cases: &[&[&str]] = &[
        &["--help"],
        &["agent", "--help"],
        &["--version"],
        &["completions", "bash"],
    ];
    for args in cases {
        let out = run(args);
        let combined = out.combined();
        let hits = legacy_hits(&combined);
        assert!(
            hits.is_empty(),
            "`deepseek-build {}` prints legacy branding: {hits:#?}\n--- output ---\n{combined}",
            args.join(" ")
        );
    }
}

/// Keyless headless run must fail fast with the API-key error and no legacy
/// branding anywhere in stdout/stderr. API keys are removed from the child so
/// the test can never reach the network (or print a real key) even when the
/// developer has one exported.
#[test]
fn keyless_headless_run_avoids_old_branding() {
    let mut cmd = Command::new(bin());
    cmd.args(["-p", "branding gate", "--output-format", "json"])
        .env_remove("DEEPSEEK_API_KEY")
        .env_remove("DEEPSEEK_BUILD_API_KEY")
        .env_remove("GROK_API_KEY")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let scratch = std::env::temp_dir().join(format!(
        "deepseek-build-branding-gate-{}",
        std::process::id()
    ));
    let _ = std::fs::create_dir_all(&scratch);
    cmd.env("DEEPSEEK_BUILD_HOME", &scratch);
    let out = run_child(cmd);
    let _ = std::fs::remove_dir_all(&scratch);

    let combined = out.combined();
    assert!(
        !out.status.success(),
        "keyless headless run unexpectedly succeeded:\n{combined}"
    );
    let hits = legacy_hits(&combined);
    assert!(
        hits.is_empty(),
        "keyless headless run prints legacy branding: {hits:#?}\n--- output ---\n{combined}"
    );
}
