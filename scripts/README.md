# scripts

- `check-egress.sh` — HARD/SOFT network-egress grep gate.
- `sandbox-run.sh` — run the binary with only the provider + loopback reachable.
- `encrypt_templates.py` — regenerate the XOR-obfuscated agent prompt templates.
- `synstrip/` — small `syn`-based Rust tool that drops top-level items referencing deleted symbols (`--line`, `--line-info` modes).
- `line_loop.py` — compiler-driven test-compile repair loop over `synstrip`.
