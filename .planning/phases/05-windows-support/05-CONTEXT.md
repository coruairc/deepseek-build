# Phase 5: Windows Support - Context

**Gathered:** 2026-11-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Make `x86_64-pc-windows-msvc` a supported target at a **named tier**: it builds, ships a `.zip`, installs
via a PowerShell installer, and refuses the sandbox honestly instead of accepting `--sandbox` and
silently ignoring it.

**Not in scope:** a Windows sandbox backend (a weeks-sized job, ~35-50 files), feature parity claims,
or any statement that Windows has the same protection as Linux.

**Evidence base:** `05-RESEARCH.md`. The tree was already ported upstream: 183 `cfg(windows)` occurrences,
Win32 Job Objects for process teardown, named pipes for leader transport, ConPTY via `portable-pty`, a
pwsh → powershell.exe → Git Bash → cmd.exe cascade, and crash capture. The port is mostly present; the
proof, the packaging and the sandbox honesty are what is missing.

</domain>

<decisions>
## Implementation Decisions

### The tier, stated plainly
- Windows ships as **"builds and runs, sandbox unsupported"**. No form of "sandboxed" may appear in any
  Windows-facing artifact until a backend exists with an enforcement test. — **Reversibility:** costly — the tier is a public claim; raising it later is easy, retracting a false one is not.

### Compile spike first
- Plan 05-01 is a **no-product-change compile spike**: `rustup target add x86_64-pc-windows-msvc` then
  `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc`, triaging every error into
  (a) real gap, (b) missing `cfg`, (c) test-only, and producing the definitive file list. The whole
  phase's size is decided there, so nothing else is planned in detail until it runs.
- **The spike cannot run on the Linux build host (measured 2026-11-26).** `aws-lc-sys` 0.39.1 compiles C
  for the target and `cc-rs` aborts with `GNU compiler is not supported for this target`; there is no
  MSVC C toolchain here. The spike must run on a `windows-latest` runner (or a Windows host), which
  means the first Windows evidence comes from CI, not from a local command. A `build-windows` spike job
  that does **not** gate `publish` (so a failure cannot erase the release) is the way to get it.

### Sandbox honesty before distribution
- The misleading fallback (`"Sandbox not supported on this platform, continuing without sandbox"`) and
  the accepted-but-ignored `--sandbox` flag are fixed **before** any Windows artifact exists: on Windows
  only `--sandbox off` is accepted; any other value is a hard error naming the unsupported platform.
- The README and the sandbox user-guide page gain the platform note. This is the honesty-bar deliverable
  of the phase.

### Distribution
- A `build-windows` job on `windows-latest`, a `.zip` (not `.tar.gz`), artifact name `release-x86_64-pc-windows-msvc`, `publish: needs` extended to include it, and the publish verify loop extended to the fifth
  tarball.
- `install.ps1`: download, SHA-256 verify **before** extracting, install to `%LOCALAPPDATA%`, print a
  PATH hint, support `-Uninstall`; `scripts/test-install.ps1` is its fixture harness, and the real proof
  runs against a throwaway tag.
- Scoop is not added in this phase; the tap-equivalent is deferred to open-build.

### Acceptance and honesty
- A credible Windows acceptance run is: `cargo check --all-targets` (the spike), a `windows-latest` test
  run recorded with a **test-target count and pass/fail/unix-gated totals** (never a tail line), a
  release build whose extracted zip passes `--version`, one headless turn with a sentinel, and the two
  `*_windows.rs` PTY scenarios with `--ignored`.
- The record must state what did **not** run: 493 unix-gated tests, 14 whole-file `#![cfg(unix)]` targets,
  27 unix-only test modules, and the sandbox.

### Freeze ordering
- The author chose to put Windows **before** the freeze. The research recommends the opposite; the plan
  therefore keeps the phase bounded by the "builds and ships, sandbox refuses honestly" tier (12-18
  files) and treats a sandbox backend as explicitly out of scope, so it cannot silently double.

### Decided autonomously
- [Plan shape] Q: "Detail all plans now?" → Selected: "spike first, then size the rest" — the compile result decides the phase's size; planning the tail before it would be fiction.
- [Installer] Q: "Scoop or PowerShell?" → Selected: "PowerShell" — no new public repo, matches `install.sh`'s shape, and Scoop needs a manifest repo the author has not asked for.
- [Distribution] Q: "Add Windows to the same tag run?" → Selected: "yes, `build-windows` joins the existing tag-triggered workflow" — one release carries all platforms.

### the agent's Discretion
- The error-triage taxonomy's exact labels, whether `install.ps1` lives at the repo root (it should, next
  to `install.sh`), and the wording of the platform note.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `xai-tty-utils` Job Objects (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, `CREATE_NO_WINDOW`).
- `xai-grok-shell/src/leader/transport.rs` named-pipe transport (`\\.\pipe\grok-leader-<hash>`).
- `xai-grok-shell-terminal` + `portable-pty` ConPTY backend; `pty.rs:146-167` Windows spawn arm.
- The 67 `target_os = "windows"` branches concentrated in `xai-grok-pager-render`, `xai-grok-pager`,
  `xai-grok-pager-pty-harness`.

### Established Patterns
- Platform behaviour behind `cfg` gates with a parent-module gate rather than per-line gates.
- Installers verify SHA-256 before extracting and install to a user path with no elevation.
- Release artifacts are `release-<target>` and `publish` verifies each by name.

### Integration Points
- `publish` downloads `pattern: release-*`; adding a fifth target is a wiring change, not a new contract.
- `SHA256SUMS` generation must wildcard the new `.zip`.
- The sandbox refusal sits at the flag-parse/validation boundary, before any tool runs.

</code_context>

<specifics>
## Specific Ideas

- The single dominant unknown is the first `cargo check --target x86_64-pc-windows-msvc`; 161 src files
  carry a unix gate with no Windows branch in the same file, and static reading cannot tell a
  parent-gated module from a real error.
- The two `*_windows.rs` PTY tests are `#[ignore]`d today; the acceptance run must pass `--ignored`.
- No docs currently claim Windows support and `install.sh` rejects it, so nothing is false today — the
  risk starts the moment a `.zip` exists.

</specifics>

<deferred>
## Deferred Ideas

- A Windows sandbox backend (AppContainer / restricted token / WFP) — open-build, 35-50 files.
- Scoop and winget manifests — open-build.
- Running the full suite on Windows — the acceptance run records what did not run rather than claiming
  parity.

</deferred>
