# Phase 2: Loop-Item Verification - Pattern Map

**Mapped:** 2026-11-23
**Files analyzed:** 24 (14 create, 6 modify, 4 read-only contracts, plus 7 evidence item dirs)
**Analogs found:** 24 / 24 (0 with no analog; 9 are "extend this file" self-analogs)
**Tree state:** `main` @ `1efb05ac` (`docs(02): research + validation strategy`); working tree clean except
`.planning/STATE.md` (the orchestrator's own edit). Every path below was verified git-tracked with
`git ls-files --error-unmatch <path>`; all 24 returned `TRACKED`.

**Corrections to `02-RESEARCH.md` (verified this session, follow the tree):**

| Research says | Tree says | Consequence for the planner |
|---|---|---|
| `headless.rs:476-498` / `:1538` | `permission_cancel_hint` at **:479-498**; the `anyhow!` at **:1539** | Line refs only; content matches. |
| "the three rendered labels … `permission_cursor.rs`" (code_context bullet) | `permission_cursor.rs` labels are the **settings picker** strings (`"Allow once"`, `"Always allow this command"`, `"Reject"`, `:43-50`) — they are **not** the prompt row labels | Row labels come from `xai-grok-workspace/src/permission/prompter.rs` option `label`s (`"Yes, proceed"`, `"Always allow: …"`) and `permission_view.rs::build_reject_once_line` (`:1519-1528`). Assert the prompter/view strings, never the settings strings. |
| `tests/pty_e2e_shell_tools.rs:1-2` carries the `#[ignore]` | The file is **56 lines of `#[path] mod …;` only**; every `#[ignore]` lives in the module file (e.g. `configured_bash_filename_arguments.rs:259`) | Registering a new scenario = one `#[path] mod` line in `pty_e2e_shell_tools.rs`, plus `#[ignore]` on the test fn in the new file. |
| — | `xai-grok-pager` already dev-depends on `xai-grok-test-support` (`Cargo.toml:198-199`), as do `pty-harness` (`:32`) and `pager-bin` (`:97`) | A headless-mock test can live in any of the three; `pager-bin/tests/header_capture.rs` is the cheapest precedent (no `PAGER_BINARY` gate). |
| — | `run_headless` resolves the binary through `xai_grok_test_support::env::grok_binary()` | The headless-mock test still needs a built/resolved pager binary: it is a build-host test, not a planning-host test. |

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `docs/evidence/02-loop-item-verification/{live-smoke,runtime-egress,key-leak,permission-prompt,permission-mode,resume-fidelity,plan-read-only}/` *(new, 7 dirs)* | evidence artifact (report + cast + notes + step logs) | file-I/O | `docs/evidence/01-truth-baseline/ipv6-report/` (+ `cost-estimate/`, `determinism/`) and `docs/evidence/README.md` | exact |
| `scripts/evidence.py` | dev script (recorder) | file-I/O | itself — `begin/step/artifact/finish/cast/normalize` | **read-only** (no edit) |
| `scripts/egress-check.sh` | dev script (live egress audit) | event-driven (LD_PRELOAD interpose) | itself | **read-only** (call, capture; edit only on a real defect) |
| `scripts/smoke-test.sh` | dev script (live adapter smoke) | request-response | itself | **read-only** |
| `scripts/key-leak-check.sh` | dev script (secret scan) | file-I/O | itself — default canary mode + `--scan <paths>` | **read-only** |
| `scripts/test-gate.sh` | dev script (gate) | batch | itself — already runs `--scan docs/evidence` as step 3 (`:294-305`) | **read-only** |
| `crates/codegen/xai-grok-pager/src/headless.rs` | service (CLI denial path) | request-response | itself — `permission_cancel_hint:479-498`, denial branch `:1531-1541` | extend (self) |
| `crates/codegen/xai-grok-pager/src/headless_tests.rs` | test (unit) | request-response | `permission_cancel_hint_only_fires_for_unapproved_gated_cancel:1015-1043`; `headless_plan_mode_sends_set_session_mode_only_for_plan:1045-1087` | extend (self) |
| `crates/codegen/xai-grok-pager-bin/tests/<new>.rs` *(new; PERM-02/PERM-04/SESS-01 headless-mock)* | test (integration, subprocess + mock server) | request-response | `crates/codegen/xai-grok-pager-bin/tests/header_capture.rs:11-32` | exact |
| `crates/codegen/xai-grok-pager/src/app/acp_handler/permissions.rs` | TUI handler (prompt assembly) | request-response | itself — `build_permission_display:186-269` (the `is_execute` arm at `:221-232` drops `acp_title`) | extend (self) |
| `crates/codegen/xai-grok-pager/src/views/permission_view.rs` | TUI view (render) | transform | itself — `PermissionViewState:138-172`, `permission_chrome_height:307-329`, title row `:474-485`, tests `:1984-2078` | extend (self) |
| `crates/codegen/xai-grok-workspace/src/permission/prompter.rs` | service (warning source) | transform | itself — `with_security_warning:837-850`, test `:1075-1108` | **read-only** (copy, then leave the warning on the wire) |
| `crates/codegen/xai-grok-pager-render/src/appearance/permission_cursor.rs` | library (cursor policy) | transform | itself — `resolve_initial_cursor` | **read-only** (do not change cursor behaviour) |
| `crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/permission_prompt_outcomes.rs` *(new, PERM-01)* | test (pty e2e) | streaming | `configured_bash_filename_arguments.rs:132-171` (label-driven row pick) + `permission_prompt_hook_chimes_only_on_real_wait.rs:58-114` | exact |
| `crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/permission_mode_resume.rs` *(new, PERM-03)* | test (pty e2e) | streaming | `src/scenarios/plan_approval_resume.rs:35-126` (quit → resume → assert frame) | exact |
| `crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/plan_mode_read_only.rs` *(new, SESS-02)* | test (pty e2e, parameterized) | streaming | `configured_bash_filename_arguments.rs` (turn + assert) + `plan_mode_edit_gate_tests.rs:41-64` | role-match |
| `crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e_shell_tools.rs` | test root (registry) | — | itself (one `#[path] mod` per file) | extend (self) |
| `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/plan_approval_resume.rs` | scenario lib (SESS-01/03) | streaming | itself — whole file `:1-187` | extend (self) |
| `crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/common.rs` | test helper | — | itself — `write_cast_if_requested:1560-1575`, `expect_tool_turn:733-747` | **read-only** |
| `crates/codegen/xai-grok-pager-pty-harness/src/lib.rs` | library (PTY driver) | streaming | itself — `write_cast:484-514`, `spawn_with_content_in_dir:190-199`, `wait_for_text:447-452` | **read-only** contract |
| `crates/codegen/xai-grok-shell/src/agent/mvp_agent/session_setup.rs` | service (resume restore) | file-I/O | itself — `restore_persisted_model:1674-1727` | **read-only** (assert only) |
| `crates/codegen/xai-grok-shell/src/session/persistence.rs` | service (persistence) | file-I/O | itself — `CurrentModel:260-267`, `PlanModeState:2155-2161` | **read-only** |
| `crates/codegen/xai-grok-shell/src/session/plan_mode.rs` | library (plan tracker) | file-I/O | itself — `PlanModeSnapshot:30-66`, `plan_file_path:69-79` | **read-only** |
| `crates/codegen/xai-grok-shell/src/session/acp_session_tests/plan_mode_edit_gate_tests.rs` | test (unit) | — | itself — `:41-103` | **read-only** / extend if SESS-02 wants a mode matrix |
| `crates/codegen/xai-grok-shell/src/session/acp_session_tests/plan_approval_resume_tests.rs` | test (unit) | — | itself — `request_plan_approval_issues_reverse_request_and_clears_flag:44` | **read-only** |
| `KNOWN-ISSUES.md` (lines 209-213) | docs (ledger) | — | itself — the section voice of "Step 4 — real-use results" / "Phase 2" | extend (self) |
| `crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md` | docs (user guide) | — | itself — `### How to set the mode` `:43-62`; sibling fact `19-plan-mode.md:125` | extend (self) |
| `docs/evidence/README.md` | docs (convention) | — | itself — `## Layout`, `## Recording an item`, `## Re-running the Phase-1 items` | extend (self, optional) |

---

## Pattern Assignments

### 1. Evidence items (7 dirs) — the Phase 1 item is the whole pattern

#### `docs/evidence/02-loop-item-verification/<slug>/` (evidence artifact, file-I/O)

**Analog:** `docs/evidence/01-truth-baseline/ipv6-report/` (script capture) and
`docs/evidence/01-truth-baseline/cost-estimate/` + `determinism/` (test-driven capture).

`report.json` — fixed key set **and order** (`docs/evidence/README.md:21-34`):

```json
// docs/evidence/01-truth-baseline/ipv6-report/report.json:1-9 (head)
{
  "item": "ipv6-report",
  "requirements": ["SBOX-02"],
  "date": "2026-11-21T19:52:49Z",
  "host": "Linux/x86_64",
  "command": "sh scripts/test-sandbox-run.sh (captured with script -T)",
  "steps": [ { "name": "…", "ok": true, "detail": "exit 0: PASS: …" } ],
  "artifacts": [ { "path": "docs/evidence/01-truth-baseline/ipv6-report/ipv6-report.cast",
                   "sha256": "e18c867a…" } ],
  "result": "ok"
}
```

`notes.md` — the re-run recipe is a fenced block, and the provenance sentence says which
command produced the cast (`ipv6-report/notes.md:1-46`):

```markdown
# ipv6-report

Proves SBOX-02's second half: …

## What the cast shows
## Steps
| Step | Assertion |
## Re-run

```sh
sh scripts/test-sandbox-run.sh        # expect: ALL PASS (6 assertions), exit 0
```
```

**Exact `step` / `finish` invocation shape** (from `docs/evidence/README.md:79-97`; the
parser strips a leading `--`, `scripts/evidence.py:162-170`):

```sh
ITEM=docs/evidence/02-loop-item-verification/runtime-egress
scripts/evidence.py begin --item-dir "$ITEM" --item runtime-egress \
    --requirements PROOF-02 --command "scripts/egress-check.sh (captured with script -T)"
script -qec 'scripts/egress-check.sh' -T "$ITEM/run.timing" "$ITEM/run.typescript"
scripts/evidence.py cast --typescript "$ITEM/run.typescript" --timing "$ITEM/run.timing" \
    --out "$ITEM/runtime-egress.cast"
scripts/evidence.py step --item-dir "$ITEM" --name "egress: zero violations" \
    -- sh -c "grep -F 'EGRESS_VIOLATIONS=0' '$ITEM/run.typescript'"
scripts/evidence.py artifact --item-dir "$ITEM" --path "$ITEM/runtime-egress.cast"
scripts/evidence.py finish --item-dir "$ITEM" --result ok
```

`step` is a real subprocess whose exit status is recorded, its merged output saved to
`<name-slug>.log` **before** echoing (`scripts/evidence.py:162-209`):

```python
def cmd_step(args):
    ...
    if command and command[0] == "--":
        command = command[1:]
    proc = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    ...
    log_path = item_dir / f"{slug(args.name)}.log"
    log_path.write_text(output, encoding="utf-8")
    state["steps"].append({"name": args.name, "ok": proc.returncode == 0,
                           "detail": summarize(proc.returncode, output)})
```

**Constraints specific to evidence items**
- `grep -F` (not `-q`) so `detail` carries the matched line — every Phase 1 item does this.
- `run.typescript` / `run.timing` are intermediates and are **not** committed.
- `host` is `platform.system()/machine()`; never a hostname/user (`evidence.py:153`).
- `finish --result ok` refuses over a failed step, over **zero** steps, or a missing artifact
  (`evidence.py:230-268`) — so a claim cannot be overstated.
- A pty item's step **is** the `--ignored` test run, with `GROK_PTY_CAST_DIR` set on it, so the
  cast and the recorded exit status come from one execution (`02-RESEARCH.md` Pattern 2).
- `scripts/evidence.py normalize --path "$ITEM/<slug>.cast"` is mandatory for harness casts
  (raw screen bytes can carry the sandbox temp path); casts are text JSONL so the text-only
  scan caveat applies (`docs/evidence/README.md:56-75`).
- Seven item slugs are fixed by CONTEXT: `live-smoke`, `runtime-egress`, `key-leak`,
  `permission-prompt`, `permission-mode`, `resume-fidelity`, `plan-read-only`.

#### Live scripts — read-only, "call, capture, never edit"

`scripts/smoke-test.sh` prints one `PASS  <check>` / `FAIL  <check>` per adapter check and ends
with the line the `live-smoke` step must match (`:38-51, :176-177`):

```sh
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); printf 'PASS  %s\n' "$1"; }
bad()  { FAIL=$((FAIL+1)); printf 'FAIL  %s\n' "$1"; }
...
printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
```

`scripts/egress-check.sh` prints an "activity evidence" section for the session it measured
(`:329`) and its verdict lines (`:474`, `:560-563`):

```sh
print("EGRESS_VIOLATIONS=%d" % len(violations))
...
ok "EGRESS-CHECK PASS"      # else bad "EGRESS-CHECK FAIL"
```

Neither script is edited in this phase. A defect found in a live script is fixed minimally with
the defect policy in CONTEXT §Defect policy; a flaky/unclear result is recorded in
`KNOWN-ISSUES.md`, not papered over.

---

### 2. Headless denial copy (PERM-02) — one shared constant, two call sites

#### `crates/codegen/xai-grok-pager/src/headless.rs` (service, request-response)

**Analog:** itself. Today there are **two independent literals**:

```rust
// crates/codegen/xai-grok-pager/src/headless.rs:479-498
fn permission_cancel_hint(
    stop_reason: &str,
    cancellation_category: Option<&str>,
    is_yolo: bool,
) -> Option<&'static str> {
    use xai_grok_shell::session::commands::{
        PERMISSION_CANCELLED_CATEGORY, PERMISSION_REJECTED_CATEGORY,
    };
    if is_yolo || stop_reason != "cancelled" {
        return None;
    }
    match cancellation_category {
        Some(c) if c == PERMISSION_CANCELLED_CATEGORY || c == PERMISSION_REJECTED_CATEGORY => Some(
            "Turn ended: a permission-gated tool was cancelled because headless mode cannot prompt \
             for approval. Re-run with --always-approve, or choose a non-interactive permission \
             mode such as --permission-mode auto.",
        ),
        _ => None,
    }
}
```

```rust
// crates/codegen/xai-grok-pager/src/headless.rs:1531-1541
} else if let Some(hint) =
    permission_cancel_hint(&stop_reason, cancellation_category, options.yolo)
{
    emitter.on_end(&stop_reason, sid, rid);
    eprint_line(hint);
    Err(anyhow::anyhow!(
        "permission-gated tool cancelled; pass --always-approve to approve it"
    ))
}
```

The fix the UI-SPEC prescribes (`02-UI-SPEC.md` §Copywriting Contract → "The headless
denial"): extract the flags sentence **once** (e.g. `const PERMISSION_DENIAL_HINT: &str`) and
compose both messages from it, keeping the call sites' capitalised `Re-run` vs lower-case
`re-run` distinction. `main` renders the payload as `Error: {error}` (research
`xai-grok-pager-bin/src/main.rs:1745`), so no other file changes.

**Constraints**
- Never suggest `--permission-mode always-approve`: clap accepts only
  `default | acceptEdits | auto | dontAsk | bypassPermissions | plan`.
- Do not add a key to the JSON result (`build_json_result` field set is fixed) and do not
  change the yolo exemption (`is_yolo` → `None`; the run ends `Ok(())`).
- Error text is user-visible prose with no brand token, so `xai-grok-brand` is untouched and
  `branding_surface` is unaffected.

#### `crates/codegen/xai-grok-pager/src/headless_tests.rs` (test, unit)

**Analog:** the neighbouring test in the same module — it asserts `Some/None` but **not** the
text (`:1015-1043`):

```rust
#[test]
fn permission_cancel_hint_only_fires_for_unapproved_gated_cancel() {
    ...
    assert!(
        super::permission_cancel_hint("cancelled", Some(PERMISSION_CANCELLED_CATEGORY), false)
            .is_some()
    );
```

Extend it (or add a sibling) with the byte assertions the UI-SPEC requires:
`contains("--always-approve") && contains("--permission-mode auto")` on **both** rendered
strings. The recall-only test style (`let hint = …unwrap(); assert!(hint.starts_with(...),
"…{hint}")`) is the house pattern; assertions carry a message.

Also copy the sibling's shape for the plan-mode unit proof (already exists, keep green):

```rust
// crates/codegen/xai-grok-pager/src/headless_tests.rs:1045-1087
#[tokio::test]
async fn headless_plan_mode_sends_set_session_mode_only_for_plan() {
    ...
    assert_eq!(seen.lock().unwrap().clone(), vec!["plan".to_string()]);
}
```

#### `crates/codegen/xai-grok-pager-bin/tests/<new>.rs` (integration, mock server) — PERM-02 / PERM-04 / SESS-01

**Analog:** `crates/codegen/xai-grok-pager-bin/tests/header_capture.rs:1-32` — the in-tree
pattern for "real binary, mock provider, assert exit + output, no key, no network":

```rust
use xai_grok_test_support::{
    MockInferenceServer, MockModelEntry, assert_headless_success, run_headless,
};

#[tokio::test]
async fn headless_chat_headers_are_brand_clean() {
    let server =
        MockInferenceServer::start_with_models(vec![MockModelEntry::new("deepseek-v4-pro")])
            .await
            .expect("start mock inference server");
    server.set_response("hello from the mock server");
    ...
    let out = run_headless(&server, &["-p", "say hello", "--yolo", "--output-format", "json"], &cwd).await;
    assert_headless_success(&out, "header capture headless run", Some(&server));
    server.wait_for_inference_requests(1).await;
    let requests = server.requests();
```

For a **gated** tool turn (the denial path) script the tool call on both backends the way the
pty tests do — `ScriptedResponse::sse(responses_api_tool_call_events(...))` and
`ScriptedResponse::sse(chat_completions_tool_call_events_with_id(...))` — via
`content.expect_agent_turn_with_responses(...)` (`tests/pty_e2e/common.rs:733-747`
`expect_tool_turn`), or `run_headless_in_sandbox_borrowed(cmd, content.sandbox())` when the
same sandbox/HOME must be shared (`xai-grok-test-support/src/headless.rs:75-88`).
`HeadlessResult` carries `status`, `stdout`, `stderr`, `timed_out` (`headless.rs:15-23`) —
assert `status.code() == Some(1)`, both stderr lines, and (JSON run) `"stopReason": "cancelled"`
on stdout.

---

### 3. Permission prompt: keep the warning, then paint it (PERM-01)

#### `crates/codegen/xai-grok-pager/src/app/acp_handler/permissions.rs` (TUI handler)

**Analog:** itself. The defect is a branch that discards the title:

```rust
// crates/codegen/xai-grok-pager/src/app/acp_handler/permissions.rs:198-232
let acp_title = req
    .tool_call
    .fields
    .title
    .as_deref()
    .map(|title| match &ask {
        Some(ask) => ask.strip_prompt_header(title),
        None => title,
    });
...
let title = if is_execute {
    bash_description
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .unwrap_or_else(|| match bash_highlights.and_then(|h| h.highlighted_words.first()) {
            Some(bin) => format!("Allow `{bin}`?"),
            None => "Allow Execute?".to_string(),
        })
} else if is_edit_permission(req) { ... }
```

The warning is on the wire as a `\n`-joined prefix inside `fields.title`
(`xai-grok-workspace/src/permission/prompter.rs:837-850`), so `acp_title` already contains it;
the `is_execute` arm simply never looks. The seam the UI-SPEC prescribes: split a leading
`⚠ Security warning: …` line into its own field on the tuple `build_permission_display`
returns (or on `PermissionViewState`), keep `title` the question row, and keep the copy
byte-exact. The function returns `(String, Vec<String>, Option<String>)` and the single caller
(`enqueue_permission:113-163`) pushes those into `PermissionViewState`; that is the only
other site to touch.

**Constraints**
- `#![deny(clippy::indexing_slicing)]` is on for the crate: use `split_once('\n')` /
  `strip_prefix(SECURITY_WARNING_PREFIX)`, never index.
- Do not reorder rows, do not touch the always-approve/cursor policy, do not alter the
  `hook_ask` header path (it has its own test).

#### `crates/codegen/xai-grok-pager/src/views/permission_view.rs` (TUI view)

**Analog:** itself. Three sites:

```rust
// :138-172 — state; a `warning: Option<String>` field goes beside `title`
pub struct PermissionViewState {
    ...
    pub title: String,
    pub description: Vec<String>,
    ...
}
```

```rust
// :307-329 — chrome height; the per-row math has NO spare row for a warning
fn permission_chrome_height(state: &PermissionViewState, content_w: usize) -> u16 {
    ...
    let mut h: u16 = 1;
    if state.subagent_label.is_some() { h += 1; }
    h += 1;                       // ← the title row
    ...
}
```

```rust
// :474-485 — the title row, the only place the question is painted
if y < area_bottom {
    let title_style = Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD);
    buf.set_line(content_x, y, &Line::from(Span::styled(state.title.clone(), title_style)), content_width);
}
y += 1;
```

Render the warning row **before** the title (mirroring the `subagent_label` block at `:461-472`,
which is the exact structural precedent: guarded `if y < area_bottom`, `y += 1`), in
`theme.warning`, normal weight, and add `h += 1` (+ the same guarded increment) when present.
`permission_chrome_height_pub:296-305` caps by `area_h` so an extra row cannot overflow.

**Rendered-frame assertion pattern to copy** — `render_to_text` (`:2046-2062`):

```rust
fn render_to_text(state: &PermissionViewState, area: Rect) -> String {
    let theme = Theme::current();
    let mut buf = Buffer::empty(area);
    let _ = render_permission_view(&mut buf, area, state, "", None, None, &theme, true);
    (0..area.height)
        .map(|row| {
            (area.x..area.x + area.width)
                .map(|col| buf.cell((col, row)).map(|c| c.symbol().to_string()).unwrap_or_default())
                .collect::<String>()
                + "\n"
        })
        .collect()
}
```

and the row assertion style (`:2064-2078`): `assert!(text.contains("... Ctrl-F to expand"),
"indicator missing:\n{text}");`. New tests: warning row present + title stripped; `+1` chrome
height when present (`chrome_height_counts_mcp_args_lines:1969-1981` is the delta style).

#### `crates/codegen/xai-grok-workspace/src/permission/prompter.rs` (read-only source)

The copy is owned here; do not change it. Its test is the pin for the prefix and the
high-signal filter (`:1075-1108`):

```rust
let dangerous: BashSecurityAssessment = [ClassifierSecurityFinding::DangerousCommand].into_iter().collect();
let warned = with_security_warning(&titled, Some(&dangerous));
let title = warned.fields.title.unwrap();
assert!(
    title.starts_with("⚠ Security warning:") && title.ends_with("Run rm -rf build"),
    "warning must precede the original title: {title}"
);
```

Row labels (what the pty cast must contain) also live here — e.g. `"Yes"` for edit
(`:339-351`), `REJECT_ONCE_LABEL` for reject — and the view replaces the reject label with
`"No, reject (type to add feedback)"` (`permission_view.rs:1519-1528`). The
`permission_cursor.rs` strings (`:43-50`) are settings-picker labels and must not be asserted.

---

### 4. PTY scenarios and tests

#### `tests/pty_e2e/permission_prompt_outcomes.rs` (new, PERM-01)

**Analog:** `configured_bash_filename_arguments.rs:132-171` — the only in-tree code that
answers a permission prompt without hard-coding a digit:

```rust
fn reject(&mut self, call_id: &str, name: &str, args: serde_json::Value) {
    let ToolTurn { expectation, .. } = self.submit(call_id, name, args);
    self.harness.wait_for_text(REJECT_ROW, Duration::from_secs(60)).expect("actual permission menu");
    let screen = self.harness.screen_contents();
    let row = screen.lines().find(|line| line.contains(REJECT_ROW)).expect("reject row");
    let index = row.split_whitespace().find_map(|word| word.parse::<usize>().ok())
        .expect("displayed option number");
    assert!((1..=9).contains(&index), "unexpected reject row: {row}");
    let keys = format!("{}{}", "\x1b[A".repeat(9), "\x1b[B".repeat(index - 1));
    self.harness.inject_keys(keys.as_bytes()).expect("navigate to displayed reject option");
    self.harness.wait_until("reject option selected", Duration::from_secs(10), |h| {
        h.screen_contents().lines().any(|line| {
            line.contains(REJECT_ROW) && (line.contains("(●)") || line.contains("(•)"))
        })
    }).expect("reject row owns the selection before Enter");
    self.harness.inject_keys(b"\r").expect("reject once");
    ...
    assert!(self.harness.contains_text("permission was denied"), ...);
}
```

Second analog for turn scripting + wait discipline:
`permission_prompt_hook_chimes_only_on_real_wait.rs:58-114` (signature `expect_tool_turn`,
`content.set_response(...)`, `harness.wait_for_text(...)`, then advance with `harness.update`).

**Per-case shape (adopt verbatim):**
- file header: `use super::common::*;` then `const REJECT_ROW: &str = "No, reject";` (and
  allow-once / always-allow substrings from the prompter/view).
- `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`
- `#[ignore = "PTY e2e; requires PAGER_BINARY"]` (add `and RG_BIN_PATH` only if the fixture
  needs a real `rg`)
- `pager_binary().expect("supplied pager binary")`, `ContentController::start().await`,
  `PtyHarness::spawn_with_content_in_dir(&binary, rows, cols, &content, &[], Some(cwd))`
- end with `write_cast_if_requested(&harness, "permission-prompt.cast");` (or the `.cast`
  name the item dir expects) and `harness.quit()`.
- assert the three labels by substring (`Yes, proceed` / `Always allow:` /
  `No, reject (type to add feedback)`) and `Security warning:` (never the `⚠` glyph alone).

**Register** in `tests/pty_e2e_shell_tools.rs` — the file is a 56-line list of plain
`#[path]` mods (only `common.rs`/`scroll.rs` carry `#[allow(dead_code, unused_imports)]`).
Append in the same form, next to the existing permission module (`:51-52`):

```rust
#[path = "pty_e2e/permission_prompt_outcomes.rs"]
mod permission_prompt_outcomes;
```

#### `tests/pty_e2e/permission_mode_resume.rs` (new, PERM-03)

**Analog:** `src/scenarios/plan_approval_resume.rs:35-126` — the quit/resume choreography:

```rust
let mut first = PtyHarness::spawn_with_content_in_dir(&binary, DEFAULT_ROWS, DEFAULT_COLS,
    &content, &[], Some(project.path()))?;
wait_for_welcome(&mut first).await?;
first.inject_keys(b"go\r")?;
first.wait_for_text(SETUP_SENTINEL, Duration::from_secs(30))?;
// Quit and reap BEFORE seeding …
first.inject_keys(b"\x11")?; first.update(Duration::from_millis(200));
first.inject_keys(b"\x11")?; first.quit()?;
...
let mut resumed = PtyHarness::spawn_with_content_in_dir(&binary, DEFAULT_ROWS, DEFAULT_COLS,
    &content, &["--continue"], Some(project.path()))?;
resumed.wait_for_text("request changes", WELCOME_TIMEOUT)?;
```

For PERM-03 assert the chip pair (`deepseek-flash · plan · always-approve` style, or the pair
the launch config produces) in the last pre-quit frame and again after resume. Critical:
launch **both** runs with the **same** effective config/flag set (no `--permission-mode` on
resume), because the permission chip is config/CLI-resolved (`event_loop.rs:1098-1103,
1504-1518`) while only the plan chip is session-persisted. Write it into the item's
`notes.md` (SESS-04 provenance).

#### `tests/pty_e2e/plan_mode_read_only.rs` (new, parameterized, SESS-02)

**Analog:** `configured_bash_filename_arguments.rs` (drive a turn, wait, assert screen text)
for the pty side; `plan_mode_edit_gate_tests.rs:41-64` for the assertion's meaning:

```rust
// crates/codegen/xai-grok-shell/src/session/acp_session_tests/plan_mode_edit_gate_tests.rs:41-64
#[tokio::test(flavor = "current_thread")]
async fn plan_mode_rejects_grok_edit_outside_plan_file_despite_allow_all_permissions() {
    let actor = build_gate_actor().await;
    activate_plan_mode(&actor);
    let result = prepare(&actor, search_replace_call_at("call_gate", "/tmp/src/main.rs")).await;
    assert!(matches!(result, Err(ToolLoop::Continue)), "gate must reject with Continue …");
    let text = tool_result_text(&actor, "call_gate").await;
    assert!(text.contains("/tmp/test-session/plan.md"), "must name the plan file: {text}");
}
```

The gate runs **before** permission resolution (`tool_calls.rs:1655-1670`) and rejects
`ApplyPatch` outright + any edit outside `should_auto_approve_edit` (`:170-190`), so the pty
assertion for `default`, `always-approve` and `auto` is the same refusal string:
`Rejected: file edits are not allowed in plan mode - the only editable file is the plan file (…)`
(`plan_mode.rs:332`). One test fn, a `for mode in ["default", "always-approve", "auto"]` loop
with a fresh `ContentController`+sandbox per iteration and one cast per mode
(`plan-read-only-<mode>.cast`), plus the plan-file-writable half.

#### `src/scenarios/plan_approval_resume.rs` (extend, SESS-01/03)

**Analog:** itself (`:1-187`, read above). Extend — do not rewrite:
- add model + reasoning-effort assertions after resume (today the scenario only checks
  approval chrome). The restore path is `session_setup.rs:1674-1727`
  (`restore_effort = initial_reasoning_effort.or(summary.reasoning_effort)`, then
  `model_switch::apply(... SwitchEffort::Set(restore_effort) …)`); the observable in a pty is
  the model name / effort chip in the info row.
- keep `seed_parked_approval` / `write_awaiting_plan_mode` byte-for-byte; they are the
  precedent for "flip one field, preserve the rest, fresh snapshot if absent".
- end the scenario with `write_cast_if_requested(...)` only if the test wrapper owns the
  harness (today the harness is local to the scenario fn — either add a `&Path`/cast-name
  parameter or take the cast in the `tests/plan_approval_resume.rs` wrapper;
  `tests/plan_approval_resume.rs` is a 16-line `.expect(...)` wrapper and is the place to
  keep that constant).
- `tests/plan_approval_resume.rs` has **no** `#[ignore]`; it runs under plain cargo (needs a
  built pager). For evidence it is still run explicitly so the step log carries
  `test result: ok`.

#### `src/lib.rs` `write_cast` — the cast contract (read-only)

```rust
// crates/codegen/xai-grok-pager-pty-harness/src/lib.rs:484-514
pub fn write_cast(&self, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
    let (cols, rows) = self.cast_size;
    out.push_str(&serde_json::json!({"version": 2, "width": cols, "height": rows}).to_string());
    ...
    out.push_str(&serde_json::json!([elapsed, "o", data]).to_string());
```

It creates the parent dir, so `GROK_PTY_CAST_DIR=docs/evidence/02-loop-item-verification/<item>`
works with the directory absent. `write_cast_if_requested` is non-fatal by design
(`tests/pty_e2e/common.rs:1560-1575`) — the test's own assertions, not the cast, are the gate.

---

### 5. Plan mode / resume / edit gate in `xai-grok-shell` (assert-only, no edits)

| Fact the evidence claims | Pin it with | Path:line |
|---|---|---|
| Resume restores model + effort | existing unit test | `agent/mvp_agent/tests.rs:1108` `model_state_prefers_session_reasoning_effort_over_global`; restore code `session_setup.rs:1674-1727` |
| Plan state persists (`plan_mode.json`) | snapshot shape + writer | `session/plan_mode.rs:30-66` (`PlanModeSnapshot`, `awaiting_plan_approval`, `#[serde(default)]`), `storage/mod.rs:32` `PLAN_MODE_FILE`, `persistence.rs:2155-2161` |
| Approval re-parked on resume | unit + pty | `plan_approval_resume_tests.rs:44` `request_plan_approval_issues_reverse_request_and_clears_flag`; pty scenario above |
| Plan mode is read-only in every mode | pure predicate + gate order | `tool_calls.rs:170-190` (`PlanEditGate`, `ApplyPatch` always rejected), `:1655-1670` (gate before permission) |
| `--permission-mode plan` really arms headless | existing unit test + new mock-run transcript | `headless.rs:856-875` (`apply_headless_permission_mode`), call site `:1277`; `headless_tests.rs:1045-1087` |

**Read-only rule:** none of these files change. SESS-01/02/03 are proven by the tests above
plus the pty casts; if a test is found weak, extend it in the same file with the same
`#[tokio::test(flavor = "current_thread")]` + `LocalSet` idiom (`plan_mode_edit_gate_tests.rs`
also proves "no hang means no permission prompt" with a 10 s timeout wrapper at `:33-40`).

The plan-file path never hard-codes `.grok` in tests: the shell computes it
(`plan_mode.rs:69-79`), and the test sandbox HOME is `DEEPSEEK_BUILD_HOME=<sandbox>/.grok`
(`xai-grok-test-support/src/sandbox.rs:212,325`) — write each item's real per-item path into
`notes.md` rather than assuming `~/.deepseek-build`.

---

### 6. Docs

#### `KNOWN-ISSUES.md` (lines 209-213) — PERM-04 deliverable

Current, **stale** text (verbatim):

```markdown
- **Plan mode gap:** `--permission-mode plan` is read-only (no file changed) but
  does not send `session/set_mode`, write `plan_mode.json`, or inject the
  plan-mode reminder; a bare "change X" request therefore ends as a cancelled
  turn with empty output instead of a plan. A prompt that explicitly asks for a
  plan produces one.
```

Correct it in the ledger's own voice (past-tense status line + what the tree now does + the
evidence item that proves it). Keep the neighbouring entries untouched — notably the headless
entry that follows it stays true once the `Error:` line names both flags (`02-UI-SPEC.md`
§Owned Elsewhere says do **not** rewrite it to quote the message). Do not edit
`TESTING.md:93` (same claim; it belongs to DOCS-02/03 in Phase 5).

#### `crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md` — SESS-04 paragraph

House shape of the page: `### How to set the mode` (`:43-62`) lists TUI / CLI / config, then a
one-line precedence sentence ("CLI overrides config for that process."). The provenance
paragraph this phase owns belongs there (or immediately under it), stating: the **permission**
mode on resume comes from CLI → `[ui] permission_mode` → default, i.e. **config**, while the
**plan** state comes from the session record (`plan_mode.json`). Ground the plan half in the
sibling page's existing sentence (`19-plan-mode.md:125`):

```markdown
Plan mode state is persisted to disk and survives process restarts. Transient states
(`Pending`, `ExitPending`) are collapsed to `Inactive` on restart since they depend on in-flight interactions.
```

Constraints: hand-edited page; do **not** touch `26-config-reference.md` (no key added/removed;
`config_docs` gate); do not re-describe the prompt/chips/denial (DOCS-04 owns that);
`check-egress.sh` excludes `*.md` so new prose cannot trip the gate.

#### `docs/evidence/README.md` — convention doc (optional extend)

It already covers "a later phase **appends** a new `NN-<phase-slug>/` tree; it never rewrites
an earlier item" (`:16-19`) and the whole recording recipe. Only add a
`## Re-running the Phase-2 items` section (mirroring `:108-124`) if the planner wants the
seven items re-runnable from one place. No schema change.

---

## Shared Patterns

### Evidence discipline (applies to all 7 items)
- One directory per claim; a correction is a **new** directory, never a re-run of `finish`
  (`docs/evidence/README.md:16-19, :68-75`).
- `report.json` key set/order is fixed; `result: "ok"` only after every step passed.
- Normalize before hashing; casts are text-only; `--scan` before commit.
- The step log is the proof; `notes.md` names the exact re-run command.

### Secrets
- Key from `DEEPSEEK_API_KEY` / `DEEPSEEK_BUILD_API_KEY` only; exported in the **parent**
  shell before `script -qec`, never inside the capture, never with `set -x`.
- `--scan` takes explicit paths (`docs/evidence`, a `git ls-files` list) — never `.` (it would
  descend into `target/`); `grep -I` skips binary files.
- Scrub anything that could carry a path: `evidence.py normalize` handles `<REPO>/<HOME>/<TMP>`.

### Rust conventions in every touched crate
- `#![deny(clippy::indexing_slicing)]` — use `get`/iterators/`split_once`; tests are also
  linted by the Bazel aspect.
- `dunce::canonicalize`, `xai_dirs::home_dir`, `xai_tty_utils::ProcessScope`,
  `xai_grok_extra_ca` — the clippy-banned alternatives.
- Unit tests live beside the code (`*_tests.rs`, wired with
  `#[cfg(test)] #[path = "…_tests.rs"] mod tests;`); assert with messages; test names are
  sentences describing the invariant.
- No new dependency of any kind (no `Cargo.toml`, lockfile, `deny.toml` change). No new
  outbound destination (a change that adds one is stop-and-ask).
- User-visible strings, if any are added, stay plain prose: `xai-grok-brand` untouched,
  `branding_surface` unaffected.

### Scripts
- Read-only this phase; if a real defect forces an edit, keep the script's own conventions
  (header block naming the dialect, `set -euo pipefail` or `set -eu`, `mktemp -d` + `trap`,
  `PASS:`/`FAIL:` + a final count line, non-zero exit on failure) and `shellcheck --shell=sh`.

### Build hygiene for the executor
- `scripts/disk-guard.sh 150` before any workspace build (gate also enforces 150,
  `test-gate.sh:59-63`); one shared `CARGO_TARGET_DIR`; heavy runs under
  `flock -w 900 /tmp/team-build.lock`.
- PTY cases are **not** in `cargo test --workspace`: they need `--ignored`, so an item's step
  must be the explicit `--ignored` run (`02-RESEARCH.md` Pitfall 7).
- Every runtime task states its precondition (toolchain / `PAGER_BINARY` / key / disk) and that
  it runs on the **build host** — the planning host has no toolchain, 31 GB free and no key.

---

## No Analog Found

Nothing is entirely without an analog. The weakest matches:

| File | Role | Why weak | Fallback |
|---|---|---|---|
| `tests/pty_e2e/plan_mode_read_only.rs` (SESS-02 pty) | test (pty e2e) | No pty test asserts a plan-mode **refusal** today (the gate is unit-tested only) | `configured_bash_filename_arguments.rs` drive/assert skeleton + `plan_mode_edit_gate_tests.rs:41-64` for the string and meaning |
| `permission-mode/*.cast` resume chip assertion (PERM-03) | evidence artifact | No existing cast asserts an info-row chip pair across resume | `plan_approval_resume.rs` quit/resume choreography + `views/prompt_widget/tests.rs::mode_flags_show_plan_and_permission_together` for the exact chip vocabulary |
| `resume-fidelity/notes.md` provenance table (SESS-04) | docs in evidence item | No prior item carries a provenance table | `ipv6-report/notes.md` step table shape + `19-plan-mode.md:125` / `22-permissions-and-safety.md:43-62` for the facts |

## Executor Checklist (per plan)

- **02-01 live smoke:** `script -qec 'scripts/smoke-test.sh'` on the build host with the key
  exported outside the capture → `cast` → step asserting `8 passed, 0 failed` → `finish ok`;
  `live-smoke/notes.md` says the smoke script is the *separate adapter* smoke.
- **02-02 runtime egress:** same recipe with `scripts/egress-check.sh`; steps
  `EGRESS_VIOLATIONS=0` and `EGRESS-CHECK PASS`; `notes.md` says the measured session is the
  egress-check session and its activity proof is in the same cast.
- **02-03 key leak:** default-mode canary run + `--scan docs/evidence` + a tracked-file key-shape grep
  recorded as a **measurement** (2 files / 4 lines at HEAD `1efb05ac`, all in `#[cfg(test)]` redaction
  code — never an equality gate) + dated revocation attestation in `notes.md`; the canary contacts
  `api.deepseek.com` with a fake key, so this plan needs the network.
- **02-04 permission surface (PERM-01/02):** land the **warning split** (permissions.rs →
  permission_view.rs + `+1` chrome height, unit + frame test), the **one-const denial copy**
  (`headless.rs` + both composed-string assertions in `headless_tests.rs`), the new headless-mock test
  (plain + JSON + non-asserting `auto` probe), the pty outcomes scenario registered in
  `pty_e2e_shell_tools.rs`, then record the `permission-prompt` item with a `GROK_PTY_CAST_DIR` step and
  refresh `scripts/test-gate.expected` once.
- **02-05 resume/plan (SESS-01/03/04):** add `headless_resume_restores_model_effort_and_plan_state` on one
  `TestSandbox` via `run_headless_in_sandbox_borrowed`, extend `scenarios/plan_approval_resume.rs`
  (model/plan-body assertions + cast), record the `resume-fidelity` item, and write the SESS-04 provenance
  paragraph in `22-permissions-and-safety.md`.
- **02-06 mode + read-only (PERM-03/04, SESS-02):** the `-r <sessionId>` chip test, the headless plan-arming
  test, the pure `super::tool_calls::plan_mode_edit_gate` test, the parameterized three-mode read-only pty
  test, the `permission-mode` and `plan-read-only` items, and the `KNOWN-ISSUES.md` plan-mode correction.
  No gate-count refresh (no new `[[test]]` target).
- **Every item:** `normalize` → `artifact` → `finish ok` → `key-leak-check.sh --scan` green;
  `report.json` frozen once finished.

## Metadata

**Analog search scope:** `docs/evidence/**`, `scripts/*`, `crates/codegen/{xai-grok-pager,
xai-grok-pager-bin,xai-grok-pager-render,xai-grok-pager-pty-harness,xai-grok-workspace,
xai-grok-shell,xai-grok-test-support}`, `crates/codegen/xai-grok-pager/docs/user-guide/`,
repo-root docs.
**Files scanned:** 30+ (all cited excerpts read this session; every cited path is git-tracked).
**Pattern extraction date:** 2026-11-23.
