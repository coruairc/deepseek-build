# HANDOFF — 2026-11-23 evening (remote planning session)

Pick-up notes for continuing the `deepseek-build` v0.1.0 milestone on the build PC.
Supersedes `HANDOFF-2026-11-23.md` for *planning state only* — that file still holds
the Phase-01 close-out detail. Start with [`AGENTS.md`](AGENTS.md) §0 and
`.planning/STATE.md`; this file records what the remote session did and exactly how
to continue.

## 1. Where this session ran, and what it could not do

- Ran on the Zotac Alpine box (`zotac`, 192.168.1.11) through **OpenCode 2**
  (`opencode v2.0.30`) with agent `build` on **`opencode-go/deepseek-v4.1-flash`** —
  all six project agents and the global default were repointed to Flash for this run.
- **No Rust toolchain on that host** (Alpine ships 1.96.1; the repo pins 1.94.0), no
  `protoc`, and only **~31 GB free** against this repo's `scripts/disk-guard.sh`
  default threshold of 60 GB. So: **planning only, zero builds, zero tests, zero cargo
  processes.** Disk stayed flat at 31 GB for the whole session.
- Consequence: **Phase 02 is planned and checker-verified but NOT executed.** Nothing
  in the milestone advanced past `status: blocked`.

## 2. State pushed

- Branch `main`, working tree clean at push time, **7 commits** on top of the previous
  push:

  ```
  47698812 docs(02): smart discuss context
  018bf648 docs(02): UI design contract for the permission prompt, mode indicator and headless denial
  f50a0976 docs(02): UI-SPEC approved and checker FLAGs folded
  1efb05ac docs(02): research + validation strategy
  50dbacc3 docs(02): revise plans to six-plan split and fix checker findings
  470c51b1 docs(02): checker residue fixes — measure the baseline properly, align verify needles
  36ee4679 docs(02): state — phase 2 planned, execution blocked on build host
  ```

- Milestone v0.1.0: Phase 01 complete (6/6 plans, 24/24 must-haves). **Phase 02
  (Loop-Item Verification) = `status: blocked`**, planned, 6 plans / 4 waves,
  planner-checker **0 blockers**.
- No tags, no releases, no `target/` on disk.

## 3. Phase 02 as planned

| Plan | Wave | Title |
|---|---|---|
| 02-01 | 1 | Live smoke on a real model-driven session |
| 02-04 | 1 | Permission prompt — warning row, three outcomes, and the headless denial copy |
| 02-02 | 2 | Runtime egress measured on the live session |
| 02-05 | 2 | Resume fidelity — model, reasoning effort, plan state, and the provenance of each |
| 02-06 | 3 | Permission mode across resume, headless plan arming, and plan mode read-only |
| 02-03 | 4 | Key-leak scan over the tree and every committed artifact |

Artefacts: `02-CONTEXT.md`, `02-DISCUSSION-LOG.md`, `02-RESEARCH.md`, `02-UI-SPEC.md`,
`02-VALIDATION.md`, `02-PATTERNS.md`, plus the six `02-0N-PLAN.md` files under
`.planning/phases/02-loop-item-verification/`.

### Three real findings the planning pass produced

1. **PERM-01 is a genuine bug, not missing evidence.** The shell prepends
   `⚠ Security warning: …` to the ACP title
   (`crates/codegen/xai-grok-workspace/src/permission/prompter.rs:837-850`), but the
   pager rebuilds the title for execute prompts and discards the ACP one
   (`crates/codegen/xai-grok-pager/src/app/acp_handler/permissions.rs:186-269`) — so on
   `rm -rf base` the warning never paints. Plan 02-04 adds the warning row and the `+1`
   chrome height.
2. **PERM-02 asymmetry is a one-constant fix.** `headless.rs:479-498` names both flags;
   the `Err(anyhow!(…))` at `headless.rs:1539` names only `--always-approve`. And
   `--permission-mode` accepts only
   `default|acceptEdits|auto|dontAsk|bypassPermissions|plan`, so the copy must say
   **`auto`** — `--permission-mode always-approve` would not parse.
3. **PERM-04 is doc-only.** `headless.rs:856-875` already sends
   `SetSessionModeRequest("plan")` and a unit test pins it; `KNOWN-ISSUES.md:209` is
   stale. Plan 02-06 corrects the ledger and explicitly forbids the tempting second
   set-mode call.

## 4. How to continue on the build PC

```sh
git pull                                  # brings the 7 commits above
rustup toolchain install 1.94.0 --component rustfmt clippy
source "$HOME/.cargo/env"
export CARGO_TARGET_DIR="$HOME/Projects/deepseek-build/target"   # one shared target dir
scripts/disk-guard.sh                    # default 60 GB; a full workspace build+tests is ~24 GB
/gsd-execute-phase 2
```

- **02-01 and 02-03 pause for the `DEEPSEEK_API_KEY`** — arm it before, and rotate it on
  platform.deepseek.com after 02-01's live run. No key material may land in any
  committed artefact (key-leak scan in 02-03 enforces this).
- Per-crate `cargo test -p <crate>` during work; one `cargo test --workspace` only at the
  final gate. A fresh workspace build is ~24 GB in ~7 min — reclaim with
  `cargo clean` when done.

## 5. What still blocks a fully autonomous run (phases 3–5)

- **Phase 3** (Pipeline Decoupling & Version) pushes a workflow change and needs CI runs.
- **Phase 4** (Distribution Proof) pushes a throwaway tag and pulls from GHCR.
- **Phase 5** (Documentation Truth & Freeze) needs a **20–30 minute human TUI session**
  (ACC-01) before the freeze marker may exist.

`/gsd-autonomous` will plan them, but it stops at the same kind of gate until those
credentials and that session are available on the machine that runs it.
