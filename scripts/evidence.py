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
status. A command that fails to launch (OSError) is recorded as a failed
step, not an unrecorded crash. The step record is saved before the output
is echoed, so a broken stdout pipe cannot lose the record of a completed
run. `finish` refuses `--result ok` when any step failed, no steps were
recorded, or a recorded artifact is missing from disk (a report cannot
vouch for a file it cannot read); a non-ok result drops a missing artifact
with a note instead of keeping a stale digest. It recomputes artifact
sha256s after normalization so a digest recorded over a raw log cannot go
stale. `cast` converts
`script(1)`'s typescript plus its `-T` timing file into asciinema v2: one
`{"version":2,"width":80,"height":24}` header line, then `[elapsed, "o", data]`
records derived from the timing file's `elapsed bytes` pairs (the per-write
delays are accumulated into absolute elapsed seconds). The leading
`Script started` line and any trailing `Script done` framing line are
dropped explicitly; a missing timing file, or one with no positive-length
pairs over a non-empty body, is an error (exit 2). Normalization runs over the
assembled body with a partial-prefix carry between chunks, so an absolute
path (or multi-byte UTF-8 sequence) split across two writes is still
substituted. `normalize` applies
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

    try:
        proc = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    except OSError as exc:
        # A command that never launched is a failed step, not an unrecorded
        # crash: record it so `finish --result ok` cannot succeed over it.
        state["steps"].append(
            {
                "name": args.name,
                "ok": False,
                "detail": f"exit n/a: {exc}",
            }
        )
        save_pending(item_dir, state)
        sys.stderr.write(f"evidence: step failed to launch: {exc}\n")
        raise SystemExit(1)
    output = proc.stdout.decode("utf-8", "replace")
    log_path = item_dir / f"{slug(args.name)}.log"
    log_path.write_text(output, encoding="utf-8")

    # Persist the step record BEFORE echoing: a broken stdout pipe must not
    # lose the record of a command that ran to completion.
    state["steps"].append(
        {
            "name": args.name,
            "ok": proc.returncode == 0,
            "detail": summarize(proc.returncode, output),
        }
    )
    save_pending(item_dir, state)
    try:
        sys.stdout.write(output)
        sys.stdout.flush()
    except BrokenPipeError:
        # The executor's pipe is gone; the record is already saved. Point
        # stdout at devnull so the interpreter's final flush stays quiet.
        devnull = os.open(os.devnull, os.O_WRONLY)
        os.dup2(devnull, sys.stdout.fileno())
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
        if not state["steps"]:
            sys.stderr.write("evidence: refusing --result ok; no steps were recorded\n")
            raise SystemExit(2)
        failed = [step["name"] for step in state["steps"] if not step["ok"]]
        if failed:
            sys.stderr.write(
                "evidence: refusing --result ok; failed step(s): " + ", ".join(failed) + "\n"
            )
            raise SystemExit(2)

    # A recorded artifact that vanished must not keep a digest the report
    # cannot back. `--result ok` refuses (fail closed, like a failed step);
    # other results drop it with a note rather than assert a stale digest.
    missing = [
        artifact["path"]
        for artifact in state["artifacts"]
        if not (REPO_ROOT / artifact["path"]).is_file()
    ]
    if missing:
        shown = ", ".join(normalize_text(path) for path in missing)
        if args.result == "ok":
            sys.stderr.write(
                "evidence: refusing --result ok; recorded artifact(s) missing: "
                + shown
                + "\n"
            )
            raise SystemExit(2)
        sys.stderr.write(
            "evidence: dropping missing artifact(s) from the report: " + shown + "\n"
        )
        state["artifacts"] = [
            artifact
            for artifact in state["artifacts"]
            if (REPO_ROOT / artifact["path"]).is_file()
        ]

    for step in state["steps"]:
        log_path = item_dir / f"{slug(step['name'])}.log"
        if log_path.is_file():
            text = log_path.read_text(encoding="utf-8", errors="replace")
            log_path.write_text(normalize_text(scrub_log(text)), encoding="utf-8")

    # Recompute artifact digests AFTER normalization: `finish` rewrites step
    # logs, so a digest recorded over the raw log would otherwise go stale.
    for artifact in state["artifacts"]:
        path = REPO_ROOT / artifact["path"]
        if path.is_file():
            actual = hashlib.sha256(path.read_bytes()).hexdigest()
            if actual != artifact["sha256"]:
                artifact["sha256"] = actual
                print(
                    "evidence: refreshed sha256 for "
                    f"{normalize_text(artifact['path'])} (changed after normalization)"
                )

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


def _strip_script_trailer(body):
    """Drop script(1)'s trailing `Script done …` framing line, if present.

    The timing file usually does not account for the trailer, but one that
    does must not leak the framing line into the cast.
    """
    if not body:
        return body
    line_end = len(body) - 1 if body.endswith(b"\n") else len(body)
    line_start = body.rfind(b"\n", 0, line_end) + 1
    if body[line_start:line_end].startswith(b"Script done"):
        return body[:line_start]
    return body


def _split_carry(chunk, targets):
    """Split raw bytes into (emit, carry) for cast normalization.

    `carry` holds trailing bytes that could begin a substitution prefix
    (an absolute repo/home/temp path split across two script writes) or an
    incomplete UTF-8 sequence; they are prepended to the next chunk so the
    assembled body is normalized exactly, while event boundaries keep the
    timing file's shape.
    """
    hold = 0
    for target in targets:
        for k in range(min(len(target) - 1, len(chunk)), 0, -1):
            if chunk.endswith(target[:k]):
                hold = max(hold, k)
                break
    emit = chunk[: len(chunk) - hold] if hold else chunk
    try:
        emit.decode("utf-8")
    except UnicodeDecodeError as exc:
        if exc.reason == "unexpected end of data":
            hold += len(emit) - exc.start
            emit = chunk[: len(chunk) - hold]
    carry = chunk[len(chunk) - hold :] if hold else b""
    return emit, carry


def cmd_cast(args):
    typescript = Path(args.typescript).read_bytes()
    newline = typescript.find(b"\n")
    body = typescript[newline + 1 :] if newline >= 0 else typescript
    body = _strip_script_trailer(body)

    timing_path = Path(args.timing)
    if not timing_path.is_file():
        sys.stderr.write(f"evidence: timing file not found: {timing_path}\n")
        raise SystemExit(2)
    timing = []
    for line in timing_path.read_text(encoding="utf-8").splitlines():
        parts = line.split()
        if len(parts) != 2:
            continue
        try:
            timing.append((float(parts[0]), int(parts[1])))
        except ValueError:
            continue
    if body and not any(length > 0 for _, length in timing):
        sys.stderr.write(
            "evidence: timing file has no usable `elapsed bytes` pairs "
            "(no positive byte counts) but the typescript body is non-empty\n"
        )
        raise SystemExit(2)

    # Normalize the assembled body, not each chunk: a write boundary can
    # fall inside an absolute path (or a multi-byte UTF-8 sequence), and
    # per-chunk substitution would miss the split prefix. The carry keeps
    # any partial prefix for the next chunk; the concatenated event data
    # is exactly the normalized body.
    targets = [
        str(REPO_ROOT).encode("utf-8"),
        str(HOME_DIR).encode("utf-8"),
        str(TMP_DIR).encode("utf-8"),
    ]
    targets = [target for target in targets if target and target != b"/"]

    records = []
    position = 0
    running = 0.0
    carry = b""
    for elapsed, length in timing:
        if length <= 0:
            continue
        if position >= len(body):
            break
        chunk = carry + body[position : position + length]
        position += length
        # script(1)'s timing file holds per-write delays; asciinema v2 wants
        # absolute elapsed seconds, so accumulate.
        running += elapsed
        emit, carry = _split_carry(chunk, targets)
        data = normalize_text(emit.decode("utf-8", "replace"))
        if data:
            records.append([round(running, 6), "o", data])
    # Flush any bytes the timing file did not cover (plus the held carry),
    # so no output is silently dropped.
    if position < len(body):
        carry += body[position:]
    if carry:
        data = normalize_text(carry.decode("utf-8", "replace"))
        if data:
            records.append([round(running, 6), "o", data])

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
        text = path.read_text(encoding="utf-8", errors="replace")
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
