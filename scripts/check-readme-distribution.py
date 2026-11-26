#!/usr/bin/env python3
"""check-readme-distribution.py — assert README.md's distribution claims.

The README is prose read by a user who will copy a command from it. These
checks pin the sentences that must stay true (the pinned release, the unpinned
404, the macOS signature reality) and the stale claims that must not come back.

Matching is whitespace-normalized: every run of whitespace collapses to one
space, so a phrase split across two Markdown lines still matches (and a stale
claim cannot hide by being line-wrapped).

Usage:
    python3 scripts/check-readme-distribution.py --stage install
    python3 scripts/check-readme-distribution.py --stage container \\
        --digest sha256:... [--visibility private|public]

Stages:
    install    this phase's installer proof: v0.1.0-proof1 is the installable
               pre-release, VERSION= is required, latest 404s, the macOS
               binaries are ad-hoc signed and not notarized.
    container  04-04's container proof: the published image digest, the
               headless/run recipe, and (for a private package) the login step.

Exit status: 0 when every assertion passed, 1 otherwise.
"""

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
README = REPO_ROOT / "README.md"


class Checks:
    def __init__(self):
        self.results = []

    def ok(self, name, passed, detail=""):
        self.results.append((name, bool(passed), detail))

    def contains(self, text, needle, label=None):
        self.ok(label or f"contains {needle!r}", needle in text)

    def absent(self, text, needle):
        self.ok(f"does not contain {needle!r}", needle not in text)

    def report(self, stage):
        passed = sum(1 for _, ok, _ in self.results if ok)
        total = len(self.results)
        for name, ok, detail in self.results:
            line = f"{'PASS' if ok else 'FAIL'}: {name}"
            if detail:
                line += f" ({detail})"
            print(line)
        print(f"readme-distribution[{stage}]: {passed}/{total} PASS")
        return 0 if passed == total else 1


def normalize(text):
    return re.sub(r"\s+", " ", text)


def stage_install(checks, text):
    count = text.count("v0.1.0-proof1")
    checks.ok(f"names v0.1.0-proof1 at least 3 times (found {count})", count >= 3)
    checks.contains(text, "VERSION=v0.1.0-proof1")
    checks.contains(text, "releases/download/v0.1.0-proof1/install.sh")
    checks.contains(text, "pre-release")
    checks.contains(text, "latest")
    checks.contains(text, "404")
    checks.contains(text, "ad-hoc")
    checks.contains(text, "not notarized")
    checks.absent(text, "No release has been published yet")
    checks.absent(text, "not codesigned")
    checks.absent(text, "is currently empty")
    checks.contains(text, "not affiliated with DeepSeek")


def stage_container(checks, text, digest, visibility):
    if not digest:
        checks.ok("--digest was provided for --stage container", False)
    else:
        checks.contains(text, digest)
    checks.contains(text, "headless")
    checks.contains(text, "/work")
    checks.contains(text, "--sandbox devbox")
    checks.contains(text, "/home/app/.deepseek-build")
    if visibility == "private":
        checks.contains(text, "docker login ghcr.io")
    checks.absent(text, "Not yet built or published")
    checks.absent(text, "ghcr.io/<owner>")


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--stage", required=True, choices=["install", "container"])
    parser.add_argument("--readme", default=str(README))
    parser.add_argument("--digest", default="", help="image digest for --stage container")
    parser.add_argument(
        "--visibility",
        default="private",
        choices=["private", "public"],
        help="GHCR package visibility (default: private)",
    )
    args = parser.parse_args()

    readme = Path(args.readme)
    if not readme.is_file():
        sys.stderr.write(f"readme-distribution: no README at {readme}\n")
        return 1

    text = normalize(readme.read_text(encoding="utf-8"))
    checks = Checks()
    if args.stage == "install":
        stage_install(checks, text)
    else:
        stage_container(checks, text, args.digest, args.visibility)
    return checks.report(args.stage)


if __name__ == "__main__":
    sys.exit(main())
