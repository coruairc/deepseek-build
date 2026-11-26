# Windows compile spike — run log

Throwaway branch `ci/windows-spike`, workflow `.github/workflows/windows-spike.yml`,
job `cargo check (x86_64-pc-windows-msvc)` on `windows-latest`, Rust 1.94.0.
The job deliberately exits 0 to capture the full log, so **a green job is not a green check** —
the result is `check_exit` in the "Error summary" step.

| # | Run | Result | First failure | Fix |
|---|-----|--------|---------------|-----|
| 1 | 45766443725 | check_exit 101, 2.1 min | `cc` 1.2.48 won't compile: `expected u32, found i32` at `tempfile.rs:33` | `find-msvc-tools` 0.1.13 → 0.1.15 (vendors `FILE_ATTRIBUTE_TEMPORARY` as `u32`) |
| 2 | 45768898474 | check_exit 101, 4.0 min | `xai-grok-tools-api` build script: `bin/protoc … not a valid Win32 application (os error 193)`; no `protoc` on PATH | install protoc on the runner (`choco install protoc`), set `PROTOC` — same contract the Linux leg already uses |
| 3 | 45770292086 | check_exit 101, 4.9 min | `xai-proto-build`: `/dev/stdout: No such file or directory` when invoking protoc | write `--dependency_out`/`--descriptor_set_out` to a tempdir, read the dep file back, strip the exact target prefix (both separators) |
| 4 | 45772140860 | check_exit 101, 6.6 min | `xai-grok-pager-pty-harness`: E0432 `process_has_exited_without_reap` unresolved (a `#[cfg(unix)]` helper imported unconditionally) | gate the import; every use already sits in a `cfg(unix)` block |
| 5 | 45773337297 | see run | — | running |

## What each failure means

1. **A build-dependency, never compiled on Linux.** Bumping it is a `Cargo.lock`-only change and
   `cargo check --workspace --locked` still passes on Linux. It must ride with the Windows work, not
   be reverted.
2. **Distribution, not product.** A `build-windows` job must install protoc explicitly. This is the
   same rule the release workflow documents for Linux (`PROTOC` set to the real binary, never the
   repo's dotslash wrapper).
3. **A real cross-platform defect in our own build helper.** `xai-proto-build` hardcoded `/dev/stdout`
   and `/dev/null`, so every crate whose build script calls `xai_proto_build()` failed on Windows —
   not just one. This is the first genuine product change the Windows phase needs.

## Standing constraints discovered

- `workflow_dispatch` requires the workflow file on the **default branch**; main is not pushed, so the
  spike is triggered by a `push` to its own branch instead. A real `build-windows` job in
  `release.yml` will be dispatched normally once main is pushed.
- The branch and its workflow are throwaway. `main` carries none of these three changes yet; they
  were proven on the branch first.
