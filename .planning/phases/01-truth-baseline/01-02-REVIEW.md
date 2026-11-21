---
phase: 01-truth-baseline
plan: 02
reviewed: 2026-11-21T19:36:40Z
depth: deep
files_reviewed: 7
files_reviewed_list:
  - scripts/egress_guard.c
  - scripts/test-sandbox-run.sh
  - scripts/sandbox-run.sh
  - scripts/egress-check.sh
  - scripts/README.md
  - .planning/phases/01-truth-baseline/01-02-SUMMARY.md
  - .planning/phases/01-truth-baseline/deferred-items.md
findings:
  critical: 0
  warning: 4
  info: 8
  total: 12
status: issues_found
---

# Phase 01 Plan 02: Code Review Report (`sandbox-run.sh` IPv6 truth)

**Reviewed:** 2026-11-21T19:36:40Z
**Base:** `0bf46c84` (all change under review is uncommitted in the working tree)
**Depth:** deep
**Files Reviewed:** 7
**Status:** issues_found (no blockers)

## Summary

SBOX-02's second half is implemented and the claim survives adversarial probing. I read the whole
C shim against the old heredoc, compiled both, and drove each through real `connect()` calls with
`LD_PRELOAD`; I re-ran every command the SUMMARY records. The intended behaviour holds:

- The constructor resolves `EGRESS_ALLOW_HOST` with `AF_UNSPEC`/`SOCK_STREAM` and stores raw,
  family-tagged bytes; matching is `family + memcmp` (4 bytes `AF_INET`, 16 bytes `AF_INET6`).
  `store()` and the `connect` path normalise `::ffff:a.b.c.d` in **both** directions, so
  `::ffff:203.0.113.9` and `203.0.113.9` are one entry (I confirmed a v4 connect is permitted when
  the allow host is given as the mapped spelling, and that a mapped connect hits a v4 entry).
- The blocked line names the destination per family. `2001:db8::1` prints
  `sandbox-run: BLOCKED connect to 2001:db8::1`; `::ffff:8.8.8.8` prints
  `…to ::ffff:8.8.8.8`; `AF_NETLINK` still prints `…to non-ip`, which is now the literal's only
  remaining use. `non-ip` is not reachable for either IP family.
- Behavioural equivalence with the old shim: identical for AF_UNIX, `127/8`, `0/8`, the v4
  allowlist, the v4 refuse/format path and `::1`. The only newly-allowed destinations are the ones
  the plan mandates (v4-mapped loopback / `0/8` / allowlisted v4 spellings); nothing else is newly
  allowed, and nothing the old shim allowed is now refused. I verified this by rebuilding the
  pre-change shim from `git show HEAD:scripts/sandbox-run.sh`.
- Red-first reproduces independently. Against the old shim: `2001:db8::1` → `BLOCKED connect to
  non-ip` (fails assertions 1 and 2), `EGRESS_ALLOW_HOST=2001:db8::2` → no `family=inet6` line and
  the destination is refused (fails assertion 4), and `::ffff:127.0.0.1` is refused (so assertion 3
  fails too — the SUMMARY understates this as "1, 2 and 4"). Assertions 5 and 6 pass on the old code
  by design. The harness therefore measures the fix.
- `sh scripts/test-sandbox-run.sh` → `ALL PASS (6 assertions)`, rc=0, 2.2 s, deterministic across two
  runs. `sh -n` / `bash -n` clean; `shellcheck --shell=sh` rc=0 on both shell scripts; the shim
  compiles clean with `-Wall -Wextra` (pedantic warnings are the standard POSIX `dlsym`/function-
  pointer ones and are not defects).
- `sandbox-run.sh` is a genuine extraction: `cd`, `BIN`, `HOST`, `WORK`, `trap`, the startup echo and
  the `exec env … LD_PRELOAD=… "$BIN" "$@"` line are byte-identical to HEAD; only the heredoc and its
  `cc` line changed. `egress-check.sh`'s diff is prose only — no gate, count line, helper or exit
  code moved. No dependency, lockfile or `Cargo.toml` change; no new egress destination (the shim
  still only refuses, and its only network action is the pre-existing `getaddrinfo`); no secrets.

Verification honesty is high: every number, command and diff excerpt in the SUMMARY reproduces,
including the two "unfavourable" results it did not hide (the red HARD gate and the `bash -n`
re-check). The remaining findings are about robustness at the edges of the proof, and about prose
outside the plan's `files_modified` that now contradicts the code.

## Warnings

### WR-01: `dlsym` failure would crash the guard instead of refusing

**File:** `scripts/egress_guard.c:160-161` (also `:100`, and the pass-through calls at `:167`, `:173`, `:177`)
**Issue:** `real_connect` is re-resolved lazily and then called unconditionally:

```c
if (!real_connect) real_connect = dlsym(RTLD_NEXT, "connect");
if (!addr) return real_connect(fd, addr, len);
```

If `dlsym(RTLD_NEXT, "connect")` ever returns `NULL` (a statically linked target, a libc without an
interposable `connect`, or an unusual loader state), every path — including the "allowed" branches —
calls through a `NULL` function pointer. That is UB and in practice a `SIGSEGV`, i.e. the guard
fails *open by dying* rather than failing closed. It is inherited from the old shim, but the old
shim did not claim "fail closed" as its contract, and this file does (`:19-21`). A crash here takes
down the sandboxed process with no `BLOCKED` line, so an operator cannot tell a guard fault from a
provider fault.
**Fix:** refuse when the real symbol is unavailable, and keep the constructor's result:

```c
if (!real_connect) real_connect = dlsym(RTLD_NEXT, "connect");
if (!real_connect) {
  errno = ENOSYS;
  return -1;
}
```
(Leave the `!addr` fast path below it, so a `NULL` address is still passed through.)

### WR-02: the harness can abort silently (no verdict line) on a host without IPv6 sockets

**File:** `scripts/test-sandbox-run.sh:123-124` (and `:138-141`, `:155-158`, `:173-174`, `:185-186`)
**Issue:** every capture is a bare assignment, e.g. `OUT=$(env … python3 "$WORK/connect.py" …)`. Under
`set -eu` a failing command substitution aborts the script — I confirmed `sh -c 'set -e; OUT=$(sh -c
"exit 3"); echo x'` exits 3 without reaching `echo`. The connector only catches `OSError` from
`connect()`; its socket creation (`connect.py:103`, `:106`) is outside the `try`. On a kernel built
without IPv6 (`ipv6.disable=1`, some minimal containers) `socket(AF_INET6, …)` raises
`EAFNOSUPPORT`, python exits 1, and the harness dies at assertion 3 with **no `FAIL` line, no
`FAILED: n of m` summary and no diagnostic** — just rc=1. The same pattern hides any harness-
infrastructure fault (`python3` or `bash` missing, `cc` output ok but connector unrunnable).

This also contradicts the SUMMARY's Risks entry (`01-02-SUMMARY.md:228`), which states that on a
"host with IPv6 disabled at the kernel level the harness still passes" — it does not pass, it aborts.
An aborting harness is fail-safe for CI, but it is not the behaviour the report claims, and a
maintainer will not know which assertion died.
**Fix:** capture the status the way assertion 6 already does, and let the case report a failure:

```sh
OUT=$(env … python3 "$WORK/connect.py" inet6 ::1 2>&1) && RC=0 || RC=$?
if [ "$RC" -ne 0 ]; then
    fail "ipv6-loopback: ::1 and ::ffff:127.0.0.1 permitted" "connector rc=$RC: $(flat "$OUT")"
elif has "$OUT" "sandbox-run: BLOCKED connect"; then
    fail "…" "$(flat "$OUT")"
else
    pass "…"
fi
```

Applied to assertions 1, 3, 4 and 5 (`assert_contains` can take the rc as a third argument). Then
correct the Risks bullet: on a v6-less host the harness reports a FAIL with the connector error, or
(if you prefer) skips with an explicit `SKIP:` line — but it must not vanish.

### WR-03: two shipped docs still state the behaviour this plan removed, and the deferral record misses them

**File:** `TESTING.md:78`, `KNOWN-ISSUES.md:91-95`; deferral gap in `.planning/phases/01-truth-baseline/deferred-items.md:30-35`
**Issue:** Both user-facing docs now contradict the code:

- `TESTING.md:78` — "(IPv6 refusals print as `non-ip` — cosmetic gap …)"
- `KNOWN-ISSUES.md:91-95` — "It has a cosmetic gap: its shim formats only AF_INET, so IPv6 refusals
  print as `non-ip`, and the provider's IPv6 NAT64 attempt is refused before the client falls back to
  the allowlisted IPv4 address."

`AGENTS.md` §8 explicitly requires "Update `KNOWN-ISSUES.md` when status changes", and the plan's own
premise is "the coupled prose updates that would otherwise contradict the code". `deferred-items.md`
records only `.planning/codebase/CONCERNS.md`, so the two more visible stale docs are not routed
anywhere and will be read by the next operator as current. `KNOWN-ISSUES.md` is also where the
NAT64/IPv6 fallback story lives — after this change the provider's IPv6 address *is* allowlisted, so
the fallback sentence is actively wrong.
**Fix:** edit the two bullets to the new truth (both families formatted; the provider's IPv6/NAT64
address is allowlisted, so the IPv6 path is now the primary one), or add them to `deferred-items.md`
with an owner. The first is a two-line edit and keeps the repo's own status document honest; prefer
it over deferral. (`SECURITY.md:50-55` and `README.md:334` remain accurate — no change needed.)

### WR-04: `scripts/check-egress.sh --strict` is red on the delivered tree

**File:** `scripts/check-egress.sh` (HARD scan); delivered tree at `0bf46c84` + this working tree
**Issue:** I re-ran the gate: rc=**1**, with exactly the three HARD hits the SUMMARY records
(`xai-grok-shell-base/src/util/mod.rs:422-423` `api.x.ai` / `grok.com`, `xai-grok-shell/src/agent/
config_tests.rs:2768` `gs://bucket`). The plan's `<verification>` bullet "`scripts/check-egress.sh
--strict` still exits 0" is therefore **NOT MET**. The attribution is correct and honest — all three
are in committed files that `git status` does not touch, and the gate scans only `crates prod
third_party` (`check-egress.sh:37-41`), so this plan cannot have caused them — but the deliverable is
the tree, and the tree fails a gate the project treats as binary (`AGENTS.md` §0: "HARD OK"). This is
not a defect in the 01-02 diff; it is a decision someone must take before the phase gate closes.
**Fix:** none in this plan (respecting `files_modified`). Keep item 1 in `deferred-items.md`, give it
an owner and a deadline, and make it a phase-gate entry condition: either teach the HARD scan to
ignore rejected-string fixtures in `*_tests.rs`/`#[cfg(test)]` modules, or construct those literals at
runtime. Do not close the phase with a red HARD gate unowned.

## Info

### IN-01: the debug line reports the resolver's answer, not the stored entry

**File:** `scripts/egress_guard.c:79-80`, `:84-96` vs `:66-72`
**Issue:** `store()` normalises a v4-mapped answer to `AF_INET`, but `debug_entry(ai)` prints the
original `addrinfo`. With `EGRESS_ALLOW_HOST=::ffff:203.0.113.9 EGRESS_ALLOW_DEBUG=1` I get
`sandbox-run: allow family=inet6 addr=::ffff:203.0.113.9 port=443` while the table holds a 4-byte
`AF_INET` entry. The header comment says "one line per resolved allowlist entry", so the family in
the line is not the family in the allowlist.
**Fix:** either print the normalised form (`family=inet addr=203.0.113.9` when mapped) or reword the
comment to "one line per resolved answer". Cosmetic, but this line is what an operator uses to
believe the allowlist.

### IN-02: misleading comment on the v4-mapped path

**File:** `scripts/egress_guard.c:134-135`
**Issue:** "`v4-mapped addresses never reach here; connect() normalises them to the v4 path.`" The
kernel does not normalise these on `connect`; this interposer does (`:170-173`). A future reader could
reason from the kernel and delete the branch.
**Fix:** "…; the `connect` interposer below rewrites them to the v4 path before calling `v6_allowed`."

### IN-03: no assertion that a *different* IPv6 address is refused while an IPv6 entry is allowlisted

**File:** `scripts/test-sandbox-run.sh:155-167`
**Issue:** Assertion 1 refuses `2001:db8::1` while the allowlist is `localhost`; assertion 4 allows
`2001:db8::2` while the allowlist is `2001:db8::2`. Nothing refuses `2001:db8::3` *in the same
process* that has `2001:db8::2` allowlisted, so a matching bug that compares a prefix (e.g.
`memcmp(…, 4)`) instead of all 16 bytes would still be green. That is precisely the T-01-05 class.
**Fix:** extend assertion 4 with a second connector in the same `ALLOW6` capture:
`python3 connect.py inet6 2001:db8::3` and assert the output *does* contain
`BLOCKED connect to 2001:db8::3`. It stays one PASS/FAIL and keeps `ALL PASS (6 assertions)`.

### IN-04: assertion 6 never proves what the real script passes to the shim

**File:** `scripts/test-sandbox-run.sh:185-192`
**Issue:** The end-to-end case checks only the startup echo, which is built from `$HOST` in the
script. A regression that stopped exporting `EGRESS_ALLOW_HOST` (or mangled it) would leave all six
assertions green, because 1-5 set the variable directly.
**Fix:** run `DEEPSEEK_BUILD_EGRESS_HOST=2001:db8::2 EGRESS_ALLOW_DEBUG=1 BIN=/bin/echo bash
scripts/sandbox-run.sh --version` and assert the output contains
`family=inet6 addr=2001:db8::2 port=443` — still offline, still one verdict.

### IN-05: `MAX_ALLOW` is now shared between A and AAAA answers, silently

**File:** `scripts/egress_guard.c:48`, `:111`
**Issue:** Resolution is dual-stack but the table still holds 16 entries; once A+AAAA answers exceed
that, the rest are dropped silently. Fail-closed (a dropped provider address becomes unreachable, not
reachable), so this is not a security issue, and `api.deepseek.com` returns a handful of addresses.
Still, the cap's new meaning is undocumented in a file whose header documents everything else.
**Fix:** one comment line at `:48` ("16 total across both families"), or log a debug line when the
cap truncates, so an operator seeing "provider unreachable" has a clue.

### IN-06: the allowlist is published without synchronisation

**File:** `scripts/egress_guard.c:56-57`, `:99-122`
**Issue:** `allowed[]`/`allowed_n` are plain statics written by the constructor and read by
`connect`. Safe for the intended use (constructor runs under the loader lock before `main`), but if
the shim is ever `dlopen`ed into a multithreaded process, a concurrent `connect` can observe a
partially populated table. The old shim had the same shape, so this is not a regression.
**Fix:** either document "LD_PRELOAD at exec only; not supported via `dlopen`", or store `allowed_n`
in an `_Atomic int` and increment after the entry is fully written.

### IN-07: the harness compiles the shim without warnings enabled, and red-first is manual

**File:** `scripts/test-sandbox-run.sh:87`; `.planning/phases/01-truth-baseline/01-02-SUMMARY.md:154`
**Issue:** The only automated compile of `egress_guard.c` is `cc -shared -fPIC -O2`, so a future
warning (a real `-Wformat` bug, say) is invisible; the SUMMARY's `-Wall -Wextra` run was one-off.
Separately, the non-vacuity proof is a manual paragraph — I reproduced it, but nothing in the repo
does.
**Fix:** add `-Wall -Wextra` to the harness compile (not to `sandbox-run.sh`, where warnings would
pollute every run). Optionally keep the red-first extract (`git show HEAD:scripts/sandbox-run.sh |
sed -n "/<<'C'/,/^C$/p"`) as a commented recipe in the harness header; it is four lines and settles
the "does the harness measure the fix?" question for the next reviewer.

### IN-08: `EGRESS_ALLOW_DEBUG=0` enables debug output

**File:** `scripts/egress_guard.c:82-83`
**Issue:** The test is "non-empty", so `EGRESS_ALLOW_DEBUG=0` prints. Documented as `=1`.
**Fix:** `if (!env || strcmp(env, "1") != 0) return;` — or reword the docs to "any non-empty value".

## Verification (reproduced)

```
sh -n scripts/test-sandbox-run.sh                                  -> rc=0
bash -n scripts/sandbox-run.sh                                     -> rc=0
bash -n scripts/egress-check.sh                                    -> rc=0
shellcheck --shell=sh scripts/sandbox-run.sh scripts/test-sandbox-run.sh -> rc=0, no output
sh scripts/test-sandbox-run.sh                                     -> ALL PASS (6 assertions), rc=0, 2.2 s
cc -shared -fPIC -O2 -Wall -Wextra -o /tmp/x.so scripts/egress_guard.c -ldl -> rc=0, no warnings
grep -n 'print as' scripts/egress-check.sh                         -> rc=1 (no match)
git diff HEAD -- scripts/egress-check.sh | grep -E '^[-+](BLOCKED|if \[|exit|ok|bad)' -> no matches
scripts/check-egress.sh --strict                                   -> rc=1, 3 HARD hits (all pre-existing) [WR-04]
```

Ad-hoc behavioural matrix, new shim (`EGRESS_ALLOW_HOST` → destination → observed):

| allow host | connect | observed |
|---|---|---|
| `localhost` | `2001:db8::1` | `BLOCKED connect to 2001:db8::1` |
| `localhost` | `::ffff:8.8.8.8` | `BLOCKED connect to ::ffff:8.8.8.8` |
| `localhost` | `192.0.2.1` | `BLOCKED connect to 192.0.2.1` |
| `localhost` | `AF_NETLINK` | `BLOCKED connect to non-ip` |
| `2001:db8::2` | `2001:db8::2` | allowed (`allow family=inet6 addr=2001:db8::2 port=443`), no BLOCKED |
| `2001:db8::2` | `2001:db8::3` | `BLOCKED connect to 2001:db8::3` |
| `203.0.113.9` | `::ffff:203.0.113.9` | allowed (normalised to the v4 entry) |
| `::ffff:203.0.113.9` | `203.0.113.9` | allowed (normalised on store) |
| `localhost` | `::1`, `127.0.0.1` | allowed, no BLOCKED |

Old shim (rebuilt from `git show HEAD:scripts/sandbox-run.sh`): `2001:db8::1` → `non-ip`;
`EGRESS_ALLOW_HOST=2001:db8::2` → no `family=inet6`, destination refused; `::ffff:127.0.0.1` →
refused; `203.0.113.9` → named correctly; `::1` → allowed.

## Verdict

**PASS_WITH_NOTES** — SBOX-02's second half is implemented correctly, with a proof that is offline,
deterministic, red-first and honest. The shim resolves both families, matches by family-tagged raw
bytes with bidirectional v4-mapped normalisation, names every refused destination, and neither widens
nor narrows any destination the old shim allowed beyond the two v4-mapped allowances the plan
mandates. `non-ip` is now reachable only for a genuinely non-IP family. No new egress, dependency or
secret. No blocker.

Ship after WR-01 (three lines: refuse when `dlsym` fails) and WR-02 (stop the harness aborting
silently — the fix is mechanical and makes the report's own risk note true). WR-03 should be done in
this change rather than deferred: `KNOWN-ISSUES.md` and `TESTING.md` now describe the bug the plan
fixed, and `AGENTS.md` §8 requires that file to track status. WR-04 is pre-existing and routed, but
it must be owned before the phase gate closes, because the phase's verification bullet says the HARD
gate exits 0 and it does not. IN-03 and IN-04 are cheap and would close the two real gaps in the
proof.

---

_Reviewed: 2026-11-21T19:36:40Z_
_Reviewer: gsd-code-reviewer (subagent)_
_Depth: deep_
