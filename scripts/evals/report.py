#!/usr/bin/env python3
"""Aggregate eval results into a comparison table.

Reads one or more JSONL result files (or the whole `results/` directory) and
prints a per-configuration comparison: pass rate, mean wall time, mean tokens,
cache hit rate, reasoning tokens, and estimated cost per task.

Usage:
    python3 scripts/evals/report.py                 # all results/*.jsonl
    python3 scripts/evals/report.py results/a.jsonl results/b.jsonl
    python3 scripts/evals/report.py --by-task       # per-task detail rows
    python3 scripts/evals/report.py --out report.md # write a Markdown table
"""

from __future__ import annotations

import argparse
import json
import statistics
from collections import defaultdict
from pathlib import Path

RESULTS_DIR = Path(__file__).resolve().parent / "results"


def load_records(paths: list[Path]) -> list[dict]:
    records = []
    for path in paths:
        if path.is_dir():
            files = sorted(path.glob("*.jsonl"))
        else:
            files = [path]
        for file in files:
            for line in file.read_text().splitlines():
                line = line.strip()
                if line:
                    records.append(json.loads(line))
    return records


def mean(values: list[float]) -> float | None:
    values = [v for v in values if v is not None]
    return statistics.fmean(values) if values else None


def summarize(records: list[dict]) -> dict:
    passes = [r for r in records if r.get("pass")]
    costs = [r.get("estimated_cost_usd") for r in records if r.get("estimated_cost_usd") is not None]
    return {
        "runs": len(records),
        "passed": len(passes),
        "pass_rate": len(passes) / len(records) if records else 0.0,
        "wall_s": mean([r.get("wall_s") for r in records]),
        "input_tokens": mean([r.get("input_tokens") for r in records]),
        "cache_read_input_tokens": mean([r.get("cache_read_input_tokens") for r in records]),
        "cache_hit_rate": mean([r.get("cache_hit_rate") for r in records]),
        "output_tokens": mean([r.get("output_tokens") for r in records]),
        "reasoning_tokens": mean([r.get("reasoning_tokens") for r in records]),
        "cost_usd": sum(costs) if costs else None,
        "cost_per_task": mean(costs),
        "cost_reported": len(costs),
    }


def fmt(value, spec=".2f", dash="—"):
    if value is None:
        return dash
    if isinstance(value, float):
        return format(value, spec)
    return str(value)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("paths", nargs="*", type=Path, help="result JSONL files or directories (default: results/)")
    parser.add_argument("--by-task", action="store_true", help="also print per-task, per-config rows")
    parser.add_argument("--out", type=Path, help="write the Markdown table to this file")
    args = parser.parse_args(argv)

    paths = args.paths or [RESULTS_DIR]
    for path in paths:
        if not path.exists():
            parser.error(f"no such file or directory: {path}")
    records = load_records(paths)
    if not records:
        parser.error("no records found")

    # Group by (config, model).
    groups: dict[tuple[str, str], list[dict]] = defaultdict(list)
    for record in records:
        groups[(record.get("config", "?"), record.get("model", "?"))].append(record)

    lines: list[str] = []
    lines.append("| configuration | model | runs | pass | pass rate | wall/task | tokens in | cache read | hit rate | out | reasoning | est. USD/task |")
    lines.append("|---|---|---|---|---|---|---|---|---|---|---|---|")
    for (config, model), group in sorted(groups.items()):
        s = summarize(group)
        hit = f"{s['cache_hit_rate'] * 100:.0f}%" if s["cache_hit_rate"] is not None else "—"
        lines.append(
            f"| {config} | {model} | {s['runs']} | {s['passed']} | {s['pass_rate'] * 100:.0f}% | "
            f"{fmt(s['wall_s'], '.1f')}s | {fmt(s['input_tokens'], '.0f')} | "
            f"{fmt(s['cache_read_input_tokens'], '.0f')} | {hit} | {fmt(s['output_tokens'], '.0f')} | "
            f"{fmt(s['reasoning_tokens'], '.0f')} | {fmt(s['cost_per_task'], '.5f')} |"
        )
    table = "\n".join(lines)
    print(table)

    if args.by_task:
        print()
        task_groups: dict[tuple[str, str, str], list[dict]] = defaultdict(list)
        for record in records:
            task_groups[(record.get("task", "?"), record.get("config", "?"), record.get("model", "?"))].append(record)
        detail = ["| task | configuration | model | pass | runs | wall/task | est. USD/task |", "|---|---|---|---|---|---|---|"]
        for (task, config, model), group in sorted(task_groups.items()):
            s = summarize(group)
            detail.append(
                f"| {task} | {config} | {model} | {s['passed']}/{s['runs']} | {s['runs']} | "
                f"{fmt(s['wall_s'], '.1f')}s | {fmt(s['cost_per_task'], '.5f')} |"
            )
        print("\n".join(detail))

    if args.out:
        args.out.write_text(table + "\n")
        print(f"\nwrote {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
