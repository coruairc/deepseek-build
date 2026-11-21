# Phase 1: Truth Baseline - Research

**Researched:** 2026-11-21
**Domain:** Rust workspace (95 members) — sandbox deny materialization, workspace test gate, evidence artifacts, price/model truth
**Confidence:** HIGH for repo mechanics (all claims read from this tree), MEDIUM for the historical
`/run/containerd` abort (not reproducible on this host), HIGH for the empirical baseline run supplied
by the orchestrator (independently re-checked below)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
**Sandbox deny materialization**
- Skip a candidate only when `canonicalize(parent)` fails with `PermissionDenied` (EACCES/EPERM) — the
  root-owned, non-traversable parent case. Every other error stays fatal.
- A traversable parent with an unreadable endpoint (EACCES from `symlink_metadata`) stays fatal:
  fail-closed is preserved.
- A skipped candidate emits one `tracing::debug!` with the path only — no stderr noise.
- Tests: a unit matrix over missing parent (NotFound), traversable-but-unreadable endpoint,
  non-traversable parent, symlink endpoint, and parent-alias dedupe; plus
  `xai-grok-sandbox --test deny_paths_e2e` unmodified and green on the author's host.

**Test gate**
- New `scripts/test-gate.sh`: `scripts/disk-guard.sh` first, then `flock -w 900 /tmp/team-build.lock`,
  then `cargo test --workspace --no-fail-fast`, then
  `cargo test -p xai-grok-shell --lib --features config-docs`.
- Completeness is asserted by counting `test result:` lines against a checked-in expected target count;
  the expected count is refreshed only via an explicit flag, never silently.
- `git_gate`/`restore_fetch`: attempt a deterministic fix first; if still flaky after two attempts,
  quarantine with `#[ignore = "<recorded reason>"]` and a `KNOWN-ISSUES.md` entry.
- A recorded run commits a normalized log plus `report.json` under
  `docs/evidence/01-truth-baseline/test-gate/`.

**Evidence convention**
- Casts are produced by the existing pty harness `write_cast` (asciinema v2) — no external `asciinema`
  binary is installed. Headless-only items use `script(1)` where a pty is required.
- Layout: `docs/evidence/NN-<phase-slug>/<item-slug>/` containing `<item-slug>.cast`, `report.json`,
  optional `notes.md`.
- `report.json` fixed keys: `item`, `requirements[]`, `date`, `host`, `command`, `steps[]` (`name`,
  `ok`, `detail`), `artifacts[]` (`path`, `sha256`), `result` (`ok` | `fail` | `skipped`).
- Normalization: absolute repo/home/tmp prefixes become `<REPO>`/`<HOME>`/`<TMP>`; env dumps are never
  captured; `scripts/key-leak-check.sh` must pass over the committed artifacts.

**Cost honesty and default model**
- Price `deepseek-v4-flash` explicitly alongside `deepseek-flash` at Flash rates in the shipped table,
  so the hidden catalog alias stops inheriting Pro rates.
- The TUI cost item carries an `est.` label, e.g. `~$0.0042 est.`.
- Headless resolves the same `[ui.status_line.pricing]` table as the TUI for `attach_cost_estimate` and
  the streaming reducer, instead of `PricingTable::default()`.
- Flip the catalog `"default"` to `deepseek-flash`; document the at-ceiling caveat (63/63 both models)
  and the revert path (`/model deepseek-v4-pro`, `-m deepseek-v4-pro`) in the README quickstart, the
  user guide and `docs/TUNING.md`.

### the agent's Discretion
- Exact wording and placement of the estimate label, the report.json helper shape, the expected-count
  file name, and whether the IPv6 report reuses the existing `egress_log.c` shim or extends
  `sandbox-run.sh`'s inline shim.

### Deferred Ideas (OUT OF SCOPE)
- Automatic model routing, `[models] routing`, session pins, `Route` status-line item, `/route`,
  per-model cost accounting — v2 / open-build.
- Installing `asciinema` or any new external tool — the pty harness already emits the format.
- `ci.yml` and debug-symbol sidecars — tracked as REL-V2-01/02.
- Renaming `GROK_*` env names or `xai-grok-*` crates — explicitly out of scope (D2).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SBOX-01 | Runtime-socket deny materialization skips non-traversable parents, stays fail-closed otherwise | Findings 1a–1d; plan 01-01 |
| SBOX-02 | A test proves a root-only deny-path parent does not abort bwrap; `sandbox-run.sh` reports IPv6 | Findings 1e (fixtures), 2 (IPv6); plans 01-01, 01-02 |
| TEST-01 | `cargo test --workspace` exits 0, completeness asserted by target count | Findings 3a–3c; plan 01-05 |
| TEST-02 | The gate runs `config-docs` tests | Findings 3d; plan 01-05 |
| TEST-03 | `git_gate`/`restore_fetch` flake deterministic or quarantined with a reason | Findings 3e–3f; plan 01-05 |
| EVID-01 | Every item carries a committed cast + `report.json`, paths normalized | Findings 4a–4d; plan 01-05 |
| EVID-02 | The convention is documented and re-runnable | Finding 4e; plan 01-05 |
| COST-01 | `deepseek-v4-flash` priced in the table | Finding 5a; plan 01-03 |
| COST-02 | The TUI cost figure is labelled an estimate | Finding 5b; plan 01-03 |
| COST-03 | Headless honours `[ui.status_line.pricing]` | Finding 5c–5f; plan 01-03 |
| DEF-01 | `deepseek-flash` is the catalog default, caveat + revert path documented | Finding 6a–6e; plan 01-04 |
</phase_requirements>

## Summary

Three of the five plan areas are straightforward, single-file changes with an obvious test seam
(pricing table, cost label, headless pricing injection). Two are not, and both were mis-scoped by the
context document in ways that matter:

1. **The sandbox fix as decided does not cover the shape of the abort.** `dunce::canonicalize` is
   `std::fs::canonicalize` (i.e. `realpath(3)`) on Linux, and `realpath(parent)` **succeeds** when the
   parent itself is the unsearchable directory — verified on this host with a Rust probe. The EACCES
   then comes from `symlink_metadata(candidate)` at `runtime_sockets.rs:141-145`, which today is
   unconditional-fatal. So "skip when `canonicalize(parent)` is `PermissionDenied`" only fixes the case
   where an *ancestor* of the parent is unsearchable, not the `/run/containerd`-shaped case where the
   parent is the last component. A second, narrow branch (EACCES on the endpoint with a parent that is
   provably not traversable) is required to satisfy SBOX-01's own wording ("parent path is not
   traversable") without widening the predicate. The fail-closed clause stays satisfiable because a
   `lstat` EACCES can only be produced by a non-traversable ancestor — never by an unreadable *file*.

2. **`deny_paths_e2e` is red on this host for a completely different reason than the recorded one.**
   The empirical baseline is `12 passed; 1 failed; 1 ignored`, and the single failure is
   `devbox_genuine_reexec_applies_enforcement` with `FAIL: bwrap_reexec_for_profile returned None
   outside bwrap`. This host has no `/data` and no `/run/containerd`; the devbox profile therefore has
   an *empty* deny plan and `bwrap_reexec_for_profile` correctly returns `None`
   (`xai-grok-sandbox/src/lib.rs:639-645`). The test asserts the plan is always non-empty, which is a
   host-state assumption, not a product invariant. The recorded "Can't mkdir parents for
   /run/containerd/containerd.sock" cannot be produced by the devbox profile in *this* tree at all —
   devbox has `restrict_network: false` and `skip_dbus = true`, so runtime sockets never enter its
   deny set. That stderr line belongs to `read_deny_empty_set_verifies_inside_bwrap` (which passes here
   because `/run/containerd` is absent).

The remaining surprise is smaller but load-bearing: flipping `default_models.json` `"default"` does
**not** change the session default. With no `models.default`/env/CLI preference, the catalog's *first
visible entry* wins (`model_catalog/resolution.rs:93-121`), and the JSON array order puts
`deepseek-v4-pro` first. DEF-01 needs either the array reordered or the resolution changed.

**Primary recommendation:** implement 01-03 and 01-04 first (small, self-contained, immediately
provable), then 01-01 with the two-branch traversal predicate and a fixture that reproduces EACCES
without root, then 01-02, then 01-05 last (it can only be green once 01-01 and 01-03/04 land).
`deny_paths_e2e` must be resolved by an explicit, recorded decision about
`devbox_genuine_reexec_applies_enforcement` before the gate is trusted.

## Empirical Baseline (this host, 2026-11-21)

Supplied by the orchestrator, then independently re-checked here.

```
Command: flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-sandbox --test deny_paths_e2e
Result:  12 passed; 1 failed; 1 ignored
Failure: devbox_genuine_reexec_applies_enforcement
         stderr: "FAIL: bwrap_reexec_for_profile returned None outside bwrap"
```

Host facts, re-verified in this session:

| Fact | Command | Result |
|---|---|---|
| `/run/containerd` absent | `ls -ld /run/containerd` | `No such file or directory` |
| `/data` absent | `ls -ld /data` | `No such file or directory` |
| non-root euid | `id -u` | `1000` |
| bwrap works | `bwrap --version` + `bwrap --bind / / -- true` | `bubblewrap 0.13.1`, exit 0 |
| no runtime sockets present | `ls -l /run/docker.sock /var/run/docker.sock` | absent (`/run/dbus/system_bus_socket` and `/run/systemd/private` do exist) |
| rust/cargo | `rustc --version` | `1.94.0 (4a4ef493e 2026-03-02)` |
| disk | `df -h .` / `du -sh target` | 1018 GB free, target 43 GB |

Derivation of the failure (read from this tree):

- `ProfileName::Devbox` resolves with `deny: vec![]`, `write_deny: vec![]`, `restrict_network: false`
  (`crates/codegen/xai-grok-sandbox/src/profiles.rs:421-448`).
- `resolve_profile_with_runtime_sockets` sets `skip_dbus = true` for Devbox and only appends
  container-runtime sockets when `profile.restrict_network`
  (`profiles.rs:366-399`) — so the devbox policy is empty and `append_socket_denies` returns early
  (`runtime_sockets.rs:251-253`).
- `requires_hook_write_deny(Devbox) == false` (`lib.rs:54-66`, `ProfileName::Devbox => false`).
- `requires_read_deny(Devbox) == false` (`lib.rs:453-473`, non-`Custom` arm is `false`).
- `requires_data_write_deny(Devbox)` is `is_devbox_based && /data.try_exists()`
  (`lib.rs:491-501`) → `false` because `/data` is absent.
- `bwrap_deny_plan(Devbox, ws)` therefore yields an empty plan, and
  `bwrap_reexec_for_profile` returns `None` at `lib.rs:639-645` because the emptiness test omits
  nothing that is set.

So `None` is the *correct* answer on a host without `/data`; the test's expectation is what is wrong.
Note the ordering detail that made the historical record plausible but is impossible here:
`bwrap_deny_plan` prints `error: sandbox profile resolve failed: …` (`lib.rs:535`) and refuses via
`route_bwrap_startup` (`xai-grok-shell/src/config/mod.rs:1278-1298,1345-1395`), so a resolve-time EACCES
never reaches bwrap at all; only a *traversable* candidate reaches bwrap and can produce
`Can't mkdir parents`.

## Key Findings

### 1. Sandbox: the full chain from `canonicalize` to the abort

**1a. Materialization.** `materialize_runtime_socket_deny_paths_from`
(`crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs:100-157`) is the only place a permission error
becomes fatal. Verbatim, the two decision points:

```rust
let canonical_parent = match dunce::canonicalize(parent) {
    Ok(parent) => parent,
    Err(error) if error.kind() == io::ErrorKind::NotFound => { … }
    Err(error) => return Err(with_context(error)),
};
…
let metadata = match std::fs::symlink_metadata(&path) {
    Ok(metadata) => metadata,
    Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
    Err(error) => return Err(with_context(error)),
};
```

`dunce::canonicalize` is a no-op wrapper on Linux — verbatim from
`dunce-1.0.5/src/lib.rs:65-76`:

```rust
pub fn canonicalize<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    let path = path.as_ref();
    #[cfg(not(windows))]
    {
        fs::canonicalize(path)
    }
    …
}
```

**1b. The gap in the locked predicate, proved.** `realpath(parent)` needs search permission on every
component *except the last*, so it succeeds when the parent itself is the unsearchable directory. A
Rust probe against an owned `chmod 000` directory (same std path `dunce` calls on Linux):

```
canonicalize(/tmp/opencode/nt3/p) ok -> /tmp/opencode/nt3/p
lstat(/tmp/opencode/nt3/p/x.sock) err PermissionDenied
lstat(/tmp/opencode/nt3/p/.) err PermissionDenied (not traversable)

canonicalize(/tmp/opencode/nt4/hidden/p) err PermissionDenied      # ancestor unsearchable
lstat(/tmp/opencode/nt4/hidden/p/x.sock) err PermissionDenied
lstat(/tmp/opencode/nt4/hidden/p/.) err PermissionDenied (not traversable)
```

`[VERIFIED: rustc probe in /tmp/opencode on this host, 2026-11-21]` — the program calls
`std::fs::canonicalize`, `std::fs::symlink_metadata(child)` and
`std::fs::symlink_metadata(parent.join("."))`. The `parent/.` probe is a no-new-dependency traversal
test: `lstat("p/.")` requires execute on `p`, and returns `EACCES` exactly when `p` is not searchable
(confirmed for modes `000`, `711`, `555`: only `000` fails).

Consequence: shape (A) — parent is the unsearchable directory, the `/run/containerd` shape — is
handled by neither branch of the locked predicate. Shape (B) — an ancestor is unsearchable — is
handled by the `canonicalize` branch. Both are "parent not traversable" in the requirement's sense.

Also note: `lstat` on the *endpoint* never fails for the endpoint's own mode, so "traversable parent
with an unreadable endpoint" cannot produce EACCES at this call site at all. The existing
`materialized_socket_path_is_stable_after_mode_zero_mask` test
(`runtime_sockets_tests.rs:219-240`) proves the mode-000 file is still materialized. The fail-closed
clause therefore has to be expressed against the *parent traversal* check (and any LSM denial), not
against endpoint readability.

**1c. Caller and user-visible text.** `resolve_profile_with_runtime_sockets`
(`profiles.rs:366-399`) wraps the error:

```rust
.map_err(|error| anyhow::anyhow!("socket deny resolution failed: {error}"))?;
```

`bwrap_deny_plan` (`lib.rs:520-560`) prints `error: sandbox profile resolve failed: {e}` at line 535
when `resolve_failure_must_refuse` (`lib.rs:506-510`: hook deny ∨ read deny ∨ `/data` deny) is true,
and returns `None`. `bwrap_reexec_for_profile` (`lib.rs:625-645`) then returns `None`, and the shell's
`route_bwrap_startup` (`xai-grok-shell/src/config/mod.rs:1286-1298`) turns `(None, false, true)` into
`Refuse`, printing `error: this sandbox could not enforce its deny list on Linux: … Refusing to start
with denied paths unprotected.` (`config/mod.rs:1345-1352`). That pair of lines is the abort a user
sees.

**1d. What must keep rejecting.** `verify_runtime_socket_deny`
(`read_deny_verify.rs:433-453`) is the inside-bwrap check: NotFound → Ok, inspect error → Err, symlink
→ Err, socket → Ok, mode-000 → `verify_path_masked`, anything else → Err ("exposed"). It must stay as
is: a candidate that was skipped in the outer process is simply absent from the handoff and from the
deny set, so no new leniency is introduced there. `verify_resolved_read_deny_masks`
(`read_deny_verify.rs:398-427`) re-resolves inside bwrap, where the handoff path is used and no
filesystem EACCES can occur (`socket_deny_paths_for_resolution`, `runtime_sockets.rs:76-84`).

**1e. Fixtures without root.** The existing tests build fixtures in `temp_runtime_root`
(`runtime_sockets_tests.rs:104-116`) and use real `UnixListener::bind`, `symlink`, and `chmod 000`
(`:219-240`, `:242-258`, `:260-278`) — all as the unprivileged test user. `chmod 000` on a directory
the test owns produces `EACCES` for that same user (verified above), so both shapes are constructible:

| Fixture | Shape | Expected outcome |
|---|---|---|
| `root/missing/docker.sock` | parent NotFound | skip (today + after) |
| `root/p` (000) holding `x.sock` | shape A: parent unsearchable, canonicalize succeeds | **new**: skip |
| `root/hidden` (000) → `root/hidden/p` → `x.sock` | shape B: canonicalize EACCES | **new**: skip |
| `root/p` (0755) + `x.sock` mode 000 | traversable, unreadable endpoint | materialize (unchanged) |
| `root/p/x.sock` → symlink | endpoint symlink | fatal (unchanged) |
| `root/p` (0755) + `x.sock`, plus `var/run → ../p` alias | alias dedupe | one path (unchanged) |

The "traversable parent + unreadable endpoint stays fatal" arm is not reachable through the filesystem;
make the predicate a pure function over `(error_kind, parent_traversable)` so that arm can be unit
tested by construction.

**1f. Can the historical bwrap abort be reproduced?** No, and it cannot even be attributed to devbox in
this tree. Measured here with bwrap 0.13.1:

```
bwrap --bind / / --ro-bind /dev/null /tmp/opencode/bw/ro/x.sock -- sh -c true   # parent 0555, live socket → rc=0
bwrap --bind / / --ro-bind /dev/null /tmp/opencode/bw2/hidden/x.sock -- sh -c true  # parent 000 → rc=0
bwrap --bind / / --ro-bind /dev/null /tmp/opencode/missing/x.sock -- sh -c true     # missing → rc=0
```

`[VERIFIED: probe in /tmp/opencode on this host, 2026-11-21]`. So on this host/version bwrap neither
aborts on an unreadable parent nor on a missing destination. The record in `KNOWN-ISSUES.md:25-31`
attributes one bwrap stderr line to two tests; `read_deny_empty_set` (restrict_network, so runtime
sockets enter the plan) is the only one that can produce a `/run/containerd` bind in this tree. The
devbox half of that attribution is stale — most plausibly interleaved stderr from the parallel test
threads inside one test binary.

### 2. `scripts/sandbox-run.sh` IPv6

**2a. Where it breaks.** `scripts/sandbox-run.sh:45-61` resolves the allow host with
`hints.ai_family = AF_INET` and stores only `inet_ntop(AF_INET, …)` strings in `allowed[16][16]`.
`connect` (`:63-90`) allows `AF_INET` loopback + allowlist, allows only `::1` for `AF_INET6`
(`:75-80`), and the blocked report (`:81-89`) formats only `AF_INET`, leaving `what = "non-ip"` for any
IPv6 destination.

Reproduced against the current inline shim (extracted and compiled to `/tmp/opencode/eg/guard.so`):

```
$ EGRESS_ALLOW_HOST=api.deepseek.com LD_PRELOAD=…/guard.so \
    python3 -c "import socket;s=socket.socket(socket.AF_INET6);s.connect(('2001:db8::1',443,0,0))"
sandbox-run: BLOCKED connect to non-ip
```

**2b. `scripts/egress_log.c` already does it correctly.** It handles `AF_INET6` in both
`log_sockaddr` (`:77-82`) and `connect` (`:136-141`) with `inet_ntop(AF_INET6, …, INET6_ADDRSTRLEN)`.
It is a *logger* (never blocks, `:5-16`), so it cannot be reused verbatim. Two reusable pieces: the
family switch, and the `addr=… port=…` line shape that `egress-check.sh` already parses.

**2c. A test with no network.** The shim is a `connect` interposer, so the proof needs no route. Add
`scripts/test-sandbox-run.sh` modelled on `scripts/test-install.sh` (self-contained harness, prints
`n/n assertions`):

1. compile the shim (extract the heredoc, or move it to `scripts/egress_guard.c` — see 2d);
2. `EGRESS_ALLOW_HOST=api.deepseek.com` + `LD_PRELOAD` a `python3` connector to `2001:db8::1:443`;
3. assert stderr contains `BLOCKED connect to 2001:db8::1` and does **not** contain `non-ip`;
4. assert `::1` is allowed (no blocked line);
5. assert the allowlist itself carries an IPv6 entry: with `EGRESS_ALLOW_HOST=localhost` (AAAA `::1` in
   `/etc/hosts`), an `EGRESS_ALLOW_DEBUG=1` startup line lists `family=inet6` — this is the only
   offline way to prove "resolves IPv6", so add that one debug print.

**2d. Structural choice (agent's discretion).** Extracting the C to `scripts/egress_guard.c` mirrors
`egress_log.c`, makes the harness simple, and keeps `sandbox-run.sh` short; keeping it inline forces
the test to `sed` the heredoc out. Recommend extraction.

**2e. Coupled documentation.** `scripts/egress-check.sh:552-558` currently *documents* the bug in
prose ("formats only AF_INET destinations; its AF_INET6/other refusals print as 'non-ip'"). Plan 01-02
must update those notes in the same change or the repo contradicts itself.

### 3. Test gate

**3a. No existing completion machinery.** `grep` over `scripts/`, `TESTING.md`, `AGENTS.md`: there is
no gate script, no expected-count file, no parser. `scripts/disk-guard.sh` defaults to **60 GB**
(`min="${1:-60}"`) while the documented floor for this tree is 150 GB (`AGENTS.md` §8, PITFALLS M5) —
the gate must call `scripts/disk-guard.sh 150` explicitly.

**3b. Exact line format for this toolchain** `[VERIFIED: cargo/rustc 1.94.0 on this host]`:

```
     Running unittests src/lib.rs (target/debug/deps/xai_grok_status_line-0a6f…)
running 24 tests
…
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

and for doctests (`cargo test -p xai-grok-status-line --doc`, a crate with zero doctests):

```
   Doc-tests xai_grok_status_line

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

So a zero-doctest lib still emits a `test result:` line — doctest targets are countable, and both
forms start at column 0 with `test result:`.

**3c. Enumerating targets reliably.** `cargo metadata` is not enough: it reports no
`required-features`, and 9 `xai-grok-shell` `[[test]]` targets are skipped unless `test-support` is on
(`crates/codegen/xai-grok-shell/Cargo.toml`, 9 `[[test]]` blocks with
`required-features = ["test-support"]`), and `xai-grok-markdown`/`xai-fast-worktree` have
required-features bins. Current metadata totals:

| Measure | Count | Command |
|---|---|---|
| workspace packages | 95 | `cargo metadata --no-deps` |
| targets with `test == true` | 223 (93 lib, 16 bin, 114 test) | same |
| targets with `doctest == true` | 89 (all lib) | same |
| derived upper bound on `test result:` lines | 312 | 223 + 89 |

The recorded **307** is from a different tree (2026-10-09) and must not be carried forward. The
authoritative enumeration is the build graph, not metadata:

```sh
cargo test --workspace --no-run --message-format=json \
  | jq -r 'select(.reason=="compiler-artifact" and .profile.test==true and .executable!=null) | .executable' \
  | sort -u | wc -l
```

This counts exactly the test binaries cargo will run (skipped `required-features` targets, `harness =
false` targets and non-test artefacts are excluded). Cross-check inside the gate log: count
`^test result:` and `^   Doc-tests ` lines, and assert
`results == executables + doctests` and `results == expected`. That is a self-consistent completeness
proof that does not depend on a hard-coded number; the checked-in expected count then guards against a
whole family of targets disappearing.

**3d. `default = []` for `xai-grok-shell`** `[VERIFIED: crates/codegen/xai-grok-shell/Cargo.toml:7-30]`:

```toml
[features]
default = []
…
config-docs = []
default-bazel = [
    "local-workspace",
    "test-support",
    "config-docs",
]
```

`config-docs` gates `crates/codegen/xai-grok-shell/src/config_docs/mod.rs` (8 `#[test]`s) against
`crates/codegen/xai-grok-pager/docs/user-guide/26-config-reference.md`. `cargo test --workspace` never
enables it. `cargo test -p xai-grok-shell --lib --features config-docs` is the second gate step; note
it also pulls in the two pre-existing shell flakes (see 3f-adjacent note in Pitfalls).

**3e. The `git_gate`/`restore_fetch` flake is a wall-clock tolerance problem, not a logic problem.**
The gate stores `std::time::Instant` (`git_gate.rs:15`), so a paused tokio clock cannot drive it —
`#[tokio::test(start_paused = true)]` would not help. The tests that can fail purely from scheduling
delay:

| Test | File:line | Margin |
|---|---|---|
| `root_cache_expires_so_a_new_nested_repo_is_rediscovered` | `git_gate_tests.rs:513-524` | test `ROOT_CACHE_TTL` = 80 ms (`git_gate.rs:34-38`), sleeps 100 ms → 20 ms |
| `walk_timeout_after_ttl_does_not_resurrect_snapshot` | `:257-281` | 20 ms ttl / 40 ms walk timeout / 40 ms sleep |
| `late_ok_after_waiter_timeout_is_snapshotted` | `:285-325` | 40 ms walk timeout / 50 ms sleep |
| `waiter_timeout_after_invalidate_does_not_return_stale_ok` | `:461-505` | 40 ms walk timeout vs 2 s outer |
| `joiners_do_not_start_a_second_walk` | `:142-192` | 500 ms ttl / 20 ms sleep |
| `snapshot_within_ttl_skips_second_walk` | `:118-138` | two calls must land inside 5 s |
| `status_and_diffs_never_overlap_odb_walks` | `:196-233` | 1 ms ttl / 60 ms sleep |
| `sustained_invalidate_during_inflight_returns_after_at_most_two_walks` | `:526-578` | 1 ms ttl, `timeout(1500 ms)` |
| `fetch_timeout_kills_process_group_with_grandchild` | `restore_fetch_tests.rs:551-591` | 250 ms timeout, asserts `elapsed < 2 s` |
| `fetch_child_drop_tears_down_process` | `:668-690` | asserts `elapsed < 5 s` |
| `fetch_timeout_sigkills_term_immune_grandchild` | `:692-730` | asserts `elapsed < 5 s` |
| `assert_timeout_about` callers | `restore_fetch_tests.rs:78-85` + 181/362/416/441 | ±2 s band on budget-derived deadlines |

`KNOWN-ISSUES.md:44-48` records the flake as "rotates under whole-suite saturation and passes in
isolation" and names no tests. Two deterministic attempts are available before quarantine:
(i) raise only the under-margin constants so the *ordering* invariant is unchanged (e.g. root-cache
sleep 100 ms → 500 ms; sub-100 ms ttl/walk windows → 250–500 ms; `elapsed < 2 s` → `< 10 s`), and
(ii) `#[serial]`-ise the tests that share no state but compete for the same tokio worker. If neither
holds after a recorded repetition (5× crate run under the full gate's load), quarantine the named
tests with `#[ignore = "<reason>; tracked in KNOWN-ISSUES.md>"]`. Do **not** quarantine blind: a
quarantine without a named test and a reproducer is the pattern PITFALL 3 forbids.

**3f. Determinism preconditions the gate must set.** `KNOWN-ISSUES.md:5-17` records that three
`xai-grok-shell` credential tests fail when `DEEPSEEK_API_KEY` is exported, that
`agent::folder_trust::tests::record_and_lookup_round_trip` failed 3/3 in a full run, and that
`util::config::consent::tests::set_consent_answer_is_monotonic_per_account` failed 1/3. The first two
are fixable and both sit in the `config-docs` step:

- the gate must run with `env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY`
  (PITFALLS §2's recipe);
- `record_and_lookup_round_trip` (`crates/codegen/xai-grok-shell/src/agent/folder_trust.rs:451-462`)
  is the only test in that module with no `#[serial_test::serial]`, while its neighbours set the
  process-global `xai_grok_version::TEST_VERSION_ENV` (`:445-449`) and `DEEPSEEK_BUILD_HOME`
  (`:464-479`). Adding `#[serial]` is a one-line, assertion-preserving fix.
- `set_consent_answer_is_monotonic_per_account`
  (`crates/codegen/xai-grok-shell/src/util/config/consent_tests.rs:44-47`) sets
  `DEEPSEEK_BUILD_HOME` without serialising; same one-line fix.

Without those two, TEST-02's gate step is red on this host regardless of the sandbox work.

### 4. Evidence convention

**4a. `write_cast`.** `crates/codegen/xai-grok-pager-pty-harness/src/lib.rs:483-514` writes asciinema
v2: a `{"version":2,"width":cols,"height":rows}` header line, then one `[elapsed, "o", data]` JSON
array per recorded PTY read, with a UTF-8 boundary fix-up. It is a method on `PtyHarness`, not gated to
tests, and it creates the parent directory.

**4b. Existing usage is opt-in and test-only.** `crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/common.rs:1563-1576`:

```rust
pub(crate) fn write_cast_if_requested(harness: &PtyHarness, file_name: &str) {
    let Ok(dir) = std::env::var("GROK_PTY_CAST_DIR") else { return; };
    …
}
```

28 call sites across `tests/pty_e2e/*`. So the recording recipe for a TUI item is: run one
`#[ignore]`d pty test with `GROK_PTY_CAST_DIR=docs/evidence/01-truth-baseline/<item>`. There is **no**
existing pty test that renders the status-line cost item (`grep status_line` over
`tests/pty_e2e/` returns nothing), so COST-02's evidence needs a new small pty test (or a `script(1)`
capture of a manual TUI run).

**4c. Can a non-test binary drive it?** `pty-scenario`
(`src/bin/pty_scenario.rs`) runs declarative scenarios via `ScriptedScenarioRunner`, which writes
`report.json` (`src/scripted.rs:1136-1173`) plus text/html/svg artifacts but **no cast** —
`grep write_cast src/scripted.rs` is empty. So no single existing entry point emits both halves of the
convention, despite `PROJECT.md:173` saying "the pty harness already produces both". The evidence
helper has to compose them (cast from a pty test run, report.json authored by the helper) or a
`--cast` flag must be added to the scripted runner. Given "no new product surface", composing in a
script is cheaper.

**4d. `script(1)` for headless items.** `script from util-linux 2.42.4` is present; `asciinema` is not
(confirmed). `script -qec '<cmd>' -T <timing> <typescript>` gives a typescript with a
`Script started on …` header and `Script done on …` footer plus a timing file of `elapsed bytes` pairs:

```
0.028614 7
0.200935 7
```

Converting to cast v2 is ~20 lines (strip the header line and the trailing `Script done` record, then
walk the byte offsets). `jq`, `python3 3.14.10`, `sha256sum`, `cc` and `gcc` are all available, which
makes a `scripts/evidence.sh` (or `.py`) helper feasible without new dependencies.

**4e. Nothing exists yet, and git will accept it.** `docs/evidence` does not exist; `.gitignore` does
not cover `docs/`, so committed evidence is tracked normally. `docs/` holds `TUNING.md`,
`DROPPED-TESTS.md`, `INTERNAL-GROK-ENV-NAMES.md`, `LICENSE-AUDIT.md`, `license-deny-report.txt`.
`scripts/check-egress.sh` scans only `crates prod third_party` (`EXCLUDES` also drops `*.md`), so an
evidence file naming `api.deepseek.com` cannot trip the HARD gate.

**4f. `key-leak-check.sh` does not look at `docs/`.** `scripts/key-leak-check.sh:37-47` greps only the
isolated `$HOME_DIR`, the captured stdout and stderr for one canary
(`sk-FAKE-LEAKCANARY-…`). It also requires the release binary to exist (`:24-27`). To satisfy
"pass over the committed artifacts", the script needs an additive mode — e.g. accept extra paths as
arguments, or a `LEAK_PATTERNS` file — scanning `docs/evidence` for the canary plus any
`sk-[A-Za-z0-9]{20,}` shape, printing file names only (the script already refuses to print matched
lines, `:45`). Without that change, EVID-01's leak assertion is unprovable and PROOF-03 (Phase 2)
inherits the gap.

### 5. Pricing and headless cost

**5a. The table.** `crates/codegen/xai-grok-status-line/src/pricing.rs`:
`DEFAULT_PRICING_MODEL = "deepseek-v4-pro"` (`:24`), `FLASH_PRICING_MODEL = "deepseek-flash"` (`:27`),
`ModelPricing::default()` = Pro peak `0.044/1.32/3.96` (`:49-57`), `ModelPricing::flash()` =
`0.006/0.30/1.20` (`:61-68`), shipped table = exactly those two entries (`:78-83`), and
`for_model` falls back to the Pro entry for any unnamed model (`:97-102`). So `deepseek-v4-flash` —
the hidden third catalog entry (`default_models.json:98-141`, `"hidden": true`) — currently prices at
Pro rates, 4.6× over. `with_overrides` replaces entries outright (`:86-90`), so a user override for
`deepseek-flash` will not propagate to `deepseek-v4-flash`; state that in the docs or alias the two
keys in `for_model`.

**5b. Where the TUI number is formatted.**
`crates/codegen/xai-grok-pager/src/views/status_line/segments.rs:18` and `:104-108`:

```rust
const MIN_DISPLAYED_COST_USD: f64 = 0.005;
…
StatusLineItem::Cost => ctx.cost.total_cost_usd
    .filter(|usd| *usd >= MIN_DISPLAYED_COST_USD)
    .map(|usd| StatusSegment::dim(format!("${usd:.2}"))),
```

Two decimals and a 0.005 floor mean a Flash turn (~$0.0015) renders **nothing**. The requested
`~$0.0042 est.` therefore requires a precision and a floor change, not just a suffix. Only
`crates/codegen/xai-grok-pager/src/views/status_line/segments_tests.rs` pins the strings
(`:27` `"… │ $0.37 │ …"`, `:34-42` for the 0.004/0.006 boundary), so the blast radius is two tests
plus `docs/user-guide/25-status-line.md:21` ("hidden below $0.005 so it never shows a misleading
`$0.00`").

**5c. How the TUI resolves the table.** `load_initial_ui_config`
(`crates/codegen/xai-grok-pager/src/app/event_loop.rs:2804-2818`) reads
`xai_grok_shell::config::load_effective_config()`, extracts `root["ui"]` and tries to deserialize
`UiConfig`; `current_ui` is set at `event_loop.rs:1493`. `UiConfig.status_line` is
`crates/codegen/xai-grok-shared/src/ui_config.rs:172-174`, and `StatusLineConfig::pricing()`
(`xai-grok-status-line/src/config.rs:199-204`) layers `[ui.status_line.pricing]` over the defaults.
The cost is filled in `AppView::fill_estimated_cost`
(`crates/codegen/xai-grok-pager/src/app/status_line_policy.rs:275-290`) from
`self.current_ui.status_line.pricing()`, and **a provider-reported cost always wins** (`:276-278`).
That is the resolution the TUI shows and therefore the one headless must match.

**5d. What headless has today.** Two hard-coded sites, both `PricingTable::default()`:

- `headless.rs:308-340` `HeadlessEmitter::attach_cost_estimate` — uses
  `self.session_model` and the projected `usage`, writes `estimated_cost_usd` + `estimate: true`;
  the default table is line 332.
- `headless/reducer/messages/usage.rs:103-117` `MessagesReducer::estimated_cost_usd` — same maths,
  default table at line 116.

**5e. Smallest correct injection.** `HeadlessEmitter::new` (`headless.rs:116-134`) constructs the
reducer via `reducer_for(format)` (`headless/reducer/mod.rs:432-437`) *before*
`run_single_turn` loads `raw_config` at `headless.rs:898`. The minimal shape:

1. `HeadlessEmitter::new(format, schema, pricing: PricingTable)`; add a `pricing` field; move the
   `let mut emitter = …` line below the `raw_config` load (2-line move, no logic change).
2. Derive the table from the already-loaded layers — `raw_config.get("ui")` →
   `UiConfig` → `.status_line.pricing()` — i.e. exactly what `load_initial_ui_config` does, without a
   second file read.
3. `reducer_for(format, pricing)` and `MessagesReducer::new_with_pricing(pricing)`, keeping
   `MessagesReducer::new()` as `PricingTable::default()` so the ~6 reducer tests are unchanged.
4. `attach_cost_estimate` and `estimated_cost_usd` use the injected table.

Do **not** convert the TUI to a shared cached helper: `current_ui` is deliberately re-read on config
reload, and caching it would regress `/config` changes mid-session.

**5f. Existing test coverage to preserve.**
`headless_tests.rs:665-704` (`json_result_carries_local_cost_estimate_marked_estimate`,
`…omits_estimate_when_no_tokens_recorded`) and
`headless/reducer/messages/tests/result_usage.rs:150-181` (pro-default fallback for an unknown model).
These pass with a default table; add one new case per surface proving a `[ui.status_line.pricing]`
override changes the number, and one cross-surface equality case (same usage + same override → same
USD in TUI-context and headless paths).

### 6. Default model

**6a. The JSON.** `crates/codegen/xai-grok-models/default_models.json:2` `"default": "deepseek-v4-pro"`;
array order is `deepseek-v4-pro` (`:8`), `deepseek-flash` (`:53`), `deepseek-v4-flash` (`:98`,
`"hidden": true` at `:141`). `default_model()` returns the field verbatim
(`xai-grok-models/src/lib.rs:47-49`) and `DEFAULTS` asserts the default is one of `models[].model`.

**6b. Who consumes `default_model()`.** `session/persistence.rs:1270-1272`
(`default_model_id()`), `session/acp_session_impl/context_snapshot.rs:21-23` (tokenizer budget),
`model_catalog/resolution.rs:32-37` (`fallback_model_id`), plus tests that follow it dynamically
(`config_tests.rs` ~20 sites use `let dm = crate::models::default_model();`).

**6c. The default does *not* come from that field at runtime.**
`resolve_default_model` (`model_catalog/resolution.rs:75-145`) resolves a preference from
CLI/env/`models.default`; with no preference it calls `first_or_fallback()`, which returns
`visible.first()` (`:93-100`). The catalog is built by `resolve_model_list`
(`agent/config.rs:2454-2469`), which does `resolved.extend(defaults)` — JSON order preserved. So with
no `[models] default`, the session model is the **first visible entry**, `deepseek-v4-pro`, and
flipping `"default"` alone changes only `fallback_model_id`, the tokenizer budget and
`persistence::default_model_id`, not the session default.

Options (state the risk explicitly):

| Option | Change | Risk |
|---|---|---|
| A | Set `"default": "deepseek-flash"` **and** move the `deepseek-flash` object above `deepseek-v4-pro` in the array | Changes picker order (flash first); no test pins the order today (`grep` found none), but the picker's first row changes for users |
| B | Set `"default"` and make `first_or_fallback` prefer `xai_grok_models::default_model()` when it is visible | Larger change inside the resolver; makes the JSON authoritative and keeps picker order; needs new tests for "default present", "default hidden", "default not in catalog" |
| C | Set `"default"` only | **Does not satisfy DEF-01** on a default config — the session still starts on Pro |

Recommend B if the planner wants one source of truth; A if the change must stay a data edit. Either
way, add a test asserting the resolved default with no preference, which does not exist today.

**6d. Tests pinned to the old default.** `context_snapshot.rs:375-377`
`assert_eq!(crate::models::default_model(), "deepseek-v4-pro");` is the only hard pin and must be
updated. Everything else resolves dynamically and follows the flip.

**6e. Documentation that states the default.**

| File:line | Current text |
|---|---|
| `README.md:25` | "DeepSeek catalog (default `deepseek-v4-pro`)" |
| `README.md:247` | "Models: `deepseek-v4-pro` (default) and `deepseek-flash`, plus a hidden `deepseek-v4-flash` legacy alias" |
| `README.md:270` | `/model` row: "switch between `deepseek-v4-pro` and `deepseek-flash`" |
| `crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md:9` | "new sessions start with `deepseek-v4-pro`" |
| `crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md:51` | `default = "deepseek-v4-pro"` example |
| `crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md:51` | `default = "deepseek-v4-pro"` example |
| `docs/TUNING.md:19-37` | baseline table + findings ("Both models pass every task") — the right home for the caveat and revert path |
| `crates/codegen/xai-grok-pager/docs/user-guide/26-config-reference.md:352` | `models.default` row — no default value stated, **no edit needed** |
| `KNOWN-ISSUES.md:128,170` | list the catalog models; no default claim, optional touch |

`26-config-reference.md` is hand-edited and guarded by the `config_docs` tests; since the
`models.default` row states no default value, DEF-01 needs no change there.

## Recommended Implementation Approach

### 01-01 — Sandbox deny materialization (SBOX-01, SBOX-02 test half)

1. Extract the decision to a pure function so every branch is testable, e.g.
   `enum SocketCandidate { Materialize(PathBuf), Skip }` +
   `fn classify_parent_error(kind: io::ErrorKind, parent_traversable: bool) -> bool` (skip?).
2. `materialize_runtime_socket_deny_paths_from` (`runtime_sockets.rs:100-157`):
   - keep `NotFound` handling exactly as is;
   - add `Err(error) if error.kind() == PermissionDenied` on the `canonicalize(parent)` arm →
     `tracing::debug!(path = %candidate.display(), "skipping runtime-socket deny: parent not traversable"); continue;`
   - add the shape-A branch: on `symlink_metadata(&path)` `PermissionDenied`, skip only when
     `parent_is_traversable(parent)` is false, where
     `parent_is_traversable(p) = std::fs::symlink_metadata(&p.join(".")).is_ok()`; otherwise keep the
     fatal `with_context` error. This is strictly narrower than "EACCES like NotFound" and preserves
     the fail-closed arm for an LSM denial on a traversable parent.
   - keep symlink, dedupe and `InvalidInput` arms untouched.
3. Extend `runtime_sockets_tests.rs` with the six fixtures from 1e, including one that asserts a
   `tracing` debug event is emitted (or at least that no stderr is produced) and one that pins
   "skip shapes are absent from the returned vec while a 0755-parent candidate is present".
4. Integration proof for SBOX-02: extend `deny_paths_e2e` **only if** you decide
   `devbox_genuine_reexec_applies_enforcement` needs a guard (see 01-05 item 4); otherwise add a
   sandbox-unit test that builds `bwrap_reexec_for_profile` for a `Custom("strict")`-style profile
   whose only automatic candidate sits under an owned `chmod 000` parent, and assert the returned
   command is `Some` and does **not** contain that path in its arguments.
5. Verify: `cargo test -p xai-grok-sandbox --lib`, then
   `flock -w 900 /tmp/team-build.lock cargo test -p xai-grok-sandbox --test deny_paths_e2e`.

### 01-02 — `sandbox-run.sh` IPv6 (SBOX-02 second half)

1. Resolve with `hints.ai_family = AF_UNSPEC`; store `(family, bytes)` (or `sockaddr_storage`) rather
   than strings, and match by family + bytes so `::ffff:1.2.3.4` maps onto the same IPv4 entry.
2. Keep loopback allowances (`127.*`, `0.*`, `::1`, and mapped loopback).
3. Format the blocked line per family with `inet_ntop(AF_INET6, …, 46)`; `non-ip` remains only for
   genuinely non-IP families.
4. Optional `EGRESS_ALLOW_DEBUG=1` startup print of the resolved allowlist (needed for the offline
   "resolves IPv6" assertion).
5. New `scripts/test-sandbox-run.sh` (modelled on `scripts/test-install.sh`), 6 assertions from 2c;
   print `N/N assertions` and exit non-zero on any failure.
6. Update `scripts/egress-check.sh:552-558` prose and `scripts/README.md`.

### 01-03 — Cost honesty (COST-01, COST-02, COST-03)

1. `pricing.rs`: add `pub const FLASH_ALIAS_PRICING_MODEL: &str = "deepseek-v4-flash";` and insert it
   with `ModelPricing::flash()` in `deepseek_defaults()`; extend
   `default_table_prices_both_labelled_models_at_peak` and add a `deepseek-v4-flash` worked example
   (same buckets → `0.001206`). Decide and document whether `with_overrides` aliases the two keys.
2. `segments.rs`: label + precision. Suggested: `~$0.0042 est.` at 4 dp below $0.01, `~$0.37 est.` at
   2 dp at/above, floor lowered to the smallest displayable value (or drop the floor and rely on
   precision). Update `segments_tests.rs` and `docs/user-guide/25-status-line.md:21`.
3. Headless: inject the resolved table (5e) into `HeadlessEmitter` and `MessagesReducer`; assert a
   `[ui.status_line.pricing.deepseek-flash]` override changes `estimated_cost_usd` and that TUI and
   headless produce the same number for the same usage. Update
   `docs/user-guide/14-headless-mode.md:194-201` to say the table honours the override.

### 01-04 — Default model (DEF-01)

1. Apply option A or B from 6c (recommend B: make `default_models.json` authoritative in
   `first_or_fallback`).
2. Update `context_snapshot.rs:376`; add the missing "resolved default with no preference" test.
3. Docs: `README.md:25,247,270`, `11-custom-models.md:9,51`, `05-configuration.md:51`,
   `docs/TUNING.md` (caveat paragraph: 63/63 both models at ceiling → the eval set cannot prove Flash
   sufficient for hard work; revert with `/model deepseek-v4-pro` or `-m deepseek-v4-pro`).
4. Re-run `cargo test -p xai-grok-shell --lib` (config tests follow `default_model()` dynamically) and
   the `config-docs` step.

### 01-05 — Test gate + evidence (TEST-01/02/03, EVID-01/02)

1. `scripts/test-gate.sh`:
   - `scripts/disk-guard.sh 150 || exit 1`;
   - `flock -w 900 /tmp/team-build.lock` around the whole run;
   - `env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY` for both cargo steps;
   - step 1 `cargo test --workspace --no-fail-fast -- --test-threads=4`, tee to a log;
   - step 2 `cargo test -p xai-grok-shell --lib --features config-docs -j 1`;
   - completeness: `results=$(grep -c '^test result:' log)`,
     `doctests=$(grep -c '^   Doc-tests ' log)`, `binaries=$(cargo test --workspace --no-run
     --message-format=json | jq … | wc -l)`; assert `results == binaries + doctests` and
     `results == <checked-in expected>`; print all four numbers;
   - `--refresh-expected` re-runs and rewrites the expected file **only** when both steps are green,
     recording commit, date, and the two derived numbers in the file header;
   - exit non-zero on a step failure *or* a count mismatch, and never read the tail of the log.
2. Expected-count file: `scripts/test-gate.expected` (or `docs/evidence/01-truth-baseline/test-gate/
   expected.txt`) with the provenance header; seed it from the first green run — do not carry 307.
3. Determinism: apply the two `#[serial]` fixes (3f) and the tolerance widening (3e-i); record a
   5× repetition of `cargo test -p xai-grok-workspace --lib` under load. If a named test still
   rotates, quarantine it with `#[ignore = "<reason>; see KNOWN-ISSUES.md>"]` and add the entry.
4. Decide, explicitly and in writing, what to do about `devbox_genuine_reexec_applies_enforcement`:
   - **Option 1 (test's assumption wrong):** guard the bwrap expectation on the plan being non-empty
     (e.g. skip only when `!Path::new("/data").exists()` **and** the profile resolves to an empty plan),
     with a comment naming the required host state. This is a *skip*, so it needs a reviewer note and a
     `KNOWN-ISSUES.md` line; it does not weaken any sandbox assertion because there is nothing to
     assert on this host.
   - **Option 2 (product always re-execs for devbox):** make `bwrap_reexec_for_profile` return a
     command whenever the profile is not `Off` (e.g. `bwrap --bind / / --dev-bind /dev /dev --proc /proc`
     with no deny binds). The test then passes unmodified and `verify_data_write_deny_enforced` is a
     no-op (`read_deny_verify.rs:387-389`). Cost: one user namespace per devbox launch and a
     `__GROK_INSIDE_BWRAP=1` marker on hosts that previously had none; risk of new bwrap-availability
     failures (mitigated by `BwrapStartup::ReexecOptional`).
   - **Option 3 (do nothing):** the gate stays red on this host → TEST-01 unprovable.
   Recommend Option 1 (recorded skip keyed on the *plan being empty*, not on "bwrap failed"), because
   Option 2 changes runtime behaviour of every devbox session to satisfy a test.
5. Evidence: add `docs/evidence/README.md` (layout, fixed `report.json` keys, normalization table,
   re-run recipe) and one helper (`scripts/evidence.sh` with `jq`, or `scripts/evidence.py`) that runs
   a command, captures the log, computes sha256, normalizes `<REPO>/<HOME>/<TMP>` and writes
   `report.json`. Phase-1 items:
   - `01-truth-baseline/test-gate/` — `test-gate.cast` (via `script -T` → cast v2) or
     `test-gate.log`, `report.json`, `notes.md`;
   - `01-truth-baseline/ipv6-report/` — the `scripts/test-sandbox-run.sh` transcript;
   - `01-truth-baseline/cost-estimate/` — a pty test cast showing `est.` (new test) or a `script`-captured
     TUI session.
6. `scripts/key-leak-check.sh`: accept extra paths (or a `LEAK_PATTERNS` file) and scan
   `docs/evidence`; keep printing file names only; run it as the last gate step and record the result.

## Architecture Patterns

### Data flow (sandbox)

```
profile name + workspace
  └─ resolve_profile_with_runtime_sockets (profiles.rs:366)
       ├─ resolve() → SandboxProfile { deny, restrict_network, … }
       ├─ policy = dbus sockets (non-devbox) + runtime sockets (restrict_network)
       └─ append_socket_denies (runtime_sockets.rs:246)
            └─ socket_deny_paths_for_resolution (runtime_sockets.rs:76)
                 ├─ outside bwrap → materialize_runtime_socket_deny_paths_from   ← the fix lands here
                 └─ inside bwrap  → decode handoff against the static policy
  └─ bwrap_reexec_for_profile (lib.rs:625)
       ├─ empty plan → None
       └─ bwrap … --ro-bind <placeholder> <deny> … -- __GROK_INSIDE_BWRAP=1 self
            └─ route_bwrap_startup (shell config/mod.rs:1286): Required | Optional | Verify | Refuse | Continue
```

### Data flow (cost)

```
config.toml [ui.status_line.pricing.<model>]
  ├─ TUI: load_initial_ui_config → current_ui.status_line.pricing() → fill_estimated_cost
  │        → ctx.cost.total_cost_usd → segments::Cost → "~$… est."
  └─ headless: load_effective_config (headless.rs:898) → [ui] → UiConfig.status_line.pricing()
           ├─ HeadlessEmitter::attach_cost_estimate → estimated_cost_usd + estimate:true
           └─ MessagesReducer::estimated_cost_usd → result.usage.estimated_cost_usd
```

### Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|---|---|---|---|
| Deny-set materialization | Sandbox library (`xai-grok-sandbox`) | — | Pure path resolution; no UI, no shell |
| bwrap startup refusal | Shell config (`xai-grok-shell::config::apply_sandbox`) | Sandbox library | Library returns `Option`; the policy of refusing lives at process startup |
| IPv6 destination reporting | Dev script (`scripts/sandbox-run.sh`) | — | Test/dev tooling, not product |
| Test-suite completeness | Dev script (`scripts/test-gate.sh`) | — | No `ci.yml` in this milestone |
| Price table | `xai-grok-status-line` | — | Single source of truth for both surfaces |
| Cost rendering | Pager views (`views/status_line/segments.rs`) | — | Formatting is presentation |
| Headless cost | Pager headless (`headless.rs`, reducer) | `xai-grok-status-line` | Headless is a client surface, must reuse the same resolver |
| Default model id | Model catalog (`xai-grok-models`, `model_catalog`) | Shell config | Resolution is catalog policy |
| Evidence artifacts | `docs/evidence/` + `scripts/` | — | Committed proof, not product |

## Standard Stack

No new dependencies. Everything needed is already in the tree or on the host.

| Need | Use | Evidence |
|---|---|---|
| Temp-dir permission fixtures | `std::fs::set_permissions` + `PermissionsExt` | `runtime_sockets_tests.rs:219-240` |
| Canonicalize | `dunce::canonicalize` (workspace dep `dunce = "1"`) | `Cargo.toml:153` |
| Debug logging | `tracing::debug!` (already used in the crate) | `lib.rs:550`, `profiles.rs` |
| Shell for the gate | POSIX-ish bash like `scripts/test-install.sh`; `flock`, `jq`, `python3` present | host check |
| Cast generation | `PtyHarness::write_cast` (asciinema v2) | `pty-harness/src/lib.rs:483-514` |
| Headless capture | `script from util-linux 2.42.4` (`-T` timing) | host check |
| JSON evidence | `jq` 1.x or `python3` json | host check |

**Installation:** none.

**Version verification:** no package is installed or added by this phase, so the package-legitimacy
gate has no candidates. `[VERIFIED: no dependency change in the locked decisions]`

## Package Legitimacy Audit

Not applicable — this phase adds no external package and the workspace `.cargo`/`Cargo.toml` graph is
unchanged (`[VERIFIED: CONTEXT decisions; no install step in any plan]`).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---|---|---|---|
| Cast recording | A new PTY recorder | `PtyHarness::write_cast` | Already emits asciinema v2 with UTF-8 boundary handling |
| IPv6 formatting | Custom hex grouping | `inet_ntop(AF_INET6, …)` as `egress_log.c:77-82` does | Handles compressed forms, scopes |
| Target enumeration | A hard-coded number from docs | `cargo test --no-run --message-format=json` + `Doc-tests` line count | required-features and feature gating make static counts wrong |
| JSON evidence assembly | Ad-hoc `echo` blocks | `jq -n` or `python3 -c json.dump` | Fixed schema must stay valid |
| Pricing lookup | A second table in headless | `StatusLineConfig::pricing()` | One resolver keeps TUI and JSON equal |

## Common Pitfalls

1. **Treating `canonicalize(parent)` EACCES as the whole story** — it misses the `/run/containerd`
   shape (2a/1b). Symptom: the unit test passes, the host still aborts. Guard: a fixture where the
   parent itself is `chmod 000`.
2. **Widening to "any EACCES is NotFound"** — silently drops a deny the process *can* see; PITFALL 12.
   Guard: keep the parent-traversal check and a unit test for the fail-closed arm.
3. **Making the suite green by editing the test** — `skip_if_enforcement_unavailable` must not gain a
   condition (`deny_paths_e2e.rs:84-110`). The devbox decision (01-05 item 4) is the one place a test
   change is unavoidable; it needs a reviewer note.
4. **Counting targets from `cargo metadata`** — overcounts by the 9 `test-support`-gated shell
   targets and any required-features bin.
5. **Reading the tail of the gate log** — PITFALL 2. The gate must assert
   `results == binaries + doctests`.
6. **Running the gate with an exported key** — three shell credential tests fail
   (`KNOWN-ISSUES.md:5-17`).
7. **`disk-guard.sh` with its default 60 GB** — the documented floor is 150; call it explicitly.
8. **Assuming `"default"` in JSON drives the session model** — it does not (6c).
9. **Dropping the `est.` suffix on the provider-reported path** — a ledger `total_cost_usd` is not an
   estimate; only the table-derived figure gets the label.
10. **Leaving the cost floor at 0.005 with two decimals** — Flash sessions show no cost at all.
11. **Committing evidence with absolute paths** — normalization is a listed requirement; the helper
    must rewrite `<REPO>/<HOME>/<TMP>` in every captured file.
12. **Assuming `key-leak-check.sh` covers `docs/`** — it does not (4f).

## Runtime State Inventory

Not a rename/refactor phase. Nothing to migrate: no stored data, service config, OS registration,
secret key or build artefact embeds the strings this phase changes. The only persisted state touched
is `default_models.json` → the resolved session model, which is read at startup, and session files that
record the model id they were started with (resume keeps the recorded model — SESS-01 territory).

## Validation Architecture

### Test Framework

| Property | Value |
|---|---|
| Framework | Rust `libtest` (cargo test), `tokio::test` for async, `serial_test` where env is global |
| Config file | none (workspace `Cargo.toml`, `.cargo/config.toml` sets `RUST_MIN_STACK`) |
| Quick run command | `cargo test -p xai-grok-sandbox --lib` / `cargo test -p xai-grok-status-line --lib` / `cargo test -p xai-grok-pager --lib` |
| Full suite command | `scripts/test-gate.sh` (see 3b/3c) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|---|---|---|---|---|
| SBOX-01 | skip non-traversable parent (both shapes) | unit | `cargo test -p xai-grok-sandbox --lib runtime_sockets::tests` | extends `runtime_sockets_tests.rs` |
| SBOX-01 | traversable + unreadable endpoint fatal | unit (pure predicate) | same | ❌ new |
| SBOX-02 | root-only parent does not abort bwrap | unit/integration | `cargo test -p xai-grok-sandbox --lib` + `--test deny_paths_e2e` | ✅ extend |
| SBOX-02 | IPv6 destination reported | script | `scripts/test-sandbox-run.sh` | ❌ new |
| TEST-01 | completeness assertion | script | `scripts/test-gate.sh` | ❌ new |
| TEST-02 | config-docs reached | script | step 2 of `scripts/test-gate.sh` | ❌ new (tests exist) |
| TEST-03 | flake deterministic/quarantined | repeated run | `for i in 1..5; cargo test -p xai-grok-workspace --lib` | ✅ tests exist |
| EVID-01 | cast + report.json committed | script + manual | `scripts/evidence.sh` in each item | ❌ new |
| EVID-02 | convention documented | docs | `docs/evidence/README.md` | ❌ new |
| COST-01 | flash alias priced at Flash | unit | `cargo test -p xai-grok-status-line --lib pricing` | ✅ extend |
| COST-02 | label present | unit + pty | `cargo test -p xai-grok-pager --lib status_line` | ✅ extend (`segments_tests.rs`) |
| COST-03 | headless honours override | unit | `cargo test -p xai-grok-pager --lib headless` | ✅ extend |
| DEF-01 | flash is the resolved default | unit | `cargo test -p xai-grok-shell --lib model_catalog` | ❌ new |
| DEF-01 | docs state caveat + revert | manual/review | docs diff | ❌ new |

### Sampling Rate

- **Per task commit:** the crate-local command for the crate touched.
- **Per wave merge:** `cargo test -p xai-grok-sandbox --lib && cargo test -p xai-grok-status-line --lib && cargo test -p xai-grok-pager --lib`.
- **Phase gate:** `scripts/test-gate.sh` green, with the recorded evidence committed.

### Wave 0 Gaps

- [ ] `crates/codegen/xai-grok-sandbox/src/runtime_sockets_tests.rs` — new fixture matrix.
- [ ] `crates/codegen/xai-grok-status-line/src/pricing.rs` — alias entry + worked example.
- [ ] `crates/codegen/xai-grok-pager/src/views/status_line/segments_tests.rs` — new label format.
- [ ] `crates/codegen/xai-grok-pager/src/headless_tests.rs` + `reducer/messages/tests/result_usage.rs` — override cases.
- [ ] `crates/codegen/xai-grok-shell` model-catalog test for the no-preference default.
- [ ] `scripts/test-gate.sh`, `scripts/test-sandbox-run.sh`, `scripts/evidence.sh`, `docs/evidence/README.md`.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|---|---|---|---|---|
| `rustc`/`cargo` | everything | ✓ | 1.94.0 | — |
| `bwrap` | deny_paths_e2e, sandbox | ✓ | 0.13.1 | test soft-skips (`deny_paths_e2e.rs:84-110`) |
| `flock` | gate serialisation | ✓ | util-linux | none — required by team rules |
| `jq` | evidence + counts | ✓ | ✓ | `python3` |
| `python3` | evidence, IPv6 probe | ✓ | 3.14.10 | — |
| `cc`/`gcc` | shim compile | ✓ | ✓ | pre-built `.so` (not preferred) |
| `script` | headless cast | ✓ | util-linux 2.42.4 | log-only evidence with a reason |
| `asciinema` | — | ✗ | — | pty harness `write_cast` (decided) |
| `strace` | — | ✗ | — | not needed this phase |
| `/data` | devbox bwrap plan | ✗ | — | see 01-05 item 4 |
| free disk | gate | ✓ | 1018 GB | `disk-guard.sh 150` |

**Missing dependencies with no fallback:** none.
**Missing dependencies with fallback:** `asciinema` (harness), `/data` (devbox decision), `strace`
(not needed).

## Security Domain

`security_enforcement` is absent from `.planning/config.json` → treated as enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---|---|---|
| V2 Authentication | no | env-key only, untouched |
| V3 Session Management | no | untouched |
| V4 Access Control | yes | sandbox deny set must stay fail-closed; the fix narrows, never widens |
| V5 Input Validation | yes | path parsing/validation in `runtime_sockets.rs`; no new input surface |
| V6 Cryptography | no | no crypto change |
| V7 Error handling / logging | yes | skipped candidate logs path only via `tracing::debug!`; no secrets |
| V12 Files/resources | yes | `chmod 000` fixtures are temp-dir scoped and removed |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---|---|---|
| Deny-list relaxation exposing a runtime socket | Elevation of privilege | Skip only when the parent is provably not traversable; keep `verify_runtime_socket_deny` strict |
| Symlink endpoint redirecting a mask | Tampering | existing `endpoint is a symlink` fatal (`runtime_sockets.rs:146-151`) |
| Gate green because tests were skipped | Repudiation | count `test result:` against binaries + doctests; no new skip conditions without a note |
| Key material in committed evidence | Information disclosure | extend `key-leak-check.sh` to `docs/evidence`; normalize paths |
| Budget/`estimate` mislabel | (integrity of claim) | `estimate: true` stays beside every table-derived figure |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|---|---|---|
| A1 | The author's host failure is the shape-A (`/run/containerd` as final component) EACCES; the historical bwrap message implies a traversable-but-unwritable parent, which the locked predicate does not cover | 1b/1f | The host still aborts after 01-01 and a second predicate decision is needed |
| A2 | `devbox_genuine_reexec_applies_enforcement` is a host-state assumption and can be guarded on an empty plan | 3f/01-05.4 | If the reviewer rejects a skip, only a product change (Option 2) keeps the test unmodified |
| A3 | `resolve_default_model`'s `visible.first()` is the effective default path for a clean config | 6c | If some layer always sets `models.default`, flipping the JSON is sufficient and option A/B is over-engineering |
| A4 | Adaptive precision (`4 dp` below $0.01, `2 dp` above) is acceptable for "labelled as an estimate" | 5b | If the author wants the literal `~$0.0042 est.` at all magnitudes, drop the adaptive branch |
| A5 | A 5× repetition under load is enough evidence to call the workspace flake "not reproducible" | 3e | If it still rotates, quarantine is the fallback and must name tests |
| A6 | `script(1)` typescript → cast v2 conversion is acceptable evidence for non-TUI items | 4d | Otherwise only TUI items can carry casts, and EVID-01 needs a narrower reading |

## Open Questions

1. **[devbox test disposition]** Guard with a recorded skip, or always re-exec for devbox?
   - Known: `None` is correct on a `/data`-less host; the test asserts the opposite.
   - Unclear: which option the author accepts for `deny_paths_e2e` "unmodified".
   - Recommendation: recorded skip keyed on the empty plan (Option 1), with the `KNOWN-ISSUES.md` entry.
2. **[shape-A predicate]** Confirm that skipping an endpoint inside a non-searchable parent is
   acceptable even though the locked wording names `canonicalize(parent)`.
   - Recommendation: yes — it is the same "not traversable" condition and strictly narrower than
     treating EACCES as NotFound.
3. **[default-model mechanism]** Reorder the JSON or change the resolver (6c options A/B)?
4. **[cost precision]** Fixed 4 dp everywhere vs adaptive (A4).
5. **[evidence helper language]** `sh` + `jq` or `python3`? Both available; `jq` keeps it shell-native,
   Python makes the cast conversion cleaner.

## Sources

### Primary (HIGH confidence)
- `crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs` (read in full), `profiles.rs:366-399`,
  `lib.rs:54-66,440-560,625-650`, `read_deny_verify.rs:380-495`
- `crates/codegen/xai-grok-sandbox/tests/deny_paths_e2e.rs` (read in full), `src/runtime_sockets_tests.rs`
- `crates/codegen/xai-grok-shell/src/config/mod.rs:1276-1400`
- `crates/codegen/xai-grok-status-line/src/pricing.rs`, `config.rs:199-204`
- `crates/codegen/xai-grok-pager/src/headless.rs:100-140,280-340,890-905`,
  `headless/reducer/messages/usage.rs`, `headless/reducer/mod.rs:432-437`,
  `app/status_line_policy.rs:275-290`, `app/event_loop.rs:2804-2818`, `views/status_line/segments.rs`
- `crates/codegen/xai-grok-pty-harness/src/lib.rs:483-514`, `tests/pty_e2e/common.rs:1563-1597`,
  `src/scripted.rs:1136-1173`, `src/bin/pty_scenario.rs`
- `crates/codegen/xai-grok-models/default_models.json`, `src/lib.rs`,
  `xai-grok-shell/src/agent/model_catalog/resolution.rs:32-145`,
  `xai-grok-shell/src/agent/config.rs:2454-2470,2824-2900`
- `scripts/sandbox-run.sh`, `scripts/egress_log.c`, `scripts/egress-check.sh:505-560`,
  `scripts/disk-guard.sh`, `scripts/key-leak-check.sh`
- `dunce-1.0.5/src/lib.rs:65-76` (registry source)
- Host probes: Rust `canonicalize`/`lstat`/`parent/.` matrix; `bwrap 0.13.1` bind-over tests;
  `cargo test -p xai-grok-status-line --lib|--doc` line format; `cargo metadata` counts

### Secondary (MEDIUM confidence)
- `.planning/research/PITFALLS.md` §2, §3, §4, §12, M5, M8 (phase guidance, consistent with this tree)
- `.planning/codebase/CONCERNS.md` (known bugs section)

### Tertiary (LOW confidence)
- The historical `/run/containerd` attribution in `KNOWN-ISSUES.md:25-31` — contradicted by this
  tree's devbox policy (see 1f); not reproducible on this host.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependency, all tools verified present
- Architecture: HIGH — every path and line read in this session
- Pitfalls: HIGH for repo mechanics; MEDIUM for the historical sandbox abort (A1)
- Sandbox predicate gap: HIGH — proved with a Rust probe on this host

**Research date:** 2026-11-21
**Valid until:** 14 days (fast-moving phase; the sandbox/deny code and the model catalog are the two
areas most likely to drift)
