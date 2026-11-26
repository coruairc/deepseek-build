#!/bin/sh
# Re-assert the retention invariants recorded in DEVIATION-keep-prerelease.md:
# no :latest, no v0.1.0, and the deepseek branch / deepseek-v0.1-snapshot tag
# untouched. Read-only: anonymous registry reads, git ls-remote and gh api GETs.
set -eu

d=$(mktemp -d)
trap 'rm -rf "$d"' EXIT

python3 scripts/distribution-proof.py baseline --out "$d/baseline.json" >"$d/baseline.log"
grep -F "tags=['0.1.0-proof1', 'v0.1.0-proof1'] latest=absent" "$d/baseline.log"
echo "registry: exactly the two proof tags, :latest absent"

if git ls-remote origin refs/tags/v0.1.0 'refs/tags/v0.1.0^{}' | grep -q .; then
  echo "FAIL: refs/tags/v0.1.0 exists on origin" >&2
  exit 1
fi
echo "remote: no refs/tags/v0.1.0"

snapshot=$(git ls-remote origin 'refs/tags/deepseek-v0.1-snapshot^{}')
case "$snapshot" in
  4b305bd0c1ab2fbb55468df14ab6a7e94d7c8d0b*)
    echo "remote: deepseek-v0.1-snapshot still peels to 4b305bd0"
    ;;
  *)
    echo "FAIL: deepseek-v0.1-snapshot moved: $snapshot" >&2
    exit 1
    ;;
esac

branch=$(git ls-remote origin refs/heads/deepseek)
case "$branch" in
  4b305bd0c1ab2fbb55468df14ab6a7e94d7c8d0b*)
    echo "remote: deepseek branch still 4b305bd0"
    ;;
  *)
    echo "FAIL: deepseek branch moved: $branch" >&2
    exit 1
    ;;
esac

releases=$(gh api repos/coruairc/deepseek-build/releases --jq '.[].tag_name')
if echo "$releases" | grep -qx 'v0.1.0'; then
  echo "FAIL: a v0.1.0 release exists" >&2
  exit 1
fi
echo "releases: no v0.1.0 (tags: $(echo "$releases" | tr '\n' ' '))"

if gh api repos/coruairc/deepseek-build/releases/latest >/dev/null 2>&1; then
  echo "FAIL: releases/latest resolves, so a stable release exists" >&2
  exit 1
fi
echo "releases/latest: 404 (every release is a pre-release)"
