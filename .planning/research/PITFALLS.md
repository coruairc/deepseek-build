# Pitfalls Research

**Domain:** Freezing a mature Rust terminal coding agent (TUI + headless) at v0.1.0 —
test-suite truth, release pipeline, packaging, and security-invariant preservation
**Project:** deepseek-build (fork of xai-org/grok-build, DeepSeek-only)
**Researched:** 2026-11-21
**Confidence:** HIGH for repo-grounded findings; MEDIUM/LOW where marked UNVERIFIED
**Tree:** `main` at `9e6780d9` (rc5 lineage `4b305bd0` + 5 commits, 4 of them docs)

> **How to read this.** Every "What goes wrong" claim cites a file read in this session, or is
> marked **UNVERIFIED** (recall/external behaviour not checkable without network). Phase names in
> the mapping table are **provisional** — `.planning/ROADMAP.md` does not exist yet, so the
> roadmap should adopt or rename them. Each pitfall carries a **Blocker for freeze** verdict,
> because "documented risk" and "cannot tag" are different outcomes.

---

## Critical Pitfalls

### Pitfall 1: The freeze tag already exists — on the wrong commit, and already public

**What goes wrong:**
The roadmap's final step ("create tag `deepseek-v0.1-snapshot`") cannot succeed as written.
The tag exists today as an **annotated** tag pointing at `4b305bd0` (the rc5 release commit),
five commits behind `main`:

- `git rev-parse deepseek-v0.1-snapshot^{}` → `4b305bd0c1ab…`
- `git rev-parse HEAD` → `9e6780d9…`
- `git merge-base --is-ancestor 221a1a59 deepseek-v0.1-snapshot` → **fails** (main's rc5 doc
  commit is not in the tag's history)
- `git ls-remote --tags origin` → the tag **is already pushed** (`68d8590f…`).
- The tag message already asserts a claim the milestone contradicts:
  `deepseek-v0.1-snapshot deepseek-build v0.1 snapshot: Linux-only v0.0.1-rc5`

So the artifact named "v0.1 snapshot" is a Linux-only rc5 marker that predates every freeze fix
(routing, sandbox profile, macOS pipeline, container, docs reconciliation).

**Why it happens:**
The tag was created as an *anticipatory* marker during the rc5 handoff, and the planning docs
described it as a future deliverable. `PROJECT.md` then listed it under Active requirements
("A `deepseek` branch exists with tag `deepseek-v0.1-snapshot` on it"), which reads as
"not yet done". Nobody re-checked `git show-ref --tags`.

**How to avoid:**
Decide the tag *identity* before the freeze phase starts, and record it in `DECISIONS.md`:
1. **Preferred:** use a new, unambiguously-frozen name (e.g. `deepseek-v0.1.0-freeze`) created on
   branch `deepseek` at the freeze tip, and mark the existing `deepseek-v0.1-snapshot` as
   superseded in `KNOWN-ISSUES.md`.
2. **Only with explicit approval:** move the existing tag (`git tag -f` locally, then a forced
   remote ref update). That is a destructive remote operation; team rules require user approval,
   and it rewrites a ref other clones may already have.
Never do either silently inside an automated phase.

**Warning signs:**
- A phase plan says "tag `deepseek-v0.1-snapshot`" and `git tag -l` already lists it.
- `git tag deepseek-v0.1-snapshot` → `fatal: tag 'deepseek-v0.1-snapshot' already exists`.
- `git push origin deepseek-v0.1-snapshot` → `! [rejected] … (already exists)`.
- The tag's own message mentions "Linux-only".

**Phase to address:** the final freeze phase, but the *decision* must be made in the phase that
sets the version/tag shape (it changes what `release.yml` must learn).

**Blocker for freeze:** **YES.** As written, the freeze step fails; and if forced, the "frozen"
ref is not the tested tree.

---

### Pitfall 2: Declaring the suite green from a run that never finished

**What goes wrong:**
The project has already done this once. `AGENTS.md` §0 records that a follow-up full run on
`cc6e6948` "was described as in-flight then and **was never recorded as finished**". A
`cargo test --workspace` over 307 targets takes long enough to hit a shell/tool timeout; when it
is killed there is **no summary line at all**, and the last thing in the log is a wall of
`test … ok`. A reader then quotes the *previous* run's numbers (31,939 / 2 / 388) as current.

**Why it happens:**
Three separate amplifiers:
- 307 test binaries, `RUST_MIN_STACK=67108864` per thread (`.cargo/config.toml [env]`), so the
  run is memory- and time-heavy on a ~16 GB box.
- The summary is per-target (`test result: …`), so "looks green" is judged from the tail of the
  log, which is one target out of 307.
- Interrupting mid-run leaves `target/` partially populated, so the next attempt is *not* a
  clean comparison either.

**How to avoid:**
Make the gate a script, not a command typed into a chat, and make it assert **completeness**, not
just absence of `FAILED`:

```sh
scripts/disk-guard.sh 150 || exit 1
export CARGO_TARGET_DIR="$HOME/Projects/deepseek-build/target"
env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY \
  flock -w 900 /tmp/team-build.lock \
  cargo test --workspace --no-fail-fast -j 2 -- --test-threads=4 2>&1 | tee /tmp/full-test.log
grep -c '^test result:' /tmp/full-test.log     # must equal the target count (307 today)
grep -E '^test result: FAILED' /tmp/full-test.log
```

Run it in the background and poll; never foreground it. Record the three numbers plus the exact
command and env in `KNOWN-ISSUES.md` as a dated entry, and treat any other invocation as advisory.

**Warning signs:**
- Log has no `test result:` line for every target (count < expected).
- Log ends mid-target with no `Running …`/`test result:` pair.
- The recorded result has no command line, no date, no target count.
- Someone says "the last run was green" while quoting a run from a different commit.

**Phase to address:** the test-suite-truth phase (first).

**Blocker for freeze:** **YES.** "Zero failures" is unprovable without a completed run.

---

### Pitfall 3: Zero failures achieved by weakening the suite

**What goes wrong:**
The two known failures are `xai-grok-sandbox --test deny_paths_e2e`
(`read_deny_empty_set_verifies_inside_bwrap`, `devbox_genuine_reexec_applies_enforcement`), caused
by `bwrap` failing on a root-owned `/run/containerd`. The cheap path to "zero failures" is to
widen `skip_if_enforcement_unavailable` to swallow `EACCES`, add `#[ignore]`, or delete the tests.
All three make the suite green while removing the assertion that guards **fail-closed sandbox
startup** — the exact security property the milestone must preserve.

There is a second, subtler variant: `KNOWN-ISSUES.md` records that three `xai-grok-shell`
credential tests and `xai-grok-hooks::validate_hook_url_accepts_and_rejects` fail only under
specific host conditions (an exported `DEEPSEEK_API_KEY`; fake-IP DNS resolving
`nonexistent.invalid` to `198.18.0.1`). "Fixing" those by loosening the assertions converts a
host quirk into a permanent blind spot.

**Why it happens:**
The goal is stated as a number ("zero failures"), and the cheapest way to hit a number is to
change what is counted. Two prior handoffs already over-claimed completion, so there is local
precedent for accepting a green number at face value.

**How to avoid:**
- Classify each failure into exactly one of: **real regression** (fix), **host-environment**
  (documented allowlist keyed on *test name + exact error string*, never a blanket skip), or
  **test defect** (fix the test, e.g. the `git_gate`/`restore_fetch` wall-clock joins that should
  use a controllable clock).
- For the bwrap pair: fix the *product* condition instead — skip candidates whose parent is not
  traversable when materializing runtime-socket denies (see Pitfall 12) — and add a test that a
  root-only parent does not abort `bwrap`.
- Any change to a skip predicate or an `#[ignore]` in the freeze phase requires a reviewer note
  saying which assertion is lost and what replaces it.
- Keep the mutation-check habit already used once: break the behavior deliberately and confirm a
  test goes red (`KNOWN-ISSUES.md` "Step 6 mutation check").

**Warning signs:**
- Diff in the freeze branch touches `skip_if_enforcement_unavailable`, `#[ignore]`, or asserts
  being relaxed.
- The failure count drops while the `test result:` line shows fewer tests *run* than before
  (e.g. `31,939 passed` → `31,93x passed` with a matching drop in executed tests).
- `cargo test -p xai-grok-sandbox --test deny_paths_e2e -- --list` still lists the tests but the
  run reports them as ignored/skipped.

**Phase to address:** test-suite-truth phase; the sandbox product fix belongs to the sandbox phase.

**Blocker for freeze:** **YES** (integrity of the central claim).

---

### Pitfall 4: The gate command does not run the tests that guard the docs you are reconciling

**What goes wrong:**
`config-docs` is **not** in `xai-grok-shell`'s default features — the crate declares
`default = []` and puts `config-docs` only in the `default-bazel` set
(`crates/codegen/xai-grok-shell/Cargo.toml`). So `cargo test --workspace` never compiles or runs
the `config_docs` tests that check
`crates/codegen/xai-grok-pager/docs/user-guide/26-config-reference.md` against the live
`FEATURES` / `UNMIRRORED_BOOLEAN_FEATURES` / `KNOWN_MCP_SERVER_FIELDS` registries.
The same feature-gated class of test is exactly what `KNOWN-ISSUES.md` warns about:
"workspace feature unification compiles feature-gated tests that per-crate runs miss (the
`xai-fast-worktree` case)".

Consequence: the milestone can declare "zero failures" **and** "docs in line with the tree" while
the only automated guard between the registry and the hand-edited config page was never executed.

**Why it happens:**
`cargo test --workspace` is assumed to mean "everything". Feature-gated tests are invisible to
that assumption, and the `[features] default = []` line is far from the release checklist.

**How to avoid:**
Add a second explicit gate command to the freeze evidence:

```sh
cargo test -p xai-grok-shell --lib --features config-docs -j 1
```

and record its result separately. Treat the pair (workspace + config-docs) as the definition of
"green". Do not make `config-docs` a default feature to save a command — that changes the
workspace build for everyone mid-freeze.

**Warning signs:**
- Evidence lists only `cargo test --workspace`.
- `grep -n 'config_docs' /tmp/full-test.log` returns nothing.
- `26-config-reference.md` changed in the freeze branch but no `config_docs` run is recorded.

**Phase to address:** test-suite-truth phase; re-check in the docs-reconciliation phase.

**Blocker for freeze:** **YES** for the docs claim; the code claim is unaffected.

---

### Pitfall 5: The macOS arm64 timeout is a cold-cache loop, not a slow build

**What goes wrong:**
`build` has `timeout-minutes: 90` (`.github/workflows/release.yml:101`) and
`aarch64-apple-darwin` on `macos-15` has never finished across rc2–rc5. The instinct is to raise
the timeout. That usually does not help, because a timed-out job is cancelled **before** its
post-steps run, so `Swatinem/rust-cache` never saves — every retry starts cold again and hits the
same wall. The build is also the heaviest configuration the repo has: `release-dist` is
`lto = "thin"` + `codegen-units = 1` (`Cargo.toml`), which the profile comment itself calls
"~2.2x" slower than plain `--release`, plus `debug = 1` with `split-debuginfo = "off"` (DWARF
embedded in the binary) and `strip = false`. On a 3-core Apple runner that is a very long link.

**Why it happens:**
Four independent costs stack (cold cache, LTO, codegen-units=1, embedded debug info), and the
timeout hides which one dominates. Raising the ceiling makes the red run 3 hours instead of 90
minutes without changing the outcome.

**How to avoid:**
1. **Measure before changing.** Read the macOS job log and find which step consumed the time
   (compile vs link vs `brew install protobuf` vs rustup). Note whether `Restore cargo cache`
   reported a hit.
2. **Break the cache deadlock without touching the timeout:** raise `timeout-minutes` *once*, with
   `cache-on-failure: true` already present (it is, line 179), so the first long run can save a
   cache for the next one. Accept one ~2-hour run as a cache-warming run, not as a release run.
3. **Cut the work, not the ceiling:** the shipped macOS artifact does not need `debug = 1`. A
   darwin-only profile (or a post-link `strip` + re-sign step, see Pitfall M3) removes embedded
   DWARF from the link and from the tarball.
4. **Do not ship a shorter-but-untested artifact.** Whatever changes, the existing
   `Smoke-check the built binary` step (lines 192–206) must still run **on the packaged binary**
   on the target machine.

**Warning signs:**
- macOS job log shows no cache-hit line for `aarch64-apple-darwin` on successive runs.
- The 90-minute cancel lands during `Linking` (not during compile).
- `x86_64-apple-darwin` finishes but `aarch64-apple-darwin` does not — asymmetry points at runner
  capacity or a target-specific link, not at the code.
- A proposed "fix" is only a larger `timeout-minutes` value.

**Phase to address:** release-pipeline phase.

**Blocker for freeze:** **YES** — the milestone requires both macOS arches and a green workflow.

---

### Pitfall 6: The container job is hostage to the macOS leg

**What goes wrong:**
`container` declares `needs: [version, build]` (line 349). `build` is the whole 4-target matrix,
so **any** leg failing or timing out skips the container job — which is exactly what happened on
rc4 and rc5 ("no GHCR image exists", `KNOWN-ISSUES.md`). The container only depends on the two
Linux tarballs, which have succeeded on every rc that got that far. A macOS problem therefore
erases an unrelated deliverable, and the milestone's two requirements ("macOS green" and
"container proven") become a single point of failure.

**Why it happens:**
`needs:` is job-level, not matrix-leg-level. The natural way to write "after the builds" is
`needs: build`, which silently couples all artifacts to the slowest platform.

**How to avoid:**
Split the matrix by platform so the container can depend on only what it consumes:
- `build-linux` (2 legs) → feeds `container` and `publish`
- `build-macos` (2 legs) → feeds `publish`
- `container: needs: [version, build-linux]`
- `publish: needs: [version, build-linux, build-macos]` (still requires all four tarballs)

This is a workflow edit, so read Pitfall M6 first (credential scope) and re-run `actionlint`.

**Warning signs:**
- A run where Linux tarballs exist as artifacts but no GHCR image was pushed.
- `gh run view <id>` shows `container` as "skipped" with the reason "needs build".
- The container job has never appeared in any run's job list.

**Phase to address:** release-pipeline phase, before any throwaway tag.

**Blocker for freeze:** **YES** for the container requirement.

---

### Pitfall 7: Tag shape is a six-way contract, and the freeze tag is the odd one out

**What goes wrong:**
Changing the release tag shape touches six independent consumers. Adding `'deepseek-v*'` to the
trigger without changing the rest produces a cascade of failures — and one dangerous success:

| # | Consumer | Location | Behaviour with `deepseek-v0.1.0` |
|---|----------|----------|--------------------------------|
| 1 | Trigger | `release.yml:42` `tags: ['v*']` | Does **not** fire today. Adding `'deepseek-v*'` makes the freeze tag a release trigger. |
| 2 | Version derive | `release.yml:72-79` `case … v?*)` | `::error::release tags must match 'v*'` → version job fails in seconds. |
| 3 | Prerelease classification | `release.yml:86` `^[0-9]+\.[0-9]+\.[0-9]+-` | `deepseek-v0.1-snapshot` → `is_prerelease=false` → release is **not** marked prerelease and **`:latest` moves**. |
| 4 | Installer | `install.sh:278-281` `v[0-9]*` | `VERSION=deepseek-v0.1.0` → `Invalid VERSION`. |
| 5 | Container tags | `release.yml:406-410` `type=semver` | Non-semver tag yields no semver tags; `imagetools inspect ":${version}"` (line 462) then fails after the image was already pushed. |
| 6 | Tarball naming / latest resolution | `release.yml:243`, `install.sh:156-161` | `deepseek-build-deepseek-v0.1.0-<triple>.tar.gz`; `version_from_tarball` returns `deepseek-v0.1.0`, then `VERSION_TAG="vdeepseek-v0.1.0"` → 404. |

The dangerous case is #3: if the freeze tag is `deepseek-v0.1-snapshot` **and** the trigger learns
`deepseek-v*`, pushing the freeze tag publishes a **non-prerelease "Latest release"** and moves
the container `:latest` tag to a snapshot.

**Why it happens:**
Tag shape looks like a one-line regex change. It is actually a protocol shared by CI, the
installer, the container tagging scheme, and the release notes semantics, with no single test
covering all six.

**How to avoid:**
- **Separate the two concepts.** The *freeze* tag (`deepseek-v0.1-snapshot`, on branch
  `deepseek`) is a git marker and must **not** be a release trigger. The *publish* tag
  (`v0.1.0`) is what `release.yml` consumes. Keeping `tags: ['v*']` unchanged achieves this today
  and requires no workflow edit.
- If a `deepseek-` prefixed *publish* tag is genuinely wanted, change all six sites in one commit
  and add: a unit-ish test for the derive step (bash snippet with `GITHUB_REF_NAME` fixtures), a
  `scripts/test-install.sh` case with a `deepseek-` tag, and a `metadata-action` tag assertion.
- Add an explicit guard in the `version` job that refuses `*snapshot*` tags.

**Warning signs:**
- The tag pattern in `release.yml` differs from the tag you intend to push.
- `is_prerelease` is `false` for a tag containing `-snapshot` or `-rc`.
- `install.sh` prints `Invalid VERSION` for a tag the release actually produced.
- `docker buildx imagetools inspect` fails with `manifest unknown` right after a successful push.

**Phase to address:** version/tag-shape phase, **before** the pipeline is exercised by any tag.

**Blocker for freeze:** **YES** — a wrong shape either blocks the freeze tag or publishes it.

---

### Pitfall 8: The cross-compile / Rosetta "shortcut" silently deletes the only runtime check

**What goes wrong:**
The milestone asks for Rosetta/QEMU and cross-compile to be evaluated. Two facts settle it:
- **Rosetta cannot produce an arm64 artifact.** Rosetta 2 translates x86_64 → arm64 execution on
  Apple Silicon; it does not let an Intel runner build or run arm64. Using it to "get macOS green"
  is a category error.
- **Any cross-compile removes the ability to execute the artifact**, and the workflow's macOS
  correctness gate *is* execution: `Smoke-check the built binary` runs
  `target/release-dist/deepseek-build --version` and asserts both the stamped version and the
  `deepseek-build <ver> (<commit>)` shape (lines 192–206). A cross-compiled arm64 binary cannot
  run on an x86_64 host, so the step is either skipped (gate deleted) or made conditional
  (gate weakened), and the tarball ships unverified.

There is a second-order risk specific to macOS: an arm64 Mach-O needs at least an ad-hoc code
signature, and post-link `strip`/modification invalidates it (**UNVERIFIED** — recall, not
checkable here). A binary that links fine and fails at `exec` on the user's Mac is exactly the
class of failure the runtime smoke check exists to catch.

**Why it happens:**
Cross-compiling looks like a way to avoid a slow native runner, and the smoke step is treated as
a nicety rather than as the platform's only correctness gate.

**How to avoid:**
- **Keep native runners.** The four-target matrix already does this deliberately (the workflow
  header states "no cross-compilation, no qemu"). Fix time via Pitfall 5 instead.
- If a cross-compile is ever adopted for a *third* platform, replace the runtime smoke check with
  a structural one (`file`, `lipo -info`, `codesign -dv`) **and** record in `KNOWN-ISSUES.md` that
  the binary was never executed before shipping.
- If you strip on macOS, re-sign ad-hoc afterwards and run `--version` on the binary **extracted
  from the final tarball**, not from `target/`.

**Warning signs:**
- A matrix entry whose runner arch ≠ target arch.
- `Smoke-check the built binary` gaining an `if:` condition.
- macOS job "green" with no `binary reports: …` line in the log.
- Tarball produced by a job that never executed the binary.

**Phase to address:** release-pipeline phase (as an explicit rejection, recorded in `DECISIONS.md`).

**Blocker for freeze:** **YES** if adopted as the fix; otherwise a recorded non-decision.

---

### Pitfall 9: "Container proven" measured by `--version`, and the sandbox fails closed inside Docker

**What goes wrong:**
The tempting proof is `docker run --rm <image> --version` → exits 0 → "container works". That
proves the dynamic loader and glibc. It does not prove the product. Two specific gaps:

1. **The sandbox is on by default and `bwrap` generally cannot create user namespaces under the
   default Docker seccomp profile** (**UNVERIFIED** mechanism, but the project's own recorded
   behaviour is that the sandbox *fails closed* rather than degrading:
   `CONCERNS.md` — "sandbox startup fails closed rather than skipping the unreadable socket — that
   is safer than a partial sandbox, but it means 'sandbox on' cannot start on this class of
   host"). A container is precisely that class of host. So the first real session that runs a
   shell command inside the image can die at sandbox startup, while `--version` is green.
2. **`~/.deepseek-build` inside the container is ephemeral** and `WORKDIR /work` is a mount point
   that may not exist on the host; sessions, config, and `.grok/` project state have nowhere to
   live unless the user mounts them.

**Why it happens:**
`--version` is the cheapest command that proves "the image runs", and the image is assembled from
tarballs that were already smoke-checked on the host. The sandbox dependency is invisible until a
tool executes.

**How to avoid:**
Define "container proven" as **a headless turn that executes a tool**, not `--version`:
```sh
docker run --rm -e DEEPSEEK_API_KEY -v "$PWD:/work" <image> \
  -p "run the command 'echo ok' and report the output" --always-approve
```
Record the exact command and output. Document the sandbox interaction explicitly (either
`DEEPSEEK_BUILD_SANDBOX=off` for the container with a stated reason, or the required
`--security-opt`), and document the mounts for config/sessions. Do not silently flip the sandbox
default in the image — the project decision is "sandbox stays on by default".

**Warning signs:**
- Container evidence consists only of `--version`.
- First in-container tool call fails with a sandbox/bwrap/namespace error.
- README GHCR example has no `-v` mount and no sandbox note.
- Image size/uid assumptions differ between `docker/Dockerfile` (`USER 1000:1000`) and the run
  command in the docs.

**Phase to address:** container/installer-proof phase.

**Blocker for freeze:** **YES** for the "container proven" requirement.

---

### Pitfall 10: The installer's `latest` path cannot work while every release is a prerelease

**What goes wrong:**
`install.sh` with no `VERSION` fetches `…/releases/latest/download/SHA256SUMS`
(`install.sh:263-272`). GitHub's `/releases/latest` resolves to the newest **non-prerelease**
release. Every release this repo has published (rc2–rc5) is a prerelease, and `release.yml` marks
any `X.Y.Z-*` tag as prerelease (`release.yml:86-90`). So the documented no-argument install path
resolves to *nothing* today.

`scripts/test-install.sh` cannot catch this: `build_fake_tree` **creates** a
`latest/download/` directory and populates it (`scripts/test-install.sh:144-150`). The harness
asserts a behaviour GitHub does not provide when all releases are prereleases. That is the
canonical "green against a fake that is kinder than reality".

**Why it happens:**
The fake tree models the URL *layout* faithfully but not the *release-selection semantics*.
`latest` is treated as a directory, not as "newest non-prerelease".

**How to avoid:**
- Prove the installer with **`VERSION=` pinned** for every throwaway tag
  (`VERSION=v0.1.0-rc0 sh install.sh`), and state in the README that an unpinned install requires
  a non-prerelease release to exist.
- Add one real-network check before the freeze:
  `curl -fsSL -o /dev/null -w '%{http_code}\n' https://github.com/coruairc/deepseek-build/releases/latest/download/SHA256SUMS`
  — a non-200 confirms the unpinned path is currently broken and must be documented as such.
- Keep the fake-tree `latest` case, but label it in the harness as "layout only; does not model
  prerelease exclusion" so nobody cites it as proof.

**Warning signs:**
- README one-liner uses `releases/latest` while `gh release list` shows only prereleases.
- Installer evidence is only `scripts/test-install.sh`.
- The `latest` resolution logs `Latest release: v…` in the harness but 404s against GitHub.

**Phase to address:** container/installer-proof phase.

**Blocker for freeze:** **NO** if the pinned-`VERSION` path is proven and the limitation is
documented; **YES** if the README promises the unpinned one-liner.

---

### Pitfall 11: Wiring `route_turn` without one decision point per turn

**What goes wrong:**
`ModelRoutingPolicy` defaults to `Off`, and every shell construction site passes
`model_routing: Default::default()` (`crates/codegen/xai-grok-shell/src/tools/config.rs`,
`src/agent/config.rs`, `src/agent/subagent/mod.rs`, `src/session/acp_session_impl/sampler_turn.rs`).
Wiring it in invites four distinct failures:

1. **Double-routing** — the decision is computed in the turn loop *and* re-computed in the status
   line or request assembly, so the model shown is not the model requested.
2. **Per-call routing inside a tool chain** — the recorded live run is `num_turns=5, modelCalls=5`.
   Routing per *model call* switches model mid-chain, invalidating the prefix cache that produced
   the 97–99% hit rate in `docs/TUNING.md`.
3. **Stale pin** — `/model` or `/effort` sets a session pin, but the routing policy reads a
   snapshot taken before the pin, so routing overrides a pin (explicitly forbidden by the
   project decision "routing never overrides a pin").
4. **Cache-cost inversion** — the entire justification for routing is Flash-vs-Pro savings; a
   mid-session switch that drops cache hit rate can cost *more* than staying on Pro.

**Why it happens:**
`route_turn` is a method on `SamplingClient`, which makes it look like a per-call concern. It is
actually a per-turn policy decision. The pin state lives in session/config layers that the sampler
does not own.

**How to avoid:**
- Compute the decision **once, at turn start**, in `sampler_turn.rs`, and pass it down as a value
  (the chosen model) rather than re-deriving it. Keep `route_turn`'s pure `resolve_route` function
  as the only policy logic.
- Make the pin an explicit input to the decision function with a test: policy `On` + pin set →
  pinned model wins, for every policy variant.
- Add a wiremock test asserting the outbound request body's `model` equals the value the status
  line reports for that turn.
- Decide and record whether routing applies per turn or per model call; the cache economics argue
  for per turn.

**Warning signs:**
- Two call sites for `route_turn` (or one in `sampler_turn.rs` and one in a view/status module).
- `usage` cache-hit ratio drops after enabling routing.
- Status line model ≠ the `model` field in the request (visible with the sampler's debug log or a
  wiremock capture).
- A pinned session changes model mid-conversation.

**Phase to address:** routing phase (before the sandbox phase, because both touch session state).

**Blocker for freeze:** **YES** for the routing requirement; it is a correctness and trust issue,
not a cost issue.

---

### Pitfall 12: Relaxing runtime-socket deny materialization widens a fail-closed boundary

**What goes wrong:**
The planned sandbox fix is "materialize runtime-socket denies only where the parent path is
traversable, fail-closed everywhere else". The failure mode is over-broad skipping. Current code
(`crates/codegen/xai-grok-sandbox/src/lib.rs` `bwrap_reexec_command_ex`) `--ro-bind`s deny paths
and refuses to start on a partial sandbox. If the traversal check is written as "if anything about
this path is not accessible, skip it", then a deny the process *can* see but cannot mask is
dropped, and the socket becomes **visible inside the sandbox** — a silent weakening of the one
security property the milestone is supposed to preserve. The same trap exists one level up: the
two failing tests are the only thing currently asserting fail-closed startup, so a fix that also
touches `skip_if_enforcement_unavailable` can make the suite green by not testing.

**Why it happens:**
The observed error is `EACCES` on a *parent directory* (`/run/containerd`), and the natural patch
is "treat EACCES like NotFound". But EACCES on the parent (cannot traverse) and EACCES on the
target (can traverse, cannot read) have opposite correct answers: skip vs deny.

**How to avoid:**
- Gate the skip on **`access(parent, X_OK)` / traversal only**, never on the target's readability.
  Keep `NotFound` and non-traversable-parent as the only skip reasons.
- Keep the fail-closed path for every deny whose parent *is* traversable but whose mask cannot be
  applied.
- Add a pure unit test over the "which denies to materialize" function with fixtures: missing
  parent, root-only parent, traversable-but-unreadable target, normal file. Assert the decision for
  each.
- Add one test that enumerates the materialized deny set inside `bwrap` and asserts a known
  traversable deny is present.
- Review rule: a diff touching `skip_if_enforcement_unavailable` in the freeze branch is a
  stop-and-ask.

**Warning signs:**
- The materialized deny set shrinks on the author's host (log the count before/after).
- `skip_if_enforcement_unavailable` gains a new condition, or the two `deny_paths_e2e` tests report
  "skipped" rather than "ok".
- A deny that used to appear in the `bwrap` command line disappears from it.
- The fix's justification is "the tests now pass" rather than "this host cannot mask that path".

**Phase to address:** sandbox phase.

**Blocker for freeze:** **YES** — this is the security invariant, and the project decision says
sandbox stays on by default with fail-closed behaviour everywhere else.

---

## Moderate Pitfalls

### Pitfall M1: The version bump touches 95 manifests and three other version sources

**What goes wrong:** Bumping `0.0.1-rc5` → `0.1.0` across a 95-member workspace leaves drift:
`Cargo.lock` versions, the `xai-grok-version` crate's fallback string, and the fact that release
builds take the version from `GROK_VERSION` (stamped by `release.yml:188`) while a source build
prints the manifest version. A partial bump yields a binary whose `--version` disagrees with the
tag, and `release.yml`'s smoke check only validates the *stamped* path.

**How to avoid:** Bump with `cargo set-version`/`cargo release`-style tooling or a scripted
`[workspace.package] version` if one exists; then assert both paths:
`cargo build -p xai-grok-pager-bin && ./target/debug/deepseek-build --version` (manifest path) and
one CI-stamped build. Commit the bump separately from behavior changes so the 95-file diff is
reviewable.

**Warning signs:** `git diff --stat` shows a version bump mixed with code; `--version` from a
source build still says `0.0.1-rc5`; `Cargo.lock` unchanged after the bump.

**Phase to address:** version/tag-shape phase. **Blocker:** YES (a wrong stamp fails the release
smoke check, or ships a mislabeled binary).

---

### Pitfall M2: The installer harness only ever exercises the host triple

**What goes wrong:** `build_fake_tree` builds a tarball for `$(host_target)` only
(`scripts/test-install.sh:139-147`), so on the author's x86_64 Linux the `Darwin` branches of
`map_target`, the `shasum` fallback in `sha256_of`, and the macOS tarball-name path are never
executed. The 39/39 result is a Linux-x86_64 result.

**How to avoid:** Extend the fake tree to contain all four triples and run the success case four
times with `DSB_INSTALL_TEST_UNAME_S`/`_M` overrides (the hooks already exist,
`install.sh:50-60`). Assert each resolves the right tarball name. The `shasum` fallback can be
forced by running with a `PATH` that shadows `sha256sum`.

**Warning signs:** Harness header prints only one host target; no test name mentions
`aarch64-apple-darwin`; the macOS install path is only proven by the release run itself.

**Phase to address:** container/installer-proof phase. **Blocker:** NO (documented risk) — but it
is the cheapest way to avoid a macOS-only install break.

---

### Pitfall M3: `release-dist` ships unstripped debug info, and stripping on macOS is a trap

**What goes wrong:** `[profile.release-dist]` sets `strip = false`, `debug = 1`,
`split-debuginfo = "off"`, with a comment saying CI can "extract .debug sidecars … before
stripping post-build". `release.yml` has **no such step**. So every tarball carries embedded DWARF:
larger uploads, slower links (feeding Pitfall 5), and symbol/path disclosure in a distributed
binary. Conversely, adding a naive post-build `strip` on macOS arm64 can invalidate the ad-hoc
code signature (**UNVERIFIED** — recall) and produce a binary that fails at `exec`.

**How to avoid:** Decide explicitly. Either (a) accept unstripped for v0.1.0 and say so, or
(b) add a documented strip step: Linux `strip --strip-debug` (keep `.symtab` if wanted), macOS
`strip -x` **followed by** `codesign -s - --force`, then re-run the `--version` smoke check on the
binary extracted from the final tarball. Never strip without the post-strip execution check.

**Warning signs:** Tarball size ≫ the recorded 147 MiB `--release` figure; `file`/`nm` shows full
symbols; a macOS artifact that `--version`s from `target/` but not after extraction.

**Phase to address:** release-pipeline phase. **Blocker:** NO (size/hygiene), unless a strip step
is added carelessly.

---

### Pitfall M4: The linker wrapper makes the CI binary differ from the local one

**What goes wrong:** `[target.x86_64-unknown-linux-gnu] linker = "scripts/cargo-linker.sh"`,
which "prefers lld when present and silently falls back to cc". A local box with `lld` and a
GitHub runner without it link the same sources differently. The freeze claim is about the *CI*
artifact, so a locally verified binary is not the artifact being frozen.

**How to avoid:** Record which linker was used (the wrapper should log it; if it does not, add one
line). Treat the CI-built tarball as the only artifact that may be called "verified", and install
*that* in the final installer proof rather than a locally built binary.

**Warning signs:** No linker line in the build log; local and CI `sha256sum` differ for the same
commit (expected — but then say so); installer proof run against `target/release-dist`.

**Phase to address:** release-pipeline phase. **Blocker:** NO.

---

### Pitfall M5: `disk-guard.sh` defaults to 60 GB while the documented floor is 150 GB

**What goes wrong:** `scripts/disk-guard.sh` uses `min="${1:-60}"`, and `TESTING.md` even suggests
passing `60` on a smaller disk. `AGENTS.md` §8 and `CONCERNS.md` require 150 GB, and
`CONCERNS.md` explicitly says "this tree still requires 150 GB — keep the stricter guard until a
measured plan says otherwise". A bare `scripts/disk-guard.sh` passes at 80 GB free; a fresh
`--tests` build is ~24 GB and a second `target/` doubles it, so the guard can green-light a build
that fills the filesystem.

**How to avoid:** Always invoke `scripts/disk-guard.sh 150` in freeze evidence and in any script
that wraps the gate. Consider making 150 the default (a one-line change, and the doc comment
already says 60) — but only if the author confirms no smaller disk is in play.

**Warning signs:** Gate log shows `minimum 60 GB`; `CARGO_TARGET_DIR` unset or pointing inside a
worktree; more than one `target/` under `$HOME`.

**Phase to address:** test-suite-truth phase (first gate). **Blocker:** YES operationally — a
full disk mid-run produces a red run indistinguishable from a code failure.

---

### Pitfall M6: Any workflow edit may be rejected by GitHub before it runs

**What goes wrong:** `KNOWN-ISSUES.md` records: "a per-target timeout change needs a token with
the `workflow` scope". GitHub rejects pushes that create or modify files under
`.github/workflows/` when the pushing credential lacks that scope. The release-pipeline phase
*requires* editing `release.yml` (Pitfall 5/6), so the first fix can fail at `git push` with a
message that looks like a permissions problem, not a code problem.

**How to avoid:** Confirm the push credential's scopes **before** writing the change (or use a
credential known to have `workflow`, e.g. `gh auth login` with the `workflow` scope). If it cannot
be obtained, the pipeline fixes cannot land and the macOS/container requirements must be recorded
as blocked rather than attempted repeatedly.

**Warning signs:** Push rejected with "refusing to allow a Personal Access Token to create or
update workflow"; or a local commit that exists but never appears on `origin`.

**Phase to address:** release-pipeline phase (precondition). **Blocker:** YES if the scope is
unavailable — it blocks two milestone requirements.

---

### Pitfall M7: GNU release binaries still ship without RELRO/BIND_NOW

**What goes wrong:** `[profile.release-dist]`'s comment says linker hardening is per-target in
`.cargo/config.toml`, but the `-Wl,-z,relro,-z,now,-z,noexecstack` flags exist **only** under the
`*-unknown-linux-musl` target sections. The shipped targets are `x86_64-unknown-linux-gnu` and
`aarch64-unknown-linux-gnu`, whose rustflags are `force-unwind-tables` (plus
`target-cpu=neoverse-v2`). The comment therefore over-promises relative to the tree.

**How to avoid:** Either add the link args to both gnu sections and verify with
`readelf -l <bin> | grep GNU_RELRO` and `readelf -d <bin> | grep BIND_NOW`, or amend the profile
comment and record the gap as a documented risk. Do not leave the comment claiming a guarantee the
binary does not have — that is the same class of error as the docs drift.

**Warning signs:** `readelf -l` shows no `GNU_RELRO`; the comment and `.cargo/config.toml`
disagree; `readelf -d` lacks `BIND_NOW`.

**Phase to address:** release-pipeline phase (a decision, not necessarily a change).
**Blocker:** NO — but shipping a comment that claims hardening you do not have is a truthfulness
defect in a freeze.

---

### Pitfall M8: Runtime egress evidence is LD_PRELOAD-based, key-dependent, and easy to over-claim

**What goes wrong:** `scripts/egress-check.sh` needs a real key (`KEY="${DEEPSEEK_API_KEY:-…}"`,
and it `FAIL`s when absent) and is an `LD_PRELOAD` logger that misses raw `syscall(SYS_connect)`,
`io_uring`, static/setuid children, and children that scrub `LD_PRELOAD`; `strace` is absent.
The recorded `EGRESS_VIOLATIONS=0` is from 2026-10-07 on the author's machine. The milestone
wants the egress boundary preserved while the sandbox and routing change — and both touch the code
paths that make connections. A freeze note that says "egress verified" without saying *when, on
what commit, with what tool* repeats the over-claim pattern.

**How to avoid:** Record egress evidence as a dated triple: commit, command, result — and state
the tool's limits in the same sentence. If no key is available, record **NOT RUN** with the reason
(team rules: a check you could not run is never reported as passing). `scripts/check-egress.sh
--strict` is the static gate that *can* always run; treat it as necessary, not sufficient.

**Warning signs:** Egress claim with no date/commit; a change to `check-egress.sh`'s `HARD`/`SOFT`
patterns or `EXCLUDES` in the freeze branch (that is a stop-and-ask); "egress verified" appearing
next to a sandbox or routing change with no re-run.

**Phase to address:** final freeze evidence phase. **Blocker:** NO (documented risk) — unless a
change adds a destination, which is a hard stop.

---

## Minor Pitfalls

### Pitfall m1: `dsb` shortcut / uninstall edges
`install.sh --uninstall` removes `$INSTALL_DIR/deepseek-build` unconditionally, even if the user
replaced it; and the `dsb` symlink logic treats a dangling symlink (`_dest=""`) as "ours". Both
are acceptable for a personal tool but should be one line in the README, not a surprise.
**Prevention:** document the two behaviours; leave the harness cases as-is.

### Pitfall m2: Leftover `login` / `logout` / `/login` surface
`KNOWN-ISSUES.md` records that `login --help` still describes OAuth via `api.deepseek.com` although
the OIDC stack was deleted and auth is env-only. In a freeze, a help string that describes a
deleted flow is a trust defect, not a cosmetic one.
**Prevention:** either remove the subcommands or make `--help` say "auth is `DEEPSEEK_API_KEY`".
Cheap, and it is exactly the kind of claim the docs phase is meant to fix.

### Pitfall m3: `.grok/` project directory vs `~/.deepseek-build`
Per-repo config/skills/hooks/checkpoints still live in `<repo>/.grok/`. The decision is "keep" for
0.0.1, but a v0.1.0 user guide that says `~/.deepseek-build` everywhere without mentioning
`.grok/` will read as a rebrand that is incomplete. **Prevention:** one explicit paragraph; do not
rename anything (out of scope).

### Pitfall m4: Cost is always the peak rate
`estimated_cost_usd` uses the peak table with no clock, so off-peak is overstated 2x. Documented
in D14 and `CONCERNS.md`. **Prevention:** keep `estimate: true`; do not "fix" it by fetching
prices (new outbound destination).

### Pitfall m5: The annotated tag message makes a claim that will become false
`deepseek-v0.1-snapshot`'s message says "Linux-only v0.0.1-rc5". If that ref is reused as the
freeze marker, the annotation documents the wrong artifact. **Prevention:** covered by Pitfall 1's
decision; if the tag is kept, its message must be rewritten (which means moving the tag).

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| `#[ignore]` the bwrap e2e tests to reach zero | Green suite today | Deletes the only fail-closed sandbox assertion; the invariant becomes untested | **Never** in a freeze |
| Raise `timeout-minutes` as the macOS "fix" | Run no longer red at 90 min | 3-hour red runs; cache still cold; cause unmeasured | Only as a one-time cache-warming step, with the cause recorded |
| Point the installer proof at a locally built binary | No tag needed | The verified artifact is not the shipped artifact (Pitfall M4) | Never for the freeze claim |
| `docker run --version` as container proof | Seconds | Sandbox/tool path unverified; first user session can die | Only as a first smoke, never as the evidence |
| Add `'deepseek-v*'` to the trigger to "support" the freeze tag | One-line change | Freeze tag becomes a release trigger; `:latest` and "Latest release" move (Pitfall 7) | **Never** without the other five sites |
| Widen `skip_if_enforcement_unavailable` | Two failures vanish | Host-dependent skip becomes a permanent blind spot | Never without a replacement assertion |
| Document "egress verified" from a prior session | No key needed | Over-claim; two prior handoffs already did this | Only with date + commit + tool + limits |
| Skip the `config-docs` run because the workspace suite is green | One fewer command | The hand-edited config page loses its only guard | Never during a docs reconciliation |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| GitHub Releases `latest` | Assuming `/releases/latest` resolves while only prereleases exist | Prove with pinned `VERSION=`; check the real URL's HTTP status |
| GHCR | Assuming a pushed package is anonymously pullable. Package visibility is independent of repo visibility and defaults to private (**UNVERIFIED** — recall) | `docker logout ghcr.io` then `docker pull` the tag; if it needs auth, flip visibility and re-check |
| GHCR | Assuming `packages: write` is sufficient | It is scoped correctly here (`release.yml:353-355`, workflow-level `permissions: {}`); if login/push 403s, the failure is repo/package policy, not the YAML |
| `docker/metadata-action` | Assuming `type=semver` produces tags for a non-semver ref | Non-semver tags yield only `type=ref` tags; the final `imagetools inspect ":${version}"` then fails *after* the push |
| `actions/upload-artifact` + `needs: build` | Assuming per-leg dependencies | `needs` is per job; split the matrix by platform (Pitfall 6) |
| `Swatinem/rust-cache` | Assuming a cache exists after a timeout | Verify a cache-hit line in the log; treat the first long run as cache warming |
| `bwrap` inside Docker | Assuming the sandbox works because `--version` does | Run a tool-executing headless turn inside the image (Pitfall 9) |
| `softprops/action-gh-release` | Assuming `prerelease` is inferred from the tag | It is passed explicitly (`release.yml:328`); the derivation is the only thing keeping `:latest` stable |
| `tar` with member selection | Assuming GNU tar semantics on macOS | `tar -xzf … deepseek-build` works with bsdtar, but only the `install.sh` fake-tree harness would catch a regression, and it never runs on macOS |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Full workspace run on a 16 GB box without `-j` | OOM-killed rustc/test binary, or a run that never prints a summary | `-j 2`, `--test-threads=4`, one `flock` build lock, background + poll | Immediately at 307 targets with 64 MB stacks |
| Second `target/` (worktree or unset `CARGO_TARGET_DIR`) | Disk drops ~24 GB per copy; guard fails mid-build | One shared `CARGO_TARGET_DIR`; `scripts/disk-guard.sh 150` | On the second build |
| Measuring `release-dist` locally | Thin LTO at `-j 1` exceeds any reasonable timeout; fat LTO OOMs | Do size measurement in CI, or `-j 1` with a hard time box and no fat LTO | The moment `codegen-units = 1` is compiled on ≤16 GB |
| Cold macOS cache after every timeout | Same 90-minute wall every run | Cache-warming run; cut `debug=1` from the darwin link | Every retry |
| `git status` walks in a saturated workspace suite | `git_gate`/`restore_fetch` flake | Run the crate in isolation before calling it a regression; fix the test clock rather than raising the timeout | Whole-suite saturation |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Skipping a deny path whose *target* is unreadable | The socket becomes visible inside the sandbox — silent weakening of fail-closed | Gate the skip on parent traversal only (Pitfall 12); unit-test the decision function |
| Widening `skip_if_enforcement_unavailable` to hide `EACCES` | Fail-closed startup stops being asserted | Stop-and-ask on any change to that predicate in the freeze branch |
| Adding an outbound destination to gain confidence (telemetry, version check, price fetch) | Violates D6/D10/D13; unfreezes the core guarantee | Explicitly out of scope; any such change is stop-and-ask |
| Editing `check-egress.sh`'s `HARD`/`SOFT`/`EXCLUDES` to clear a hit | The static gate stops meaning anything | Treat allowlist edits as stop-and-ask |
| `strip` after signing on macOS arm64 | Binary may fail at `exec` on the user's machine (**UNVERIFIED**) | Re-sign ad-hoc, then `--version` the extracted tarball binary |
| Shipping `debug = 1` symbols | Path/symbol disclosure in a distributed binary | Decide explicitly (Pitfall M3) |
| Recording "egress verified" from an old session | False security claim in the freeze record | Date + commit + command + tool limits, or NOT RUN |
| Unpinned plugin/marketplace sources | Branch tip changes under an install | Not in freeze scope; do not enable by default |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Routing changes the model silently | User cannot tell why cost/quality changed | Status line shows the routed model every turn; `/model` pins and is visibly pinned |
| Pin appears set but routing overrides it | Trust loss; the documented control does nothing | Test the pin against every policy variant; show "pinned" in the status line |
| Container docs show `docker run` with no mounts | Sessions/config vanish on exit | Document `-v` for repo, config, and `.grok/` |
| README one-liner fails because no non-prerelease exists | First impression is a 404 | Pin `VERSION=` in the README until a stable release exists |
| `login --help` describes OAuth that does not exist | User tries a flow that cannot work | Rewrite the help text to name `DEEPSEEK_API_KEY` |
| `dsb` shortcut silently not created (foreign file present) | User cannot run `dsb` and does not know why | The installer already warns; repeat it in the README |

## "Looks Done But Isn't" Checklist

- [ ] **Full suite green:** the log has one `test result:` line per target (count == 307) and no
      `FAILED` line — not just a quiet tail. Verify: `grep -c '^test result:' /tmp/full-test.log`.
- [ ] **Docs-sync guard ran:** `config-docs` is off by default, so verify a separate
      `cargo test -p xai-grok-shell --lib --features config-docs` run exists.
- [ ] **Sandbox still fail-closed:** verify the two `deny_paths_e2e` tests report *ok* (not
      ignored/skipped) and that the materialized deny set did not shrink.
- [ ] **macOS artifact exists and ran:** verify the release has an `aarch64-apple-darwin` tarball
      **and** the job log contains `binary reports: deepseek-build <ver> (<sha>)`.
- [ ] **Container proven:** verify a headless turn that executes a shell command inside the image,
      not `--version`.
- [ ] **Installer proven from the real release:** verify a pinned `VERSION=` install from the
      throwaway tag, then `deepseek-build --version` prints that tag's version.
- [ ] **Tag shape consistent:** verify `git ls-remote --tags origin` has no unintended `v*` tag,
      and the freeze tag did not trigger a workflow run (`gh run list --workflow=release.yml`).
- [ ] **Freeze tag points at the frozen tree:** verify
      `git rev-parse <tag>^{} == git rev-parse <freeze-branch>`.
- [ ] **Version stamped everywhere:** verify a source build's `--version` and the CI-stamped
      `--version` both report `0.1.0`.
- [ ] **Docs describe the tree:** verify every state claim in `DECISIONS.md` / `TESTING.md` /
      `KNOWN-ISSUES.md` has a command + date, and that no claim contradicts `git tag` / `gh release
      list`.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Freeze tag on the wrong commit | LOW if caught early / HIGH if already reused | Decide new tag name (preferred) or move it with explicit approval; re-verify `rev-parse`; record supersession |
| Green from an unfinished run | LOW | Re-run the scripted gate to completion; record command + numbers + commit |
| Suite weakened to reach zero | MEDIUM | Revert the skip/ignore; fix the product condition; add the replacement assertion; re-run |
| macOS timeout persists | MEDIUM | Cache-warming run; cut `debug=1` from darwin; only then consider a larger timeout |
| Container skipped by macOS failure | LOW (after split) | Split the matrix; re-run with a throwaway `vX.Y.Z-rcN` tag |
| `:latest` moved by a snapshot tag | MEDIUM | Re-point `:latest` with `imagetools create` to the last intended digest; mark the release prerelease; fix the classification |
| Installer 404 on the real release | LOW | Use pinned `VERSION=`; document; fix the README |
| Routing changed model mid-session | MEDIUM | Move the decision to turn start; re-run the cache-hit measurement; verify the pin test |
| Sandbox deny dropped | HIGH | Revert the traversal predicate; restore the fail-closed path; add the enumerate-deny test |
| Workflow edit rejected for scope | LOW (if a scoped credential exists) / HIGH otherwise | Use a `workflow`-scoped credential; if impossible, record the requirement as blocked |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase (provisional) | Verification |
|---------|-------------------------------|--------------|
| 1 Stale freeze tag | Version/tag-shape phase (decision) → freeze phase (execution) | `git rev-parse <tag>^{}` equals the freeze-branch tip; tag appears in no release run |
| 2 Green from unfinished run | Test-suite-truth phase | Gate script exits 0; `grep -c '^test result:'` equals the target count |
| 3 Suite weakened | Test-suite-truth + sandbox phases | Diff review: no new skip/ignore; mutation check still red |
| 4 `config-docs` not run | Test-suite-truth + docs phases | Separate `--features config-docs` run recorded |
| 5 macOS timeout | Release-pipeline phase | `gh run view`: macOS leg completed; `binary reports:` line present |
| 6 Container hostage to macOS | Release-pipeline phase | Run shows `container` executed while macOS was red (or both green) |
| 7 Tag-shape contract | Version/tag-shape phase | Freeze tag triggers no run; `install.sh` accepts the publish tag; `imagetools inspect` succeeds |
| 8 Cross-compile shortcut | Release-pipeline phase (rejection recorded) | Matrix runner arch == target arch for every leg |
| 9 Container proof by `--version` | Container/installer-proof phase | Headless tool-executing turn transcript committed |
| 10 Installer `latest` path | Container/installer-proof phase | Pinned install from the real release; `curl -w '%{http_code}'` on `latest` recorded |
| 11 Routing wiring | Routing phase | Pin test; wiremock `model` == status-line model; cache-hit ratio held |
| 12 Sandbox deny relaxation | Sandbox phase | Decision-function unit tests; deny-set enumeration test; e2e tests *ok* not skipped |
| M1 Version bump | Version/tag-shape phase | Source `--version` and CI `--version` both `0.1.0` |
| M2 Installer host-only harness | Container/installer-proof phase | Four-triple success cases in `test-install.sh` |
| M3 Unstripped debug info / macOS strip | Release-pipeline phase | Explicit decision recorded; if stripped, extracted binary `--version`s |
| M4 Linker drift | Release-pipeline phase | Linker line in the build log; installer proof uses the CI artifact |
| M5 Disk guard default | Test-suite-truth phase | Gate log shows `minimum 150 GB` |
| M6 Workflow scope | Release-pipeline phase (precondition) | `release.yml` change appears on `origin` |
| M7 RELRO gap | Release-pipeline phase (decision) | `readelf -l`/`-d` output, or the comment corrected |
| M8 Egress evidence | Freeze-evidence phase | Dated commit+command+result, or NOT RUN with reason |

## Sources

Repository (read in this session):

- `.planning/PROJECT.md` — Active requirements, out-of-scope list, "trust the code" rule, freeze
  tag requirement.
- `.planning/codebase/CONCERNS.md` — full read; sandbox e2e shape, routing latent, disk floor
  150 GB vs guard 60 GB, release CI scaling limits, RELRO gap, test-coverage gaps.
- `KNOWN-ISSUES.md` — full read; the 31,939/2/388 run, the `cc6e6948` run never finished, flaky
  `xai-grok-shell` tests, key-exported test failures, rc2–rc5 outcomes, container never built,
  `workflow` scope note, LTO/OOM measurements.
- `.github/workflows/release.yml` — trigger `'v*'`, version derive, `is_prerelease` regex,
  90-minute build timeout, matrix runners, smoke-check, tarball naming, `publish` four-tarball
  check, `container` `needs: [version, build]`, metadata-action tags, `imagetools inspect`.
- `install.sh` — `VERSION` guard `v[0-9]*`, `latest` resolution via `/releases/latest/download`,
  two-space SHA256SUMS parsing, `shasum` fallback, `dsb` shortcut and uninstall semantics.
- `scripts/test-install.sh` — fake tree built for `$(host_target)` only; `latest/download/`
  synthesized; 39 assertions all against the loopback tree.
- `docker/Dockerfile`, `docker/Dockerfile.localtest` — tarball-as-context, `TARGETARCH` staging,
  non-root uid 1000.
- `Cargo.toml` `[profile.release]` / `[profile.release-dist]` — `strip=false`, `debug=1`,
  `split-debuginfo="off"`, thin LTO, `codegen-units=1`.
- `.cargo/config.toml` — gnu targets lack RELRO flags (present only under musl), linker wrapper,
  `RUST_MIN_STACK=67108864`, aarch64 jemalloc page-size env.
- `crates/codegen/xai-grok-shell/Cargo.toml` — `default = []`, `config-docs` only in
  `default-bazel`.
- `scripts/disk-guard.sh` — default 60 GB.
- `scripts/check-egress.sh` — static HARD/SOFT patterns and `EXCLUDES`.
- `scripts/egress-check.sh` — requires `DEEPSEEK_API_KEY`, `LD_PRELOAD` mechanism.
- `DECISIONS.md` — D1–D14, execution rules, stale §E/§F phase log (last updated 2026-10-07).
- `TESTING.md` — disk budget, recorded gate command `--no-fail-fast -- --test-threads=4`.
- `AGENTS.md` — §0 snapshot, §3 remaining work, §8 disk/clippy/egress rules.
- `git`: `show-ref --tags`, `ls-remote --tags origin`, `rev-parse`, `merge-base --is-ancestor`,
  tag annotation text.

Marked UNVERIFIED (recall / external behaviour, not checkable without network in this session):

- GitHub `/releases/latest` excludes prereleases (well-known; verify with the `curl -w` check).
- GHCR package visibility defaults to private independent of repo visibility.
- A timed-out job does not run cache-save post-steps.
- `strip` invalidating an ad-hoc Mach-O signature on arm64 macOS.
- `bwrap` user-namespace creation failing under Docker's default seccomp profile.
- Apple runner core counts (3-core assumption).

---
*Pitfalls research for: freezing a mature Rust terminal coding agent at v0.1.0*
*Researched: 2026-11-21 against `main` @ `9e6780d9`*
