# scripts

- `check-egress.sh` — HARD/SOFT network-egress grep gate.
- `sandbox-run.sh` — run the binary with only the provider + loopback reachable.
- `encrypt_templates.py` — regenerate the XOR-obfuscated agent prompt templates.
- `smoke-test.sh` — live DeepSeek adapter smoke test (key required).
- `egress-check.sh` — runtime egress audit (LD_PRELOAD connect/send logger).
- `test-install.sh` — local fake-release installer test suite.
- `key-leak-check.sh` — secret-leak scan of logs and saved sessions.
- `evals/` — Phase E tuning/eval harness: `harness.py` (21 auto-checked tasks,
  JSONL metrics), `report.py` (comparison table), `tasks/`, `results/`.
- `synstrip/` — small `syn`-based Rust tool that drops top-level items referencing deleted symbols (`--line`, `--line-info` modes).
- `line_loop.py` — compiler-driven test-compile repair loop over `synstrip`.
