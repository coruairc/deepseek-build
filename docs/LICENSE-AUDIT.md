# License audit — deepseek-build dependency tree

- **Date:** 2026-10-07
- **Tool:** cargo-deny 0.20.2 (`cargo-deny check licenses bans sources`, exit 0)
- **Full log:** [`docs/license-deny-report.txt`](license-deny-report.txt)
- **Config:** [`deny.toml`](../deny.toml)
- **Graph targets:** `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin` (dev-deps included)

## Policy summary

| License / expression | Why allowed |
|---|---|
| Apache-2.0, MIT, MIT-0, BSD-2/3-Clause, ISC, Zlib, CC0-1.0, 0BSD, BSL-1.0, Apache-2.0 WITH LLVM-exception | Standard permissive set; satisfy with notice/attribution. |
| MPL-2.0 | Weak copyleft, **allowed by policy**; obligations = preserve notice + provide source of modified MPL files. |
| Unicode-3.0, Unicode-DFS-2016 | Permissive (Unicode/icu4x ecosystem). |
| Unlicense | Public-domain-style dedication (rust-lang crates: memchr, aho-corasick, ignore, jiff, walkdir…). |
| CDLA-Permissive-2.0 | Permissive data license used by `webpki-roots` (bundled root-CA certificate data, not code). |
| BSD-1-Clause | One clause of BSD-3; `fiat-crypto` (kept for target robustness; not hit on the 4 checked targets). |

**Deliberately not allowed:** strong copyleft (GPL, LGPL, AGPL, etc.). If any crate ever
declares one, `cargo-deny check licenses` fails and it becomes a finding here.

## Copyleft findings

### 1. libgit2 — GPL-2.0-only WITH libgit2-linking-exception (the stakeholder callout)

- **Crate:** `libgit2-sys 0.18.2+1.9.1` → reached via `git2 0.20` (used by first-party git/worktree code).
- **Nuance:** `libgit2-sys`'s own Cargo manifest declares `MIT OR Apache-2.0`, so cargo-deny
  resolves the Rust crate as permissive. The **GPL-2.0-only** term applies to the **vendored
  C libgit2 library** compiled into the crate.
- **Why this is distributable in an Apache-2.0 project:** libgit2 ships under
  `GPL-2.0-only WITH libgit2-linking-exception`. The linking exception explicitly permits
  conveying a combined work that links libgit2 without the whole work falling under GPL-2.0.
  This is exactly why git2-rs is usable in permissively-licensed projects.
- **Where it is documented:** full license text of libgit2 (GPL-2.0 with exception) is
  already shipped in `THIRD-PARTY-NOTICES` (search "libgit2"), which is included in release
  tarballs — satisfying GPL-2.0 exception notice obligations for distribution.
- **cargo-deny note:** 0.20.2's embedded SPDX list does not contain the
  `libgit2-linking-exception` term, so the expression cannot be written in `deny.toml`
  (config parsing rejects it). Since the crate resolves permissive, no exception entry is
  needed; this section plus `THIRD-PARTY-NOTICES` carry the obligation.

### Other copyleft findings

**None.** No crate in the checked tree (any of the 4 targets) requires GPL-3.0, AGPL, LGPL
or any other strong copyleft. The only LGPL expression seen is `r-efi 5.3.0`
(LGPL-2.1-or-later), which is **boot-efi-only** — it appears only under
`x86_64-*/aarch64-*-uefi` targets, none of which are in the checked release targets, so it
is not part of the shipped dependency tree.

## Weak copyleft in use (MPL-2.0) — allowed, obligations noted

| Crate | Version | Pulled in by |
|---|---|---|
| cssparser | 0.34.0 | scraper ← `xai-grok-tools` |
| cssparser-macros | 0.6.1 | cssparser |
| dtoa-short | 0.3.5 | cssparser |
| selectors | 0.26.0 | scraper |
| option-ext | 0.2.0 | dirs / similar ecosystem ← `xai-dirs` |
| nucleo | 0.5.0 | git rev-pinned ← `xai-fuzzy-file-search` ← `xai-grok-workspace` |
| nucleo-matcher | 0.3.1 | nucleo |

Obligations: keep license notices; if MPL-covered files are modified, make the modified
files' source available (file-level copyleft only — does not infect the rest).

## Notable permissive-but-notable

- **Unicode-3.0 (19 crates)** — icu4x / unicode data crates; permissive, attribution-style.
- **CDLA-Permissive-2.0** — `webpki-roots 0.26.11 / 1.0.3` (root CA cert data; share + notice).
- **BSL-1.0** — ryu, ryu-js, whoami, clipboard-win, error-code, wasite.
- **CC0-1.0** — blake3, tiny-keccak, constant_time_eq, dunce, notify.
- **Unlicense (12 crates)** — memchr, aho-corasick, byteorder, globset, ignore, jiff, walkdir, etc.
- **MIT-0** — aws-lc-sys, borrow-or-share, constant_time_eq, dunce.
- No **CDDL** crates found.

## Sources

- crates.io is the only registry (validated by `unknown-registry = "deny"`).
- Exactly **two** pinned git sources are allowed, both patch overrides of crates-io crates:
  1. `https://github.com/our-forks/async-openai.git` @ rev `95b52ebdedf42143083cf3d6f0e0be7c84e9c808` — internal fork patch (`[patch.crates-io]`). Allowed because it replaces the registry crate wholesale; pinned by full rev.
  2. `https://github.com/helix-editor/nucleo.git` @ rev `5b74652` — upstream repo, rev-pinned in the same `[patch.crates-io]` block. `nucleo` (MPL-2.0) and `nucleo-matcher` (MPL-2.0) come from here.
- Any other git or registry source fails the check (`unknown-git/unknown-registry = "deny"`).

## Advisories (non-blocking, warnings only — `cargo-deny --offline --locked check advisories`)

- **Vulnerabilities (4):** `quick-xml 0.39.4` (RUSTSEC-2026-0194 quadratic duplicate-attr
  check, RUSTSEC-2026-0195 unbounded ns-declaration allocation DoS), `rsa 0.9.10`
  (RUSTSEC-2023-0071 Marvin timing sidechannel), `rustls 0.23.37` (RUSTSEC-2026-0285
  TLS 1.3 handshake across encryption levels).
- **Unsound (3):** `git2 0.20.2` (RUSTSEC-2026-0008/0183/0184).
- **Unmaintained (10):** async-std, backoff, bincode 1.3.3, fxhash, instant, paste,
  rustybuzz, smartstring, ttf-parser, yaml-rust.
- None block the build; remediation is dependency-version work (tracked separately), not
  a license-policy issue.

## Tarball licensing

Release tarballs / installed distributions include `LICENSE`, `NOTICE`, and
`THIRD-PARTY-NOTICES` (763 KB generated file listing every bundled crate and its license
text, **including libgit2's GPL-2.0 WITH linking-exception full text**). Together with the
permissive notice obligations above, this satisfies:

- Apache-2.0 §4 (redistribution of first-party code with NOTICE),
- GPL-2.0-only + libgit2-linking-exception notice requirements for the vendored C library,
- MPL-2.0/EPL notice obligations (license texts shipped), and
- CDLA/Unicode/CC0 attribution requirements (texts shipped).

## Verdict

**Distribution-safe under the current policy.** The full dependency tree across all 4
release targets contains no strong-copyleft code; the single GPL entry point (libgit2's
vendored C sources) is neutralized by its linking exception and its license text ships in
`THIRD-PARTY-NOTICES`. Remaining obligations are notice/attribution-level only
(MPL file-level source availability if modified, CDLA share for root-cert data), all
covered by the packaged notices.