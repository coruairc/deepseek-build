#!/usr/bin/env python3
"""Regenerate `crates/codegen/xai-grok-agent/src/prompt/prompt_encrypted.rs`.

The base system-prompt templates under `crates/codegen/xai-grok-agent/templates/`
are XOR-obfuscated so they do not show up as obvious plaintext in `strings`.
This is obfuscation, not security.

Run after editing any file in `crates/codegen/xai-grok-agent/templates/`:

    python3 scripts/encrypt_templates.py
"""

from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TEMPLATES = ROOT / "crates/codegen/xai-grok-agent/templates"
OUT = ROOT / "crates/codegen/xai-grok-agent/src/prompt/prompt_encrypted.rs"

SEEDS = [0x5A, 0x7B, 0x3D]
TEMPLATE_FILES = ["prompt.md", "apply_patch_prompt.md", "subagent_prompt.md"]
CONSTS = ["BASE_PROMPT_ENC", "CODEX_PROMPT_ENC", "SUBAGENT_PROMPT_ENC"]


def xor_encrypt(data: bytes, seed: int) -> bytes:
    return bytes(b ^ ((seed + i) & 0xFF) for i, b in enumerate(data))


def rust_byte_array(data: bytes) -> str:
    return "&[" + ", ".join(str(b) for b in data) + "]"


def main() -> None:
    parts = [
        "// Auto-generated -- do not edit.",
        "// Regenerate: python3 scripts/encrypt_templates.py",
        "// XOR-encrypted prompt templates (key = position-dependent seed).",
        "",
    ]
    for name, const in zip(TEMPLATE_FILES, CONSTS):
        raw = (TEMPLATES / name).read_bytes()
        enc = xor_encrypt(raw, SEEDS[TEMPLATE_FILES.index(name)])
        parts.append("#[rustfmt::skip]")
        parts.append(f"pub(crate) const {const}: &[u8] = {rust_byte_array(enc)};")
        parts.append("")
    seeds = ", ".join(f"0x{s:02X}" for s in SEEDS)
    parts.append(f"pub(crate) const PROMPT_SEEDS: [u8; 3] = [{seeds}];")
    parts.append("")
    OUT.write_text("\n".join(parts))


if __name__ == "__main__":
    main()
