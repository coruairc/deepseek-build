#!/usr/bin/env python3
"""Run one E3 experiment: a variant config over all tasks, next to the baseline.

Wraps `harness.py` with the one-variable-at-a-time discipline from the Phase E
spec: it warns when the invocation changes more than one dimension (model,
prompt, extra args, config file), runs the variant, then prints the baseline
vs. variant comparison.

Usage:
    # E3c: reasoning effort
    python3 scripts/evals/experiment.py --name effort-low --model deepseek-v4-pro \
        --extra-args "--effort low"

    # E3a: short system prompt
    python3 scripts/evals/experiment.py --name short-prompt --model deepseek-v4-pro \
        --system-prompt-file scripts/evals/prompts/short-deepseek.md

Baseline records are read from `--baseline-config` (default `baseline`).
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

EVALS = Path(__file__).resolve().parent
sys.path.insert(0, str(EVALS))

import report  # noqa: E402


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--name", required=True, help="variant label (results filename and table row)")
    parser.add_argument("--bin", type=Path, default=EVALS.parents[1] / "target" / "release" / "deepseek-build")
    parser.add_argument("--model", default="deepseek-v4-pro")
    parser.add_argument("--runs", type=int, default=3)
    parser.add_argument("--timeout", type=int, default=420)
    parser.add_argument("--tasks", help="comma-separated task ids (default: all)")
    parser.add_argument("--extra-args", default="")
    parser.add_argument("--config-file", type=Path)
    parser.add_argument("--system-prompt-file", type=Path)
    parser.add_argument("--baseline-config", default="baseline")
    parser.add_argument("--baseline-model", default=None, help="model to compare against (default: same as --model)")
    parser.add_argument("--dry-run", action="store_true", help="print the command and exit")
    args = parser.parse_args(argv)

    dimensions = {
        "model": args.model,
        "extra_args": args.extra_args,
        "config_file": str(args.config_file) if args.config_file else None,
        "system_prompt_file": str(args.system_prompt_file) if args.system_prompt_file else None,
    }
    changed = [k for k, v in dimensions.items() if v and k != "model"]
    if len(changed) > 1:
        print(f"warning: this experiment varies {len(changed)} dimensions at once: {', '.join(changed)}",
              file=sys.stderr)
        print("(the spec's discipline is one variable at a time; results will be hard to attribute)",
              file=sys.stderr)

    command = [
        sys.executable, str(EVALS / "harness.py"),
        "--bin", str(args.bin),
        "--model", args.model,
        "--config", args.name,
        "--runs", str(args.runs),
        "--timeout", str(args.timeout),
    ]
    if args.tasks:
        command += ["--tasks", args.tasks]
    if args.extra_args:
        command += ["--extra-args", args.extra_args]
    if args.config_file:
        command += ["--config-file", str(args.config_file)]
    if args.system_prompt_file:
        command += ["--system-prompt-file", str(args.system_prompt_file)]

    print("running:", " ".join(command))
    if args.dry_run:
        return 0
    rc = subprocess.call(command)

    baseline_model = args.baseline_model or args.model
    baseline_files = sorted((EVALS / "results").glob(f"*-{args.baseline_config}-{baseline_model}.jsonl"))
    variant_files = sorted((EVALS / "results").glob(f"*-{args.name}-{args.model}.jsonl"))
    print("\n== baseline vs variant ==")
    if not baseline_files:
        print(f"no baseline results for config={args.baseline_config} model={baseline_model}; "
              f"run scripts/evals/run-baseline.sh first")
    else:
        rc = report.main(["--by-task", *[str(p) for p in baseline_files], *[str(p) for p in variant_files]]) or rc
    return rc


if __name__ == "__main__":
    sys.exit(main())
