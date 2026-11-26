# scripts

- `check-egress.sh` — HARD/SOFT network-egress grep gate.
- `check-release-pipeline.sh` — one command that runs every static release gate (actionlint, shellcheck, the two checkers, the guard fixture, the installer harness) with each tally pinned.
- `check-release-workflow.py` — static assertions over `.github/workflows/release.yml`: job graph, `needs:`, matrix legs, artifact names, tag-guard placement, SHA pins.
- `check-release-tag-shape.py` — the `v0.1.0` tag shape: trigger, the `version` job's own derive body run against stable/prerelease/rejection fixtures, and release gating.
- `distribution-proof.py` — daemonless release/run/registry proofs with an offline fixture mode (`fixtures`, `probe`, `baseline`, `run`, `release`, `registry`).
- `prove-install.sh` — real-release pinned installer proof and the unpinned-`latest` behaviour probe; refuses the fake-tree hooks.
- `check-readme-distribution.py` — whitespace-normalized README assertions, `install` and `container` stages.
- `check-homebrew-formula.py` — formula text/hash cross-check against the release `SHA256SUMS`, with a self-test and an optional `--online` mode.
- `check-distribution-evidence.py` — evidence schema/hash/text validator for a phase directory, with a four-case `--self-test`.
- `guard-image-tags.sh` — pre-push guard for the `docker/metadata-action` tag list (`:latest` only on a stable release; prints the resolved names).
- `test-guard-image-tags.sh` — fixture suite for `guard-image-tags.sh` (stable, prerelease 2-name and 4-name incl. `0.1.0-proof1`, and every rejection).
- `test-gate.sh` — the phase test gate: workspace + `config-docs` runs, completeness checked against the build graph and a checked-in count.
- `evidence.py` — record a `docs/evidence/` item: steps, artifacts, normalized `report.json`, `script(1)` → asciinema v2 cast.
- `sandbox-run.sh` — run the binary with only the provider (IPv4 and IPv6) + loopback reachable.
- `test-sandbox-run.sh` — offline IPv4/IPv6 refusal harness for `sandbox-run.sh`'s `LD_PRELOAD` guard.
- `encrypt_templates.py` — regenerate the XOR-obfuscated agent prompt templates.
- `smoke-test.sh` — live DeepSeek adapter smoke test (key required).
- `egress-check.sh` — runtime egress audit (LD_PRELOAD connect/send logger).
- `test-install.sh` — local fake-release installer test suite, including the pinned stable `0.1.0` / `0.1.0-proof1` cases.
- `key-leak-check.sh` — secret-leak scan of logs and saved sessions; `--scan <path>...` scans committed evidence without a release binary.
- `evals/` — Phase E tuning/eval harness: `harness.py` (21 auto-checked tasks,
  JSONL metrics), `report.py` (comparison table), `tasks/`, `results/`.
- `synstrip/` — small `syn`-based Rust tool that drops top-level items referencing deleted symbols (`--line`, `--line-info` modes).
- `line_loop.py` — compiler-driven test-compile repair loop over `synstrip`.
