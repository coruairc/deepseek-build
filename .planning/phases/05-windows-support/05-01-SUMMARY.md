# 05-01 — Compile spike (WIN-02)

**Executed:** 2026-11-26 · **Status:** complete

`cargo check --workspace --lib --bins --examples --target x86_64-pc-windows-msvc` on
`windows-latest` is **green** (run 45797190306 onward). Every test target compiles; the two
remaining targets were Unix-only code, now gated. Log: `05-SPIKE-LOG.md`.

Seven defects were found and fixed to get there — four environmental/build-system
(`find-msvc-tools` 0.1.13 → 0.1.15, protoc on the runner, `/dev/*` in `xai-proto-build`, a
`String + &String` expression) and three Unix-only test/example/bench targets. All are on
`main`; the throwaway branch `ci/windows-spike*` was only the vehicle.

**What this does not prove:** the product has never been *built* as a release for Windows
(`cargo build --profile release-dist`), only checked. That is 05-04's job.
