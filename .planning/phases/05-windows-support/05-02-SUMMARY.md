# 05-02 — Sandbox honesty (WIN-04)

**Executed:** 2026-11-26 · **Status:** complete (`822086c4`)

Windows had no sandbox backend, yet every non-`off` profile was accepted, logged at `info`
and enforced nothing. Now:

- an **explicit** non-`off` request (CLI, env, config file, requirement) is a hard error before
  any tool runs: `the sandbox is not supported on Windows; only --sandbox off is accepted`;
- no request on Windows resolves to `off`, and `/session-info` / the header report
  `sandbox:unsupported`;
- the non-enforcing stub errors for non-`off` and names the **build feature** on a Unix build
  without `enforce` rather than claiming a platform limit;
- Linux/macOS resolution is unchanged for every profile (tested), including `Custom`.

`is_explicit_sandbox_source` is unit-tested for every `ConfigSource` variant. Suite: 389
passed. Docs: README platform note and `18-sandbox.md`.
