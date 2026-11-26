# seal — the mechanical close of the Phase 4 evidence

This item records the seal itself, not a new distribution run. It proves the
other eight items still validate against the eight-key contract, their
artifacts still hash to their recorded sha256, the tree is leak-scan clean, and
the keep-the-prerelease decision is recorded against the roadmap line it
contradicts.

`brew install`, the image pull and the in-image turn are **not** part of this
item; the first is an author step on macOS and the other two stay REL-05
environment-blocked.

## Validator

`python3 scripts/check-distribution-evidence.py docs/evidence/04-distribution-proof --self-test`
runs four mutations on a temp copy — delete an artifact, corrupt a recorded
sha256, flip `image-run-blocked` to `ok`, write a NUL byte into an artifact —
and requires a named rejection for each:

```
self-test: 4/4 PASS
```

At seal time the real tree held the eight items produced by 04-01…04-04:

```
distribution-evidence: 8/8 PASS
8 reports validated
```

After this `seal` report was written the same command was re-run over nine
reports, `seal` included: `distribution-evidence: 9/9 PASS`, `9 reports
validated`. [`seal.log`](seal.log) records the eight-report run, because a
report cannot hash a log that reports on the report that hashes it: the
nine-report run is a command, not an artifact of this item.

Exactly one item is `skipped`: `image-run-blocked`, requirement `REL-05`, one
`docker info` step that exits 1 because no daemon is running. The checker
refuses any item that claims `REL-05` as `ok` without a recorded successful
`pull` step, so REL-05 cannot drift into a false pass. For that reason this
`seal` item's own `requirements` are `REL-04, REL-06, PKG-04` — it validates
the evidence for those three; REL-05 is recorded as open, not proven, and
adding it here would be the false claim the checker exists to reject.

## Size gate

```
$ find docs/evidence/04-distribution-proof -type f -size +1M | wc -l
0
```

No tarball, image layer or cast-sized recording is committed. Every artifact
is text (`UTF-8`, no NUL byte) and at most 1 MiB, and each recorded `sha256`
equals the file's current hash.

## Leak scan

`scripts/key-leak-check.sh --scan` (file names only; fails closed on a grep
error). Three runs, all exit 0:

```
$ scripts/key-leak-check.sh --scan docs/evidence/04-distribution-proof
PASS  no key-shaped string in: docs/evidence/04-distribution-proof
(exit 0)

$ scripts/key-leak-check.sh --scan docs/evidence .planning/phases/04-distribution-proof
PASS  no key-shaped string in: docs/evidence .planning/phases/04-distribution-proof
(exit 0)

$ scripts/key-leak-check.sh --scan docs/evidence
PASS  no key-shaped string in: docs/evidence
(exit 0)
```

## Retention invariants

[`retention-check.sh`](retention-check.sh) re-reads the live refs and the
registry (read-only; no mutation) and asserts what
[`DEVIATION-keep-prerelease.md`](../DEVIATION-keep-prerelease.md) promises:

```
baseline: transport=anonymous-http tags_status=200 tags=['0.1.0-proof1', 'v0.1.0-proof1'] latest=absent digest=None
registry: exactly the two proof tags, :latest absent
remote: no refs/tags/v0.1.0
remote: deepseek-v0.1-snapshot still peels to 4b305bd0
remote: deepseek branch still 4b305bd0
releases: no v0.1.0 (tags: v0.1.0-proof1 v0.0.1-rc5 v0.0.1-rc4 )
releases/latest: 404 (every release is a pre-release)
```

No cleanup was performed by this phase: the tag, the pre-release and the GHCR
tags are retained by decision, `v0.1.0` is unpublished, and `:latest` is
unchanged from the pre-push baseline.

## Aggregate requirement status

| Requirement | Status |
|-------------|--------|
| REL-04 | **Partial** — built and published; registry side proven (`image-registry`, 8/8); the pull did not happen (no daemon) |
| REL-05 | **Open / environment-blocked** — `image-run-blocked` is `skipped`; exact digest-pinned commands recorded |
| REL-06 | **Closed** — pinned install from the real release prints `0.1.0-proof1 (b14d28eb2bc7)`; unpinned `latest` observed to 404 |
| PKG-04 | **Closed for the in-tree formula** — Homebrew-checked, tap activation documented; `brew install` is an author step on macOS |

Phase 4 is 5/5 plans executed with two requirements partially/blocked. It is
not a fully complete phase; ROADMAP Phase 4 criteria 1 (pulled) and 2 (in-image
turn) remain unmet.

## Deviation

[`../DEVIATION-keep-prerelease.md`](../DEVIATION-keep-prerelease.md) names
`ROADMAP.md` → `### Phase 4: Distribution Proof` → `Success Criteria` item 4
(the cleanup line) and `04-CONTEXT.md` §Cleanup / §Decided autonomously, lists
what is retained, gives the reason (the pre-release is the milestone's only
macOS artifact), and records the invariants the retention keeps. It contains no
deletion command.

## Files

| File | Meaning |
|------|---------|
| `seal.log` | validator output over the eight produced reports |
| `retention-check.sh` | read-only re-assertion of no `:latest`, no `v0.1.0`, untouched `deepseek` / `deepseek-v0.1-snapshot` |
| `notes.md` | this file |
| `report.json` | the item record (`scripts/evidence.py finish --result ok`) |
