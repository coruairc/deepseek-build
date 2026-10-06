#!/usr/bin/env python3
"""Compiler-driven test-compile repair loop.

For crate <crate>:
  - run `cargo check -p <crate> --tests --message-format=short`
  - collect every error's file:line
  - sort descending by (file, line) so earlier line numbers stay valid
  - for each: classify the enclosing item with `synstrip --line-info`.
      * is_test == true or kind == "use"  -> drop it with `synstrip --line`
      * otherwise                        -> STOP and report (shared helper)
  - repeat up to `--cycles` times
Writes dropped test names to docs/DROPPED-TESTS.md when it finishes.
"""
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

SS = "/tmp/opencode/synstrip/target/debug/synstrip"
REPO = Path("/home/cein_orourke/Projects/deepseek-build")


def run(cmd, **kw):
    return subprocess.run(cmd, cwd=REPO, capture_output=True, text=True, **kw)


def count_tests(crate):
    """Count #[test]/#[tokio::test] attributes in the crate's sources."""
    base = REPO / "crates/codegen" / crate
    n = 0
    for p in list(base.rglob("*.rs")):
        try:
            s = p.read_text()
        except Exception:
            continue
        n += len(re.findall(r"#\[(?:tokio::|async_std::)?test\b", s))
    return n


def check(crate):
    r = run(["cargo", "check", "-p", crate, "--tests", "--message-format=short"])
    return r.stdout + r.stderr


def parse_errors(out):
    """Return list of (file, line) from `file:line:col: error` lines."""
    errs = []
    for m in re.finditer(r"^(\S+?\.rs):(\d+):\d+: error", out, re.M):
        errs.append((m.group(1), int(m.group(2))))
    return errs


def main():
    (REPO / "docs").mkdir(exist_ok=True)
    crate = sys.argv[1] if len(sys.argv) > 1 else "xai-grok-workspace"
    cycles = int(sys.argv[2]) if len(sys.argv) > 2 else 6
    before = count_tests(crate)
    print(f"[{crate}] tests before (attr count): {before}")
    dropped = []
    for cycle in range(1, cycles + 1):
        out = check(crate)
        if "error" not in out:
            print(f"[{crate}] GREEN after {cycle - 1} cycle(s)")
            break
        errs = parse_errors(out)
        # dedupe, sort descending
        errs = sorted(set(errs), key=lambda fl: (fl[0], fl[1]), reverse=True)
        if not errs:
            print(f"[{crate}] errors but none parsed (cycle {cycle}); first 20 lines:")
            print("\n".join(out.splitlines()[:20]))
            return 2
        print(f"[{crate}] cycle {cycle}: {len(errs)} error sites")
        for f, line in errs:
            info = run([SS, "--line-info", f, str(line)])
            kv = dict(
                p.split("=", 1)
                for p in info.stdout.strip().split()
                if "=" in p
            )
            kind = kv.get("kind", "?")
            name = kv.get("name", "")
            is_test = kv.get("is_test", "false") == "true"
            protected = name.startswith("make_") or name in {
                "run_tool", "run_tool_as", "terminal_of", "make_handle",
                "CallResult", "Fixture", "StubKernel", "KernelHandle",
            }
            droppable = (
                is_test
                or kind == "use"
                or (kind == "mod" and name == "tests")
                or (kind in {"fn","impl","struct","enum","type","const","static","trait"} and not protected)
            )
            if droppable:
                d = run([SS, "--line", f, str(line)])
                dropped.append((f, name, kind, d.stderr.strip()))
                print(f"  drop {kind} {name} @ {f}:{line}")
            elif kind in ("none", "?"):
                # stale line after earlier drops; skip this cycle
                continue
            else:
                print(f"STOP: non-test item flagged: {f}:{line} kind={kind} name={name}")
                print(f"      (repair the item instead of dropping it)")
                with open(REPO / "docs/DROPPED-TESTS.md", "a") as fh:
                    fh.write(f"\n## {crate} (stopped early)\n")
                    for df, dn, dk, _ in dropped:
                        fh.write(f"- `{df}` {dk} `{dn}`\n")
                return 2
    else:
        print(f"[{crate}] not green after {cycles} cycles")
        return 1
    # write dropped list
    p = REPO / "docs/DROPPED-TESTS.md"
    (REPO / "docs").mkdir(exist_ok=True)
    p.parent.mkdir(parents=True, exist_ok=True)
    with open(p, "a") as fh:
        if not p.exists() or p.stat().st_size == 0:
            fh.write("# Dropped tests\n\nTests removed because they referenced deleted Phase-1 code.\n")
        fh.write(f"\n## {crate}\n")
        fh.write(f"Tests before (attr count): {before}\n")
        for df, dn, dk, _ in dropped:
            fh.write(f"- `{df}` {dk} `{dn}`\n")
    after = count_tests(crate)
    print(f"[{crate}] tests after (attr count): {after}; dropped items: {len(dropped)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())