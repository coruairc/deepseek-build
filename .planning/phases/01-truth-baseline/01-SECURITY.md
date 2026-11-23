---
phase: "01"
slug: "truth-baseline"
status: verified
# threats_open = count of OPEN threats at or above workflow.security_block_on severity (the blocking gate)
threats_open: 0
asvs_level: 1
created: "2026-11-23"
---

# Phase 01 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| sandbox -> host filesystem | bwrap deny materialization must stay fail-closed while skipping only provably non-traversable parents | runtime socket paths (no user data) |
| egress guard -> network | LD_PRELOAD connect() guard allows only the configured provider host | destination IP/port (no payload) |
| gate -> human judgement | a green gate is taken as proof the suite ran; the gate must not be able to skip silently | test counts, exit codes |
| test evidence -> committed repository | captured terminal output can carry secrets, absolute paths and hostnames | normalized logs, casts, reports |
| evidence helper -> report.json | the report is the artifact a reviewer trusts instead of re-running | real exit statuses, sha256s |
| cost surface -> user | the rendered figure must never imply a bill | estimated USD, user-supplied rates |
| catalog -> session model | the default the docs promise must be the default the binary resolves | model identifiers |

---

## Threat Register

| Threat ID | Category | Component | Severity | Disposition | Mitigation | Status |
|-----------|----------|-----------|----------|-------------|------------|--------|
| T-01-01 | Tampering | `materialize_runtime_socket_deny_paths_from` | medium | mitigate | `lstat("parent/.")` traversability probe; skip only when provably non-traversable (`runtime_sockets.rs:131,173`); symlink/InvalidInput/dedupe arms unchanged | closed |
| T-01-02 | Elevation of privilege | skipping a maskable socket | medium | mitigate | EACCES under a traversable parent stays fatal; `endpoint_error_classification_table` pins classification; NotFound never conflated with PermissionDenied | closed |
| T-01-03 | Repudiation | making the e2e green by weakening it | high | mitigate | devbox scenario retargeted to a devbox-extending profile with a real deny obligation; the one `#[ignore]` is the pre-existing subprocess entry helper (run via `--ignored --exact`), no test ignored or predicate widened by 01-01 (diff of `0bf46c84` verified) | closed |
| T-01-04 | Information disclosure | skip log content | low | accept | one `tracing::debug!` with the candidate path only; no env, no user data | closed |
| T-01-05 | Tampering | IPv6 allowlist matching | medium | mitigate | allowlist stores raw address bytes tagged by family, never formatted strings; v4-mapped normalized to 4-byte form (`scripts/egress_guard.c:23`) | closed |
| T-01-06 | Information disclosure | blocked-destination report | low | accept | line names an IP the process already tried to reach; no payload, no credentials | closed |
| T-01-07 | Repudiation | harness proving a private copy of the shim | medium | mitigate | harness compiles tracked `scripts/egress_guard.c`; assertion 6 runs the real `scripts/sandbox-run.sh` end to end (`BIN=/bin/echo`, localhost) | closed |
| T-01-08 | Integrity | `segments.rs` cost arm | low | mitigate | unconditional `~$… est.` label (`segments.rs:141-153`); guide documents marker and floor | closed |
| T-01-09 | Integrity | `HeadlessEmitter` / `MessagesReducer` | medium | mitigate | both surfaces read `StatusLineConfig::pricing()` (`config.rs:200`); `tui_and_headless_report_the_same_cost_for_one_override` fails on divergence | closed |
| T-01-10 | Information disclosure | pricing override values | low | accept | rates are user-supplied config; nothing new logged, no key material | closed |
| T-01-11 | Integrity | `default_models.json` vs `resolve_default_model` | high | mitigate | `first_or_fallback` reads the JSON-declared field (`resolution.rs:94-150`); present/hidden/absent tests cover divergence | closed |
| T-01-12 | Integrity | model picker / selection surface | medium | mitigate | JSON array order untouched; hidden/disabled cases fall back to first visible entry | closed |
| T-01-13 | Repudiation | `scripts/test-gate.sh` completeness | high | mitigate | counted assertion `results == binaries + doctests == expected`; log tail never the verdict; run #3 confirmed 308/219/89 exit 0 | closed |
| T-01-14 | Information disclosure | committed evidence | high | mitigate | `key-leak-check.sh --scan docs/evidence` is the gate's last step (additive mode, file names only); `evidence.py` normalizes `<REPO>/<HOME>/<TMP>`, host as system/arch only; scan green | closed |
| T-01-15 | Repudiation | flake "fixed" by weakening a test | medium | mitigate | task-2 diff adds `#[serial]` and widens waits only; no assertion direction changed or dropped; `restore_fetch_tests.rs` untouched; 5x repetition green (1694 passed) | closed |
| T-01-16 | Tampering | `report.json` overstating a result | medium | mitigate | `evidence.py step` records real exit statuses; `finish` refuses `--result ok` over a failed step (`evidence.py:199`) | closed |
| T-01-17 | Repudiation | expected count refreshed silently | medium | mitigate | `--refresh-expected` full-run only, writes only when green, stamps commit/date/three numbers; expected file provenance run #2 at `882ad0de`, confirmed by run #3 | closed |
| T-01-SC | Tampering | npm/pip/cargo installs | high | mitigate | no dependency or tool installed in any plan; `git diff 882ad0de..HEAD` + working tree show no Cargo.toml/lockfile/deny.toml change; helper is stdlib-only Python | closed |

*Status: open · closed · open — below high threshold (non-blocking)*
*Severity: critical > high > medium > low — only open threats at or above workflow.security_block_on count toward threats_open*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-01-01 | T-01-04 | skip log carries only the candidate path at debug level; no env or user data | author (plan approval) | 2026-11-21 |
| AR-01-02 | T-01-06 | blocked-destination line names an IP already attempted; no payload or credentials | author (plan approval) | 2026-11-21 |
| AR-01-03 | T-01-10 | pricing rates are user-supplied config; nothing new logged | author (plan approval) | 2026-11-21 |

*Accepted risks do not resurface in future audit runs.*

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-11-23 | 18 | 18 | 0 | orchestrator L1 grep-depth verification (ASVS L1 short-circuit: register authored at plan time, threats_open 0) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-11-23
