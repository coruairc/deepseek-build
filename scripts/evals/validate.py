#!/usr/bin/env python3
"""Validate eval tasks both ways (no model calls).

For every task:

1. the pristine fixture must FAIL its checks (`fails_before: true`), and
2. a reference fix must make every check PASS.

The reference fixes live in `PATCHES` below. A new task needs an entry here
before it can be trusted in a comparison.

Usage:
    python3 scripts/evals/validate.py
    python3 scripts/evals/validate.py --tasks py-fix-add
"""

from __future__ import annotations

import argparse
import importlib.util
import sys
import tempfile
from pathlib import Path

EVALS = Path(__file__).resolve().parent


def load_harness():
    spec = importlib.util.spec_from_file_location("harness", EVALS / "harness.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


PATCHES: dict[str, dict[str, str]] = {
    "py-fix-add": {"calc.py": "def add(a, b):\n    return a + b\n\n\ndef sub(a, b):\n    return a - b\n"},
    "py-fix-offbyone": {"seq.py": "def count_to(n):\n    return list(range(1, n + 1))\n"},
    "py-fix-mutable-default": {
        "store.py": "def add_item(item, items=None):\n    if items is None:\n        items = []\n    items.append(item)\n    return items\n"
    },
    "py-fix-regex": {
        "parse.py": "import re\n\n\nPATTERN = re.compile(r\"\\d+\")\n\n\ndef extract_number(text):\n    m = PATTERN.search(text)\n    return m.group(0) if m else None\n"
    },
    "py-fix-json-datetime": {
        "records.py": "import datetime\nimport json\n\n\nclass _Encoder(json.JSONEncoder):\n    def default(self, obj):\n        if isinstance(obj, datetime.date):\n            return obj.isoformat()\n        return super().default(obj)\n\n\ndef to_json(record):\n    return json.dumps(record, cls=_Encoder)\n"
    },
    "py-fix-sort-key": {
        "ranking.py": "def rank_players(players):\n    return sorted(players, key=lambda p: p['score'], reverse=True)\n"
    },
    "py-fix-recursion": {"mathx.py": "def factorial(n):\n    if n == 0:\n        return 1\n    return n * factorial(n - 1)\n"},
    "py-fix-csv-parse": {
        "csvline.py": "import csv\nimport io\n\n\ndef split_row(line):\n    return next(csv.reader(io.StringIO(line)))\n"
    },
    "py-add-mean": {
        "stats.py": "def total(values):\n    return sum(values)\n\n\ndef mean(values):\n    if not values:\n        raise ValueError('mean() of empty sequence')\n    return sum(values) / len(values)\n"
    },
    "py-add-stack": {
        "stack.py": "class Stack:\n    def __init__(self):\n        self._items = []\n\n    def push(self, x):\n        self._items.append(x)\n\n    def pop(self):\n        if not self._items:\n            raise IndexError('pop from empty stack')\n        return self._items.pop()\n\n    def peek(self):\n        if not self._items:\n            raise IndexError('peek from empty stack')\n        return self._items[-1]\n\n    def is_empty(self):\n        return not self._items\n\n    def __len__(self):\n        return len(self._items)\n"
    },
    "py-refactor-rename": {
        "users.py": "def get_user(uid):\n    return {'id': uid, 'name': 'user' + str(uid)}\n",
        "main.py": "from users import get_user\n\n\n\ndef describe(uid):\n    return get_user(uid)['name']\n",
    },
    "py-refactor-dedupe": {
        "report.py": "def normalize_name(name):\n    return name.strip().lower()\n\n\ndef header(name):\n    return '[ ' + normalize_name(name) + ' ]'\n\n\ndef footer(name):\n    return '( ' + normalize_name(name) + ' )'\n\n\ndef body(name):\n    return '  ' + normalize_name(name)\n"
    },
    "py-multifile-move": {
        "textutil.py": "def slugify(title):\n    return '-'.join(w.lower() for w in title.split())\n",
        "blog.py": "from textutil import slugify  # noqa: F401  (re-export)\n\n\ndef make_url(title):\n    return '/posts/' + slugify(title)\n",
    },
    "py-fix-exception": {"config.py": "def get_value(cfg, key, default=None):\n    return cfg.get(key, default)\n"},
    "py-fix-path-join": {"paths.py": "import os\n\n\ndef join_path(*parts):\n    return os.path.join(*parts)\n"},
    "py-fix-timestamp": {
        "clock.py": "import datetime\n\n\ndef from_epoch(seconds):\n    return datetime.datetime.fromtimestamp(seconds, datetime.timezone.utc)\n"
    },
    "py-fix-generator": {
        "stream.py": "def first_even(limit):\n    for n in range(limit):\n        if n % 2 == 0:\n            yield n\n"
    },
    "py-fix-context-manager": {
        "resource.py": "class Resource:\n    def __init__(self):\n        self.closed = False\n\n    def __enter__(self):\n        return self\n\n    def __exit__(self, exc_type, exc, tb):\n        self.closed = True\n        return False\n"
    },
    "py-fix-cli-args": {
        "cli.py": "import argparse\n\n\ndef main(argv=None):\n    parser = argparse.ArgumentParser()\n    parser.add_argument('--count', type=int, default=1)\n    args = parser.parse_args(argv)\n    return args.count\n"
    },
    "py-fix-dedupe-case": {
        "dedupe.py": "def dedupe(items):\n    seen = set()\n    out = []\n    for item in items:\n        if not item:\n            continue\n        key = item.lower()\n        if key not in seen:\n            out.append(item)\n            seen.add(key)\n    return out\n"
    },
    "sh-fix-script": {
        "check.sh": '#!/bin/sh\ncode=0\nfor f in "$@"; do\n  if ! test -r "$f"; then\n    code=1\n  fi\ndone\nexit $code\n'
    },
}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--tasks-dir", type=Path, default=EVALS / "tasks")
    parser.add_argument("--tasks", help="comma-separated task ids (default: all)")
    args = parser.parse_args(argv)

    harness = load_harness()
    try:
        tasks = harness.load_tasks(args.tasks_dir, args.tasks)
    except ValueError as exc:
        parser.error(str(exc))

    bad = 0
    for task in tasks:
        task_id = task["id"]
        with tempfile.TemporaryDirectory(prefix="eval-validate-") as tmp:
            scratch = Path(tmp) / "repo"
            harness.build_fixture(task, scratch)
            harness.clear_pycache(scratch)
            pristine = harness.run_checks(task, scratch, 120)
            patch = PATCHES.get(task_id)
            if patch is None:
                print(f"[skip]     {task_id}: no reference patch registered")
                bad += 1
                continue
            for rel, content in patch.items():
                target = scratch / rel
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(content)
            # The pristine run may have byte-compiled the fixture; a same-second
            # rewrite can otherwise be served from the stale `__pycache__`.
            harness.clear_pycache(scratch)
            fixed = harness.run_checks(task, scratch, 120)

        pristine_fail = not all(c["pass"] for c in pristine)
        fixed_pass = all(c["pass"] for c in fixed)
        if pristine_fail and fixed_pass:
            print(f"[ok]       {task_id}")
            continue
        bad += 1
        print(f"[INVALID]  {task_id}"
              f" (pristine{' fails' if pristine_fail else ' PASSES'}," 
              f" reference fix{' PASSES' if fixed_pass else ' FAILS'})")
        for check in fixed:
            if not check["pass"]:
                print(f"    fix: {check['type']}: {check.get('detail', check.get('pattern', ''))}"[:200])

    print(f"\nvalidate: {len(tasks) - bad}/{len(tasks)} task(s) valid")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
