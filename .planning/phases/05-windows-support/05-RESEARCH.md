# Phase 5 (inserted): Windows Support — Research

**Researched:** 2026-11-26
**Domain:** Cross-platform port of a Rust TUI coding agent to `x86_64-pc-windows-msvc`
**Confidence:** MEDIUM-HIGH on the shape of the work, LOW on whether it compiles (see A1)

> No `CONTEXT.md` exists for this phase (the directory did not exist before this research). Nothing
> here is constrained by a locked user decision. The brief supplied by the orchestrator is the
> authority; where the brief's counts differ from what the tree measures, the measured number is
> reported and the discrepancy named.

---

## Summary

deepseek-build is **not an unported Unix codebase**. It was ported to Windows upstream and then
forked, re-branded, and shipped Linux-only. The Windows machinery is production code: Win32 **Job
Objects** for process-group teardown, **named pipes** for the leader transport, a **pwsh →
powershell.exe → Git Bash → cmd.exe** cascade, ConPTY via `portable-pty` with
`ENABLE_VIRTUAL_TERMINAL_PROCESSING` console setup, `SetUnhandledExceptionFilter` crash capture, and a
`capability/windows.rs` for foreign-session reads. 183 `cfg(windows)` occurrences exist, 160 in `src/`.

What is missing is not the port. It is **proof, packaging, and one honesty hole**: no workflow has
ever mentioned Windows, the installer rejects it, no docs mention it, and the sandbox has no Windows
backend while `--sandbox` is still accepted and silently ignored there.

**Primary recommendation:** do **not** let Windows block the v0.1.0 freeze. Ship it as a follow-up
whose first task is a `cargo check --target x86_64-pc-windows-msvc` (the single unknown that
dominates the estimate), and whose first *product* decision is whether Windows means "builds and
refuses the sandbox honestly" or "feature parity including a sandbox backend" — those differ ~3×.

---

## 1. Today's Windows posture

### Counts (measured this session)

| Signal | Occurrences | Files | Notes |
|---|---|---|---|
| `target_os = "windows"` | **67** | 26 | matches the brief's 67 |
| `cfg(windows)` | **183** | 66 | 160 in `src/`, 23 in tests |
| `cfg(not(unix))` | 152 | — | the "else" arm of a unix gate |
| `cfg(unix)` | **1,247** | **326** | brief said 327; measured 326. 204 src files, 122 test files |
| `cfg(unix)` + `cfg(windows)` in the same file | — | **48** | these are the already-ported files |
| `UnixStream`/`UnixListener`/`UnixDatagram`/`AF_UNIX` | **57** | 12 | brief said 55; 14 occurrences are outside tests |

The 67 `target_os = "windows"` occurrences are concentrated in the UI layer, the strongest single
signal that upstream shipped a Windows build: `xai-grok-pager-render` 22, `xai-grok-pager` 22,
`xai-grok-pager-pty-harness` 10, `xai-ratatui-textarea` 5, `xai-grok-tools` 3, `xai-grok-shared` 3,
`xai-system-power` 2.

The 326-file `cfg(unix)` number is misleading on its own: **161 of the 204 src files carry a unix gate
with no `cfg(windows)` in the same file**, but most of those sit inside a module whose parent is
`#[cfg(unix)]` (e.g. `xai-grok-tools/src/computer/local/mod.rs:6-9` gates `shell_state` and
`static_shell`), so they are not individually broken — they are simply absent on Windows.

### What already works (verified in-tree)

| Capability | Evidence |
|---|---|
| Node-level process teardown | `xai-tty-utils/src/lib.rs:506-530` builds a Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`; `:185-190` sets `CREATE_NO_WINDOW` |
| Leader IPC | `xai-grok-shell/src/leader/transport.rs:1-10` — "**Windows:** wraps `tokio::net::windows::named_pipe::*` … `\\.\pipe\grok-leader-<hash>`" |
| Shell selection | `xai-grok-config/src/shell.rs:3` — "Default cascade: pwsh, then powershell.exe, then Git Bash, then powershell.exe" |
| Shell tool spawn | `xai-grok-tools/src/computer/local/terminal.rs:3377-3455` — `CREATE_NEW_PROCESS_GROUP`, `CREATE_BREAKAWAY_FROM_JOB` with an `ERROR_ACCESS_DENIED` retry, Job attach |
| Child-exit wake | `xai-grok-tools/src/computer/local/terminal.rs:112-160` — per-child `OpenProcess(PROCESS_SYNCHRONIZE)` + `WaitForSingleObject` instead of `SIGCHLD` |
| Console/TUI | `xai-grok-pager/src/app/mod.rs:1145-1165` — `ENABLE_VIRTUAL_TERMINAL_PROCESSING`, `SetConsoleOutputCP(CP_UTF8)` |
| Crash capture | `xai-crash-handler/src/handler.rs:751` — `SetUnhandledExceptionFilter`; `handler.rs:7` documents `EXCEPTION_ACCESS_VIOLATION` |
| Diagnostics server | `xai-grok-diag-server/src/lib.rs:26-31` — "a Unix socket on Linux, loopback TCP on Windows", `DEFAULT_DIAG_PORT: u16 = 6016` |
| Atomic writes | `xai-grok-config/src/fs_atomic.rs:15-18,426-432` — Windows symlink-follow limits, reparse tags, volume serial + file id |
| Foreign-session reads | `xai-grok-foreign-sessions/src/capability/windows.rs` — 166 lines, `FILE_FLAG_OPEN_REPARSE_POINT` |
| Daemonization | `xai-grok-workspace-daemon/src/daemonize.rs:165` — Windows variant that only redirects stdio |

### Windows-specific dependencies

**13 crates** declare a `[target.'cfg(windows)'.dependencies]` section; **28** declare
`cfg(unix)`. The workspace-level `windows` crate is already pinned with the features a sandbox
backend would need:

```
Cargo.toml:303  windows = { version = "0.61", features = ["Win32_Security", "Win32_Security_Authorization",
                "Win32_Foundation", "Win32_System_Threading", "Win32_System_JobObjects",
                "Win32_System_Console", "Win32_System_Pipes"] }
```
[VERIFIED: Cargo.toml:303]

Resolved versions: `windows 0.61.3` (`Cargo.lock:11219-11221`), `windows-sys 0.59` (workspace
declarations at `xai-grok-pager/Cargo.toml:177`, `xai-grok-pager-render/Cargo.toml:106`).

**No crate declares a unix-only dependency outside a target-gated section.** A script over all 90
member manifests found 7 ungated hits, all of them inert `[workspace.dependencies]` lines
(`Cargo.toml:141,198,249,264,265,266,305`). There is no `openssl`/`native-tls`; `reqwest` is
`rustls-tls` (`Cargo.toml:226`). The dependency graph is already Windows-clean.

**54 of 90** workspace members contain no Windows signal at all. Nearly all are pure-logic crates
(`xai-grok-models`, `xai-grok-sampler`, `xai-grok-config-types`, `xai-grok-markdown*`,
`xai-tool-*`) that need none. Only 9 members are unix-gated with no Windows branch anywhere
(`ptyctl`, `xai-fsnotify`, `xai-grok-diag-server`, `xai-grok-external-agent-migration`,
`xai-grok-lifecycle`, `xai-grok-permission-rules`, `xai-grok-session-search`, `xai-hunk-tracker`,
`xai-workflow`) — and inspection shows the gating in those is **test-only** (symlink fixtures,
`/bin/sh` scripts, `mkfifo`). `xai-grok-diag-server` is a false positive: its TCP path is
unconditional.

### What is stubbed or absent

- **Sandbox enforcement**: `xai-grok-sandbox` compiles to a no-op on non-unix (§4).
- **Bundled search binaries**: `xai-grok-shell/build.rs:46-48` returns early on Windows ("ripgrep
  ships .zip archives there and this script only extracts .tar.gz … falls back to `rg` on PATH");
  `xai-grok-tools/build.rs:63-67` does the same for `fd`, "mirroring the bfs/ugrep skip".
- **Image paste to clipboard**: `xai-grok-shared/src/clipboard.rs:251` — "On Windows, returns an
  error (not yet supported)".
- **Terminal capability probe**: `xai-grok-pager-render/src/terminal/probe.rs` is `#[cfg(unix)]`
  item-by-item with **zero** `#[cfg(not(unix))]` arms.
- **SIGABRT capture**: `xai-crash-handler/src/lib.rs:7` — "SIGABRT capture is Unix-only".

---

## 2. The Unix-only surface, ranked

Ranked by *consequence if unaddressed*, not by count. For each group: Windows equivalent → already
abstracted? → verdict.

### 1. Sandbox enforcement (highest consequence) — see §4
`cfg(unix)` only. No trait. `SandboxManager::apply` has a stub arm. **No direct equivalent; needs a
new backend.** Not abstracted.

### 2. Signal-based lifecycle
`43` `sigaction`, `13` `tokio::signal::unix`, `31` `libc::kill`, `44` `killpg`, `60` `setsid`.
Windows equivalent: `SetConsoleCtrlHandler` (already used — `xai-grok-pager/Cargo.toml:177`),
`CREATE_NEW_PROCESS_GROUP`, `GenerateConsoleCtrlEvent`, `TerminateProcess`, Job Objects. **Mostly
abstracted**: `xai-grok-pager/src/signal_streams.rs:3-42` pairs
`tokio::signal::unix::{interrupt,terminate,hangup}` with a `#[cfg(windows)]` `CtrlC` variant;
`xai-tty-utils` `ProcessGroup` is the Job-Object peer of the unix pgid. Gap: `SIGHUP`/`SIGWINCH` have
no analogue, and the tree uses `hangup_first` semantics for terminal-owning shells.

### 3. Session / IPC transport — **no work**
`xai-grok-shell/src/leader/transport.rs:7-27` is done (named pipes + a `WaitNamedPipeW` liveness
probe at `:180-199`); `xai-grok-diag-server` is done (TCP 6016). The 14 non-test `UnixStream` uses
live in exactly those two files.

### 4. Process control inside the tool runtime
`xai-grok-tools/src/computer/local/terminal.rs` — **abstracted and done** (`#[cfg(not(unix))]` spawn
arm, Job attach). `cgroup.rs` is Linux-only but is `pub mod cgroup` unconditionally
(`local/mod.rs:1`) — not Windows-gated; a compile will judge it.

### 5. File locking
`xai-grok-file-lock` — the *lock* is `fs2` (portable; `LockFileEx` on Windows). The *acquire slot*
is unix-only by design:
```
crates/codegen/xai-grok-file-lock/src/slot.rs:10-13
#[cfg(not(unix))]
pub(crate) use stub::{SlotGuard, SlotHandle};
#[cfg(unix)]
pub(crate) use unix::{SlotGuard, SlotHandle};
```
[VERIFIED: crates/codegen/xai-grok-file-lock/src/slot.rs:10-13]

Windows equivalent: `LockFileEx` on a slot file — feasible. Verdict: **degradation, not breakage**
(a wedged sibling can stall others because the slot is absent). `xai-grok-memory/src/dream_lock.rs:46,60`
already has both arms.

### 6. Path semantics, symlinks, junctions
`std::os::unix::fs::symlink` appears 366 times; `PermissionsExt|.mode(` 541 times. Windows
equivalent: `symlink_file`, reparse tags, `FILE_FLAG_OPEN_REPARSE_POINT`. **Partially abstracted**:
`xai-fast-worktree/src/copy/cow.rs:28-36` pairs `symlink` with `symlink_file`;
`xai-grok-config/src/fs_atomic.rs` already tracks reparse tags. Real risk is *behavioural*:
case-insensitive paths and 260-char `MAX_PATH`. `xai-grok-workspace/src/file_system/file_tree.rs:218-219`
lowercases for sort order, but that is presentation, not identity.

### 7-10. Lower-consequence groups

- **Exec-in-place** (`pager/src/app/screen_mode_relaunch.rs:214-224`): unix `cmd.exec()`, Windows
  "emulate it the way cargo/rustup do" with `SetConsoleCtrlHandler`. **Done.**
- **termios / raw mode**: `termios` 33, `ioctl` 7, `TIOCGWINSZ` **0**. crossterm 0.28.1 + ratatui
  0.29.0 (`Cargo.lock:1745,7518`) handle raw mode and resize. Unabstracted: `theme/osc11.rs:112-161`
  (saves/restores `libc::termios` for the OSC-11 query) and `crash-handler/src/handler.rs:194-213`
  (`tcgetattr` on fd 0) — both degrade to "no restore", not failure.
- **`/proc` and statfs**: 119 occurrences, concentrated in the bench harness
  (`worktree_lifecycle_bench/runtime.rs:47`), `tty-utils/process_resources.rs` and
  `sandbox/src/command/protected.rs`. `xai-sqlite-journal/src/lib.rs:272-300` already pairs `statfs`
  with a `#[cfg(windows)]` arm. Bench/diagnostic only, except the sandbox file.
- **Mode bits / umask**: `umask` 18, `.mode(` 541. Windows equivalent: ACLs (`SetEntriesInAclW`,
  `Authorization/mod.rs:513`). `shell-base/util/secure_file.rs` and `config/fs_atomic.rs` are where
  "0600" matters; both are already platform-split. A review pass, not a rewrite.
- **Path semantics**: `std::os::unix::fs::symlink` 366. `fast-worktree/src/copy/cow.rs:28-36` pairs
  `symlink` with `symlink_file`; `config/fs_atomic.rs` already tracks reparse tags. The real risk is
  behavioural — case-insensitive names and `MAX_PATH`.

### Top 10 files that must change

| # | File | Why |
|---|---|---|
| 1 | `crates/codegen/xai-grok-sandbox/src/lib.rs` | stub arm `:243`, `support_info` `:205`, `is_applied` |
| 2 | `crates/codegen/xai-grok-shell/src/config/mod.rs` | `apply_sandbox` gates enforcement to linux/macos `:1419-1450` |
| 3 | `crates/codegen/xai-grok-sandbox/Cargo.toml` | `enforce` feature semantics `:55-59` |
| 4 | `.github/workflows/release.yml` | matrix `:104-116`, verify loop `:280-294`, publish `files:` `:337-341` |
| 5 | `install.ps1` (new) | no PowerShell installer exists |
| 6 | `scripts/test-install.ps1` (new) | no analogue of the 502-line harness |
| 7 | `crates/codegen/xai-grok-workspace-daemon/src/daemonize.rs` | `C:\Windows\Temp` defaults `:44-45` |
| 8 | `crates/codegen/xai-grok-file-lock/src/slot.rs` | stub slot `:10-13` |
| 9 | `crates/codegen/xai-grok-shared/src/clipboard.rs` | image paste unsupported `:249-252` |
| 10 | `crates/codegen/xai-grok-pager-render/src/terminal/probe.rs` | unix-only probe, no fallback arms |

Adjacent (docs, not code): `README.md:290-294`, `docs/user-guide/18-sandbox.md:1-10,28-40`,
`docs/user-guide/21-terminal-support.md` (already lists Windows Terminal).

---

## 3. PTY / terminal / TUI

**Resolved PTY crate:** `portable-pty 0.9.0` (`Cargo.lock:6984-6987`, checksum
`b4a596a2…08671e`). [VERIFIED: Cargo.lock:6984-6987]

`xai-grok-shell-terminal/src/pty_session.rs:16` uses `portable_pty::{CommandBuilder, MasterPty,
PtySize, native_pty_system}` and nothing platform-specific in production code: the four
`libc::getpgid` calls (`:978,979,1020,1021`) and `libc::kill` calls (`:986,1030,1064`) are all inside
`#[cfg(all(test, unix))] mod tests` (`:821`). Teardown goes through `PortablePty::Group`/`ProcessGroup`
(`:142-151`) which resolves to Job Objects on Windows.

**ConPTY support in 0.9.0 (read from the vendored source):**

| Capability | Windows (ConPTY) | Evidence |
|---|---|---|
| `resize` | ✅ | `win/conpty.rs:85` |
| `get_size` | ✅ | `win/conpty.rs:90` |
| `try_clone_reader` / `take_writer` | ✅ | `win/conpty.rs:95,99` |
| `process_group_leader` | ❌ | `lib.rs:88-90` is `#[cfg(unix)]` |
| `as_raw_fd`, `tty_name`, `get_termios` | ❌ | `lib.rs:96-110` are `#[cfg(unix)]` |

[VERIFIED: ~/.cargo/registry/src/index.crates.io-*/portable-pty-0.9.0/src/lib.rs:88-110]

The tree already respects this: `pty_session.rs:511` vs `:528` and `:694` vs `:700` are
`#[cfg(unix)]`/`#[cfg(not(unix))]` pairs around exactly those calls.

**Does the pager TUI depend on termios/ioctl directly?** Not on the hot path. `TIOCGWINSZ` is **0
occurrences**; `ioctl` is 7, of which the only production use is
`xai-grok-pager-pty-harness/src/pty_spawn.rs:245` (`TIOCSCTTY`, a test harness). crossterm owns raw
mode (`pager/src/app/mod.rs:1328`), and the console-mode bit is set by hand
(`:1145-1165`). The two genuine unix-only terminal dependencies are the OSC-11 theme probe
(`theme/osc11.rs`) and crash-time termios restore (`crash-handler/src/handler.rs:194-213`) — both
degrade to "no restore" rather than fail.

**Known fidelity gap:** ConPTY always runs a child under a pseudo-console, so a Windows PTY screen is
not byte-identical to a Unix PTY. The harness acknowledges this at
`xai-grok-pager-pty-harness/src/pty.rs:143-145`: "a very short-lived descendant may escape before
enrollment; diagnostics preserve that downgrade."

---

## 4. The sandbox on Windows — the sharp edge

### Where the fallback is chosen

Three distinct places, and the distinction matters:

**(a) The honest one** — `xai-grok-sandbox/src/lib.rs:205-216`, inside
`#[cfg(all(feature = "enforce", unix))]`:
`tracing::warn!(details = %support.details, "Sandbox not supported on this platform, continuing
without sandbox");` — [VERIFIED: crates/codegen/xai-grok-sandbox/src/lib.rs:205-216]

**(b) The stub that actually runs on Windows** — `lib.rs:242-252`:
```
/// Stub when `enforce` feature is disabled; sandbox is not applied.
#[cfg(not(all(feature = "enforce", unix)))]
pub fn apply(&mut self, _workspace: &Path) -> anyhow::Result<()> {
    tracing::info!(profile = %self.profile,
        "Sandbox enforcement unavailable (built without 'enforce' feature)");
    Ok(())
}
```
[VERIFIED: crates/codegen/xai-grok-sandbox/src/lib.rs:242-252]

On Windows `enforce` **is** enabled (a default feature), so this message is factually wrong: it blames
a build flag for a platform gap. And it is `info!`, below the default log threshold.

**(c) The caller** — `xai-grok-shell/src/config/mod.rs:1419-1450` calls `sandbox.apply(&workspace)`
unconditionally, but the fail-closed check around it is gated:
`#[cfg(any(target_os = "linux", target_os = "macos"))] let requires_protection = { … };` … then the
same gate around `std::process::exit(1)` when the profile is unappliable.
[VERIFIED: crates/codegen/xai-grok-shell/src/config/mod.rs:1419-1450]

**Net effect on Windows:** `--sandbox strict` is accepted (`xai-grok-pager/src/app/cli.rs:620-622`,
`pub sandbox: Option<String>`, no platform validation), no error is printed, no exit code changes,
and enforcement is skipped. `xai-grok-sandbox/Cargo.toml:55-59` states the intent plainly: "On
non-unix targets enabling this feature is a no-op (no kernel sandbox exists)."

### What the product promises when the sandbox is off

- `README.md:290-294`: "**Sandbox: off by default.** … Kernel enforced (Landlock on Linux, Seatbelt
  on macOS); child-network blocking in `read-only` / `strict` is Linux-only."
- `docs/user-guide/18-sandbox.md:3`: "restricts what the agent process and its spawned commands can
  access … using OS-level kernel primitives (Landlock on Linux, Seatbelt on macOS). **The kernel
  enforces these limits for the process lifetime.**"
- Profiles table (`18-sandbox.md:28-40`) promises per-profile read/write sets and, for
  `read-only`/`strict`, blocked child network.

There is **no** Windows column and **no** statement that any platform is unsupported. Today that is
harmless because no Windows binary is published (`install.sh:70-95,245-251` rejects anything that
is not Linux/Darwin). It becomes a false claim the moment a Windows artifact ships.

### What Windows can actually enforce

Verified against the vendored `windows 0.61.3` crate already in the dependency graph:

| Mechanism | Filesystem | Network | Process spawn | Symbols present | Feature needed | Size |
|---|---|---|---|---|---|---|
| **Restricted token** + DACL grants | ✅ DACL-based (path *allow*-lists via ACEs) | ⚠️ privilege drop only, no host filtering | ✅ inherited | `CreateRestrictedToken` `Win32/Security/mod.rs:272`; `SetEntriesInAclW` `Security/Authorization/mod.rs:513` | already enabled (`Win32_Security`, `…_Authorization`) | **L** |
| **AppContainer** | ✅ capability SIDs + ACL grants | ⚠️ `internetClient` is all-or-nothing | ✅ inherited | `CreateAppContainerProfile` `Security/Isolation/mod.rs:2`; `DeriveAppContainerSidFromAppContainerName` `:23` | `Win32_Security_Isolation` = ["Win32_Security"] (`windows/Cargo.toml:548`) — **not enabled today** | **L** |
| **Job Objects** | ❌ | ❌ | ✅ lifetime, active-process count, per-job memory, UI limits | `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` `System/JobObjects/mod.rs:589`; `…_ACTIVE_PROCESS` `:577`; `…_PROCESS_MEMORY` `:593`; `JOB_OBJECT_UILIMIT_HANDLES` `:728` | already enabled | **S** — but it is *containment*, not a sandbox |
| **WFP** | ❌ | ✅ per-app-id / per-SID filtering | — | `FwpmEngineOpen0`, `FWPM_FILTER0` in `Win32/NetworkManagement/WindowsFilteringPlatform/` | `Win32_NetworkManagement_WindowsFilteringPlatform` (`windows/Cargo.toml:514`) — **not enabled**; requires admin/service to install filters | **XL** |
| **WDAC / AppLocker** | ✅ (code + some path rules) | ❌ | ✅ | — | machine policy, admin-installed | Out of scope for a CLI |

[VERIFIED: ~/.cargo/registry/src/index.crates.io-*/windows-0.61.3/… cited per row]

### Recommendation — **judgement, clearly labelled**

**Do not build a Windows sandbox backend in this phase.** Ship an honest refusal instead. Evidence:

1. **Semantics do not match.** The promise is path-based, kernel-enforced read/write denial plus
   per-child network blocking for the process lifetime. Windows' closest primitive is
   DACL/AppContainer: *allow*-list based on SIDs and object ACLs. Translating the `deny` glob list
   into ACEs must happen at apply time over paths that exist; anything created later is outside it.
   That is weaker than Landlock and needs per-profile documentation caveats — new honesty text,
   which is precisely what a rushed phase gets wrong.
2. **No way to prove it.** The suite self-skips when unsupported
   (`xai-grok-sandbox/tests/deny_paths_e2e.rs:94`) and the enforcement tests are Linux-only
   (`:436,470`). A backend with no enforcement test is an unverifiable claim.
3. **Effort is a project, not a task.** The Linux/macOS side is 22 `cfg(unix)` files with a dedicated
   e2e suite. AppContainer + ACL translation + a WFP child-network filter is at least comparable,
   plus a feature flag and a matrix leg.
4. **The cheap alternative is good.** Refusing `--sandbox` on Windows with a named reason is ~3
   files: fix the stub message, add a platform check in `apply_sandbox`, and exit non-zero for any
   profile except `off` — matching how `--restore-code`/`--chat` conflicts already fail
   (`pager/src/app/session_startup.rs:305-307`).

If the author wants a *containment* story (not a sandbox), the Job Object already in `xai-tty-utils`
gives kill-on-close and memory caps for free — but it must never be described as sandboxing.

---

## 5. Distribution

### Current pipeline

`.github/workflows/release.yml` has one `build` matrix job (`:97-116`) with four native legs —
`x86_64-unknown-linux-gnu`/`ubuntu-22.04`, `aarch64-unknown-linux-gnu`/`ubuntu-22.04-arm`,
`aarch64-apple-darwin`/`macos-15`, `x86_64-apple-darwin`/`macos-15-intel` — then `publish`
(`needs: [version, build]`, `:261-263`) and `container` (`needs: [version, build]`, `:347-350`).
Packaging is `tar -czf "deepseek-build-${RELEASE_VERSION}-${RT}.tar.gz"` (`:243`) staged from
`target/release-dist/deepseek-build` plus LICENSE/NOTICE/THIRD-PARTY-NOTICES/README.txt (`:218-221`).
`publish` hard-asserts the four tarball names in a loop (`:280-294`) and uploads
`dist/deepseek-build-*.tar.gz` with `fail_on_unmatched_files: true` (`:337-341`).
**`rg -i windows .github/workflows/*.yml` returns nothing.**

### Where `build-windows` slots in

Phase 3 (REL-01) splits `build` into `build-linux` and `build-macos` and makes
`container: needs: [version, build-linux]`. After that split:

- Add `build-windows` on `windows-latest` (assumed image, see A4), `needs: version`, building
  `x86_64-pc-windows-msvc` natively — no cross-compile, mirroring the "every leg runs on a NATIVE
  runner" rule at `:92-96`.
- `publish: needs: [version, build-linux, build-macos, build-windows]`. Leaving `build-windows` out
  of `publish`'s `needs` is the failure mode to avoid: `download-artifact` would then race the
  upload.
- `container` stays `needs: [version, build-linux]` — unchanged, which is the whole point of Phase 3.

### Packaging and installer

| Item | Change |
|---|---|
| Archive | `.zip` not `.tar.gz`: `Compress-Archive` produces a zip with no Unix mode bits, which is correct on Windows. Name it `deepseek-build-<ver>-x86_64-pc-windows-msvc.zip` so `SHA256SUMS` stays greppable. |
| Checksums | `Get-FileHash -Algorithm SHA256` and emit the **same** `"<hash>  <name>"` two-space format as `sha256sum` (`install.sh:144-153` parses that shape). |
| `SHA256SUMS` generation | `sha256sum deepseek-build-*.tar.gz` (`:307-312`) must become `sha256sum deepseek-build-*` or the zip is silently excluded from the checksum file. |
| Publish verify loop | `:280-294` must add the windows filename, or a missing Windows leg goes unnoticed. |
| Publish `files:` | add `dist/deepseek-build-*.zip`. |
| `install.ps1` | new: `Invoke-WebRequest`/`Invoke-RestMethod` to `github.com` only (mirroring the one-host contract at `install.sh:8-10`), `Get-FileHash` verify **before** `Expand-Archive`, install to `$env:LOCALAPPDATA\deepseek-build\bin`, add to **user** PATH via `[Environment]::SetEnvironmentVariable(...,'User')` (never machine scope, no admin), `-Uninstall`, re-run-to-update. |
| `dsb` shortcut | the `symlink`/`dsb` shortcut (`install.sh:16-18`) has no unprivileged Windows equivalent; use a `dsb.cmd` shim that forwards `%*`, or drop the shortcut and say so. |
| `test-install.ps1` | the analogue of the 39-assertion `scripts/test-install.sh`: fake-release HTTP listener (`HttpListener`), success, latest-resolution, checksum mismatch (installs nothing), unsupported OS, 404, uninstall (removes the `dsb.cmd` shim, preserves a foreign one), re-run update, unwritable dir, pre-existing `dsb` file. |
| `install.sh` | unchanged — it already dies with "Unsupported platform" (`:245-251`). Adding a Windows branch to a POSIX `sh` script is the wrong shape. |

Also: Authenticode signing is **not** in scope but is the equivalent of the macOS signature work in
REL-03; an unsigned `.exe` will trip SmartScreen, and the README should say so.

---

## 6. Testability and honesty

### How much of the suite runs on Windows

Measured: **31,216** `#[test]`/`#[tokio::test]` attributes; **311** integration-test files under
`tests/` across 32 crates (AGENTS.md records 307 targets — consistent).

Gating that removes tests on Windows:

| Gate | Count | Effect |
|---|---|---|
| `#[cfg(unix)]` immediately before a test attr | **493** | those tests do not compile on Windows |
| whole file `#![cfg(unix)]` | **14** | entire targets vanish (incl. `xai-grok-shell/tests/test_leader_stdio_integration.rs`, `xai-crash-handler/tests/integration.rs`) |
| `#[cfg(all(test, unix))] mod …` | **27** | whole unit-test modules vanish |
| `#[cfg(windows)]` before a test attr | 31 | only three files gate the *file* on Windows (`paste_ctrl_v_*_windows.rs`) |

So roughly **1.7 % of individual tests are explicitly unix-gated**, which sounds reassuring and is
not: the bigger losses are (a) the 27 unix-only modules, and (b) the **PTY harness**, where
`crates/codegen/xai-grok-pager-pty-harness` has 178 e2e targets, 40 gated unix/linux at file level
and 3 Windows-only, with the spawn layer split across `pty_spawn.rs` (`#[cfg(unix)]`) and a
`#[cfg(windows)]` arm in `pty.rs:146-167`. Whether those 135 ungated targets actually pass on
Windows is unknown (A2).

Two Windows-only tests exist and are `#[ignore]`d
(`paste_ctrl_v_text_echoes_fast_windows.rs:10`) — so even on Windows they would not run without
`--ignored`.

### What a credible Windows acceptance run looks like

1. `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` → the gating fact.
2. `cargo test --workspace --no-fail-fast` on `windows-latest`, with the **test-target count** and
   pass/fail/pass-gated totals recorded, not a tail line (this is exactly TEST-01's lesson).
3. `cargo build --release-dist` → extract the zip → `deepseek-build --version` prints the tag
   (REL-03's shape, applied to Windows).
4. A headless turn: `deepseek-build -p "run a shell command that prints a sentinel" --output-format
   json` → assert the sentinel came from `cmd`/`pwsh` and that `estimated_cost_usd` is present.
5. A PTY smoke: the two `*_windows.rs` scenarios with `--ignored`.
6. `check-egress.sh` via Git Bash (`shell: bash`) — it is `grep -rIniE` over `crates prod third_party`
   (`scripts/check-egress.sh:39`) and Git for Windows ships `grep`. If that is judged too fragile,
   the honest substitute is "not run on Windows" in the record.
7. A written statement of what was **not** exercised.

### Claims that must NOT be made

- **"Sandboxed"** on Windows, in any form, until a backend exists with an enforcement test. Not
  `--sandbox strict`, not "kernel-enforced", not "the same protection as Linux".
- **"Windows support"** without a named tier — "builds and runs, sandbox unsupported" is defensible;
  "first-class" is not.
- **"All tests pass on Windows"** while 493 tests and 14 targets are `cfg(unix)`-gated out. Say
  "N tests run, M are unix-only".
- **"Same as macOS"** — macOS also has no child-network blocking (`18-sandbox.md:42`) and its
  `deny_paths_e2e` self-skips in CI. Windows would be a third tier, not a peer.
- **"Installer verified"** from a fake-tree harness (the REL-06 lesson) — `install.ps1` must be run
  against a real throwaway tag.

---

## 7. Implementation shape

Risky-first. `depends_on` is plan-level.

| Plan | Depends on | Content | Risk |
|---|---|---|---|
| **05-01 Compile spike** | — | `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc`; triage every error into (a) real gap, (b) missing `cfg`, (c) test-only. Produce the definitive file list. **No product change.** | **Highest** — this is where the estimate is decided. A small error count collapses the phase to days; a large one doubles it. |
| **05-02 Sandbox honesty** | 05-01 | Refuse `--sandbox` on Windows except `off`; fix the misleading stub message; add the platform note to `README.md` and `18-sandbox.md`. ~3-5 files. | Low code, high visibility — this is the "honesty bar" deliverable. |
| **05-03 Long-tail correctness** | 05-01 | Whatever 05-01 found: `C:\Windows\Temp` defaults (`daemonize.rs:44-45`), file-lock slot, image clipboard, search-binary fallback messaging, `cgroup.rs` gating. Bounded by 05-01's list. | Medium — unbounded until 05-01 runs. |
| **05-04 Distribution** | 05-02, 05-03 | `build-windows` job, `.zip`, `SHA256SUMS` wildcard, publish `needs` + verify loop + `files:`, `install.ps1`, `test-install.ps1`. | Medium — mechanical, but the `publish` wiring is where a silent miss hides. |
| **05-05 Acceptance + docs** | 05-04 | Real throwaway tag → install via `install.ps1` from the actual release; headless turn; PTY smoke; record what did not run. | Low, but it is the plan that must be honest. |

**Estimated size**

- *"Builds and ships, sandbox refuses honestly"* (the recommendation): **12-18 files**, dominated by
  05-01's findings; 2 new files (`install.ps1`, `test-install.ps1`); 1 new CI job.
- *"Feature parity including a sandbox backend"*: **35-50 files**, 4-6 additional plans, and a
  Windows-specific enforcement test suite that does not exist today. Weeks, not days.

**The two places most likely to blow up**

1. **The first `cargo check` for the Windows target.** 161 src files carry a unix gate with no
   Windows branch in the same file; static reading cannot tell "parent module is gated" from "real
   compile error". Nothing in this document substitutes for that compile.
2. **The sandbox story.** Either it stays a silent no-op (a shipped lie the moment a `.zip` exists)
   or it becomes a backend that is weaker than the docs' "kernel enforces these limits" sentence.
   Both failure modes are documentation failures more than code failures, which is why 05-02 must
   precede distribution.

### Freeze recommendation — **judgement**

**Windows should NOT block the v0.1.0 freeze.** Reasons, in order of weight:

1. **No requirement covers it.** `rg -i windows` returns **0** in `REQUIREMENTS.md`, `ROADMAP.md` and
   `AGENTS.md`. FREEZE-03's checklist is Linux/macOS artifact proof. Adding Windows silently changes
   REL-01/REL-03/REL-06 and the workflow that Phase 3 and Phase 4 are actively rewriting.
2. **The freeze is about truth, and today the truth is consistent.** No Windows artifact is
   published, `install.sh` rejects Windows explicitly, and no doc claims support. Nothing is
   currently false — so nothing needs fixing before the freeze. The moment a Windows `.zip` exists,
   the sandbox silent-no-op becomes a false claim, which is why 05-02 must gate 05-04.
3. **Windows as specced is unbounded until 05-01 runs.** A phase inserted before a freeze should not
   carry an unknown that can double its own size; 05-01 exists to convert that unknown into a
   number, and it can run after the freeze without invalidating any freeze criterion.
4. **What *should* happen before the freeze** (if the author wants any Windows progress at all):
   run 05-01 only, as a spike, and record the error count in `KNOWN-ISSUES.md`. That is a day of
   work and it makes the follow-up phase plannable instead of speculative.

---

## 8. Risks, unknowns, assumptions, sources

### Risks

| # | Risk | Impact | Mitigation |
|---|---|---|---|
| R1 | Windows work is scoped as "port the unix code" when the real cost is the sandbox + test coverage | 2-3× underestimate | 05-01 first; quote the file-count delta explicitly |
| R2 | A Windows `.zip` ships before the sandbox refusal lands | Shipped false security claim | hard `depends_on: 05-02 → 05-04` |
| R3 | `publish` forgets `build-windows` in `needs` | Release races artifact upload; missing zip | extend the `:280-294` verify loop in the same commit |
| R4 | `SHA256SUMS` stays `*.tar.gz`-globbing | Zip published unchecksummed | one-word change, easy to miss |
| R5 | ConPTY screen fidelity differs from Unix PTY | PTY e2e expectations fail wholesale | keep Windows PTY coverage to the 3 existing Windows-gated scenarios |
| R6 | `C:\Windows\Temp` is not writable by a standard user | workspace-daemon fails at startup on a normal account | use `%LOCALAPPDATA%\Temp` in 05-03 |
| R7 | Case-insensitive paths change permission-rule matching | deny/allow rules that work on Linux silently over- or under-match | add a Windows path-matching test; the existing symlink deny tests are `#[cfg(unix)]` (`permission-rules/src/policy.rs:2263,2315`) |
| R8 | `windows-latest` image changes under us | intermittent CI | pin to `windows-2022` once green |

### Assumptions (each with a falsifier)

| # | Assumption | Falsified by |
|---|---|---|
| **A1** | The workspace **does not currently compile** for `x86_64-pc-windows-msvc`; the error count is unknown and is the dominant cost term | `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` — if it exits 0, plans 05-01/05-03 collapse and the phase is ~3 files |
| **A2** | The PTY e2e suite mostly does **not** pass on Windows even though only 40/178 files are unix-gated | `cargo test -p xai-grok-pager-pty-harness --target x86_64-pc-windows-msvc` on a Windows host |
| **A3** | `windows-latest` provides an MSVC toolchain + Windows SDK with no extra setup | read the runner image's installed-software list, or a trivial `cargo build` on that runner |
| **A4** | A GitHub-hosted `windows-latest` runner is available to this repo and counts against the same 90-minute job budget (`release.yml:101`) | `gh api repos/coruairc/deepseek-build/actions/runners` and one `workflow_dispatch` run |
| **A5** | Restricted-token + DACL can approximate the `workspace`/`strict` profiles closely enough to be worth building *later* | a spike that applies a restricted token and confirms a write outside the workspace fails while a write inside succeeds |
| **A6** | AppContainer's `internetClient` cannot express "block child network, allow the parent's API calls" — i.e. per-process network needs WFP | a spike: AppContainer child + `Test-NetConnection` to `api.deepseek.com`, then the same from the parent |
| **A7** | The 3 Windows-gated PTY tests are the only Windows PTY coverage that exists | `rg -l 'cfg\(target_os = "windows"\)' crates/**/tests | wc -l` → 3 |
| **A8** | `install.sh`'s checksum format can be produced byte-identically by PowerShell | diff `Get-FileHash` output against `sha256sum` for the same file |

### Open questions

1. **What does "first-class" mean here?** Build + honest refusal, or sandbox parity? The brief says
   "at the same honesty bar as Linux and macOS", which reads as *honesty*, not *parity* — and that
   reading makes the phase tractable. Confirm before planning.
2. **Is `aarch64-pc-windows-msvc` in scope?** The Linux/macOS matrix ships both arches. Windows on
   ARM is a separate runner and a separate toolchain; assume x86_64 only unless told otherwise.
3. **Is Authenticode signing required?** REL-03 treats the macOS signature as a release criterion.
   The Windows peer is code signing, which needs a certificate the project does not have.
4. **Does the author have a Windows machine to run ACC-01-style acceptance on?** Without one, every
   Windows claim is CI-only, which is weaker than the Linux/macOS story.

### Sources

**Primary (read from disk this session)** — `Cargo.lock:6984-6987` (`portable-pty 0.9.0`),
`:11219-11221` (`windows 0.61.3`); `~/.cargo/registry/src/index.crates.io-*/portable-pty-0.9.0/`
(`src/lib.rs:88-110`, `src/win/conpty.rs:85-99`); `…/windows-0.61.3/src/Windows/Win32/{Security,
Authorization,Isolation,System/JobObjects,NetworkManagement}/` and its `Cargo.toml:514,548`; every tree
file cited by `path:line` in §1-§6.

**Secondary (in-repo documents)** — `README.md:290-294`; `docs/user-guide/18-sandbox.md`;
`docs/user-guide/21-terminal-support.md`; `.planning/REQUIREMENTS.md` (no Windows requirement);
`.planning/ROADMAP.md:85-145`; `AGENTS.md` §0 (31,939 passed / 388 ignored / 307 targets — this
research measured 31,216 test attributes and 311 test files, consistent).

**Not verified here** — GitHub runner images and their toolchains (A3, A4); anything requiring a
Windows host (A1, A2, A5, A6). **No web search was performed**; every claim is read from this tree or
from the vendored crate sources in `~/.cargo/registry`.

---

## Metadata

**Confidence:** today's posture **HIGH** (measured, file:line); unix surface ranking **MEDIUM-HIGH**
(counted, per-file reads, no compile); PTY/ConPTY **HIGH** (vendored source); sandbox options
**MEDIUM-HIGH** on API availability, **MEDIUM** on enforcement strength (reasoning over documented
semantics); distribution **HIGH** on the current workflow, **MEDIUM** on PowerShell specifics; size
estimate **LOW-MEDIUM** — gated entirely on A1.

**Valid until:** ~30 days for tree facts; re-run the counts after Phase 3 lands, since it rewrites the
workflow §5 describes.
