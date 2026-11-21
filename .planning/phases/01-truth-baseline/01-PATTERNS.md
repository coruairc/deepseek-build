# Phase 1: Truth Baseline - Pattern Map

**Mapped:** 2026-11-21
**Files analyzed:** 37 (29 modify, 8 create, plus evidence artifacts)
**Analogs found:** 37 / 37 (0 with no analog; 5 are "extend this file" self-analogs)
**Tree state:** `main` @ `221a1a59` lineage, working tree clean at map time (`git status --short` empty).

Every path below was verified git-tracked with `git ls-files -- <path>`.
`docs/evidence/` does not exist yet and is **not** gitignored
(`git check-ignore -v docs/evidence/...` → rc 1), so committed evidence is tracked normally.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs` | library (path policy) | transform | itself — `materialize_runtime_socket_deny_paths_from` + `normalize_existing_parent_alias` | extend (self) |
| `crates/codegen/xai-grok-sandbox/src/runtime_sockets_tests.rs` | test (unit module) | transform | itself — `temp_runtime_root` + mode-000 fixture tests | extend (self) |
| `crates/codegen/xai-grok-sandbox/tests/deny_paths_e2e.rs` | test (integration, subprocess) | request-response | itself — `run_deny_case` custom-profile writer | extend (self) |
| `scripts/sandbox-run.sh` | dev script (bash + C heredoc) | event-driven (syscall interpose) | itself; IPv6 branch from `scripts/egress_log.c` | extend (self) |
| `scripts/egress_guard.c` *(new, discretion)* | dev script (C shim) | event-driven | `scripts/egress_log.c` | role-match |
| `scripts/test-sandbox-run.sh` *(new)* | dev script (test harness) | batch | `scripts/test-install.sh` | exact |
| `scripts/test-gate.sh` *(new)* | dev script (gate) | batch | `scripts/check-egress.sh` (bash/strict) + `scripts/disk-guard.sh` (call convention) | role-match |
| `scripts/test-gate.expected` *(new)* | config data (checked-in count + provenance) | batch | `docs/LICENSE-AUDIT.md` style header; no numeric analog | partial |
| `scripts/evidence.sh` *or* `.py` *(new)* | dev script (helper) | file-I/O | `scripts/evals/harness.py` (+ `report.py`) for JSON authoring | role-match |
| `docs/evidence/README.md` *(new)* | docs | — | `TESTING.md` / `docs/DROPPED-TESTS.md` doc conventions | partial |
| `docs/evidence/01-truth-baseline/<item>/{<item>.cast,report.json,notes.md}` *(new)* | evidence artifact | file-I/O | `ScriptedRunReport` (`pty-harness/src/scripted.rs:1137-1173`) schema + `write_cast` | role-match |
| new `crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/*.rs` *(optional, for the cost cast)* | test (pty e2e) | streaming | `tests/pty_e2e/common.rs:1563-1576` `write_cast_if_requested` | exact |
| `scripts/egress-check.sh` (prose only, lines 552-558) | dev script (docs text) | — | itself | extend (self) |
| `scripts/README.md` | docs (scripts index) | — | itself (one bullet per script) | extend (self) |
| `scripts/key-leak-check.sh` | dev script (secret scan) | file-I/O | itself; extra-path arg style from `check-egress.sh` `EXCLUDES` | extend (self) |
| `crates/codegen/xai-grok-status-line/src/pricing.rs` | library (price table) | transform | itself — `deepseek_defaults`, `flash_worked_example_is_exact` | extend (self) |
| `crates/codegen/xai-grok-pager/src/views/status_line/segments.rs` | TUI view (formatting) | transform | itself — `StatusLineItem::Cost` arm, `fmt_tokens` | extend (self) |
| `crates/codegen/xai-grok-pager/src/views/status_line/segments_tests.rs` | test (unit) | transform | itself | extend (self) |
| `crates/codegen/xai-grok-pager/src/app/status_line_tests.rs` (floor pin) | test (guide-contract) | — | `the_numbers_the_guide_states_are_pinned:416-446` | extend (self) |
| `crates/codegen/xai-grok-pager/src/headless.rs` | service (emitter) | request-response | itself — `attach_cost_estimate:313-340`, `HeadlessEmitter::new:116-132` | extend (self) |
| `crates/codegen/xai-grok-pager/src/headless/reducer/mod.rs` | service (factory) | request-response | itself — `reducer_for:431-438` | extend (self) |
| `crates/codegen/xai-grok-pager/src/headless/reducer/messages/usage.rs` | service (projection) | transform | itself — `estimated_cost_usd:103-117` | extend (self) |
| `crates/codegen/xai-grok-pager/src/headless/reducer/messages/mod.rs` | service (reducer ctor) | transform | itself — `MessagesReducer::new:75-100` | extend (self) |
| `crates/codegen/xai-grok-pager/src/headless_tests.rs` | test (unit) | request-response | `json_result_carries_local_cost_estimate_marked_estimate:665-695` | extend (self) |
| `crates/codegen/xai-grok-pager/src/headless/reducer/messages/tests/result_usage.rs` | test (unit) | transform | itself (`:150-181`) | extend (self) |
| `crates/codegen/xai-grok-models/default_models.json` | config data | — | itself; invariant in `xai-grok-models/src/lib.rs:32-46` | extend (self) |
| `crates/codegen/xai-grok-shell/src/agent/model_catalog/resolution.rs` | library (resolution policy) | transform | itself — `first_or_fallback:93-114` | extend (self) |
| `crates/codegen/xai-grok-shell/src/session/acp_session_impl/context_snapshot.rs` | test (pin) | — | `tokenize_uses_baked_product_default_model:374-377` | extend (self) |
| `crates/codegen/xai-grok-shell/src/agent/folder_trust.rs` | test (env-global) | — | siblings at `:464-466`, `:481-483` | extend (self) |
| `crates/codegen/xai-grok-shell/src/util/config/consent_tests.rs` | test (env-global) | — | `:42-46` (already `serial(DEEPSEEK_BUILD_HOME)`) | extend (self) |
| `crates/codegen/xai-grok-workspace/src/session/git_gate_tests.rs` | test (timing) | — | itself, `git_gate.rs:34-38` test-only TTLs | extend (self) |
| `crates/codegen/xai-grok-workspace/src/restore_fetch_tests.rs` | test (timing) | — | itself, `assert_timeout_about:78-85` | extend (self) |
| `…/docs/user-guide/25-status-line.md` | docs (contract page) | — | itself; guarded by `app/status_line_tests.rs` | extend (self) |
| `…/docs/user-guide/11-custom-models.md` | docs | — | itself | extend (self) |
| `…/docs/user-guide/05-configuration.md` | docs | — | itself | extend (self) |
| `…/docs/user-guide/14-headless-mode.md` | docs | — | itself (bullet at `:194-201`) | extend (self) |
| `README.md`, `docs/TUNING.md`, `KNOWN-ISSUES.md`, `TESTING.md` | docs | — | themselves | extend (self) |

---

## Pattern Assignments

### 1. Sandbox deny materialization (SBOX-01 / SBOX-02)

#### `crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs` (library, transform)

**Analog:** itself. The two decision points to modify, verbatim from the tree:

```rust
// crates/codegen/xai-grok-sandbox/src/runtime_sockets.rs:127-145
let canonical_parent = match dunce::canonicalize(parent) {
    Ok(parent) => parent,
    Err(error) if error.kind() == io::ErrorKind::NotFound => {
        match std::fs::symlink_metadata(&candidate) {
            Err(metadata_error) if metadata_error.kind() == io::ErrorKind::NotFound => {
                continue;
            }
            Ok(_) => return Err(with_context(error)),
            Err(metadata_error) => return Err(with_context(metadata_error)),
        }
    }
    Err(error) => return Err(with_context(error)),
};
let path = canonical_parent.join(file_name);
let metadata = match std::fs::symlink_metadata(&path) {
    Ok(metadata) => metadata,
    Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
    Err(error) => return Err(with_context(error)),
};
```

**Error-context closure to reuse (do not invent another shape)** — `:106-114`:

```rust
let with_context = |error: io::Error| {
    io::Error::new(
        error.kind(),
        format!(
            "could not resolve runtime-socket deny path {}: {error}",
            candidate.display()
        ),
    )
};
```

**Existing `NotFound`-tolerant probe to copy for the traversal test** — `normalize_existing_parent_alias` (`:218-241`) already distinguishes `NotFound` from other kinds with the same match-arm style:

```rust
match std::fs::symlink_metadata(existing) {
    Ok(_) => break,
    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
    Err(error) => return Err(error),
}
```

**Logging convention:** structured `tracing::debug!` with named fields, path only, no stderr — precedent in the same crate at `profiles.rs` / `lib.rs:550`. Target shape:

```rust
tracing::debug!(
    path = %candidate.display(),
    "skipping runtime-socket deny: parent not traversable"
);
```

**Constraints specific to this file**
- `#![deny(clippy::indexing_slicing)]` at `src/lib.rs:26` — never index/slice; use iterators/`get`.
- `dunce::canonicalize` only. `std::fs::canonicalize`, `Path::canonicalize`, `tokio::fs::canonicalize` are disallowed methods in `clippy.toml` (repo root, duplicated in `crates/codegen/clippy.toml` — nearest-wins).
- Skip must remain strictly narrower than "EACCES == NotFound": keep the symlink (`:146-151`), `InvalidInput` (no parent / no file name, `:115-126`) and dedupe (`:152-154`) arms byte-for-byte.
- Make the predicate a pure function so the "traversable parent + unreadable endpoint stays fatal" arm is unit-testable by construction (the filesystem cannot produce it — `lstat` succeeds for a mode-000 *file*).

#### `crates/codegen/xai-grok-sandbox/src/runtime_sockets_tests.rs` (test, unit)

**Analog:** itself. Fixture + cleanup idiom to copy, `:104-116` and `:219-240`:

```rust
#[cfg(unix)]
fn temp_runtime_root(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "grok-runtime-sockets-{tag}-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    root
}

#[test]
#[cfg(unix)]
fn materialized_socket_path_is_stable_after_mode_zero_mask() {
    use std::os::unix::fs::PermissionsExt;
    let root = temp_runtime_root("masked");
    …
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o000)).unwrap();
    let after = materialize_runtime_socket_deny_paths_from([socket.clone()]).unwrap();
    assert_eq!(after, before);
    let _ = std::fs::remove_dir_all(root);
}
```

Also copy: `UnixListener::bind` for live sockets (`:129`), `std::os::unix::fs::symlink` for aliases/symlink-endpoint (`:126`, `:249`), `expect_err(...)` + substring assertion for fatal arms (`:251-257`), and the `#[serial(bwrap_env)]` + `skip_if_host_hook_write_deny_unresolvable()` guard used by profile-level tests (`:384-390`). Every test removes its temp root at the end with `let _ = std::fs::remove_dir_all(root);`.

Test names are full sentences describing the invariant (`materialized_socket_paths_propagate_parent_resolution_errors`), and assertions carry a message string.

#### `crates/codegen/xai-grok-sandbox/tests/deny_paths_e2e.rs` (integration test, subprocess)

**Analog:** itself. The custom-profile writer to model the retarget on — `run_deny_case` (`:854-880`):

```rust
let (home, grok, tmp, _ch, _cg, _cw) = fixture_homes(tag);
let deny_list = deny_entries.iter().map(|p| format!("\"{p}\"")).collect::<Vec<_>>().join(", ");
fs::create_dir_all(tmp.join(".grok")).expect("mkdir .grok");
fs::write(
    tmp.join(".grok").join(xai_grok_config::SANDBOX_CONFIG_FILENAME),
    format!("[profiles.{profile}]\nextends = \"workspace\"\ndeny = [{deny_list}]\n"),
)
.expect("write sandbox.toml");
```

Target test and its current host assumption (`:1139-1164`) — the fix must give `bwrap_reexec_for_profile` a real obligation without touching `skip_if_enforcement_unavailable` (`:84-110`):

```rust
#[test]
fn devbox_genuine_reexec_applies_enforcement() {
    if skip_if_enforcement_unavailable() { return; }
    let (home, grok, workspace, _ch, _cg, _cw) = fixture_homes("devbox-genuine");
    …
    .env(SCENARIO_ENV, "devbox_genuine")
```

Subprocess plumbing to reuse unchanged: `apply_fixture_env` (`:21-27`), `.arg("--ignored").arg("--exact").arg("--nocapture").arg("subprocess_entry")` (`:43-48`), `SCENARIO_ENV` dispatch (`:221-249`), `subprocess_devbox_genuine` (`:608-635`) and `subprocess_profile_and_bwrap_reexec` (`:252-290`). Note `subprocess_devbox_genuine` already prints `OK: devbox enforcement applied inside genuine bwrap`; keep that exact marker string, since the parent test asserts it.

Product seam if a unit proof is preferred instead (`crates/codegen/xai-grok-sandbox/src/lib.rs:625-645`): `bwrap_reexec_for_profile` returns `None` only when *all four* plan inputs are empty, so a `Custom("…")` profile with `extends = "devbox"` + one `deny` entry (or `restrict_network = true`) always produces `Some`.

---

### 2. `sandbox-run.sh` IPv6 (SBOX-02 second half)

#### `scripts/sandbox-run.sh` (dev script, syscall interpose)

**Analog:** itself. The three sites to change, `:42-90`:

```c
static char allowed[16][16];           /* IPv4 strings only */
…
hints.ai_family = AF_INET;             /* resolves v4 only */
…
if (addr->sa_family == AF_INET) { … strncmp(ip,"127.",4) … }
else if (addr->sa_family == AF_INET6) { … strcmp(ip, "::1") … }
…
char what[64] = "non-ip";              /* AF_INET6 falls through */
if (addr->sa_family == AF_INET) { inet_ntop(AF_INET, …, what, sizeof what); }
dprintf(2, "sandbox-run: BLOCKED connect to %s\n", what);
```

**IPv6 formatting to copy** — `scripts/egress_log.c:77-82` (and the identical `:136-141` in `connect`):

```c
} else if (addr->sa_family == AF_INET6) {
    struct sockaddr_in6 *s = (struct sockaddr_in6 *)addr;
    char ip[INET6_ADDRSTRLEN] = "?";
    inet_ntop(AF_INET6, &s->sin6_addr, ip, sizeof ip);
    n = snprintf(buf, sizeof buf, "%s family=inet6 addr=%s port=%u\n", kind, ip,
                 (unsigned)ntohs(s->sin6_port));
}
```

**Script header convention** every script in this repo follows (`sandbox-run.sh:1-27`): shebang + purpose block + explicit note that the repo lints with `shellcheck --shell=sh`, `# shellcheck disable=SC3040`, `set -euo pipefail`, `cd "$(dirname "$0")/.."`, `mktemp -d` + `trap 'rm -rf "$WORK"' EXIT`.

Keep: `EGRESS_ALLOW_HOST` env contract, the `sandbox-run: allowing only …` startup line, `ECONNREFUSED` refusal, and `LD_PRELOAD` + `exec env`. Add (if extraction is chosen) `EGRESS_ALLOW_DEBUG=1` printing the resolved allowlist with `family=inet6`, matching `egress_log.c`'s `addr=… port=…` line shape so `egress-check.sh`'s parser keeps working.

#### `scripts/egress_guard.c` (new, agent discretion)

**Analog:** `scripts/egress_log.c`. Copy its structure: `#define _GNU_SOURCE`, `dlsym(RTLD_NEXT, …)` cached function pointers, `open_log()`-style lazy init, a single `log_sockaddr`-equivalent helper that switches on `sa_family`, and `inet_ntop(AF_INET6, …, INET6_ADDRSTRLEN)`. Do **not** copy its never-block behaviour — this shim must still refuse (`errno = ECONNREFUSED; return -1;`).

#### `scripts/test-sandbox-run.sh` (new)

**Analog:** `scripts/test-install.sh` — this is the repo's canonical assertion harness. Copy its skeleton:

```sh
#!/bin/sh
# …purpose block: what it builds, what it asserts, "No network beyond 127.0.0.1. Cleans up via trap."
# Run: sh scripts/test-sandbox-run.sh
set -eu
ROOT=$(cd "$(dirname "$0")/.." && pwd)
PASS=0
FAIL=0
FAILED_NAMES=""
say() { printf '%s\n' "$*"; }
pass() { PASS=$((PASS + 1)); say "PASS: $1"; }
fail() { FAIL=$((FAIL + 1)); FAILED_NAMES="$FAILED_NAMES $1"; say "FAIL: $1"; if [ -n "${2:-}" ]; then say "      detail: $2"; fi; }
assert_contains() { case "$1" in *"$2"*) pass "$3" ;; *) fail "$3" "output missing: $2" ;; esac; }
assert_not_contains() { case "$1" in *"$2"*) fail "$3" "output unexpectedly contains: $2" ;; *) pass "$3" ;; esac; }
…
say ""
if [ "$FAIL" -eq 0 ]; then say "ALL PASS ($PASS assertions)"; exit 0
else say "FAILED: $FAIL of $((PASS + FAIL)) assertions ($FAILED_NAMES)"; exit 1; fi
```

(`test-install.sh:1-64` helpers, `:492-502` summary.) Also copy its subprocess-in-capture idiom `OUT=$(… 2>&1) && RC=0 || RC=$?` (`:439-446`) and its temp-dir/trap cleanup. Concrete assertions required by RESEARCH §2c: IPv6 destination reported as the literal address; no `non-ip`; `::1` allowed; allowlist carries an IPv6 entry under `EGRESS_ALLOW_DEBUG=1`.

Also: `shellcheck --shell=sh` and `dash -n` clean (AGENTS §6 lists those checks for `install.sh`/`test-install.sh`; the same gate applies here).

#### `scripts/egress-check.sh` (prose only) and `scripts/README.md`

**Analog:** themselves. `egress-check.sh:551-558` currently documents the bug:

```sh
note "note: sandbox-run.sh formats only AF_INET destinations; its AF_INET6/other"
note "      refusals print as 'non-ip'. The provider is reached over its IPv6"
```

Replace with prose describing the fixed behaviour; keep the `note`/`hdr`/`ok`/`bad` helper vocabulary already used in that file (`:534-564`). `scripts/README.md` is a one-bullet-per-script list — add `test-gate.sh`, `test-sandbox-run.sh`, `evidence.sh` in the same voice as `test-install.sh`'s bullet.

---

### 3. Cost honesty (COST-01/02/03)

#### `crates/codegen/xai-grok-status-line/src/pricing.rs` (library, transform)

**Analog:** itself. Table construction (`:76-90`) and the worked-example test style (`:182-190`):

```rust
pub fn deepseek_defaults() -> Self {
    let mut models = BTreeMap::new();
    models.insert(DEFAULT_PRICING_MODEL.to_string(), ModelPricing::default());
    models.insert(FLASH_PRICING_MODEL.to_string(), ModelPricing::flash());
    Self { models }
}
…
/// Flash, the same buckets: 1000*0.006 + 2000*0.30 + 500*1.20 = 6 + 600 + 600 = 0.001206.
#[test]
fn flash_worked_example_is_exact() {
    let table = PricingTable::default();
    let cost = table.cost_usd(FLASH_PRICING_MODEL, &usage(0, 500, 1_000, 2_000)).expect("cost");
    assert!((cost - 0.001206).abs() < 1e-12, "{cost}");
}
```

Add `pub const FLASH_ALIAS_PRICING_MODEL: &str = "deepseek-v4-flash";` next to `:24-27`, insert it with `ModelPricing::flash()`, and add a `deepseek-v4-flash` worked example (same buckets → `0.001206`) plus a table-shape assertion in `default_table_prices_both_labelled_models_at_peak` (`:148-160`). `#![deny(clippy::indexing_slicing)]` (`src/lib.rs:6`). Doc-comment style: every public item carries the provider's numbers and an "off-peak is half" note.

#### `crates/codegen/xai-grok-pager/src/views/status_line/segments.rs` (TUI view, transform)

**Analog:** itself — the arm to replace (`:18`, `:104-108`):

```rust
const MIN_DISPLAYED_COST_USD: f64 = 0.005;
…
StatusLineItem::Cost => ctx
    .cost
    .total_cost_usd
    .filter(|usd| *usd >= MIN_DISPLAYED_COST_USD)
    .map(|usd| StatusSegment::dim(format!("${usd:.2}"))),
```

Formatting-helper conventions to mirror (pure `fn`, doc comment stating the exact render, unit scale documented) — `fmt_tokens` (`:153-162`):

```rust
/// `999`, `1.2k`, `100k`, `1.2m`. Truncates rather than rounds up so the segment never overstates the bill.
fn fmt_tokens(n: u64) -> String {
```

Implementation notes bound by `01-UI-SPEC.md` (approved, binding):
- Frame `~` + `$` + number + `" est."`; 4 dp when the value **rounds to < $0.01 at 4 dp**, else 2 dp; floor `MIN_DISPLAYED_COST_USD = 0.0001`; non-finite/negative → the arm returns `None` (same as absent). `0.00996` must render `~$0.01 est.`.
- Tone stays `StatusSegment::dim`. No new theme token, no `Warn`, no second segment, no `fit_columns`.
- Do **not** add `Cost` to `varies_mid_turn`.
- Make `MIN_DISPLAYED_COST_USD` `pub(crate)` so `app/status_line_tests.rs` can pin it (spec Verification Hooks).

#### `crates/codegen/xai-grok-pager/src/views/status_line/segments_tests.rs` (test, unit)

**Analog:** itself — the two assertions that move (`:23-45`):

```rust
#[test]
fn composes_every_segment_in_order() {
    assert_eq!(
        plain(&context(), Some(Duration::from_secs(83))),
        "project │ deepseek-build │ 42% ctx │ $0.37 │ 1m23s │ status_line work"
    );
}

#[test]
fn omits_segments_whose_data_is_missing_or_rounds_to_zero() {
    let mut ctx = context();
    ctx.cost.total_cost_usd = Some(0.004);
    …
    ctx.cost.total_cost_usd = Some(0.006);
    assert!(plain(&ctx, None).contains("$0.01"));
```

New full-row string per spec: `project │ deepseek-build │ 42% ctx │ ~$0.37 est. │ 1m23s │ status_line work`. Required new cases: floor omit `0.00004`, floor show `0.0001`, promotion `0.00996 → ~$0.01 est.`, two-digit integer width (`1234.5 → ~$1234.50 est.`), non-finite omit. Reuse `context()` / `plain()` helpers (`:5-21`).

#### `crates/codegen/xai-grok-pager/src/app/status_line_tests.rs` (floor pin)

**Analog:** `the_numbers_the_guide_states_are_pinned` (`:416-446`) — append a `MIN_DISPLAYED_COST_USD` assertion in the same `assert_eq!(CONST, value, "the guide says …")` form. Note the page-contract tests at `:400,446` (`every_config_key_the_section_serializes_is_named_in_the_guide`) already read `25-status-line.md` via `include_str!`, so the edited page must keep the `cost` row as a table cell and keep every key name.

#### `crates/codegen/xai-grok-pager/src/headless.rs` (service, request-response)

**Analog:** itself. Construction (`:116-132`) and the hardcoded table (`:330-335`):

```rust
fn new(format: OutputFormat, parse_structured_output: bool) -> Self {
    Self { …, reducer: reducer_for(format), … }
}
…
let model = self.session_model.as_deref().unwrap_or("");
let Some(estimate) =
    xai_grok_status_line::PricingTable::default().cost_usd(model, &session_usage)
else { return; };
```

The resolution the headless path must match is `load_initial_ui_config` (`app/event_loop.rs:2802-2818`), whose body is the pattern to copy at the call site:

```rust
let Ok(root) = xai_grok_shell::config::load_effective_config() else {
    return UiConfig::default();
};
let Some(ui_value) = root.get("ui").cloned() else {
    return UiConfig::default();
};
ui_value.try_into::<UiConfig>().unwrap_or_default()
```

then `ui.status_line.pricing()` (`xai-grok-status-line/src/config.rs:198-204`):

```rust
pub fn pricing(&self) -> PricingTable {
    match &self.pricing {
        Some(overrides) => PricingTable::with_overrides(overrides.clone()),
        None => PricingTable::default(),
    }
}
```

Placement (RESEARCH §5e): `run_single_turn` already loads `raw_config` at `:898`, after `HeadlessEmitter::new` at `:889` — move the emitter construction below the load and pass the table in. Do not cache the TUI's `current_ui`; it is re-read on config reload by design.

#### `headless/reducer/mod.rs`, `messages/usage.rs`, `messages/mod.rs` (service, transform)

**Analog:** themselves. Factory (`reducer/mod.rs:431-438`):

```rust
pub(crate) fn reducer_for(format: OutputFormat) -> Option<Box<dyn Reducer>> {
    match format {
        OutputFormat::StreamingJson => Some(Box::new(AcpReducer)),
        OutputFormat::StreamingMessagesJson => Some(Box::new(MessagesReducer::new())),
        OutputFormat::Plain | OutputFormat::Json => None,
    }
}
```

Reducer ctor (`messages/mod.rs:75-100`) — add a table field; keep `new()` delegating to `new_with_pricing(PricingTable::default())` so the ~6 existing reducer tests stay untouched (they call `messages(false)` helpers in `tests/`).

Projection (`messages/usage.rs:103-117`): replace `PricingTable::default()` with the injected table. Keep the doc comments that explain the disjoint bucket split and the "no tokens → `None`" rule; keep `estimate: true` beside every table-derived figure.

Tests to extend, not replace: `headless_tests.rs:665-704` (`json_result_carries_local_cost_estimate_marked_estimate`, `…omits_estimate_when_no_tokens_recorded`) and `headless/reducer/messages/tests/result_usage.rs:150-181` (pro-default fallback for an unknown model). Add one override-changes-the-number case per surface plus one cross-surface equality case.

---

### 4. Default model (DEF-01)

#### `crates/codegen/xai-grok-models/default_models.json` (config data)

**Analog:** itself. `:2` `"default": "deepseek-v4-pro"`; array order `deepseek-v4-pro` (`:8`), `deepseek-flash` (`:53`), `deepseek-v4-flash` (`:98`, `"hidden": true` at `:141`). The compile-time guard that will fire if the id is wrong — `xai-grok-models/src/lib.rs:32-46`:

```rust
let model_ids: Vec<&str> = defaults.models.iter().map(|m| m.model.as_str()).collect();
assert!(
    model_ids.contains(&defaults.default.as_str()),
    "default_models.json: 'default' is '{}' but 'models' array only has {model_ids:?}",
    defaults.default,
);
```

Keep JSON order unless Option A is chosen; `default_model()` returns the field verbatim (`lib.rs:47-49`).

#### `crates/codegen/xai-grok-shell/src/agent/model_catalog/resolution.rs` (resolution policy)

**Analog:** itself. The closure to change (`:93-114`) and the fallback it currently trusts (`:32-37`):

```rust
let first_or_fallback = || -> (String, ModelEntry) {
    if let Some((key, first)) = visible.first() {
        return (key.clone(), first.clone());
    }
    …
    let default_id = fallback_model_id(cfg, local_pref.map(|p| p.value.as_str()));
    tracing::warn!(model = %default_id, "no selectable models; using the fallback model id");
```

```rust
pub(crate) fn fallback_model_id(cfg: &config::Config, preferred: Option<&str>) -> String {
    match CatalogSource::for_config(cfg) {
        CatalogSource::Standard => crate::models::default_model().to_owned(),
        CatalogSource::ModelsEndpoint => preferred.unwrap_or_default().to_owned(),
    }
}
```

Option B (recommended by RESEARCH §6c, locked by CONTEXT "resolver prefer `xai_grok_models::default_model()` when visible") means: inside `first_or_fallback`, before `visible.first()`, try `visible.get_key_value(crate::models::default_model())`. Keep the existing `tracing::warn!` fallbacks and the `config::ConfigSource` return contract unchanged. Note `models.default` in config wins over this (that path is `:116-145` and must not change).

There is **no test module in this file**; the existing coverage lives in `agent/config_tests.rs` (~20 sites using `let dm = crate::models::default_model();`) and `model_catalog/mod.rs` (integration via `resolve_default_model` at `:148,592,606`). Add the "resolved default with no preference" test where the catalog is built, following `config_tests.rs`'s dynamic-`default_model()` style rather than hardcoding an id.

#### `crates/codegen/xai-grok-shell/src/session/acp_session_impl/context_snapshot.rs` (test pin)

**Analog:** `:374-377`, the only hard pin:

```rust
#[test]
fn tokenize_uses_baked_product_default_model() {
    assert_eq!(crate::models::default_model(), "deepseek-v4-pro");
}
```

Change to `"deepseek-flash"`. Alternatively assert against `crate::models::default_model()`'s JSON field only if it still proves the baked value — but the test's purpose is to pin the literal, so keep it literal.

#### Docs that state the default (DEF-01 documentation)

Exact current lines (all verified present):

| File:line | Current text |
|---|---|
| `README.md:247` | "Models: `deepseek-v4-pro` (default) and `deepseek-flash`, plus a hidden `deepseek-v4-flash` legacy alias" |
| `README.md:~25` (phase table) | "DeepSeek catalog (default `deepseek-v4-pro`)" |
| `README.md:~270` | `/model` row: "switch between `deepseek-v4-pro` and `deepseek-flash`" |
| `…/11-custom-models.md:9` | "new sessions start with `deepseek-v4-pro`" |
| `…/11-custom-models.md:51` | `default = "deepseek-v4-pro"` example |
| `…/05-configuration.md:51` | `default = "deepseek-v4-pro"` example |
| `docs/TUNING.md:19-37` | baseline table + findings ("Both models pass every task") |

`26-config-reference.md:352` states no default value → **no edit** (and it is hand-edited + `config_docs`-guarded; see `crates/codegen/xai-grok-shell/AGENTS.md`).

`docs/TUNING.md` is where the at-ceiling caveat and revert path belong: same table/finding voice — factual bullets, measured numbers, date + command provenance. Revert path: `/model deepseek-v4-pro`, `-m deepseek-v4-pro`.

---

### 5. Test determinism (TEST-03)

#### `crates/codegen/xai-grok-shell/src/agent/folder_trust.rs` (test, env-global)

**Analog:** its siblings. `record_and_lookup_round_trip` (`:451-462`) is the only test in the module without `#[serial_test::serial]`; neighbours use the bare attribute (`:464-466`):

```rust
#[test]
#[serial_test::serial]
fn grant_folder_trust_seeds_decisions_cache() {
    let _sim = simulate_release_build();
    let home = tempfile::tempdir().unwrap();
    let _env = EnvGuard::set("DEEPSEEK_BUILD_HOME", home.path());
```

Add `#[serial_test::serial]` (bare, matching this file). Keep the existing comment block — it explains why unique keys/temp homes matter.

#### `crates/codegen/xai-grok-shell/src/util/config/consent_tests.rs` (test, env-global)

**Analog:** itself — `:42-46` **already** has the attribute, but with a *named* key:

```rust
#[tokio::test]
#[serial_test::serial(DEEPSEEK_BUILD_HOME)]
async fn set_consent_answer_is_monotonic_per_account() {
    let home = tempfile::tempdir().expect("home");
    let _guard = xai_grok_test_support::env::EnvGuard::set("DEEPSEEK_BUILD_HOME", home.path());
```

`#[serial_test::serial]` (bare, as in `folder_trust.rs`) and `#[serial(DEEPSEEK_BUILD_HOME)]` take **different keys and never exclude each other**. The deterministic fix is to make the key uniform across the two modules (add the named key in `folder_trust.rs`, or switch this one to bare) — otherwise the "add `#[serial]`" one-liner in RESEARCH §3f leaves the race intact. Prefer one convention repo-wide; check neighbours for which is dominant before choosing.

#### `crates/codegen/xai-grok-workspace/src/session/git_gate_tests.rs` + `src/restore_fetch_tests.rs` (timing)

**Analog:** themselves. The under-margin constants come from test-only TTLs — `git_gate.rs:34-38`:

```rust
const ROOT_CACHE_TTL: Duration = if cfg!(test) {
    Duration::from_millis(80)
} else {
    Duration::from_secs(30)
};
```

and the sleep that only has 20 ms of headroom (`git_gate_tests.rs:513-524`):

```rust
git2::Repository::init(&nested).unwrap();
tokio::time::sleep(Duration::from_millis(100)).await;
let after = super::canonical_git_root(&nested).await.unwrap();
assert_ne!(before, after);
```

`restore_fetch_tests.rs` has the shared band helper (`:78-85`) — widening must go through it, not per-test magic numbers:

```rust
fn assert_timeout_about(actual: Duration, target: Duration) {
    assert!(actual <= target, "timeout {actual:?} exceeds {target:?}");
    assert!(
        actual >= target.saturating_sub(Duration::from_secs(2)),
        "timeout {actual:?} is not close to {target:?}"
    );
}
```

Rule: raise only the *under-margin* constants so the ordering invariant is unchanged (RESEARCH §3e-i). `Instant`-based, so `#[tokio::test(start_paused = true)]` does not help. If still flaky after a recorded 5× repetition under load, quarantine with the repo's `#[ignore = "<reason>"]` form (precedents: `session/persistence_tests.rs:1063`, `tests/subagent_bootstrap_regression.rs:17`, `pty_e2e/*:13`) and add the named test to `KNOWN-ISSUES.md`.

---

### 6. Test gate + evidence (TEST-01/02, EVID-01/02)

#### `scripts/test-gate.sh` (new, gate)

**Analogs:** `scripts/check-egress.sh` (bash + `set -euo pipefail` + shellcheck header + explicit exit codes) and `scripts/disk-guard.sh` (the guard the gate must call):

```sh
#!/bin/sh
# Refuse to start a heavy build when … Prints the free space and exites non-zero below the threshold (default 60 GB).
# Usage: scripts/disk-guard.sh [minimum_gb]
set -eu
min="${1:-60}"
root=$(unset CDPATH; cd -- "$(dirname -- "$0")/.." && pwd)
target="${CARGO_TARGET_DIR:-$root/target}"
…
[ "$free_gb" -ge "$min" ]
```

Requirements the gate must encode (CONTEXT + RESEARCH §3a-3d, §01-05.1):
- `scripts/disk-guard.sh 150 || exit 1` — **explicit 150**, not the 60 default.
- `flock -w 900 /tmp/team-build.lock` around the whole run (team rules).
- `env -u DEEPSEEK_API_KEY -u DEEPSEEK_BUILD_API_KEY -u XAI_API_KEY` on both cargo steps.
- step 1 `cargo test --workspace --no-fail-fast -- --test-threads=4`, tee'd to a log; step 2 `cargo test -p xai-grok-shell --lib --features config-docs -j 1`.
- completeness: `results=$(grep -c '^test result:' log)`, `doctests=$(grep -c '^   Doc-tests ' log)`, `binaries=$(cargo test --workspace --no-run --message-format=json | jq -r 'select(.reason=="compiler-artifact" and .profile.test==true and .executable!=null) | .executable' | sort -u | wc -l)`; assert `results == binaries + doctests` **and** `results == expected`.
- `--refresh-expected` rewrites the expected file only when both steps are green, recording commit/date/both derived numbers in the header.
- Non-zero exit on a step failure **or** a count mismatch; never read the tail of the log; print all four numbers.

#### `scripts/test-gate.expected` (new, checked-in count)

No numeric analog exists. Use the provenance-header style of `docs/LICENSE-AUDIT.md` / `docs/license-deny-report.txt`: a short header (`# refreshed by: scripts/test-gate.sh --refresh-expected`, commit, date, `results=N binaries=N doctests=N`) followed by the count. Seed from the first green run — **do not carry 307** (RESEARCH §3c: 223 test targets + 89 doctests = 312 upper bound on this tree).

#### `scripts/evidence.sh` / `.py` (new, helper)

**Analog:** `scripts/evals/harness.py` (Python, stdlib-only, module docstring that states the file format and the usage block) and `report.py` (`argparse`, `json`, `pathlib`; `python3 scripts/evals/report.py --out report.md`). If `sh` is chosen, `jq -n` for JSON assembly. Fixed keys (CONTEXT, locked):

```json
{"item": "...", "requirements": [], "date": "", "host": "",
 "command": "", "steps": [{"name": "", "ok": true, "detail": ""}],
 "artifacts": [{"path": "", "sha256": ""}], "result": "ok|fail|skipped"}
```

The in-tree schema analog for `steps`/`artifacts` is `ScriptedRunReport` (`crates/codegen/xai-grok-pager-pty-harness/src/scripted.rs:1137-1173`), written with `serde_json::to_string_pretty` and `fs::write` (`:1161-1173`). Mirror its field naming (`status` there, `result` here per the locked key list) and pretty-print.

Normalization is mandatory: `<REPO>`, `<HOME>`, `<TMP>` prefixes; never capture env dumps; run `scripts/key-leak-check.sh` over the result.

**Cast generation:** `PtyHarness::write_cast` (`pty-harness/src/lib.rs:485-514`) emits `{"version":2,"width":…,"height":…}` + `[elapsed,"o",data]` lines and creates the parent dir. Drive it via `GROK_PTY_CAST_DIR` + a `#[ignore]`d pty test, exactly as `tests/pty_e2e/common.rs:1563-1576` does:

```rust
pub(crate) fn write_cast_if_requested(harness: &PtyHarness, file_name: &str) {
    let Ok(dir) = std::env::var("GROK_PTY_CAST_DIR") else { return; };
    if dir.is_empty() { return; }
    let path = std::path::PathBuf::from(dir).join(file_name);
    match harness.write_cast(&path) { Ok(()) => eprintln!("wrote cast: {}", path.display()), … }
}
```

No pty test renders the status line today (`grep status_line tests/pty_e2e/` is empty), so the cost item needs a new small `#[ignore]` pty test **or** a `script -T` capture; `script(1)` → cast v2 conversion is ~20 lines (RESEARCH §4d). Do not add `--cast` to `pty_scenario` ("no new product surface").

#### `docs/evidence/README.md` (new, docs)

**Analog:** `TESTING.md` (manual-checklist voice, exact strings/exit codes stated, "Status (date):" banner) and `docs/DROPPED-TESTS.md` (ledger voice, one section per item). Document: layout `docs/evidence/NN-<phase-slug>/<item-slug>/`, the fixed `report.json` keys, the `<REPO>/<HOME>/<TMP>` normalization table, the re-run recipe (`GROK_PTY_CAST_DIR=… cargo test … --ignored`, `scripts/evidence.sh …`, `scripts/key-leak-check.sh docs/evidence`).

#### `scripts/key-leak-check.sh` (dev script, file-I/O)

**Analog:** itself. Current scope (`:37-47`) greps only the isolated `$HOME_DIR` + captured stdout/stderr, and prints file names only:

```sh
if grep -RIl -- "$FAKE_KEY" "$HOME_DIR" "$WORK/stdout.txt" "$WORK/stderr.txt" >"$WORK/hits.txt" 2>/dev/null; then
  hits=$(wc -l <"$WORK/hits.txt")
fi
…
echo "FAIL  fake key found in $hits file(s):"
sed "s#$WORK#<work>#; s#$HOME_DIR#<home>#" "$WORK/hits.txt"
```

Additive change only: accept extra paths as arguments (or a `LEAK_PATTERNS` file) and scan them; keep `-l` (names only), keep redaction of the work/home prefixes, keep the `sk-FAKE-LEAKCANARY-…` canary plus a `sk-[A-Za-z0-9]{20,}` shape. Do not weaken the existing binary-required precheck (`:24-27`) or exit codes.

---

## Shared Patterns

### No new dependencies
`RESEARCH.md` "Standard Stack": none. Do not touch `Cargo.toml`, `Cargo.lock`, `deny.toml`, or `.cargo/`. Everything needed (`serde_json`, `tracing`, `serial_test`, `tempfile`, `git2`, `jq`, `python3`, `cc`, `flock`, `script`) is already present or on the host.

### Clippy bans (repo root `clippy.toml`, nearest-wins duplicate under `crates/codegen/`)
- `std::fs::canonicalize`, `std::path::Path::canonicalize`, `tokio::fs::canonicalize` → **`dunce::canonicalize`**.
- `std::env::home_dir` / `dirs::home_dir` → `xai_dirs::home_dir()` (or `xai_grok_config::grok_home`).
- `std::process::Command::spawn` / `tokio::process::Command::spawn` / `portable_pty::SlavePty::spawn_command` → `xai_tty_utils::ProcessScope::enroll` (existing `deny_paths_e2e` uses raw `Command` in a test binary — tests get the reason; do not add production spawns).
- `reqwest::Client*` → `xai_grok_extra_ca`.
- `#![deny(clippy::indexing_slicing)]` is set on **all six crates this phase touches**: `xai-grok-sandbox/src/lib.rs:26`, `xai-grok-status-line/src/lib.rs:6`, `xai-grok-pager/src/lib.rs:11` (and `main.rs` path), `xai-grok-shell/src/lib.rs:9`, `xai-grok-workspace/src/lib.rs:9`, `xai-grok-models/src/lib.rs:6`. `allow-indexing-slicing-in-tests = true` relaxes `cargo clippy`, but the Bazel lint aspect reads the repo-root config, so **inline `#[cfg(test)]` modules must not index either** — use `get`/iterators and pattern-match, as the existing tests do (`let Some(rule) = rules.first() else { panic!(…) };`).

### Error handling
- Library boundaries: `io::Error::new(kind, format!("…{}: {error}", path.display()))`, then the caller wraps with `anyhow::anyhow!("socket deny resolution failed: {error}")` (`profiles.rs:395-398`).
- Never `unwrap()` in production paths; tests use `unwrap()`/`expect("…")` freely.
- Fail-closed stays fail-closed: the sandbox change narrows a predicate, it does not convert EACCES to NotFound generally.

### Logging
- `tracing::debug!`/`warn!`/`info!`/`error!` with named fields (`path = %…`, `model = %…`). Never `println!`/`eprintln!` in library code; `eprintln!` is reserved for the CLI refusal path (`lib.rs:535`) and test subprocess markers.

### Test style
- Unit tests live beside the code in `*_tests.rs` wired with `#[cfg(test)] #[path = "…_tests.rs"] mod tests;` (`runtime_sockets.rs:296-298`, `segments.rs:164-166`).
- Integration tests live under `tests/`.
- Test names are full sentences; assertions carry a message (`"{cost}"`, `"missing {literal}: {paths:?}"`).
- Floats compare with a tolerance and a message, never `assert_eq!` on `f64`.
- Tests that mutate process-global env use `serial_test` (`EnvGuard::set`/`unset` from `xai_grok_test_support::env` or the crate's local guard) and must share one serial key when they share one variable.
- Evidence/opt-in artifacts are env-gated and non-fatal (`write_cast_if_requested`).

### Scripts
- Repo scripts declare their dialect in the header and disable the shellcheck checks they intentionally break (`# shellcheck disable=SC3040`), because `shellcheck --shell=sh` runs over them.
- Long-running/network-touching scripts use `mktemp -d` + `trap … EXIT`.
- Assertion harnesses print `PASS:`/`FAIL:` per case and a final `ALL PASS (N assertions)` / `FAILED: N of M assertions (…)`, exiting non-zero on any failure.
- No new external tools: `jq`, `python3`, `sha256sum`, `cc`, `flock`, `script` are all present (RESEARCH "Environment Availability"); `asciinema` and `strace` are not and must not be installed.

### Docs
- User guide pages are hand-edited and guarded by tests: `25-status-line.md` by `app/status_line_tests.rs` (`include_str!` + table-cell checks + pinned constants); `26-config-reference.md` by the `config-docs` feature (`crates/codegen/xai-grok-shell/AGENTS.md`). Changing a documented constant without editing the page fails a test.
- `26-config-reference.md` needs **no** change this phase (no key added/removed; `models.default` states no value).
- Tone: declarative present tense, exact rendered strings in backticks, measured numbers with date + command provenance (`docs/TUNING.md`, `KNOWN-ISSUES.md`).
- Do not print or commit secrets; evidence artifacts are normalized and scanned (`scripts/key-leak-check.sh`).

### Branding
- No new user-visible strings beyond the cost figure's `~`/`est.` frame; `xai-grok-brand` constants are untouched, so the `branding_surface` test (`crates/codegen/xai-grok-pager-bin/tests/`) is unaffected. Internal `xai-grok-*` names and `GROK_*` env names stay (D2).

### Egress
- `scripts/check-egress.sh --strict` must stay exit 0; it scans `crates prod third_party` and excludes `*.md`, so new `docs/evidence` markdown/naming cannot trip it. No new outbound destination is introduced (the C shim only *refuses* connects).

---

## No Analog Found

Nothing is without an analog. The weakest matches, with the fallback to use:

| File | Role | Why weak | Fallback |
|---|---|---|---|
| `scripts/test-gate.expected` | config data | No checked-in count file exists anywhere in the tree | Use the `docs/LICENSE-AUDIT.md` header convention; format is agent's discretion (provenance header + one integer) |
| `docs/evidence/README.md` + `docs/evidence/**` | docs / artifacts | `docs/evidence/` does not exist | `TESTING.md` voice + `ScriptedRunReport` schema + `write_cast` output format |
| new cost pty test | test (pty e2e) | No pty test renders the status line today | Copy the smallest existing `tests/pty_e2e/*.rs` + `write_cast_if_requested`; else `script -T` capture |
| IPv6 allowlist assertion | shell test assertion | Offline proof of "resolves IPv6" needs the new `EGRESS_ALLOW_DEBUG=1` line | `egress_log.c`'s `family=inet6 addr=…` line shape |

## Executor Checklist (per plan)

- **01-01 sandbox:** two-branch traversal predicate (shape A: `canonicalize` EACCES; shape B: endpoint `lstat` EACCES **and** `parent.join(".")` unsearchable) + one `tracing::debug!` per skip; six-fixture unit matrix; `deny_paths_e2e` green with `devbox_genuine` retargeted to a devbox-extending profile with an explicit deny. **No** new skip condition in `skip_if_enforcement_unavailable`.
- **01-02 IPv6:** `AF_UNSPEC` resolve, family+bytes allowlist, `inet_ntop(AF_INET6, …)` reporting, `EGRESS_ALLOW_DEBUG=1`; `scripts/test-sandbox-run.sh` on the `test-install.sh` skeleton; `egress-check.sh:551-558` prose + `scripts/README.md` updated in the same change.
- **01-03 cost:** `deepseek-v4-flash` in `deepseek_defaults()`; segment per `01-UI-SPEC.md` (4 dp < $0.01, 2 dp ≥ $0.01, promotion at `0.00996`, floor `0.0001`, `dim` tone); headless injects `ui.status_line.pricing()`; tests on both surfaces + one cross-surface equality; `25-status-line.md` copy exactly as the spec dictates.
- **01-04 default:** `"default": "deepseek-flash"` + resolver prefers it when visible; `context_snapshot.rs:376` updated; new no-preference test; README / `11-custom-models.md` / `05-configuration.md` / `docs/TUNING.md` caveat + revert path.
- **01-05 gate/evidence:** `scripts/test-gate.sh` (disk-guard 150 → flock → workspace → config-docs, key vars unset, count-vs-expected); expected file seeded from the first green run; `#[serial]`/key alignment in the two shell tests; tolerance widening via `assert_timeout_about` and the test-only TTLs; evidence helper + `docs/evidence/README.md` + three Phase-1 item dirs; `key-leak-check.sh` scans `docs/evidence`; `KNOWN-ISSUES.md` / `TESTING.md` updated when the counts or the devbox disposition change.

## Metadata

**Analog search scope:** `crates/codegen/{xai-grok-sandbox,xai-grok-status-line,xai-grok-pager,xai-grok-pager-pty-harness,xai-grok-shell,xai-grok-workspace,xai-grok-models}`, `scripts/`, `docs/`, repo root docs.
**Files scanned:** 40+ (all cited paths read in this session; every cited path is git-tracked).
**Pattern extraction date:** 2026-11-21.
