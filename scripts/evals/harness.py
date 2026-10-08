#!/usr/bin/env python3
"""DeepSeek tuning eval harness (Phase E1).

Runs small, self-contained coding tasks against a `deepseek-build` binary and
records an automatic pass/fail verdict plus cost metrics for each run.

Task format (JSON, one file per task):

    {
      "id": "py-fix-add",
      "summary": "one line",
      "prompt": "what to tell the agent",
      "files": { "calc.py": "…", "test_calc.py": "…" },
      "init_git": true,
      "timeout_s": 420,
      "max_turns": 15,
      "fails_before": true,
      "checks": [
        {"type": "command", "cmd": ["python3", "-m", "unittest", "-v"]},
        {"type": "file_contains", "path": "calc.py", "text": "a + b"}
      ]
    }

Checks:
  command           — pass when the exit code equals `expect_exit` (default 0),
                      and (optionally) stdout contains `expect_stdout_contains`
  file_contains     — file exists and contains `text` (literal)
  file_not_contains — file exists and does NOT contain `text` (literal)
  file_matches      — file exists and `pattern` (regex) matches somewhere
  file_absent       — path does not exist
  file_exists       — path exists

Each model run gets a fresh scratch repo (files written, optionally git-init'd)
and a fresh HOME, so runs never contaminate each other. One JSON line per run
is appended to the results file.

The API key is read from the environment only, never printed, and is scrubbed
from any recorded output.

Usage:
    python3 scripts/evals/harness.py --list
    python3 scripts/evals/harness.py --dry-run            # validate fixtures offline
    python3 scripts/evals/harness.py --model deepseek-flash --config baseline
    python3 scripts/evals/harness.py --model deepseek-v4-pro --runs 3
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_BIN = REPO_ROOT / "target" / "release" / "deepseek-build"
DEFAULT_TASKS_DIR = Path(__file__).resolve().parent / "tasks"
DEFAULT_RESULTS_DIR = Path(__file__).resolve().parent / "results"

CHECK_TYPES = ("command", "file_contains", "file_not_contains", "file_matches", "file_absent", "file_exists")


# --------------------------------------------------------------------------
# Task loading and validation
# --------------------------------------------------------------------------
def load_task(path: Path) -> dict:
    with path.open() as fh:
        task = json.load(fh)
    task["_path"] = str(path)
    missing = [k for k in ("id", "prompt", "checks") if k not in task]
    if missing:
        raise ValueError(f"{path}: missing required field(s): {', '.join(missing)}")
    if not task["checks"]:
        raise ValueError(f"{path}: task has no checks")
    for check in task["checks"]:
        ctype = check.get("type")
        if ctype not in CHECK_TYPES:
            raise ValueError(f"{path}: unknown check type {ctype!r}")
        if ctype == "command" and not check.get("cmd"):
            raise ValueError(f"{path}: command check needs a `cmd` list")
        if ctype in ("file_contains", "file_absent", "file_exists") and not check.get("path"):
            raise ValueError(f"{path}: {ctype} check needs a `path`")
    return task


def load_tasks(tasks_dir: Path, selector: str | None) -> list[dict]:
    paths = sorted(tasks_dir.glob("*.json"))
    if selector:
        wanted = {s.strip() for s in selector.split(",") if s.strip()}
        paths = [p for p in paths if p.stem in wanted or p.name in wanted]
    tasks = [load_task(p) for p in paths]
    ids = [t["id"] for t in tasks]
    if len(ids) != len(set(ids)):
        raise ValueError("duplicate task ids in selection")
    return tasks


def build_fixture(task: dict, dest: Path) -> None:
    """Write the task's initial files into `dest`; optionally git-init a repo."""
    dest.mkdir(parents=True, exist_ok=True)
    for rel, content in (task.get("files") or {}).items():
        target = dest / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(content)
    if task.get("init_git", True):
        run = lambda *a: subprocess.run(  # noqa: E731
            a, cwd=dest, check=True, capture_output=True, text=True
        )
        run("git", "init", "-q")
        run("git", "config", "user.email", "eval@example.com")
        run("git", "config", "user.name", "eval")
        run("git", "add", "-A")
        run("git", "commit", "-qm", "init")


# --------------------------------------------------------------------------
# Checks
# --------------------------------------------------------------------------
def run_check(check: dict, cwd: Path, timeout_s: int) -> dict:
    ctype = check["type"]
    result = {"type": ctype, "pass": False}
    if ctype == "command":
        cmd = check["cmd"]
        expect = int(check.get("expect_exit", 0))
        result["cmd"] = cmd
        try:
            proc = subprocess.run(
                cmd,
                cwd=cwd,
                capture_output=True,
                text=True,
                timeout=check.get("timeout_s", timeout_s),
            )
        except subprocess.TimeoutExpired:
            result["detail"] = "check command timed out"
            return result
        result["exit_code"] = proc.returncode
        result["pass"] = proc.returncode == expect
        if result["pass"] and check.get("expect_stdout_contains"):
            needle = check["expect_stdout_contains"]
            result["expect_stdout_contains"] = needle
            result["pass"] = needle in proc.stdout
        if not result["pass"]:
            tail = (proc.stdout + proc.stderr).strip().splitlines()
            result["detail"] = " | ".join(tail[-6:])[:600]
        return result
    path = cwd / check["path"]
    if ctype == "file_exists":
        result["pass"] = path.exists()
        return result
    if ctype == "file_absent":
        result["pass"] = not path.exists()
        return result
    if ctype == "file_contains":
        text = check["text"]
        result["text"] = text
        if not path.exists():
            result["detail"] = "file missing"
            return result
        body = path.read_text(errors="replace")
        result["pass"] = text in body
        return result
    if ctype == "file_not_contains":
        text = check["text"]
        result["text"] = text
        if not path.exists():
            result["detail"] = "file missing"
            return result
        body = path.read_text(errors="replace")
        result["pass"] = text not in body
        return result
    if ctype == "file_matches":
        pattern = check["pattern"]
        result["pattern"] = pattern
        if not path.exists():
            result["detail"] = "file missing"
            return result
        body = path.read_text(errors="replace")
        result["pass"] = re.search(pattern, body) is not None
        return result
    raise AssertionError(f"unreachable check type {ctype!r}")


def run_checks(task: dict, cwd: Path, timeout_s: int) -> list[dict]:
    return [run_check(c, cwd, timeout_s) for c in task["checks"]]


def clear_pycache(root: Path) -> None:
    """Drop byte-compiled caches so checks never score stale code.

    Python's timestamp-based invalidation has one-second granularity, so an
    agent edit landing in the same second as a previous test run could be
    served from a stale `__pycache__` of the same size.
    """
    for cache in root.rglob("__pycache__"):
        shutil.rmtree(cache, ignore_errors=True)
    for stale in root.rglob("*.pyc"):
        stale.unlink(missing_ok=True)


# --------------------------------------------------------------------------
# Running the binary
# --------------------------------------------------------------------------
def scrub(text: str, secrets: list[str]) -> str:
    for secret in secrets:
        if secret:
            text = text.replace(secret, "[REDACTED]")
    return text


def file_sha256(path: Path) -> str | None:
    """Content hash of a run input (binary, config, prompt), or None."""
    if path is None or not Path(path).exists():
        return None
    import hashlib

    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def parse_headless_json(stdout: str) -> dict | None:
    """Headless `--output-format json` prints one (pretty) JSON object."""
    stdout = stdout.strip()
    if not stdout:
        return None
    try:
        return json.loads(stdout)
    except json.JSONDecodeError:
        pass
    # Tolerate log noise on stdout: try each line as a JSON object, last wins.
    found = None
    for line in stdout.splitlines():
        line = line.strip()
        if line.startswith("{") and line.endswith("}"):
            try:
                found = json.loads(line)
            except json.JSONDecodeError:
                continue
    return found


def extract_metrics(payload: dict | None) -> dict:
    if not payload or payload.get("type") == "error":
        return {}
    usage = payload.get("usage") or {}
    inp = usage.get("input_tokens")
    cached = usage.get("cache_read_input_tokens")
    created = usage.get("cache_creation_input_tokens")
    try:
        prompt_total = int(inp or 0) + int(cached or 0) + int(created or 0)
        hit_rate = (int(cached or 0) / prompt_total) if prompt_total else None
    except (TypeError, ValueError):
        hit_rate = None
    cost = payload.get("estimated_cost_usd")
    estimate = payload.get("estimate")
    if cost is None:
        cost = payload.get("total_cost_usd")
        estimate = False
    return {
        "input_tokens": inp,
        "cache_read_input_tokens": cached,
        "cache_creation_input_tokens": created,
        "output_tokens": usage.get("output_tokens"),
        "reasoning_tokens": usage.get("reasoning_tokens"),
        "total_tokens": usage.get("total_tokens"),
        "cache_hit_rate": hit_rate,
        "estimated_cost_usd": cost,
        "estimate": estimate,
        "total_cost_usd": payload.get("total_cost_usd"),
        "num_turns": payload.get("num_turns"),
        "model_usage": payload.get("modelUsage"),
        "session_id": payload.get("sessionId"),
        "stop_reason": payload.get("stopReason"),
    }


def git_snapshot(cwd: Path) -> dict:
    try:
        status = subprocess.run(
            ["git", "status", "--porcelain"],
            cwd=cwd, capture_output=True, text=True, timeout=30,
        ).stdout.strip()
        diffstat = subprocess.run(
            ["git", "diff", "--stat"],
            cwd=cwd, capture_output=True, text=True, timeout=30,
        ).stdout.strip()
    except (subprocess.SubprocessError, OSError):
        return {"changed": None, "dirty_files": [], "diffstat": ""}
    files = [ln[3:].strip() for ln in status.splitlines() if ln.strip()]
    return {"changed": bool(files), "dirty_files": files, "diffstat": diffstat}


def run_agent(
    task: dict,
    run_index: int,
    *,
    bin_path: Path,
    model: str,
    config: str,
    workdir: Path,
    timeout_s: int,
    extra_args: list[str],
    config_file: Path | None,
    base_url: str | None,
    keep_work: bool,
    secrets: list[str],
    system_prompt_file: Path | None = None,
    rules_file: Path | None = None,
) -> dict:
    task_id = task["id"]
    run_dir = workdir / f"{task_id}-{config}-{model}-r{run_index}"
    scratch = run_dir / "repo"
    home = run_dir / "home"
    build_fixture(task, scratch)
    (home / ".deepseek-build").mkdir(parents=True, exist_ok=True)

    conf = home / ".deepseek-build" / "config.toml"
    if config_file:
        conf.write_text(config_file.read_text())
    if base_url:
        with conf.open("a") as fh:
            fh.write(f"\n[model.{model}]\nbase_url = \"{base_url}\"\n")

    env = dict(os.environ)
    env["HOME"] = str(home)
    env["DEEPSEEK_BUILD_HOME"] = str(home / ".deepseek-build")
    env["NO_COLOR"] = "1"

    cmd = [
        str(bin_path),
        "--always-approve",
        "--no-plan",
        "--no-subagents",
        "--disable-web-search",
        "--max-turns",
        str(task.get("max_turns", 15)),
        "-m", model,
        "--output-format", "json",
        *extra_args,
        "-p", task["prompt"],
    ]

    started = time.monotonic()
    timed_out = False
    try:
        proc = subprocess.run(
            cmd, cwd=scratch, env=env, capture_output=True, text=True,
            timeout=task.get("timeout_s", timeout_s),
        )
        stdout, stderr, code = proc.stdout, proc.stderr, proc.returncode
    except subprocess.TimeoutExpired as exc:
        timed_out = True
        stdout = (exc.stdout or b"").decode(errors="replace") if isinstance(exc.stdout, bytes) else (exc.stdout or "")
        stderr = (exc.stderr or b"").decode(errors="replace") if isinstance(exc.stderr, bytes) else (exc.stderr or "")
        code = None
    wall_s = time.monotonic() - started

    # Keep the record readable: files injected as flags are recorded by path+hash, not inline.
    recorded_extra_args: list[str] = []
    index = 0
    while index < len(extra_args):
        arg = extra_args[index]
        if arg in ("--system-prompt-override", "--rules") and index + 1 < len(extra_args):
            recorded_extra_args.append(arg)
            recorded_extra_args.append(f"<{len(extra_args[index + 1])} bytes from file>")
            index += 2
            continue
        recorded_extra_args.append(arg)
        index += 1

    payload = parse_headless_json(stdout)
    error = None
    if isinstance(payload, dict) and payload.get("type") == "error":
        error = str(payload.get("message", ""))[:800]
    elif code not in (0, None):
        error = (stderr or stdout).strip().splitlines()[-1][:800] if (stderr or stdout).strip() else None

    clear_pycache(scratch)
    checks = run_checks(task, scratch, timeout_s)
    record = {
        "ts": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "task": task_id,
        "config": config,
        "model": model,
        "run": run_index,
        "pass": all(c["pass"] for c in checks),
        "checks": checks,
        "wall_s": round(wall_s, 2),
        "exit_code": code,
        "timed_out": timed_out,
        "error": scrub(error, secrets) if error else None,
        "bin": str(bin_path),
        "bin_sha256": file_sha256(bin_path),
        "bin_mtime": datetime.fromtimestamp(
            bin_path.stat().st_mtime, timezone.utc
        ).isoformat(timespec="seconds") if bin_path.exists() else None,
        "config_file": str(config_file) if config_file else None,
        "config_file_sha256": file_sha256(config_file) if config_file else None,
        "extra_args": recorded_extra_args,
        "system_prompt_file": str(system_prompt_file) if system_prompt_file else None,
        "system_prompt_sha256": file_sha256(system_prompt_file) if system_prompt_file else None,
        "rules_file": str(rules_file) if rules_file else None,
        "rules_sha256": file_sha256(rules_file) if rules_file else None,
        **extract_metrics(payload),
        "git": git_snapshot(scratch),
        "workdir": str(run_dir) if keep_work else None,
    }
    if not keep_work:
        shutil.rmtree(run_dir, ignore_errors=True)
    return record


# --------------------------------------------------------------------------
# CLI
# --------------------------------------------------------------------------
def print_summary(record: dict) -> None:
    status = "pass" if record["pass"] else "FAIL"
    cost = record.get("estimated_cost_usd")
    cost_s = f"${cost:.4f}" if isinstance(cost, (int, float)) else "n/a"
    hit = record.get("cache_hit_rate")
    hit_s = f"{hit * 100:.0f}%" if isinstance(hit, (int, float)) else "n/a"
    reason = record.get("reasoning_tokens")
    turns = record.get("num_turns")
    print(
        f"[{status}] {record['task']:<28} {record['model']:<17} {record['config']:<24} "
        f"{record['wall_s']:>6.1f}s {cost_s:>8} cache={hit_s:>4} "
        f"reason={reason if reason is not None else 'n/a':>6} turns={turns if turns is not None else 'n/a'}"
    )
    if record.get("error"):
        print(f"         error: {record['error'].splitlines()[0][:160]}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--bin", type=Path, default=DEFAULT_BIN)
    parser.add_argument("--tasks-dir", type=Path, default=DEFAULT_TASKS_DIR)
    parser.add_argument("--tasks", help="comma-separated task ids (default: all)")
    parser.add_argument("--model", default="deepseek-v4-pro")
    parser.add_argument("--config", default="baseline", help="label for this configuration")
    parser.add_argument("--config-file", type=Path, help="TOML copied into the scratch HOME")
    parser.add_argument("--base-url", help="override [model.<model>] base_url (mock/self-test)")
    parser.add_argument("--system-prompt-file", type=Path,
                        help="pass this file's contents as --system-prompt-override (E3a)")
    parser.add_argument("--rules-file", type=Path,
                        help="pass this file's contents as --rules (E3d; appends to the system prompt)")
    parser.add_argument("--runs", type=int, default=1, help="runs per task")
    parser.add_argument("--timeout", type=int, default=420, help="per-run timeout (s)")
    parser.add_argument("--extra-args", default="", help="extra binary args as one shell-like string")
    parser.add_argument("--out", type=Path, help="results JSONL (default: results/<ts>-<config>-<model>.jsonl)")
    parser.add_argument("--workdir", type=Path, help="scratch parent dir (default: temp dir)")
    parser.add_argument("--keep-work", action="store_true", help="keep scratch dirs")
    parser.add_argument("--dry-run", action="store_true",
                        help="validate fixtures offline: build each task and run its checks")
    parser.add_argument("--list", action="store_true", help="list tasks and exit")
    args = parser.parse_args(argv)

    try:
        tasks = load_tasks(args.tasks_dir, args.tasks)
    except ValueError as exc:
        parser.error(str(exc))
    if args.list:
        for task in tasks:
            print(f"{task['id']:<30} {task.get('summary', '')}")
        return 0
    if not tasks:
        parser.error("no tasks selected")

    if args.dry_run:
        bad = 0
        for task in tasks:
            with tempfile.TemporaryDirectory(prefix="eval-dry-") as tmp:
                scratch = Path(tmp) / "repo"
                build_fixture(task, scratch)
                clear_pycache(scratch)
                checks = run_checks(task, scratch, args.timeout)
            verdict = "checks-fail (expected for fails_before)" if task.get("fails_before", True) else "checks-pass"
            if task.get("fails_before", True) and all(c["pass"] for c in checks):
                verdict = "UNEXPECTED: checks already pass before the agent runs"
                bad += 1
            if not task.get("fails_before", True) and not all(c["pass"] for c in checks):
                verdict = "UNEXPECTED: checks fail on the pristine fixture"
                bad += 1
            failed_detail = "; ".join(
                f"{c['type']}: {c.get('detail', 'no detail')}" for c in checks if not c["pass"]
            )
            print(f"[dry] {task['id']:<30} {verdict}" + (f" ({failed_detail[:120]})" if failed_detail else ""))
        print(f"dry-run: {len(tasks)} task(s), {bad} unexpected")
        return 1 if bad else 0

    if not args.bin.exists():
        parser.error(f"binary not found: {args.bin} (build with: cargo build --release -p xai-grok-pager-bin)")
    # Runs execute with cwd set to the scratch repo, so the binary and any
    # config paths must be absolute.
    args.bin = args.bin.resolve()
    if args.config_file:
        args.config_file = args.config_file.resolve()
    if args.system_prompt_file:
        args.system_prompt_file = args.system_prompt_file.resolve()

    if args.extra_args:
        try:
            extra_args = shlex.split(args.extra_args)
        except ValueError as exc:
            parser.error(f"--extra-args: {exc}")
    else:
        extra_args = []
    if args.system_prompt_file:
        if not args.system_prompt_file.exists():
            parser.error(f"no such system prompt file: {args.system_prompt_file}")
        extra_args = ["--system-prompt-override", args.system_prompt_file.read_text(), *extra_args]
    if args.rules_file:
        if not args.rules_file.exists():
            parser.error(f"no such rules file: {args.rules_file}")
        extra_args = ["--rules", args.rules_file.read_text(), *extra_args]
    secrets = [os.environ.get("DEEPSEEK_API_KEY", ""), os.environ.get("DEEPSEEK_BUILD_API_KEY", "")]
    workdir = args.workdir or Path(tempfile.mkdtemp(prefix="deepseek-eval-"))
    workdir.mkdir(parents=True, exist_ok=True)

    if args.out:
        out = args.out
    else:
        stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
        out = DEFAULT_RESULTS_DIR / f"{stamp}-{args.config}-{args.model}.jsonl"
    out.parent.mkdir(parents=True, exist_ok=True)

    passed = 0
    total = 0
    with out.open("a") as fh:
        for run_index in range(1, args.runs + 1):
            for task in tasks:
                record = run_agent(
                    task, run_index,
                    bin_path=args.bin,
                    model=args.model,
                    config=args.config,
                    workdir=workdir,
                    timeout_s=args.timeout,
                    extra_args=extra_args,
                    config_file=args.config_file,
                    base_url=args.base_url,
                    keep_work=args.keep_work,
                    secrets=secrets,
                    system_prompt_file=args.system_prompt_file,
                    rules_file=args.rules_file,
                )
                fh.write(json.dumps(record) + "\n")
                fh.flush()
                total += 1
                passed += int(record["pass"])
                print_summary(record)

    print(f"\n{passed}/{total} passed · results: {out}")
    if not args.keep_work:
        shutil.rmtree(workdir, ignore_errors=True)
    return 0 if passed == total else 1


if __name__ == "__main__":
    sys.exit(main())
