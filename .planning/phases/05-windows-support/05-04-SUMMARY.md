# 05-04 — Distribution (WIN-01, WIN-05)

**Executed:** 2026-11-26 · **Status:** authored, artifact NOT produced

- `build-windows` job on `windows-latest`: pinned toolchain, `choco install protoc` with
  `PROTOC` exported as a **Windows** path (`cygpath -w` — a Git-Bash path is silently ignored by
  `PathBuf::try_exists`), per-target cache, `cargo build --profile release-dist --locked`,
  `--version` smoke, top-level `.zip`, artifact `release-x86_64-pc-windows-msvc`.
- `publish` waits for it, verifies the zip, hashes it into `SHA256SUMS` and attaches it with
  `install.ps1`. `container` still depends on `version` + `build-linux` only.
- `install.ps1` (x86_64, VERSION/latest, github.com only, SHA-256 before extraction,
  `%LOCALAPPDATA%`, `-Uninstall`) and `scripts/test-install.ps1` (27 assertions).
- Gates: `check-release-workflow.py` 21/21 (A20/A21 added, mutation-tested), full
  `check-release-pipeline.sh` green, actionlint clean.

**Not proven:** no PowerShell on this host, so neither installer file has been **executed**;
no tag has run with the Windows leg, so no `.zip` exists. The README and the guide say exactly
that.
