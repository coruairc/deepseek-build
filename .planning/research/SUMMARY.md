# Project Research Summary

**Project:** deepseek-build — personal, DeepSeek-only terminal coding agent (Rust; fullscreen TUI + headless)
**Milestone:** Phase 1 freeze at `v0.1.0` — a **subsequent** milestone on a mature tree
**Domain:** distribution hardening + one latent correctness gap, not product construction
**Researched:** 2026-11-21
**Confidence:** HIGH overall, with three named MEDIUM pockets (§Confidence)

---

## Executive Summary

This is not a greenfield project. The adapter, endpoint, model catalog, rebrand, deletions, eval
harness, and a four-target release pipeline all exist and are validated (`PROJECT.md` §Validated).
The milestone's job is to make that finished thing **trustworthy and freezable**: wire the one
latent feature (model routing), fix the one host-dependent sandbox defect, prove the distribution
artifacts that have never actually shipped (macOS, GHCR image, unpinned installer), and leave
evidence for every claim. The research consensus is that this is ~5 code changes and ~6 pipeline
changes, none of which needs a new crate, a new outbound destination, or an architecture change.

The recommended approach is **dependency-ordered, evidence-first**. Every document independently
arrives at the same shape: build the evidence convention and a gate that asserts *completeness*
before touching product code; land the sandbox fix early because it is the only thing standing
between the tree and a defensible "zero failures"; then route → pin → visibility → per-model cost
in that order because each step creates the fact the next step displays or prices; and only then
touch version, tag shape, and the release pipeline, because a dry run with the current
`container needs: [version, build]` edge can never produce an image and a wrong tag shape can
publish a snapshot as "Latest release".

The risks are concentrated in four places, and they are *process* risks more than technical ones.
(1) The freeze tag `deepseek-v0.1-snapshot` **already exists on origin** at `4b305bd0`, five
commits behind HEAD, annotated "Linux-only v0.0.1-rc5" — the roadmap's final step as written
cannot succeed. (2) A `workflow`-scoped credential is a hard prerequisite for *every* pipeline
fix; without it the macOS, container, tag-shape, strip, and RELRO items all stall. (3) The suite
can be made green by weakening it, and the project has already over-claimed completion twice.
(4) Four verification commands that look conclusive are not: `cargo test --workspace`,
`docker run --version`, `scripts/test-install.sh`'s `latest` case, and pricing a session at one
model id. Those four are the milestone's real quality gate.

---

## What Is Already Settled — Do Not Re-Plan

The roadmapper should treat this list as closed. Each item is implemented in the tree and each
would waste phase time if re-opened.

| Settled | Evidence / decision | Consequence for the roadmap |
|---|---|---|
| Rust 1.94.0, edition 2024, pinned | `rust-toolchain.toml`; D8; `PROJECT.md:124` | No toolchain phase. Do not bump. |
| Four **native** runners, no cross-compile, no QEMU, no Rosetta | `release.yml:92-113`; PITFALLS §8 (Rosetta cannot *produce* an arm64 artifact) | Rejecting cross-compile is a recorded non-decision, not a phase task. |
| All third-party actions SHA-pinned; upload allowlist closed to {release, GHCR, intra-run artifacts} | `release.yml:9-35` | No sccache, no codecov, no mirror. Any addition is stop-and-ask. |
| Container *design*: `debian:bookworm-slim`, uid 1000, tarball-as-context (Dockerfile never compiles Rust), native per-arch buildx → `imagetools create` | `docker/Dockerfile`; `release.yml:347-462` | Only the digest pin and the proofs are open. No re-architecture. |
| `[workspace.package]` has **no** `version` key; 82 members are already `0.1.0`; exactly 4 shipping crates are `0.0.1-rc5` | `Cargo.toml:106-108`; verified by grep | The bump is a bounded change, not a 95-manifest sweep (see Disagreement 2). |
| Adapter core: `reasoning_content` sanitizer, thinking/effort → `tool_choice`, typographic-quote repair, cache hit/miss accounting | `PROJECT.md` §Validated; `xai-grok-sampler` | Do not touch. Routing applies an existing request field, it does not change the adapter. |
| Single wire protocol (`ChatCompletions` only); endpoints → `https://api.deepseek.com` | `PROJECT.md` §Validated | No backend work. |
| Rebrand surface done; `xai-grok-*` crate names and `GROK_*` env names **stay** (D1/D2) | `AGENTS.md` §8; `PROJECT.md` Out of Scope | Renaming is Phase 2. Not a freeze item. |
| Deletions stay deleted; egress invariants (D6/D10/D13); **no new outbound destination** — including no runtime price fetch | `PROJECT.md` Constraints; D14 | Every research doc independently re-affirms this. |
| **No new crates** anywhere in the five work items | STACK §"at a glance"; ARCHITECTURE item 1 | Routing/pin/cost all live in existing crates. |
| Prompt templates are untouched by all five items | ARCHITECTURE "Constraints" | `scripts/encrypt_templates.py` appears in **no** plan. |
| config-docs coupling is a **constraint, not a decision**: registry change + hand-edited `26-config-reference.md` in the same commit, enforced by `config_docs` | `AGENTS.md` §8; `xai-grok-shell/Cargo.toml` `default = []` | Encodes as an acceptance criterion on the routing phase. |
| `deny.toml` / cargo-deny already covers all four release targets | `deny.toml:4-14` | Local gate only; no CI job needed. |
| Phase 2 (open-build) branches from the freeze tag; **publishing** `v0.1.0` is out of scope | `PROJECT.md` Out of Scope | The roadmap ends at the freeze tag, not at a public release. |
| Freeze lineage: commits land on `main`; freeze cuts branch `deepseek` from its tip | `PROJECT.md` Key Decisions | One linear history to prove. |

**One caveat on the research base.** STACK and ARCHITECTURE read `main` @ `221a1a59`; PITFALLS
read `main` @ `9e6780d9` (four docs commits later), which is the current HEAD. File:line citations
agreed across both, but the freeze must name a commit and the roadmap should pin the research base
explicitly rather than inherit a moving one.

---

## Decisions Required BEFORE Planning

These cannot be resolved by reading the tree; each is a fork with a different failure mode. The
roadmapper should not emit plans that presuppose an answer.

### D-1. Release tag shape vs. freeze marker tag — and the tag that already exists

**The fact:** `deepseek-v0.1-snapshot` exists locally **and on origin** (`68d8590f`, annotated)
pointing at `4b305bd0` — the rc5 release commit, five commits behind HEAD. Its message says
"Linux-only v0.0.1-rc5". `PROJECT.md:71` lists "a `deepseek` branch exists with tag
`deepseek-v0.1-snapshot` on it" as an *unfinished* requirement, which is false as written.

| Option | What it means | Consequence of getting it wrong |
|---|---|---|
| **(a) Separate the concepts** (PITFALLS §7, STACK's stated alternative): keep the trigger at `'v*'`; the freeze marker is a new, non-triggering name (e.g. `deepseek-v0.1.0-freeze`) on branch `deepseek`; supersede the old tag in `KNOWN-ISSUES.md` | Zero workflow risk; the marker can never publish anything | Leaves `PROJECT.md:67` ("`release.yml` fires on the DeepSeek tag shape") unsatisfied unless PROJECT.md is amended — that amendment is itself a decision |
| **(b) Support both shapes** (STACK §6.2): one normalization regex `^(?:deepseek-)?v?([0-9]+\.[0-9]+\.[0-9]+(?:[-+].*)?)$`, add `deepseek-v*` to the trigger, run the prerelease test on the *captured* version, replace `type=semver` with `type=match`/`type=raw`, extend `install.sh` | Satisfies PROJECT.md as written | **This is the dangerous path.** PITFALLS #7 case 3: `deepseek-v0.1-snapshot` contains no `-rc`, so it classifies `is_prerelease=false` → publishes a **non-prerelease "Latest release"** and moves container `:latest` to a snapshot. Only safe if all six consumer sites change in one commit **and** the version job refuses `*snapshot*` |
| **(c) Move the existing tag** | One ref, right commit | Destructive remote ref update; rewrites a ref other clones hold; needs explicit user approval and must never happen inside an automated phase |

**Recommendation:** (b) *if and only if* the six-site change plus the `*snapshot*` guard is one
commit with a `scripts/test-install.sh` case for a `deepseek-` tag; otherwise (a) plus an explicit
PROJECT.md amendment. The tag *identity* decision must be taken in the version/tag phase, not
discovered at freeze time.

### D-2. Pin semantics

`manual_override` exists and already wins over policy (`model_routing.rs:190-192`), but nothing
sets it, and today bare `/model` does two things at once: switches the session *and* persists
`default_model` (`setters.rs:1567-1650`).

| Option | Consequence |
|---|---|
| **(a) Any explicit `/model` or `/effort` pins the whole session (model + effort) until cleared** (ARCHITECTURE recommendation) | One fact, one field (`Option<ModelRoute>` on `SessionActor`), consumed as `manual_override`. Clear path rides the existing `SetSessionModelRequest` `_meta`. Matches "routing never overrides a pin" |
| (b) Field-level pin (model only, or effort only) | Requires synthesising a full `ModelRoute` from a partial one at decision time — a second derivation, i.e. exactly the failure mode this milestone exists to remove |
| (c) No session pin; `/model` stays a config default change | Routing would override the user's explicit choice — forbidden by `PROJECT.md` Key Decisions |

Two sub-decisions ride along: **what clearing restores** (recommended: leave
`sampling_config.model` as the last explicit choice and document it, rather than silently resetting
to the catalog default), and **whether the pin persists across resume** (recommended: an explicit
`pinned: Option<bool>` with `#[serde(default)]` on `PersistenceMsg::CurrentModel` — the inference
alternative "model ≠ configured default ⇒ pinned" can flip when a release changes the default).
Getting this wrong produces either a silently un-overridable router or a resumed session that runs
a different model than it was saved with.

### D-3. Does the sandbox run inside the container image?

The sandbox is on by default and fails closed. A container is precisely the class of host where
`bwrap` cannot create user namespaces under Docker's default seccomp profile (**UNVERIFIED**
mechanism; the fail-closed behaviour is recorded in `CONCERNS.md`).

| Option | Consequence |
|---|---|
| (a) Document the required flags (`--security-opt seccomp=…` or user-ns) so the sandbox stays on | Preserves the headline safety default in the only distributed container; needs a documented, tested run command |
| (b) Ship the image with `DEEPSEEK_BUILD_SANDBOX=off` and a stated reason | Simple, but silently weakens the product's default in the artifact most likely to be tried first |
| (c) Leave it undecided | `docker run --version` passes, the first real tool call dies at sandbox startup, and the README's `-it --rm` example has no mounts anyway |

Whichever is chosen, the README GHCR block (`README.md:153-175`) must state the image is
headless-only (no TTY) and document `-v` mounts for the repo, config, and `.grok/` — `WORKDIR
/work` and `~/.deepseek-build` are ephemeral otherwise.

### D-4. `workflow`-scope credential — a hard precondition

Every change in the pipeline, tag-shape, strip, and RELRO items edits `.github/workflows/release.yml`.
`KNOWN-ISSUES.md:201-202` records that the missing `workflow` scope blocked the previous attempt
after rc4. GitHub rejects the *push*, with a message that reads like a code problem.

**Options:** a PAT / `gh auth login` with `workflow` scope; a fine-grained token or GitHub App with
workflows permission; or accept the block. **Consequence:** if unavailable, the macOS-green,
container-proven, tag-shape, strip, and RELRO requirements must be recorded as **blocked**, not
retried in a loop. STACK calls this "the single highest-risk unknown in the milestone" — schedule
the credential before Phase 1, not before Phase 7.

### D-5. Routing config surface (two documents propose different keys)

STACK §5: a new `[model_routing] policy = "off"|"low"|"high"|"auto"` section. ARCHITECTURE item 1:
`[models] routing` inside the existing `ModelsConfig`, which already holds `default` and
`default_reasoning_effort` (`agent/config.rs:387-402`). Both need the registry entry, the
hand-edited page, and a green `config_docs` run; the choice determines which page section the
routing phase edits and where the resolver module lives. ARCHITECTURE's co-location argument is
stronger; STACK's namespace is cleaner. **Pick one before the routing phase is planned** — the
config-docs acceptance criterion depends on it.

### D-6. Status-line surface: new item or reuse `Model`?

STACK §5 says reuse the existing `Model` item ("cheapest, no new config key, no doc churn").
ARCHITECTURE item 2 and FEATURES T2 both say: `Model` renders `ctx.route.model` **and** a new
`StatusLineItem::Route` renders the reason (`auto`, `auto · pinned`, `policy low`). Two of three
want the new item, and STACK's version discards the *reason* — which is what FEATURES D1 ("why this
model") sells as the cheapest trust multiplier. **Recommend ARCHITECTURE/FEATURES**, and note that
adding the enum variant obliges `varies_mid_turn`, a guide row, and confirming the default `items`
list is unchanged so existing users see no layout shift.

### Secondary decisions (cheap, but record them)

- **GHCR pull authentication** — package visibility is independent of repo visibility and defaults
  private (**UNVERIFIED**). "Pulled" needs either an authenticated pull or a visibility flip; decide
  which the proof command asserts.
- **`ci.yml`** — there is no test workflow; `release.yml` is the only one. `PROJECT.md` requires a
  green run *on the author's host*, which needs no CI. Adding fmt/clippy/test CI is a scope addition
  with the same `workflow` prerequisite — decide explicitly rather than by default.
- **Debug sidecars** — STACK assumes not wanted; PITFALLS M3 says decide explicitly either way.
- **Do subagents route?** They run the same `run_turn_via_sampler` funnel and therefore inherit the
  policy. Gate on `startup_hints.is_subagent` or let them route — but not implicitly.
- **The three crates at `0.1.220-alpha.4`** (`xai-grok-workspace`, `xai-grok-tools`,
  `xai-grok-tools-api`) — STACK says "not shipped, leave them". They *are* compiled into the binary;
  their version strings are invisible, so leaving them is defensible, but record it rather than
  asserting they are not shipped.

---

## Where the Research Disagrees (do not flatten)

1. **Container proof.** STACK §6.3 item 4: `docker run --rm <image> --version` in the pushing job
   "satisfies built **and pulled**". PITFALLS #9: `--version` proves the loader and glibc, not the
   product; the sandbox likely cannot start, so the first tool call dies while `--version` is green.
   **Both are right about different claims.** The literal requirement is "built and pulled" →
   `--version` is the pull proof. The product proof is a tool-executing headless turn. Do both; never
   let `--version` be the only evidence.
2. **Version-bump scope.** STACK measured 82 members already at `0.1.0` and prescribes
   `[workspace.package] version = "0.1.0"` + four `version.workspace = true` lines. PITFALLS M1 says
   "95 manifests and three other version sources". **STACK read the tree; PITFALLS generalised.**
   STACK wins on scope. PITFALLS' residual concern is real and survives: release builds take the
   version from `GROK_VERSION` (stamped in `release.yml`) while a source build falls back to
   `CARGO_PKG_VERSION`, so both paths must be asserted, and `Cargo.lock` regenerated.
3. **Sandbox skip predicate.** ARCHITECTURE §4: `Err(PermissionDenied) => continue` on parent
   canonicalize. STACK §8: skip only when `symlink_metadata(endpoint)` is `NotFound`/`EACCES`, error
   if visible. PITFALLS #12: gate on `access(parent, X_OK)` traversal **only**, never on target
   readability, with a fixture matrix. PITFALLS is strictest and most testable; ARCHITECTURE's
   one-liner is compatible but leaves "endpoint visible, parent untraversable" unstated. **Use
   PITFALLS' formulation at ARCHITECTURE's placement**, plus STACK's inverse test (traversable
   parent + unreadable endpoint ⇒ `Err`).
4. **Delete or keep the sampler-side routing entry point.** STACK keeps `route_turn` and adds a
   caller. ARCHITECTURE deletes `SamplingConfig.model_routing`, `ClientDefaults.model_routing`,
   `route_turn`, and `model_routing_policy` so the session is the only owner. PITFALLS #11 lists
   "two call sites for `route_turn`" as a failure signal. **ARCHITECTURE + PITFALLS agree; STACK's
   plan leaves the second source of truth in place** — which is ARCHITECTURE's anti-pattern 4.
5. **macOS remedy.** STACK: split the jobs (correctness) and env-override
   `CARGO_PROFILE_RELEASE_DIST_LTO=false` / `CODEGEN_UNITS=16` on darwin-arm64 (MEDIUM confidence).
   PITFALLS #5: the timeout is a **cold-cache loop** — a cancelled job never runs cache-save
   post-steps, so every retry hits the same wall; measure first, accept one cache-warming run, and
   cut `debug=1` from the darwin link. PITFALLS M3 adds a risk STACK misses: stripping on arm64
   macOS can invalidate the ad-hoc signature (**UNVERIFIED**), so a binary that links and
   smoke-checks from `target/` can fail at `exec` for the user. **Combine, in this order:** split
   jobs → strip/debug=0 → re-sign + `--version` from the *extracted tarball* → one cache-warming
   run → per-leg timeout → LTO override last.
6. **Risks only one document caught.**
   - PITFALLS only: `scripts/disk-guard.sh` defaults to **60 GB** while the documented floor is 150
     (M5); `scripts/test-install.sh` only ever builds the host triple, so 39/39 is a Linux-x86_64
     result (M2); `login --help` still describes a deleted OIDC flow (m2); `.grok/` vs
     `~/.deepseek-build` drift (m3); local-vs-CI linker divergence (M4).
   - STACK only: `docker/metadata-action` yields an empty tag list on a non-semver ref, so the
     manifest loop is a **no-op that still exits 0** — needs an explicit `[ -n "$tags" ]` guard.
   - FEATURES only: **headless ignores the user's `[ui.status_line.pricing]` overrides** entirely
     (`PricingTable::default()` at `headless.rs:313-340`), so TUI and JSON can disagree even before
     routing. Neither STACK nor ARCHITECTURE mentions this.
   - ARCHITECTURE only: `deepseek-v4-flash` is **absent from `deepseek_defaults()`**
     (`pricing.rs:78-83`) and `for_model` falls back to the Pro rate — a ~4× overstatement the
     moment routing can select it. FEATURES describes the symptom ("one model prices the session")
     without naming the third catalog id; STACK does not mention it at all.

---

## Dependency-Ordered Build Sequence

Nine phases. The arrows are load-bearing; the reasons are the deliverable.

```
P1 scaffolding + truth baseline ──┬──▶ P2 sandbox fix ──▶ clean-suite green
  (token, gate, evidence, 150GB)  │
                                  └──▶ P3 routing wiring (policy off)
                                              │
                                              ▼
                                        P4 routing on + pin + visibility
                                              │
                                              ▼
                                        P5 per-model cost + cache
                                              │
                                              ▼
                                        P6 loop-item verification sweep
P7 version + tag contract + pipeline decoupling ──▶ P8 distribution proof ──▶ P9 freeze
   (independent of P3–P6; must precede any tag dry run)
```

**P1 — Freeze scaffolding, truth baseline, preconditions.** No product code.
Delivers: `DECISIONS.md` entries for D-1…D-4; confirmed `workflow`-scope credential; a gate
*script* that asserts completeness (`grep -c '^test result:'` == target count, today 307) plus the
separate `cargo test -p xai-grok-shell --lib --features config-docs` command; `docs/evidence/` with
a normalizer and one worked example; `scripts/disk-guard.sh 150` in every wrapper.
*Why first:* every later phase appends an artifact, and the baseline run must be allowed to report
the two known failures **honestly** rather than be pre-cleaned.
*Avoids:* PITFALLS #2, #4, M5; FEATURES T18.

**P2 — Sandbox deny materialization.** The one resolver
(`materialize_runtime_socket_deny_paths_from`), skip non-traversable parents, keep fail-closed
everywhere else, unit fixture matrix + the two e2e tests green **unmodified**.
*Why before the clean-suite gate:* the gate's pass criterion is zero failures on the author's host,
and the only known failures are the bwrap pair. The acceptable way to zero them is the product fix;
widening `skip_if_enforcement_unavailable`, `#[ignore]`, or deleting the tests is forbidden.
*Avoids:* PITFALLS #3, #12.

**P3 — Routing decision wiring, policy still off.** Add `RouteReason` to `ModelRoute`; resolve the
policy once at spawn into a `SessionActor` field; apply the route at the **entry** of
`run_turn_via_sampler` (mutating the by-value request); delete the sampler-side entry points.
*Why before pin/visibility:* visibility renders a decision that only exists once the session
decides, and the pin is consumed as `manual_override` in the config assembly P3 creates. Landing
visibility first gives the row nothing to show and the pin nothing to suppress.
*Why the funnel and not `turn.rs`, `submit_turn_request`, the sampler, or `xai-chat-state`:*
`turn.rs` is one caller (a future subagent/retry path bypasses it); `submit_turn_request` re-decides
per HTTP attempt; the sampler cannot see the pin and would force the status line to re-derive; chat
state owns the pinned model, so routing there makes "clear the pin" unexpressible.
*Exit:* behaviour unchanged with policy `off`; exactly one place reads a policy; wiremock asserts the
request body's `model` and `reasoning_effort`.
*Avoids:* PITFALLS #11; ARCHITECTURE anti-patterns 1 and 4.

**P4 — Routing on: pin + visibility.** Policy default decided and documented (key per D-5), registry
+ hand-edited page + green `config_docs` in the same commit; `SessionActor.pin` set by the two
existing handlers; clear path via `_meta`; `StatusLineRoute` on the payload; segment per D-6;
`testdata/status_wire.json` + `25-status-line.md`; wake once per turn (not per sampling request);
resume determinism per D-2; explicit subagent decision.
*Why after P3:* see above. *Why before P5:* both edit `StatusLineContext`, the fixture, and the
guide; P4's `route` field lands first so P5's `session_usage_by_model` is a second additive change
to an already-extended contract. `STATUS_LINE_SCHEMA_VERSION` stays `1` either way.
*Avoids:* ARCHITECTURE anti-patterns 1–2; FEATURES A2 (silent routing).

**P5 — Per-model cost and cache.** `PricingTable::cost_usd_by_model`; `session_usage_by_model` on the
payload; all **three** consumers price per bucket with the aggregate as fallback; price
`deepseek-v4-flash`; label the TUI number as an estimate; make headless honour user overrides.
*Why after P3/P4 (hard):* before routing, one model per session makes the current pricing correct by
accident. *Do not* price from `route.model` — the ledger is what the provider reported; money
follows the ledger. *Do not* touch the tokens/cache segments — they are model-agnostic aggregates.
*Avoids:* FEATURES T14; ARCHITECTURE anti-pattern 3.

**P6 — Loop-item verification sweep.** Permissions prompt scope, headless fail-closed + hint,
permission-mode persistence, resume fidelity, plan-mode read-only guarantee, headless
`--permission-mode plan`, plan approval across resume, cache/token buckets. Each ends in a test or a
committed transcript plus a corrected doc line — **not** in a code change.
*Why after P4:* resume-under-routing depends on the pin decision. *Why not a code phase:* FEATURES
classifies T6–T8 and T11–T16 as verify-and-record; the tree already implements them.
*Avoids:* FEATURES A11 (a second set-mode call for a bug that is already fixed).

**P7 — Version, tag contract, pipeline decoupling.** `[workspace.package] version = "0.1.0"` + four
`version.workspace = true`; tag normalization across all six consumer sites (or the separate-marker
route) + a `*snapshot*` guard; split `build-linux`/`build-macos` with `container: needs: [version,
build-linux]`; `strip = "symbols"`/`debug = 0` (with macOS re-sign + extracted-tarball `--version`);
matrix `timeout` field; `rustup toolchain install` on both legs; RELRO/NX link-args on both gnu
targets; digest-pin the base image; manifest-loop guard; `save-if` so tag runs restore but do not
save.
*Why it must precede any release dry run:* a dry run with the current `needs: [version, build]`
can never produce an image if macOS is red; and a dry run is a real tag push — with the wrong shape
it either does not fire or publishes a non-prerelease and moves `:latest`.
*Why it can run in parallel with P3–P6:* disjoint files. But its *dry run* cannot precede them if the
dry-run artifact is meant to be the frozen binary — sequence the dry run after P5 at the earliest.
*Avoids:* PITFALLS #1, #5, #6, #7, M1, M3, M7; STACK §1–§7.

**P8 — Distribution proof.** Tool-executing headless turn inside the image (not `--version`); pull
proof (anonymous or authenticated per D-3/secondary decision); pinned-`VERSION=` install from the
**real** release; four-triple `test-install.sh` success cases; README GHCR block corrected (headless
only, `-v` mounts, sandbox note); `curl -w '%{http_code}'` on `/releases/latest`.
*Why last before freeze:* it consumes the artifacts P7 produces and it is the only step that needs a
real tag.
*Avoids:* PITFALLS #9, #10, M2.

**P9 — Freeze.** Docs reconciliation (`DECISIONS.md`, `TESTING.md`, `KNOWN-ISSUES.md`, `login
--help`, the `.grok/` paragraph, the over-promising `release-dist` comment); dated egress evidence
(commit + command + tool limits, or NOT RUN); final full gate re-run (workspace + config-docs);
branch `deepseek` from the tip; freeze tag per D-1; verify `git rev-parse <tag>^{}` equals the
freeze-branch tip; confirm the freeze tag triggered **no** workflow run.
*Avoids:* PITFALLS #1, m5, M8; the whole "Looks Done But Isn't" checklist.

### Why these constraints exist, in one line each

| Constraint | Reason |
|---|---|
| Sandbox fix **before** the clean-suite gate | The gate's criterion is zero failures on the author's host; the only red tests are the bwrap pair, and the only legitimate way to zero them is the product fix |
| Routing wiring **before** pin/visibility | Visibility renders a decision that does not exist yet; the pin is an input to the config assembly that routing creates |
| Pin/visibility **before** per-model cost | Both extend `StatusLineContext` + fixture + guide; cost is a second additive change to an already-extended contract, and per-model cost is meaningless while only one model can be sent |
| Pipeline decoupling **before** any tag dry run | `container` is gated on the whole build matrix, so a red macOS leg erases the image; and a dry run is a real push that can move `:latest` |
| Version + tag contract **before** the dry run | The smoke check asserts the stamped version; `install.sh` must accept the tag the run produces |
| Evidence convention **before** everything | Every phase must end with an artifact, not a retrofit; and the artifact format decides what "done" means |
| `workflow` credential **before** P7 | Every pipeline edit is rejected at push without it |

---

## "Looks Done But Isn't" — Traps Where a Naive Check Passes

1. **`cargo test --workspace` as the docs guard.** `config-docs` is **not** a default feature
   (`xai-grok-shell/Cargo.toml` `default = []`), so the workspace run never compiles the
   `config_docs` tests that reconcile `26-config-reference.md` with the live registries. A green
   workspace run **plus** a hand-edited config page is the exact failure. Verify a separate
   `cargo test -p xai-grok-shell --lib --features config-docs` run exists.
2. **`docker run --version` as container proof.** Passes on the dynamic loader alone. The sandbox is
   on by default and generally cannot create user namespaces under Docker's default seccomp, so the
   first tool-executing session can die while the proof is green. Proof = a headless turn that runs
   a shell command.
3. **The installer's `latest` path.** `scripts/test-install.sh` *synthesizes* `latest/download/` in
   its fake tree, so it asserts a behaviour GitHub does not provide while every published release is
   a prerelease (`/releases/latest` resolves only to non-prereleases). 39/39 proves the URL layout,
   not the selection semantics. Prove with pinned `VERSION=`; check the real URL's HTTP status.
4. **Per-model cost that prices the whole session at one model.** Three sites each price the
   aggregate at a single id: TUI `ctx.model.id`, headless `session_model`, streaming reducer
   `session.model`. Under routing a mixed Flash/Pro session is priced at one rate — and
   `deepseek-v4-flash` is not in the table at all, so it silently inherits **Pro** rates (~4×).
   A "cost looks plausible" check passes.
5. **Green from a run that never finished.** 307 targets; a killed run has **no summary line**, and
   the log tail is a wall of `test … ok`. Verify `grep -c '^test result:'` equals the target count —
   not the absence of `FAILED`.
6. **`--permission-mode plan` "fix".** `KNOWN-ISSUES.md` still claims the flag does nothing; the tree
   sends `SetSessionModeRequest` for `plan`. "Fixing" it with a second set-mode call double-applies.
   Verify with a headless transcript, then correct the doc.
7. **39/39 installer assertions.** The fake tree is built for `$(host_target)` only, so the `Darwin`
   branches, the `shasum` fallback, and the macOS tarball-name path never execute.
8. **`docker buildx imagetools inspect` succeeding.** If `type=semver` produced no tags, the manifest
   loop is a no-op and the job still exits 0 — nothing was pushed, and the inspect either fails
   afterwards or inspects a stale tag. Guard with a non-empty tag assertion.
9. **The freeze tag.** `git tag deepseek-v0.1-snapshot` fails with "already exists"; forcing it is a
   destructive remote ref update. And if the trigger learns `deepseek-v*`, pushing the *existing*
   snapshot tag shape publishes a non-prerelease and moves `:latest`.
10. **A stripped macOS binary.** Compile-time `strip = "symbols"` runs post-link; on arm64 that can
    invalidate the ad-hoc signature (**UNVERIFIED**). A binary that smoke-checks from `target/` can
    fail at `exec` on the user's Mac. Re-sign, then `--version` the binary **extracted from the final
    tarball**.
11. **Version stamping.** A source build prints `CARGO_PKG_VERSION`; a release build prints
    `GROK_VERSION` from the tag. Verifying one path proves nothing about the other.
12. **The locally built binary.** `scripts/cargo-linker.sh` prefers lld and silently falls back to
    cc, so a local artifact is not the CI artifact. Only the CI tarball may be called "verified".

---

## Implications for Roadmap

**Suggested phases: 9.** Grouping rationale: P1–P2 are *truth*; P3–P5 are *one vertical slice*
(routing reachable → visible/reversible → correctly priced); P6 is *verification*; P7–P8 are
*distribution*; P9 is *freeze*. P7 can run in parallel with P3–P6 on a second track if capacity
exists, but its dry run belongs after P5.

| # | Phase | Delivers | Addresses | Avoids |
|---|---|---|---|---|
| 1 | Scaffolding & truth baseline | Gate script (completeness + config-docs), evidence convention, decisions logged, credential confirmed, 150 GB guard | T18 | P#2, #4, M5 |
| 2 | Sandbox deny materialization | Product fix + fixture matrix; e2e green unmodified; **zero failures** | T17 | P#3, #12 |
| 3 | Routing wiring (off) | Single decision point at the funnel; sampler entry points deleted; body assertion | T1, T5 | P#11; AP-1, AP-4 |
| 4 | Routing on: pin + visibility | Config key + docs; `SessionActor.pin` + clear; `StatusLineRoute` + segment; resume determinism | T2, T3, T4, T10 | AP-1, AP-2; A2 |
| 5 | Per-model cost & cache | `cost_usd_by_model`, payload buckets, 3 consumers, hidden alias priced, estimate label | T14, T15, T16 | AP-3 |
| 6 | Loop-item verification | Test/transcript per item; corrected `KNOWN-ISSUES.md`; `login --help` | T6–T9, T11–T13 | A11 |
| 7 | Version + tag contract + pipeline | `0.1.0` bounded bump; six-site tag contract; split jobs; strip; timeouts; RELRO; digest pin | — (distribution) | P#1, #5, #6, #7, M1, M3, M7 |
| 8 | Distribution proof | In-image tool turn; pull proof; pinned install from the real release; four-triple harness | — | P#9, #10, M2 |
| 9 | Freeze | Docs reconciliation; dated egress evidence; final gate; branch + tag verified | — | P#1, M8 |

**Defer to v0.1.x** (FEATURES P2/P3): "why this model" (D1), `/route` command (D3), off-peak pricing
switch (D4), per-model spend in `/usage` (D2), route in the session log (D5).
**Defer to Phase 2:** automatic Flash-for-research subagents (D6-automatic), any provider-agnostic
routing registry (A5), learned escalation (A4 — `docs/TUNING.md` found no task class where Flash
fails and Pro succeeds, so the boundary is unmeasured).

### Research Flags

Needs a deeper research/measurement step during planning:
- **Phase 2 (sandbox)** — MEDIUM on the exact POSIX fixture semantics; the e2e can only be proven on
  the author's host. Plan the unit fixture matrix as the primary proof.
- **Phase 7 (pipeline)** — MEDIUM on macOS wall-clock arithmetic: runner core count, cache budget,
  and the `CARGO_PROFILE_*` env-override are recalled, not verified. Plan a *measure-first* step
  (read the macOS job log, find whether `Restore cargo cache` hit, whether the cancel landed in
  compile or link) before choosing between timeout, strip, and LTO override.
- **Phase 5 (cost)** — UNVERIFIED whether DeepSeek echoes the requested model id in the SSE chunk.
  Per-model bucketing depends on it; verify with one live smoke turn before relying on it.

Standard patterns, skip research-phase:
- **Phase 3/4 (routing, pin, status line)** — every seam, line number, and anti-pattern is already
  placed in ARCHITECTURE.md. This is execution, not research.
- **Phase 1 (gate + evidence)** — scripted shell and a Python normalizer.
- **Phase 9 (freeze)** — a checklist.

---

## Confidence Assessment

| Area | Confidence | Notes |
|---|---|---|
| Stack | **HIGH** on what is settled; MEDIUM on CI arithmetic | Every claim traced to a file. The open items (split jobs, strip, tag regex, RELRO, digest pin) are all high-confidence *changes*; only the macOS time budget is MEDIUM, and two figures (runner cores, cache budget) are explicitly UNVERIFIED. |
| Features | **MEDIUM-HIGH** | Grounded in files, but "good" for five daily-driver loops is a judgement; T6–T13 are verify-and-record, so the cost estimate is a floor, not a ceiling. |
| Architecture | **HIGH** for placement, MEDIUM for two design choices | All five seams are grounded at line level. The pin-persistence shape and the status-line payload shape are decisions, not facts. |
| Pitfalls | **HIGH** for repo-grounded findings | Six items explicitly UNVERIFIED and all external: `/releases/latest` prerelease exclusion, GHCR default visibility, timed-out jobs skipping cache-save, `strip` invalidating a Mach-O signature, bwrap under Docker seccomp, Apple runner cores. Each has a cheap runtime check. |

**Overall confidence: HIGH** for the sequence and the decisions; **MEDIUM** for the macOS time
budget and the exact sandbox test fixture.

### Gaps to Address

- **Tag identity is unresolved and time-sensitive** — the existing tag is public. Resolve D-1 in
  Phase 1, execute in Phase 9.
- **`workflow`-scope credential** — resolve before Phase 7 is planned, ideally in Phase 1. If
  unavailable, five requirements are *blocked*, not retried.
- **Provider model echo** — one live smoke turn before Phase 5 relies on per-model buckets.
- **macOS time budget** — measure from the job log before choosing a remedy; do not start with
  `timeout-minutes`.
- **Config key location (D-5) and status-line item (D-6)** — two documents disagree; the config-docs
  acceptance criterion depends on both.
- **`deepseek-v4-flash` pricing** — a live defect that becomes reachable the moment routing can
  select it; must land with Phase 5, not after.
- **Research base commit** — STACK/ARCHITECTURE read `221a1a59`, PITFALLS and HEAD are `9e6780d9`.
  Name the base commit in the roadmap.
- **Evidence format** — no `.cast` is committed today; ARCHITECTURE prescribes `docs/evidence/` with
  a path normalizer (absolute paths in `ScriptedRunReport` leak the home directory). Confirm the
  format in Phase 1 so Phase 2 onward can append.

---

## Sources

### Primary (HIGH confidence — read in this repository, no network)

- `.planning/PROJECT.md` — active requirements, out-of-scope, constraints, key decisions.
- `.planning/codebase/{STACK,ARCHITECTURE,CONCERNS}.md` — settled stack, seams, the four latent defects.
- `.github/workflows/release.yml` — trigger, matrix, timeouts, `needs` edges, action pins, container job, upload allowlist. *(Re-verified: trigger `'v*'`; `build` timeout 90; `container`/`publish` both `needs: [version, build]`.)*
- `docker/Dockerfile`, `docker/Dockerfile.localtest` — base image, uid, tarball context.
- `.cargo/config.toml` — per-target rustflags (RELRO only under musl), linker wrapper, `RUST_MIN_STACK`.
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `deny.toml` — workspace profile, 1,246 packages, 1.94.0 pin, license policy. *(Re-verified: `[workspace.package]` has no `version`; four crates at `0.0.1-rc5`.)*
- `install.sh`, `scripts/test-install.sh` — version guard, `latest` resolution, host-only fake tree.
- `scripts/{disk-guard,check-egress,egress-check,cargo-linker}.sh`.
- `crates/codegen/xai-grok-sampler/src/{model_routing,client,config}.rs` — pure decision, `route_turn`, `ClientDefaults`.
- `crates/codegen/xai-grok-shell/src/**` — `run_turn_via_sampler`, `build_status_context`, `model_switch.rs`, config resolution, `Cargo.toml` features. *(Re-verified: `default = []`.)*
- `crates/codegen/xai-grok-status-line/src/{context,pricing,config}.rs` — payload contract, price table, item enum. *(Re-verified: `deepseek_defaults()` seeds only pro + flash; catalog ships three ids.)*
- `crates/codegen/xai-grok-pager/src/**` — `segments.rs` TODO, `status_line_policy.rs`, `headless.rs`, `headless/reducer/messages/usage.rs`.
- `crates/codegen/xai-grok-sandbox/src/{runtime_sockets,profiles,lib}.rs` + tests.
- `crates/codegen/xai-grok-pager-pty-harness/**`, `crates/codegen/xai-grok-test-support/**` — evidence producers.
- `KNOWN-ISSUES.md`, `DECISIONS.md` (D1–D14), `AGENTS.md`, `TESTING.md`, `docs/TUNING.md`.
- `git show-ref --tags`, `git ls-remote --tags origin`, `git rev-parse`, `git merge-base --is-ancestor`. *(Re-verified: `deepseek-v0.1-snapshot` → `4b305bd0`, present on origin as annotated tag `68d8590f`; HEAD `9e6780d9`.)*

### Secondary (MEDIUM confidence)

- Four-document cross-read for the pin-persistence shape and the status-line payload shape — design
  judgement, not tree fact.
- `docker/metadata-action` semver parsing for a prefixed tag — inferred from behaviour, not tested here.

### Tertiary (LOW confidence — recalled, verify before relying)

- GitHub `/releases/latest` excludes prereleases (verify with `curl -w '%{http_code}'`).
- GHCR package visibility defaults to private, independent of repo visibility.
- A timed-out GitHub job skips cache-save post-steps.
- `strip` invalidates an ad-hoc Mach-O signature on arm64 macOS.
- `bwrap` user-namespace creation fails under Docker's default seccomp profile.
- Apple arm64 runner core count / memory; GitHub per-repo cache budget.

---

*Research completed: 2026-11-21*
*Ready for roadmap: yes — after D-1, D-2, D-3, D-4 and the `workflow`-scope credential are answered.*
