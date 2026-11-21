#!/usr/bin/env python3
"""evidence.py — record a Phase evidence item under docs/evidence/.

Layout (see docs/evidence/README.md):

    docs/evidence/NN-<phase-slug>/<item-slug>/
        <item-slug>.cast    asciinema v2 recording
        report.json         fixed-schema summary (the artifact a reviewer reads)
        notes.md            optional human notes
        <step-slug>.log     per-step captured output (normalized)

report.json has exactly these keys, in this order:

    item          str    the item slug
    requirements  [str]  requirement ids the item proves
    date          str    UTC start time, YYYY-MM-DDTHH:MM:SSZ
    host          str    system/arch only (e.g. "Linux/x86_64")
    command       str    the command that produced the item
    steps         [{name, ok, detail}]
                         detail is "exit <n>: <first failure line or last non-empty line>"
    artifacts     [{path, sha256}]
                         path is relative to the repository root
    result        str    "ok" | "fail" | "skipped"

Normalization is mandatory: the repository root, the user's home directory and
the system temp directory are replaced with `<REPO>`, `<HOME>` and `<TMP>` in
every captured log and in the report text. Per-test success lines
(`test <name> ... ok`) are dropped from logs; `Running`, `Doc-tests`,
`test result:` and failure text are kept. Environment dumps are never captured.

Usage:

    scripts/evidence.py begin --item-dir DIR --item SLUG \\
        --requirements A,B --command TEXT
    scripts/evidence.py step  --item-dir DIR --name NAME -- <cmd...>
    scripts/evidence.py artifact --item-dir DIR --path FILE
    scripts/evidence.py finish --item-dir DIR --result ok|fail|skipped
    scripts/evidence.py cast --typescript FILE --timing FILE --out FILE.cast
    scripts/evidence.py normalize --path FILE [--path FILE...]

`step` runs the command, tees its merged stdout+stderr to
`DIR/<name-slug>.log`, records the real exit status, and exits with that
status. `finish` refuses `--result ok` when any step failed. `cast` converts
`script(1)`'s typescript plus its `-T` timing file into asciinema v2: one
`{"version":2,"width":80,"height":24}` header line, then `[elapsed, "o", data]`
records sliced from the timing file's `elapsed bytes` pairs (the per-write
delays are accumulated into absolute elapsed seconds); the
`Script started`/`Script done` framing lines are dropped and the data passes
through the same normalization. `normalize` applies
the same substitution to a file produced outside `step` (for example a gate
log copied into the item directory).
"""

import argparse
import hashlib
import json
import os
import platform
import re
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
HOME_DIR = Path(os.path.expanduser("~"))
TMP_DIR = Path(tempfile.gettempdir())

PENDING_NAME = ".evidence-pending.json"
SUCCESS_LINE = re.compile(r"^test .+ \.\.\. ok\r?$")
FAILURE_HINT = re.compile(r"\b(fail|failed|error|panicked|assert)\b", re.IGNORECASE)


def normalize_text(text):
    """Replace absolute repo/home/temp prefixes with the documented tokens."""
    for actual, token in (
        (str(REPO_ROOT), "<REPO>"),
        (str(HOME_DIR), "<HOME>"),
        (str(TMP_DIR), "<TMP>"),
    ):
        if actual and actual != "/":
            text = text.replace(actual, token)
    return text


def scrub_log(text):
    """Drop per-test success lines; keep Running/Doc-tests/test result/failures."""
    lines = [line for line in text.splitlines() if not SUCCESS_LINE.match(line)]
    result = "\n".join(lines)
    if text.endswith("\n"):
        result += "\n"
    return result


def slug(name):
    cleaned = re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-")
    return cleaned or "step"


def summarize(returncode, output):
    lines = [line for line in output.splitlines() if line.strip()]
    chosen = ""
    if returncode == 0:
        chosen = lines[-1] if lines else ""
    else:
        for line in lines:
            if FAILURE_HINT.search(line):
                chosen = line
                break
        if not chosen:
            chosen = lines[-1] if lines else ""
    return f"exit {returncode}: {chosen}"


def pending_path(item_dir):
    return Path(item_dir) / PENDING_NAME


def load_pending(item_dir):
    path = pending_path(item_dir)
    if not path.is_file():
        sys.stderr.write(f"evidence: no pending state at {path}; run `begin` first\n")
        raise SystemExit(2)
    return json.loads(path.read_text(encoding="utf-8"))


def save_pending(item_dir, state):
    path = pending_path(item_dir)
    path.write_text(json.dumps(state, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def cmd_begin(args):
    item_dir = Path(args.item_dir)
    item_dir.mkdir(parents=True, exist_ok=True)
    requirements = [part.strip() for part in args.requirements.split(",") if part.strip()]
    state = {
        "item": args.item,
        "requirements": requirements,
        "date": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "host": f"{platform.system()}/{platform.machine()}",
        "command": args.command,
        "steps": [],
        "artifacts": [],
    }
    save_pending(item_dir, state)
    print(f"evidence: began {args.item} in {item_dir}")


def cmd_step(args):
    item_dir = Path(args.item_dir)
    state = load_pending(item_dir)
    command = list(args.cmd)
    if command and command[0] == "--":
        command = command[1:]
    if not command:
        sys.stderr.write("evidence: step needs a command after `--`\n")
        raise SystemExit(2)

    proc = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    output = proc.stdout.decode("utf-8", "replace")
    log_path = item_dir / f"{slug(args.name)}.log"
    log_path.write_text(output, encoding="utf-8")
    sys.stdout.write(output)
    sys.stdout.flush()

    state["steps"].append(
        {
            "name": args.name,
            "ok": proc.returncode == 0,
            "detail": summarize(proc.returncode, output),
        }
    )
    save_pending(item_dir, state)
    raise SystemExit(proc.returncode)


def cmd_artifact(args):
    item_dir = Path(args.item_dir)
    state = load_pending(item_dir)
    path = Path(args.path)
    if not path.is_file():
        sys.stderr.write(f"evidence: artifact does not exist: {path}\n")
        raise SystemExit(2)
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    try:
        display = os.path.relpath(path.resolve(), REPO_ROOT)
    except ValueError:
        display = str(path)
    display = display.replace(os.sep, "/")
    state["artifacts"].append({"path": display, "sha256": digest})
    save_pending(item_dir, state)
    print(f"evidence: artifact {display} sha256={digest}")


def cmd_finish(args):
    item_dir = Path(args.item_dir)
    state = load_pending(item_dir)
    if args.result == "ok":
        failed = [step["name"] for step in state["steps"] if not step["ok"]]
        if failed:
            sys.stderr.write(
                "evidence: refusing --result ok; failed step(s): " + ", ".join(failed) + "\n"
            )
            raise SystemExit(2)

    for step in state["steps"]:
        log_path = item_dir / f"{slug(step['name'])}.log"
        if log_path.is_file():
            text = log_path.read_text(encoding="utf-8")
            log_path.write_text(normalize_text(scrub_log(text)), encoding="utf-8")

    report = {
        "item": state["item"],
        "requirements": state["requirements"],
        "date": state["date"],
        "host": state["host"],
        "command": normalize_text(state["command"]),
        "steps": [
            {
                "name": normalize_text(step["name"]),
                "ok": step["ok"],
                "detail": normalize_text(step["detail"]),
            }
            for step in state["steps"]
        ],
        "artifacts": [
            {
                "path": normalize_text(artifact["path"]),
                "sha256": artifact["sha256"],
            }
            for artifact in state["artifacts"]
        ],
        "result": args.result,
    }
    report_path = item_dir / "report.json"
    report_path.write_text(
        json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    pending_path(item_dir).unlink()
    print(f"evidence: wrote {report_path} ({args.result})")


def cmd_cast(args):
    typescript = Path(args.typescript).read_bytes()
    newline = typescript.find(b"\n")
    body = typescript[newline + 1 :] if newline >= 0 else typescript

    timing = []
    timing_path = Path(args.timing)
    if timing_path.is_file():
        for line in timing_path.read_text(encoding="utf-8").splitlines():
            parts = line.split()
            if len(parts) != 2:
                continue
            try:
                timing.append((float(parts[0]), int(parts[1])))
            except ValueError:
                continue

    records = []
    position = 0
    running = 0.0
    for elapsed, length in timing:
        if length <= 0:
            continue
        chunk = body[position : position + length]
        if not chunk:
            break
        position += length
        # script(1)'s timing file holds per-write delays; asciinema v2 wants
        # absolute elapsed seconds, so accumulate.
        running += elapsed
        data = normalize_text(chunk.decode("utf-8", "replace"))
        records.append([round(running, 6), "o", data])
    if not records and body:
        records.append([0.0, "o", normalize_text(body.decode("utf-8", "replace"))])

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with out_path.open("w", encoding="utf-8") as handle:
        handle.write(json.dumps({"version": 2, "width": 80, "height": 24}) + "\n")
        for record in records:
            handle.write(json.dumps(record, ensure_ascii=False) + "\n")
    print(f"evidence: wrote {out_path} ({len(records)} events)")


def cmd_normalize(args):
    for name in args.path:
        path = Path(name)
        text = path.read_text(encoding="utf-8")
        path.write_text(normalize_text(scrub_log(text)), encoding="utf-8")
        print(f"evidence: normalized {path}")


def build_parser():
    parser = argparse.ArgumentParser(description="Record a Phase evidence item.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    begin = subparsers.add_parser("begin", help="start an item")
    begin.add_argument("--item-dir", required=True)
    begin.add_argument("--item", required=True)
    begin.add_argument("--requirements", default="")
    begin.add_argument("--command", required=True)
    begin.set_defaults(func=cmd_begin)

    step = subparsers.add_parser("step", help="run and record one step")
    step.add_argument("--item-dir", required=True)
    step.add_argument("--name", required=True)
    step.add_argument("cmd", nargs=argparse.REMAINDER)
    step.set_defaults(func=cmd_step)

    artifact = subparsers.add_parser("artifact", help="record one artifact")
    artifact.add_argument("--item-dir", required=True)
    artifact.add_argument("--path", required=True)
    artifact.set_defaults(func=cmd_artifact)

    finish = subparsers.add_parser("finish", help="write report.json")
    finish.add_argument("--item-dir", required=True)
    finish.add_argument("--result", required=True, choices=["ok", "fail", "skipped"])
    finish.set_defaults(func=cmd_finish)

    cast = subparsers.add_parser("cast", help="convert script(1) output to asciinema v2")
    cast.add_argument("--typescript", required=True)
    cast.add_argument("--timing", required=True)
    cast.add_argument("--out", required=True)
    cast.set_defaults(func=cmd_cast)

    normalize = subparsers.add_parser("normalize", help="normalize a file in place")
    normalize.add_argument("--path", action="append", required=True)
    normalize.set_defaults(func=cmd_normalize)

    return parser


def main():
    parser = build_parser()
    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
