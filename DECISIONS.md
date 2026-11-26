# DECISIONS — `deepseek-build`

Canonical decision log for turning the xAI Grok Build clone into a DeepSeek-first coding
agent named **`deepseek-build`**. Re-read this at the start of every phase. If a needed
decision is not covered here, stop and ask the stakeholder.

Last updated: 2026-11-26.

---

## A. Product decisions (locked)

| ID | Decision | Rationale / notes |
|----|----------|-------------------|
| D1 | **Project name = `deepseek-build`.** Binary `deepseek-build`; config dir `~/.deepseek-build`; env prefix `DEEPSEEK_BUILD_*`; ACP extension namespace `deepseek-build/*`. | Stakeholder answer. |
| D2 | **Phase 1 rename scope = user-facing surface only.** Rename binary, config dir, env prefix, and all user-visible strings. Leave internal `xai-grok-*` crate/package names unchanged for now. | Lower risk, smaller diff. A full internal crate rename may come later. |
| D3 | **Auto-update removed entirely.** No update check, no binary download. | Reduces egress; personal tool. |
| D4 | **Drop `web_search`; keep `web_fetch`, permission-gated (ask).** | `web_search` is xAI-hosted. |
| D5 | **Provider backend = `ChatCompletions` only.** Remove `Responses` and `Messages` backends and wiring. | DeepSeek and custom OpenAI-compatible hosts use Chat Completions. |
| D6 | **Delete, never disable,** all data-exfiltration / telemetry / remote-control code. | User requirement. |
| D7 | **Keep upstream licensing.** Keep `LICENSE` and all NOTICE/attribution files. Do not relicense upstream files. Add a `NOTICE` attribution to `xai-org/grok-build` (Apache-2.0). | Apache-2.0 §4 obligations. |
| D8 | **Toolchain = Rust 1.94.0 via rustup. Do not bump.** | Repo `rust-toolchain.toml` pins it. |
| D9 | **Phase 5 skipped** (benchmarks, release docs, release pipeline). **Superseded for distribution:** a release pipeline now exists (`.github/workflows/release.yml`, prerelease `v0.1.0-proof1`, a GHCR image, `install.sh`, and a Homebrew formula) because the v0.1.0 milestone needed a proven distribution path. The personal-tool constraints still hold: no telemetry, no auto-update, no hosted service. | Personal tool; the pipeline serves a personal binary, not a product. |
| D10 | **Final egress allowlist:** the configured model provider (default `https://api.deepseek.com`) and user-configured MCP servers. Nothing else. | Core security invariant. |
| D11 | **DeepSeek adapter + `reasoning_content` round-trip + cache-stable prefix is authored by ONE agent, never parallelized.** | Correctness-critical, shared state. |
| D12 | **An independent verification sub-agent** that did not write the reviewed code re-checks gates 2 & 3 and the PLAN.md deletion checklist. | Anti-groupthink. |
| D13 | **The API key must never appear in logs, panic/error text, saved sessions, crash output, commits, or command output.** Auth is env-only (`DEEPSEEK_API_KEY` / `DEEPSEEK_BUILD_API_KEY`); a missing key prints one clear line naming both vars. `SamplerConfig`'s `Debug` redacts `api_key`; header-conversion errors do not echo it. `scripts/key-leak-check.sh` runs with a fake key and scans logs + saved sessions. | Stakeholder requirement; defense against accidental secret egress into local artifacts. |
| D14 | **Cost estimates use a configurable price table with peak-rate defaults.** Three rates per model, USD per 1,000,000 tokens: `cache_hit` input, `cache_miss` input, `output` (reasoning tokens are inside `output`, never double-charged). Source: DeepSeek official pricing page, <https://api-docs.deepseek.com/quick_start/pricing>, **fetched 2026-10-07**. Peak defaults: `deepseek-flash` = 0.006 / 0.30 / 1.20; `deepseek-v4-pro` = 0.044 / 1.32 / 3.96. **Peak hours are 01:00-04:00 and 06:00-10:00 UTC Mon-Fri excluding Chinese public holidays; all other hours are off-peak, at exactly half the peak rates.** The table has no clock, so it always prices at peak (conservative, never under-estimates); an override can halve the rates to price off-peak. User-overridable via `[ui.status_line.pricing.<model>]` in `~/.deepseek-build/config.toml`; an unnamed model falls back to the v4 Pro default rather than zero. Shown in the TUI `cost` status item and in the headless result as `estimated_cost_usd` + `estimate: true` (estimated locally; any provider-reported `total_cost_usd` is left untouched). | Personal tool; DeepSeek's usage payload carries tokens but no price. Peak default is conservative. |

## B. Phase scope (locked)

- Implement **Phases 1, 2, 3**, then **Phase 4 items already present that need adapting**.
- **Skip Phase 5** (benchmarks/docs/release polish).

## C. Execution rules (locked)

1. Small, reviewable commits; one logical change each.
2. Parallelize independent work with sub-agents, each on its own git worktree/branch with a
   narrow scope:
   - upload/exfil infrastructure
   - telemetry (Mixpanel / Sentry / OTLP)
   - OAuth + remote config + remote-control relay
   - auto-update
   - xAI web/voice/imagine tools
   - plugin marketplace
   - rename of user-facing surface
   - backend removal (Responses/Messages)
3. One integrator (the lead agent) merges branches in small commits and keeps build + tests
   green after every merge.
4. Deletions must be **real deletions**, not disabled code.
5. Run `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt` clean before declaring a
   phase done.
6. Keep `PLAN.md` updated.
7. Only stop and ask if: a gate fails twice; a needed decision is not in this file; or a
   change would add an outbound network destination.

## D. Automatic gates (run after each phase; stop & report on failure)

1. `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt --check` clean.
2. `grep -rE "gs://|s3://|mixpanel|sentry|x\.ai|grok\.com"` returns nothing outside
   attribution notices.
3. Egress test: run the binary for a short scripted session with only `api.deepseek.com` (or
   a local mock) reachable; confirm no other connection attempts
   (`strace -f -e trace=connect` or equivalent).
4. DeepSeek adapter mock-server tests pass: streaming, tool-call chains, missing
   `reasoning_content`, malformed tool-call JSON, 429/5xx retries.
5. Independent verification sub-agent re-checks gates 2 & 3 and the PLAN.md deletion
   checklist.

## E. Open items

Every item from the pre-milestone list is either closed or re-homed here. The milestone
that produced the v0.1.0 tree ran 2026-11-21 … 2026-11-26; its requirement statuses live in
`.planning/REQUIREMENTS.md`.

| ID | Item | Status |
|----|------|--------|
| O1 | Legal sign-off on re-licensing the aggregate as `deepseek-build` while preserving upstream Apache-2.0 attribution. | **Open; not blocking engineering.** The `v0.1.0-proof1` prerelease and its GHCR image are already published as proof artifacts; legal sign-off is still recommended before a stable public `v0.1.0`. `LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES` and `third_party/NOTICE` are preserved. |
| O2 | Toolchain install feasibility (rustup + Rust 1.94.0) in this environment; `dotslash` absent. | **Closed.** Rust 1.94.0 is installed via rustup and pinned by `rust-toolchain.toml`; `/usr/bin/protoc` is on `PATH`. `dotslash` is still absent and not needed. |
| O3 | ~~DeepSeek exact model IDs and parameters.~~ **Resolved** against official docs (2026-10-02): base URL `https://api.deepseek.com`; models `deepseek-v4-pro` and `deepseek-flash` (legacy `deepseek-v4-flash` accepted but retired/billed as Flash); context 1M, max output 384K; `thinking: {type: enabled|disabled}`; `reasoning_effort: none|low|high|max` (`minimal`→low, `medium`/`xhigh`→high); usage `prompt_tokens_details.cached_tokens` == `prompt_cache_hit_tokens`, plus `prompt_cache_miss_tokens` and `completion_tokens_details.reasoning_tokens`; streaming deltas carry `reasoning_content`; `tool_choice` `required`/named is rejected in thinking mode. **Live-verified 2026-10-07** (`scripts/smoke-test.sh`, 8/8 PASS): `GET /models` returns exactly `deepseek-flash`, `deepseek-v4-pro`; plain + streamed chat OK; 3-turn thinking tool chain OK (`reasoning_tokens=57`); on a cache hit `cached_tokens == prompt_cache_hit_tokens` (2432) and `prompt_tokens == hit + miss`; `tool_choice=required` → HTTP 400 "Thinking mode does not support this tool_choice". **Adapter mapping confirmed, no mismatch:** the harness `usage.input_tokens` = DeepSeek `prompt_cache_miss_tokens`, `cache_read_input_tokens` = `prompt_cache_hit_tokens`, `reasoning_tokens` = `completion_tokens_details.reasoning_tokens`, `total_tokens` = input + cache_read + output. | Done. |
| O4 | Whether to rename all internal `xai-grok-*` crates later. | **Open, deferred by D2.** Belongs to the open-build branch; not part of this milestone (`.planning/REQUIREMENTS.md` out-of-scope table). |
| O5 | ~~Release / distribution pipeline.~~ | **Closed.** `.github/workflows/release.yml` builds four native targets plus a multi-arch GHCR image on a `v*` tag; prerelease `v0.1.0-proof1` (six assets), the image (`sha256:8f5c484e…`), `install.sh` and the Homebrew formula are proven in `docs/evidence/04-distribution-proof/`. `v0.1.0` itself is not published. |
| O6 | Windows distribution. | **Open (Phase 5, 0/5).** The sandbox refuses every profile except `off` on Windows and the tier is documented. The release workflow now carries a `build-windows` job, and `install.ps1` plus `scripts/test-install.ps1` are in the working tree; no tagged run has produced a Windows `.zip`, the harness has not been run on Windows, and WIN-01…06 are pending acceptance. |
| O7 | REL-05 sandbox clause for the container image. | **Open.** The image runs a real headless turn, but ships no `bubblewrap`, so `--sandbox` cannot be enforced inside it; the container boundary is the isolation. REL-05 stays partial. |
| O8 | ACC-01 human acceptance. | **Open and blocking.** The author must use the TUI for 20–30 minutes on a clone of a real repository and commit the acceptance note; the freeze does not proceed without it. |
| O9 | Freeze (FREEZE-01/02/03). | **Open.** The marker tag must be non-triggering. The pre-existing `deepseek` branch and `deepseek-v0.1-snapshot` tag at `4b305bd0` are **superseded**, not moved (FREEZE-02). |
| O10 | ~~Project-local directory name.~~ | **Closed (BRAND-01…03, 2026-11-26).** `.deepseek` is the project directory; `.grok` remains a lower-priority read fallback. `cwd/.grok/rewind-checkpoints` and `.grok-plugin/` are retained residuals, not the project config directory. |

## F. Phase log

The v0.1.0 milestone (2026-11-21 … 2026-11-26) superseded the original phase plan in §B.
"Complete" means the phase's requirement list is met and evidenced; "executed" is not the
same as the milestone being complete.

| Phase | Status | Notes |
|-------|--------|-------|
| 0 — Audit & plan | Complete | `PLAN.md` written; audit done. |
| 1 — Truth Baseline | Complete (2026-11-23) | SBOX-01/02, TEST-01/02/03, EVID-01/02, COST-01/02/03, DEF-01. `scripts/test-gate.sh` green (last recorded 308 results / 219 binaries / 89 doctests), determinism fixed, cost labelled an estimate, `deepseek-flash` default with the caveat. Evidence under `docs/evidence/01-truth-baseline/`. |
| 2 — Loop-Item Verification | Complete (2026-11-25) | PROOF-01/02/03, PERM-01…04, SESS-01…04. Live smoke, measured egress, key-leak scan, interactive permission prompt, headless denial hint, plan mode, resume. Evidence under `docs/evidence/02-loop-item-verification/`. |
| 3 — Pipeline Decoupling & Version | Complete (2026-11-25; REL-02/03 closed 2026-11-26) | REL-01/07/08, PKG-01/02/03. Split `build-linux`/`build-macos`, container depends on Linux only, version `0.1.0`. Tag run 45724842211 proved macOS (47.8 / 61.5 min against the 240 min budget) and both signatures. |
| 4 — Distribution Proof | 5/5 plans executed, phase **not** complete | REL-04 complete (image pulled by index digest and run), REL-06 complete (pinned install from the real release; `latest` observed to 404), PKG-04 complete (formula in-tree, `brew style`/`audit` clean, `brew install` an author step). REL-05 **partial**: the turn is proven, the sandbox clause is not (no `bwrap` in the image). The `v0.1.0-proof1` prerelease is retained by recorded deviation as the only macOS artifact. Evidence under `docs/evidence/04-distribution-proof/`. |
| 5 — Windows Support | In progress (0/5) | Compile-spike fixes landed on `main` (`find-msvc-tools`, `xai-proto-build` temp files, pty `cfg` gates) and the sandbox rejects non-`off` profiles on Windows with tests. A `build-windows` job, `install.ps1` and `scripts/test-install.ps1` are in the working tree; no tagged Windows `.zip`, no Windows execution of the installer, and no acceptance record yet. |
| 5.1 — Project Directory Rebrand | Complete (2026-11-26) | BRAND-01/02/03. `xai-grok-brand::PROJECT_DIR_NAME = ".deepseek"` with `.grok` as a read fallback; five crates resolve through it; open-build flips the constant to `.ob`. |
| 6 — Documentation Truth & Freeze | In progress | This work (DOCS-01…04). ACC-01 is a blocking human gate; FREEZE-01/02/03 do not run until the signed acceptance note is committed. |

**Milestone outcome:** the tree, the proof artifacts and the prerelease exist; a stable
`v0.1.0` does not. Remaining before the freeze: Phase 5 Windows distribution, REL-05's
sandbox clause (or an explicitly accepted limitation), ACC-01, then FREEZE-01/02/03.
