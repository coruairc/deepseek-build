---
schema_version: 1
open_count: 1
waived_count: 0
fixed_count: 0
total_count: 1
last_updated: 2026-11-25T19:57:37.846Z
---

# Broken Windows Ledger

> Cross-phase defect register. With `workflow.windows_enforce` enabled, `/gsd-ship` blocks while `open_count > 0`.
> Waive with `gsd-tools windows waive <id> "<reason>"` (reason required).
> Mark fixed with `gsd-tools windows fixed <id>`.

| id | phase | kind | file | line | description | status | reason | recorded_at | resolved_at |
|----|-------|------|------|------|-------------|--------|--------|-------------|-------------|
| 1 | 02 | deviation | KNOWN-ISSUES.md | 143 | evidence.py replaces every /tmp substring, so mktemp /tmp/tmp.* renders as <TMP><TMP>. | open |  | 2026-11-25T19:57:37.846Z |  |

````json
[
  {
    "id": 1,
    "kind": "deviation",
    "phase": "02",
    "file": "KNOWN-ISSUES.md",
    "line": 143,
    "description": "evidence.py replaces every /tmp substring, so mktemp /tmp/tmp.* renders as <TMP><TMP>.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-25T19:57:37.846Z",
    "resolved_at": null,
    "milestone": "v0.1.0"
  }
]
````
