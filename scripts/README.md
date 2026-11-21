# scripts

- `check-egress.sh` — HARD/SOFT network-egress grep gate.
- `test-gate.sh` — the phase test gate: workspace + `config-docs` runs, completeness checked against the build graph and a checked-in count.
- `evidence.py` — record a `docs/evidence/` item: steps, artifacts, normalized `report.json`, `script(1)` → asciinema v2 cast.
- `sandbox-run.sh` — run the binary with only the provider (IPv4 and IPv6) + loopback reachable.
- `test-sandbox-run.sh` — offline IPv4/IPv6 refusal harness for `sandbox-run.sh`'s `LD_PRELOAD` guard.
- `encrypt_templates.py` — regenerate the XOR-obfuscated agent prompt templates.
- `smoke-test.sh` — live DeepSeek adapter smoke test (key required).
- `egress-check.sh` — runtime egress audit (LD_PRELOAD connect/send logger).
- `test-install.sh` — local fake-release installer test suite.
- `key-leak-check.sh` — secret-leak scan of logs and saved sessions; `--scan <path>...` scans committed evidence without a release binary.
- `evals/` — Phase E tuning/eval harness: `harness.py` (21 auto-checked tasks,
  JSONL metrics), `report.py` (comparison table), `tasks/`, `results/`.
- `synstrip/` — small `syn`-based Rust tool that drops top-level items referencing deleted symbols (`--line`, `--line-info` modes).
- `line_loop.py` — compiler-driven test-compile repair loop over `synstrip`.
