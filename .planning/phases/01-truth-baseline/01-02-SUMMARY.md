---
phase: 01-truth-baseline
plan: "02"
subsystem: infra
tags: [sandbox-run, ld-preload, ipv6, af-unspec, getaddrinfo, v4-mapped, egress, shell-harness]

# Dependency graph
requires:
  - phase: 01-truth-baseline
    provides: "01-01 (ordering only; no artifact of 01-01 is used by this plan)"
provides:
  - "scripts/egress_guard.c — the checked-in connect() shim that sandbox-run.sh compiles and preloads"
  - "AF_UNSPEC allow-host resolution with a family-tagged, byte-compared allowlist"
  - "Blocked reports that name an IPv6 destination instead of printing `non-ip`"
  - "scripts/test-sandbox-run.sh — six offline assertions over the real shim and the real script"
  - "egress-check.sh prose and scripts/README.md that match the fixed behaviour"
affects: [01-truth-baseline plan 01-05 (runtime egress evidence), any run of scripts/egress-check.sh]

actuals:
  tokens: 5353        # chars/4 over the realized diff (21,410 chars across 5 files)
  tasks: 2
  commits: 0          # intentional: the dispatch forbids committing; changes left in the working tree
  plan_head_before: 0bf46c84d4b55d87c8a4addf014f12613d2fe2d9
  plan_head_after: 0bf46c84d4b55d87c8a4addf014f12613d2fe2d9

tech-stack:
  added: []
  patterns:
    - "Family-tagged raw-byte allowlist: { int family; unsigned char bytes[16] } compared with memcmp, never formatted strings"
    - "v4-mapped IPv6 normalised to its 4-byte IPv4 form on both store and match, so ::ffff:a.b.c.d and a.b.c.d are one entry"
    - "EGRESS_ALLOW_DEBUG=1 emits one `family=inet|inet6 addr=… port=…` line per resolved entry — the offline proof that AAAA resolution works"

key-files:
  created:
    - scripts/egress_guard.c
    - scripts/test-sandbox-run.sh
  modified:
    - scripts/sandbox-run.sh
    - scripts/egress-check.sh
    - scripts/README.md

key-decisions:
  - "The shim is extracted to scripts/egress_guard.c rather than sed-ed out of a heredoc, mirroring scripts/egress_log.c; sandbox-run.sh now compiles the tracked source."
  - "The allowlist stores 16-tagged raw address bytes; matching is family plus memcmp (4 bytes for AF_INET, 16 for AF_INET6), so no formatting difference can widen or narrow a match."
  - "A v4 allowlist entry matches a connect to its v4-mapped spelling, and a v4-mapped resolver answer is stored as the 4-byte form, making ::ffff:1.2.3.4 and 1.2.3.4 one entry in both directions."
  - "`non-ip` now only means 'a family that is neither AF_INET nor AF_INET6' (verified against AF_NETLINK); every IP refusal names the address in its own notation."
  - "The harness verdicts are six cases, one PASS/FAIL line each, so the summary line reads exactly ALL PASS (6 assertions); checks that the plan grouped into one case (loopback + v4-mapped loopback; IPv6 allowlist + v4-mapped allowlist) are composite conditions inside that case."

patterns-established:
  - "Script harnesses drive the *tracked* shim and the *real* script (BIN=/bin/echo), never a private copy, so the proof cannot drift from what ships."
  - "Offline egress proofs use numeric literals (2001:db8::2, 203.0.113.9) or localhost as the allow host, so no assertion performs a DNS lookup."

requirements-completed: [SBOX-02]

coverage:
  - id: D1
    description: "sandbox-run.sh resolves the allow host over both families and refuses an IPv6 destination while naming that destination, never `non-ip`"
    requirement: SBOX-02
    verification:
      - kind: integration
        ref: "sh scripts/test-sandbox-run.sh#assertions 1-2, 5 (refusal naming, both families) "
        status: pass
      - kind: manual
        ref: "EGRESS_ALLOW_HOST=localhost LD_PRELOAD=… python3 -c \"connect(('2001:db8::1',443,0,0))\""
        status: pass
    human_judgment: false
  - id: D2
    description: "An allowlisted IPv6 address and IPv6 loopback are permitted; IPv4 loopback and the IPv4 allowlist are unchanged"
    requirement: SBOX-02
    verification:
      - kind: integration
        ref: "sh scripts/test-sandbox-run.sh#assertions 3-4"
        status: pass
    human_judgment: false
  - id: D3
    description: "scripts/egress-check.sh no longer documents the fixed `non-ip` behaviour, and scripts/README.md lists the new harness"
    requirement: SBOX-02
    verification:
      - kind: manual
        ref: "! grep -n 'print as' scripts/egress-check.sh; git diff scripts/README.md"
        status: pass
    human_judgment: false

duration: ~30min (approximate; no harness interruption)
completed: 2026-11-21
status: complete
---

# Phase 01 Plan 02: `sandbox-run.sh` IPv6 truth Summary

**`sandbox-run.sh` now resolves the provider over A and AAAA, allowlists both families by raw bytes, permits an allowlisted or loopback IPv6 destination, and names every refused destination in its own notation — proven offline by a six-assertion harness that compiles the tracked shim and runs the real script.**

## Performance

- **Duration:** ~30 min (approximate; single harness run, no interruption)
- **Completed:** 2026-11-21T19:31Z
- **Tasks:** 2/2
- **Files changed:** 5 (all inside `files_modified`: 2 new, 3 modified)
- **Base HEAD:** `0bf46c84d4b55d87c8a4addf014f12613d2fe2d9`
- **Commits:** none — the dispatch forbids committing; everything is left in the working tree for review

## Accomplishments

- Extracted the inline heredoc shim into `scripts/egress_guard.c` (186 lines, mirrors `scripts/egress_log.c`'s structure: `_GNU_SOURCE`, the same include set, `dlsym(RTLD_NEXT, "connect")` cached in a constructor). It still refuses — `errno = ECONNREFUSED; return -1;` — unlike the logger.
- Replaced `char allowed[16][16]` (IPv4 strings) with a family-tagged byte table:

  ```c
  struct allow_entry { int family; unsigned char bytes[16]; };
  static struct allow_entry allowed[MAX_ALLOW];
  ```

  and matches with `family + memcmp` (4 bytes for `AF_INET`, 16 for `AF_INET6`). `T-01-05` mitigated: a formatted-string difference can no longer widen or narrow a match.
- Resolution is `getaddrinfo(host, "443", {AF_UNSPEC, SOCK_STREAM})` — no `AI_ADDRCONFIG`, no `AI_NUMERICHOST`. A resolution failure leaves an empty allowlist (fail closed).
- v4-mapped addresses are normalised on both sides: an `AF_INET6` answer of `::ffff:a.b.c.d` is stored as the 4-byte IPv4 address, and a connect to a v4-mapped address is compared as those 4 bytes against the IPv4 loopback rules and the IPv4 allowlist. `::ffff:1.2.3.4` and `1.2.3.4` are one entry.
- Blocked reports format per family with `inet_ntop(AF_INET6, …, INET6_ADDRSTRLEN)`; `non-ip` survives only for a family that is neither (`AF_NETLINK` verified). Prefix `sandbox-run: BLOCKED connect to ` is unchanged, so `egress-check.sh:542`'s parser is untouched.
- `EGRESS_ALLOW_DEBUG=1` prints one startup line per resolved entry, reusing `egress_log.c`'s field vocabulary:

  ```
  sandbox-run: allow family=inet6 addr=::1 port=443
  sandbox-run: allow family=inet addr=127.0.0.1 port=443
  ```

- `scripts/test-sandbox-run.sh` (new, POSIX `sh`, `test-install.sh` skeleton) compiles `scripts/egress_guard.c` into a `mktemp -d` and drives `python3` connectors through `LD_PRELOAD`.

## The six assertions (actual run, this host, 2026-11-21)

| # | Case | What it proves | Result |
|---|---|---|---|
| 1 | `EGRESS_ALLOW_HOST=localhost`, connect `2001:db8::1:443` | the blocked line is `sandbox-run: BLOCKED connect to 2001:db8::1` | PASS |
| 2 | same capture | the output contains no `non-ip` | PASS |
| 3 | `::1` and `::ffff:127.0.0.1` | IPv6 loopback permitted (plain and v4-mapped), no BLOCKED line | PASS |
| 4 | `EGRESS_ALLOW_HOST=2001:db8::2` + `EGRESS_ALLOW_DEBUG=1`, connect `2001:db8::2:443`; and `EGRESS_ALLOW_HOST=203.0.113.9`, connect `::ffff:203.0.113.9:443` | the resolver returns `family=inet6 addr=2001:db8::2 port=443`, the IPv6 entry is permitted, and a v4 entry matches its v4-mapped spelling | PASS |
| 5 | `EGRESS_ALLOW_HOST=localhost`, connect `203.0.113.9:443` | IPv4 refusal path unchanged: line names `203.0.113.9` | PASS |
| 6 | `BIN=/bin/echo DEEPSEEK_BUILD_EGRESS_HOST=localhost bash scripts/sandbox-run.sh --version` | the real script compiles the shim, prints `sandbox-run: allowing only localhost:443 + loopback; …` and exits 0 | PASS |

```
$ sh scripts/test-sandbox-run.sh
=== egress_guard.c harness ===
PASS: ipv6-refused: blocked line names 2001:db8::1
PASS: ipv6-refused: no 'non-ip' placeholder
PASS: ipv6-loopback: ::1 and ::ffff:127.0.0.1 permitted
PASS: ipv6-allowlist: resolves IPv6, permits it, and matches the v4-mapped spelling
PASS: ipv4-refused: blocked line names 203.0.113.9
PASS: script: sandbox-run.sh runs end to end (BIN=/bin/echo)

ALL PASS (6 assertions)
rc=0   (wall clock 2.3 s)
```

Every allow host is a numeric literal or `localhost`; no assertion resolves `api.deepseek.com`, so the harness is offline. A connector's own `ECONNREFUSED` / `ENETUNREACH` (printed as `connect-failed: …` by the helper) is the expected outcome, not a verdict — only the shim's BLOCKED lines are.

Non-IP control (manual, not counted as an assertion): an `AF_NETLINK` connect prints exactly `sandbox-run: BLOCKED connect to non-ip`, which is the only surviving use of the literal.

Non-vacuity control (manual): the pre-change shim, rebuilt from `git show HEAD:scripts/sandbox-run.sh`'s heredoc, prints `sandbox-run: BLOCKED connect to non-ip` for `2001:db8::1` and no `family=inet6` allow line for `2001:db8::2` — i.e. assertions 1, 2 and 4 fail against the old code, so the harness measures the fix rather than passing trivially.

## `scripts/egress-check.sh` prose diff (section 6 notes)

```diff
-note "note: sandbox-run.sh formats only AF_INET destinations; its AF_INET6/other"
-note "      refusals print as 'non-ip'. The provider is reached over its IPv6"
-note "      NAT64 address (see the allow-log table), which sandbox-run.sh does not"
-note "      allowlist, so those IPv6 attempts are refused before falling back to"
-note "      the allowlisted IPv4 provider address. The web_fetch host is refused"
-note "      as expected, so the fetch fails while the session still completes."
+note "note: sandbox-run.sh formats both AF_INET and AF_INET6 destinations, so a"
+note "      refused address is named in its own family's notation and 'non-ip' only"
+note "      appears for a non-IP address family. Its allowlist is family-tagged and"
+note "      built from getaddrinfo(AF_UNSPEC), so it carries the IPv6 addresses the"
+note "      resolver returns for the provider as well as the IPv4 ones. The"
+note "      web_fetch host is not on that allowlist and is refused as expected, so"
+note "      the fetch fails while the session still completes."
```

No gate, count line, `note`/`hdr`/`ok`/`bad` helper, or exit code was touched.

## Verification (commands and actual results, this host, 2026-11-21)

```
sh -n scripts/test-sandbox-run.sh                  -> rc=0
bash -n scripts/sandbox-run.sh                     -> rc=0
bash -n scripts/egress-check.sh                    -> rc=0   (edited prose, syntax re-checked)
shellcheck --shell=sh scripts/sandbox-run.sh scripts/test-sandbox-run.sh
                                                   -> rc=0, no output
sh scripts/test-sandbox-run.sh                     -> ALL PASS (6 assertions), rc=0
cc -shared -fPIC -O2 -Wall -Wextra -o /tmp/… scripts/egress_guard.c -ldl
                                                   -> rc=0, no warnings
grep -n 'print as' scripts/egress-check.sh         -> rc=1 (no match), as required
scripts/check-egress.sh --strict                   -> rc=1, 3 HARD hits, ALL PRE-EXISTING
                                                      (see Deviations; gate scans crates/ and
                                                       this plan touches only scripts/)
```

`git diff` review: the `EGRESS_ALLOW_HOST` env contract, the `sandbox-run: allowing only …` startup line, the `cd`/`mktemp`/`trap`, and the `exec env … LD_PRELOAD=… "$BIN" "$@"` line are byte-identical. No dependency, lockfile, or `Cargo.toml` change; no new outbound destination (the shim only refuses).

## Decisions Made

- **Extract rather than sed.** A checked-in `scripts/egress_guard.c` mirrors `egress_log.c`, keeps `sandbox-run.sh` at 36 lines, and lets the harness compile the exact source that ships (mitigates `T-01-07`: no private copy of the shim).
- **One allowlist table, two normalisations.** Storage normalises a mapped v6 answer to v4 bytes; matching normalises a mapped v6 destination to v4 bytes. Both directions, one comparison path.
- **Composite cases to keep six verdicts.** The plan specifies six assertions and the summary line must read `ALL PASS (6 assertions)`, so assertion 3 also covers `::ffff:127.0.0.1` and assertion 4 also covers the v4-mapped match; each case still emits exactly one PASS/FAIL.
- **`EGRESS_ALLOW_DEBUG` is opt-in and stderr-only.** Nothing is printed on the default path, so the startup output of a normal run is unchanged.

## Deviations from Plan

### Auto-fixed / plan-consistent adjustments

**1. [Rule 2 - Missing critical safeguard] A resolution failure must not become "allow everything"**
- **Found during:** Task 1
- **Issue:** `getaddrinfo(...) != 0` returns from the constructor with `allowed_n == 0`; that alone is safe, but the old code's shape (`if (getaddrinfo(...) == 0) { … }`) left no statement of intent.
- **Fix:** Early `return` on failure plus a comment stating fail-closed; no behavioural change versus the old shim, now explicit and documented.

**2. [Plan detail] Assertions 3 and 4 carry the v4-mapped checks**
- **Found during:** Task 2
- **Issue:** `must_haves` requires "`::ffff:1.2.3.4` and `1.2.3.4` are the same allowlist entry", which the plan's six named cases do not spell out; adding a seventh assertion would break the `ALL PASS (6 assertions)` contract.
- **Fix:** assertion 3 additionally runs a `::ffff:127.0.0.1` connector (mapped loopback), and assertion 4 additionally runs `EGRESS_ALLOW_HOST=203.0.113.9` + `::ffff:203.0.113.9` (mapped allowlist hit). Both are part of the case's single PASS/FAIL.
- **Files modified:** `scripts/test-sandbox-run.sh`
- **Verification:** the harness is green; the same cases would fail against a stored-string allowlist.

**3. [Environment] `scripts/check-egress.sh --strict` is red on the base tree**
- **Found during:** final verification
- **Issue:** 3 HARD hits in committed test files (`config_tests.rs:2768` `gs://bucket`, `xai-grok-shell-base/src/util/mod.rs:422-423` `api.x.ai` / `grok.com`), present in `git show HEAD:` and introduced upstream of this plan. The gate scans only `crates prod third_party`; every 01-02 change is under `scripts/`.
- **Fix:** none — outside `files_modified` and outside this plan's scope. Recorded in `.planning/phases/01-truth-baseline/deferred-items.md` with owner (plan 01-05 or a follow-up).
- **Impact on plan verification:** the plan's "`check-egress.sh --strict` still exits 0" expectation is **NOT MET** on this tree, for reasons independent of this change. There is no evidence that 01-02 introduced any hit.

No dependency changes. No new egress destination. No `#[ignore]`. Nothing committed.

## Risks

- **Address-family availability.** On a host with IPv6 disabled at the kernel level the harness still passes (the `::1` / mapped connectors simply fail to route, and no BLOCKED line is emitted), but assertion 4's "permitted" leg then proves only that the shim did not block. The IPv6 refusal and formatting legs remain exact.
- **The blocked line carries the address, not the port.** That matches the existing parser and the plan's `<destination>` contract; an operator distinguishing two connections to one address must read the port elsewhere.
- **`0.0.0.0/8` and `127.0.0.0/8` remain blanket allowances**, unchanged from the previous shim. A connect to any `0.x` or `127.x` address is permitted; that is deliberate (visibility into every IPv4 loopback variant) and out of this plan's scope.
- **Scope-list clients.** A process that resolves the allow host itself and then connects to a family the resolver returned is fine, but an address that arrives via a route the shim never sees (for example a socket handed in over `SCM_RIGHTS`) is not covered — the same limitation the shim had before.
- **`scripts/check-egress.sh --strict` is red on the base tree** (deferred item 1). Any later plan that requires a green HARD gate must resolve it first.
- **No commit was made**, per the dispatch; the working tree carries the five files (`M` ×3, `??` ×2) for review.

## Self-Check: PASSED

- `scripts/egress_guard.c` — created (186 lines; family-tagged allowlist, AF_UNSPEC resolution, per-family blocked report, `EGRESS_ALLOW_DEBUG`).
- `scripts/test-sandbox-run.sh` — created (205 lines; six assertions; compiles the tracked shim; `ALL PASS (6 assertions)` observed).
- `scripts/sandbox-run.sh` — modified (heredoc removed, compiles `scripts/egress_guard.c`; header/contract/startup/`exec` lines unchanged).
- `scripts/egress-check.sh` — modified (section-6 notes only).
- `scripts/README.md` — modified (`sandbox-run.sh` bullet corrected, `test-sandbox-run.sh` bullet added).
- `sh scripts/test-sandbox-run.sh` → `ALL PASS (6 assertions)`, rc=0.
- Commit check: intentionally N/A — committing was forbidden; `git status --short` shows exactly `M scripts/README.md`, `M scripts/egress-check.sh`, `M scripts/sandbox-run.sh`, `?? scripts/egress_guard.c`, `?? scripts/test-sandbox-run.sh`.

## Review follow-up (orchestrator, 2026-11-21)

- **WR-01 resolved:** `egress_guard.c` resolves `connect` through a `call_real` helper that refuses with
  `ECONNREFUSED` and a blocked line when `dlsym` returns NULL — no call through a NULL pointer.
- **WR-02 resolved:** the connector exits 77 and prints `socket-unavailable` when the kernel has no
  `AF_INET6`; the harness captures with `|| true`, prints `SKIP:` for the four IPv6 assertions, and keeps
  assertions 5–6 running. Summary line reports skips.
- **WR-03 resolved:** `KNOWN-ISSUES.md` and `TESTING.md` now describe the fixed IPv6 reporting instead of
  the `non-ip` gap.
- **WR-04 owner:** `scripts/check-egress.sh --strict` is red at base on three committed test fixtures
  (`api.x.ai`, `grok.com`, `gs://bucket`). It is independent of this plan and is owned by the Phase 5
  pre-freeze checklist; recorded in STATE.md blockers.
- Re-run: `sh scripts/test-sandbox-run.sh` → `ALL PASS (6 assertions)`; `shellcheck --shell=sh` clean;
  guard compiles with `-Wall -Wextra`.

## Commit

The frontmatter records `commits: 0` because the dispatch forbids committing;
the reviewed change landed afterwards as `882ad0de`. This note is the post-phase
record, not a change to the plan's reported state.

