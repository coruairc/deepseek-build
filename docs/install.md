<!-- generated-by: gsd-doc-writer -->

# Install and uninstall deepseek-build

`deepseek-build` ships as a single binary plus a `dsb` shortcut. This page covers
installing it from the published release on Linux and macOS, verifying the
download, upgrading, uninstalling it completely, and what to do when a step
fails. To build from source instead, see the
[README](../README.md#build-from-source).

## What you get

| Item | Detail |
|------|--------|
| Binary | `deepseek-build` — the TUI, headless mode, and agent subcommands |
| Shortcut | `dsb`, a symlink to `deepseek-build` in the same directory |
| Install location | `${INSTALL_DIR:-$HOME/.local/bin}` for `install.sh`; `$(brew --prefix)/bin` for Homebrew |
| Privileges | None. `install.sh` never calls `sudo`; it installs under your home directory by default |
| State directory | `~/.deepseek-build/` (override with `DEEPSEEK_BUILD_HOME`), created on first run, not by the installer |
| Network | The installer fetches the release tarball and `SHA256SUMS` from GitHub. No telemetry and no update check |

The tarball also contains `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES`, and
`README.txt`. `install.sh` extracts only the `deepseek-build` binary; Homebrew
installs the documentation under `$(brew --prefix)/share/doc/deepseek-build/`.

The current published release is the proof pre-release
[**`v0.1.0-proof1`**](https://github.com/coruairc/deepseek-build/releases/tag/v0.1.0-proof1)
(published 2026-11-26). `v0.1.0` is **not** published. GitHub's `latest` path
covers stable releases only, so
`https://github.com/coruairc/deepseek-build/releases/latest/download/...`
returns **404** while only pre-releases exist. Pin `VERSION` in every command
below.

### Requirements

`install.sh` uses whichever tool it finds first:

| Needed | Used for |
|--------|----------|
| `curl` or `wget` | Downloading the tarball and `SHA256SUMS` |
| `sha256sum` or `shasum` | Checksum verification; the installer refuses to continue without one |
| `tar` | Extracting the binary |

### Supported platforms

| OS | Architectures | Published artifact |
|----|---------------|--------------------|
| Linux | `x86_64`, `aarch64` (glibc) | `.tar.gz` |
| macOS | Apple silicon (`arm64`), Intel (`x86_64`) | `.tar.gz` |
| Windows | `x86_64` | None yet (see [Windows](#windows)) |

The Linux artifacts are built for glibc (`*-unknown-linux-gnu`) on the
`ubuntu-22.04` runner, so **glibc 2.35 is the practical floor**; musl-based
distributions such as Alpine are not supported by these artifacts. No macOS
minimum is recorded in the tree, and neither floor is exercised here — treat
older systems as untested.

## Linux

### Install

```sh
curl -fsSL https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/install.sh \
  | VERSION=v0.1.0-proof1 sh
```

`VERSION` accepts `v0.1.0-proof1` or `0.1.0-proof1`. The script:

1. Downloads `deepseek-build-0.1.0-proof1-<target>.tar.gz` and the release's
   `SHA256SUMS`.
2. Verifies the tarball's SHA-256 against `SHA256SUMS` **before extracting**.
   On a mismatch, or when the tarball is not listed, it aborts and installs
   nothing.
3. Extracts the binary to `${INSTALL_DIR:-$HOME/.local/bin}` (creating the
   directory if needed) and marks it executable.
4. Creates the `dsb` symlink. A real `dsb` file is never replaced, and a
   symlink that points somewhere unrelated is left alone; a `dsb` symlink whose
   target contains `deepseek-build` (or is dangling) is repointed at the
   installed binary.
5. Prints the installed version, and warns if the install directory is not on
   your `PATH`.

### Custom install directory

```sh
curl -fsSL https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/install.sh \
  | INSTALL_DIR=/opt/bin VERSION=v0.1.0-proof1 sh
```

`/opt/bin` must already be writable by you. The installer never elevates, so on
a typical system it stops with `not writable`; use a directory you own (for
example `INSTALL_DIR="$HOME/bin"`) or change the directory's ownership first.

### Verify

```sh
deepseek-build --version
# deepseek-build 0.1.0-proof1 (b14d28eb2bc7)

dsb --version
# deepseek-build 0.1.0-proof1 (b14d28eb2bc7)
```

The parenthesized value is the 12-character commit of the tagged source. If the
command is not found, see [`dsb: command not found`](#dsb-command-not-found).

### Upgrade

There is no auto-updater and the binary makes no update check; upgrading means
rerunning the installer with a newer tag:

```sh
VER=<published-tag>   # e.g. v0.1.0-proof1
curl -fsSL "https://github.com/coruairc/deepseek-build/releases/download/$VER/install.sh" \
  | VERSION="$VER" sh
```

The rerun overwrites `deepseek-build` in place and leaves the `dsb` shortcut
working. `v0.1.0-proof2` exists as a **tag only** (no GitHub release or assets),
so the newest installable release is still `v0.1.0-proof1`; there is nothing to
upgrade to yet.

### Uninstall

```sh
curl -fsSL https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/install.sh \
  | sh -s -- --uninstall
```

With a custom directory, pass the same `INSTALL_DIR`:

```sh
curl -fsSL https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/install.sh \
  | INSTALL_DIR=/opt/bin sh -s -- --uninstall
```

`--uninstall` makes no network calls of its own; the `curl` only fetches the
script. If you saved a copy, `sh install.sh --uninstall` works offline. It
removes `$INSTALL_DIR/deepseek-build` and a `dsb` symlink whose target contains
`deepseek-build` (or is dangling); a real `dsb` file, or a symlink to an
unrelated path, is left in place. Configuration and saved credentials are **not** removed — see
[Uninstall completely](#uninstall-completely).

## macOS

### Homebrew (Apple silicon and Intel)

```sh
brew tap coruairc/deepseek-build
brew trust coruairc/deepseek-build   # Homebrew's trust gate for third-party taps
brew install deepseek-build
```

Homebrew 7 refuses to load a formula from an untrusted third-party tap; without
the `brew trust` line, `brew install` stops with
`Refusing to load formula ... from untrusted tap`. `brew trust` records the tap
as trusted for this Homebrew installation.

The formula installs the prebuilt tarball for the host architecture
(`on_arm` / `on_intel`), a `dsb` symlink, and the license/notice files. There is
no source build, no `head`, and no Xcode dependency. The tap is **macOS only**:
on Homebrew for Linux the formula has no URL for the platform and `brew` reports
`formula requires at least a URL`, so Linux users should use `install.sh`.

The tap repository `coruairc/homebrew-deepseek-build` is public and carries
`Formula/deepseek-build.rb`, pinned to `0.1.0-proof1`.
<!-- VERIFY: the tap coruairc/homebrew-deepseek-build must remain public with Formula/deepseek-build.rb; reachable 2026-11-27, but its availability is outside this repository -->

### Verify

```sh
deepseek-build --version
# deepseek-build 0.1.0-proof1 (b14d28eb2bc7)
```

### Upgrade

```sh
brew upgrade deepseek-build
```

This replaces the binary when the formula's `version` moves. While the formula
stays pinned to `0.1.0-proof1`, the command is a no-op.

### Uninstall

```sh
brew uninstall deepseek-build
brew untap coruairc/deepseek-build   # optional: drop the tap as well
```

`brew uninstall` removes the binary and the `dsb` symlink. It leaves
`~/.deepseek-build/` in place — see
[Uninstall completely](#uninstall-completely).

### Tarball fallback

If you prefer not to use Homebrew, the pinned `install.sh` line works unchanged:

```sh
curl -fsSL https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/install.sh \
  | VERSION=v0.1.0-proof1 sh
```

Or download the darwin tarball and run the binary directly:

```sh
VER=0.1.0-proof1
curl -fLO "https://github.com/coruairc/deepseek-build/releases/download/v$VER/deepseek-build-$VER-aarch64-apple-darwin.tar.gz"
tar -xzf "deepseek-build-$VER-aarch64-apple-darwin.tar.gz" deepseek-build
./deepseek-build --version
```

Use `x86_64-apple-darwin` instead of `aarch64-apple-darwin` on Intel. This
manual path skips `install.sh`'s checksum verification and its `dsb` shortcut;
move the binary onto your `PATH` yourself if you want it installed.

### Signing and Gatekeeper

The macOS binaries are **ad-hoc signed** and **not notarized**; they carry no
Apple Developer ID signature. That is sufficient for a Homebrew tap.

- A tarball fetched by `brew` or `curl` carries no `com.apple.quarantine`
  attribute, so no Gatekeeper prompt is expected.
- A copy downloaded through a browser may be quarantined. If macOS blocks the
  first launch, right-click the binary and choose **Open**, or clear the
  attribute:

```sh
xattr -d com.apple.quarantine /path/to/deepseek-build
```

Notarization is not required to distribute through a Homebrew tap; it becomes
relevant only for a signed `.app` or `.pkg` distributed outside a package
manager.

## Windows

The release workflow builds and packages Windows `x86_64` artifacts, but this
path is **authored, not published or executed**:

- No Windows artifact exists on any release. `v0.1.0-proof1` carries only the
  four Linux/macOS tarballs, `SHA256SUMS`, and `install.sh`.
- `install.ps1` has not been run on a Windows host.

When a Windows artifact is published, it will be
`deepseek-build-<version>-x86_64-pc-windows-msvc.zip`, with `deepseek-build.exe`
and the license/notice files at the top level of the zip, alongside
`install.ps1`, which installs to `%LOCALAPPDATA%\deepseek-build\bin` without
elevation. No install command is documented here until an artifact exists and
the installer has been exercised on Windows.

Windows has no sandbox backend: only `--sandbox off` is accepted, and the
session is not sandboxed. User configuration lives at
`%USERPROFILE%\.deepseek-build\config.toml`.

## Uninstall completely

Removing the binary is not the same as removing your data. To leave nothing
behind:

1. **Remove the binary and shortcut.** Use the uninstall command for your
   install method:
   - `install.sh` (Linux, macOS tarball): the `--uninstall` command above, with
     the same `INSTALL_DIR` you installed to.
   - Homebrew: `brew uninstall deepseek-build` (and optionally
     `brew untap coruairc/deepseek-build`).
2. **Remove the state directory** if you also want configuration and saved
   credentials gone:

   ```sh
   rm -rf "${DEEPSEEK_BUILD_HOME:-$HOME/.deepseek-build}"
   ```

   This directory holds `config.toml`, saved sessions, logs, crash reports, and
   `credentials.json` (mode `0600`, written only when you save an API key at the
   TUI prompt). It is irreversible. Delete the directory; do not `cat`,
   `less`, or otherwise print `credentials.json` — there is no reason to read
   key material to remove it.
3. **Project-scoped files are separate.** A repository may contain
   `.deepseek/config.toml` (with legacy `.grok/` read as a fallback). Those live
   in the repository, not the global directory, and are not touched by any
   uninstall command here.

For a container install the binary is inside the image; removing the container
or image is the uninstall, and the mounted state directory follows step 2.

## Troubleshooting

### Unpinned install fails with 404

Symptom:

```
==> Resolving the latest release
curl: (22) The requested URL returned error: 404
```

`install.sh` asked for `releases/latest/download/...`, and `latest` excludes
pre-releases. Pin the tag, including the `VERSION=` variable:

```sh
curl -fsSL https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/install.sh \
  | VERSION=v0.1.0-proof1 sh
```

### `dsb: command not found`

The install directory is not on `PATH`; the installer prints a warning when this
is the case. Add it to your shell profile (`~/.profile`, `~/.bashrc`, or
`~/.zshrc`):

```sh
export PATH="$HOME/.local/bin:$PATH"
```

Use your `INSTALL_DIR` instead of `$HOME/.local/bin` if you installed elsewhere,
then open a new shell and check:

```sh
command -v deepseek-build
command -v dsb
```

### `Cannot write ... (not writable?)`

The chosen `INSTALL_DIR` is not writable by your user. The installer does not
use `sudo` and offers no elevation path. Pick a directory you own:

```sh
curl -fsSL https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/install.sh \
  | INSTALL_DIR="$HOME/bin" VERSION=v0.1.0-proof1 sh
```

If the directory is read-only, the install fails and installs nothing.

### Checksum mismatch

Symptom:

```
!!  Checksum mismatch for deepseek-build-0.1.0-proof1-<target>.tar.gz
!!  Aborting: the downloaded tarball does not match SHA256SUMS. Nothing was installed.
```

The download was truncated or altered in transit — for example by a proxy or TLS
interception. Nothing was installed, so no cleanup is needed. Retry the same
command; if it persists, inspect the network path rather than bypassing the
check. A related failure is
`<tarball> is not listed in SHA256SUMS (refusing to install an unverified tarball)`,
which means the release is missing the asset for your target.

If the script reports
`Neither sha256sum nor shasum is available to verify the checksum`, install one
of those tools; the installer will not proceed unverified.

### Homebrew refuses the tap

```
Refusing to load formula ... from untrusted tap coruairc/deepseek-build
```

Run the trust step, then install again:

```sh
brew trust coruairc/deepseek-build
brew install deepseek-build
```

### macOS blocks the binary

A browser download may carry `com.apple.quarantine`, producing
"cannot be opened because the developer cannot be verified" or "is damaged".
Homebrew- and curl-installed copies should not prompt. For a browser copy,
right-click **Open**, or:

```sh
xattr -d com.apple.quarantine /path/to/deepseek-build
```

If the prompt persists, confirm you cleared the attribute on the binary you are
actually running (`xattr -l /path/to/deepseek-build`).

### Unsupported platform

```
Unsupported platform: <os>/<arch>. Supported: Linux x86_64/arm64, macOS x86_64/arm64.
```

`install.sh` has no artifact for that OS/architecture. Build from source
following the [README](../README.md#build-from-source), or use a supported
machine. Windows is covered above and has no published artifact yet.

## Next steps

- [README quick start](../README.md#quick-start) — API key setup and the first
  session.
- [Getting started](../crates/codegen/xai-grok-pager/docs/user-guide/01-getting-started.md)
  — first launch, headless mode, and the TUI.
- [TESTING.md](../TESTING.md) — test plan and safe local runs.
- [KNOWN-ISSUES.md](../KNOWN-ISSUES.md) — known gaps and unexercised surfaces.
