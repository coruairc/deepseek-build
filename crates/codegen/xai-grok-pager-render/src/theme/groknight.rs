//! Default dark theme: a Monokai palette with the DeepSeek blue as the primary accent.
//!
//! The canonical palette is defined in RGB (`Color::Rgb`).
//! At startup [`Theme::quantized`] downgrades every color to the terminal's detected capability level (256-color, 16-color, etc.).

use ratatui::style::{Color, Modifier};

use super::tokyonight::Theme;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

// Classic Monokai base with DeepSeek blue (#4d6bfe) as the product accent.
#[allow(dead_code)]
mod palette {
    use super::*;

    // ── Backgrounds (Monokai) ───────────────────────────────────────────
    pub const BG: Color = rgb(30, 31, 28); //   #1e1f1c, deepest (terminal bg)
    pub const BG_DARK: Color = rgb(30, 31, 28); // #1e1f1c, code blocks
    pub const BG_STORM_DARK: Color = rgb(30, 31, 28); // #1e1f1c
    pub const BG_STORM: Color = rgb(39, 40, 34); //  #272822, main bg
    pub const BG_HIGHLIGHT: Color = rgb(62, 61, 50); // #3e3d32, highlight
    pub const BG_HOVER: Color = rgb(73, 72, 62); //  #49483e

    // ── Text / grays ────────────────────────────────────────────────────
    pub const FG: Color = rgb(248, 248, 242); // #f8f8f2, primary text
    pub const FG_DARK: Color = rgb(207, 207, 194); // #cfcfc2, secondary text
    pub const FG_GUTTER: Color = rgb(73, 72, 62); // #49483e, dim
    pub const COMMENT: Color = rgb(117, 113, 94); // #75715e, comments/muted
    pub const DARK3: Color = rgb(117, 113, 94); // #75715e
    pub const DARK5: Color = rgb(160, 158, 138); // #a09e8a, bright gray

    // ── Accent colors (Monokai) ─────────────────────────────────────────
    pub const PINK: Color = rgb(249, 38, 114); // #f92672, red/pink
    pub const GREEN: Color = rgb(166, 226, 46); // #a6e22e
    pub const GREEN1: Color = rgb(115, 218, 202); // #73daca
    pub const CYAN: Color = rgb(102, 217, 239); // #66d9ef
    pub const PURPLE: Color = rgb(174, 129, 255); // #ae81ff
    pub const ORANGE: Color = rgb(253, 151, 31); // #fd971f
    pub const YELLOW: Color = rgb(230, 219, 116); // #e6db74

    // DeepSeek brand blue.
    pub const DEEPSEEK_BLUE: Color = rgb(77, 107, 254); // #4d6bfe

    pub const RED_DARK: Color = rgb(62, 23, 32); // #3e1720, deep red for diff delete
    pub const GREEN_DARK: Color = rgb(38, 51, 30); // #26331e, deep green for diff insert
}
use palette::*;

impl Theme {
    pub const fn groknight() -> Self {
        Self {
            bg_base: BG_STORM,
            bg_light: BG_HIGHLIGHT,
            bg_dark: BG_DARK,
            bg_highlight: BG_HIGHLIGHT,
            bg_hover: BG_HOVER,
            bg_terminal: BG,

            accent_user: FG_DARK,
            accent_assistant: DEEPSEEK_BLUE,
            accent_thinking: PURPLE,
            accent_tool: CYAN,
            accent_system: CYAN,
            accent_error: PINK,
            accent_success: GREEN,
            accent_running: ORANGE,
            accent_skill: DEEPSEEK_BLUE,

            text_primary: FG,
            text_secondary: FG_DARK,

            gray_dim: rgb(90, 88, 76), // #5a584c
            gray: COMMENT,
            gray_bright: DARK5,

            command: YELLOW,
            path: ORANGE,
            running: CYAN,
            warning: YELLOW,

            fuzzy_accent: DEEPSEEK_BLUE,

            accent_plan: YELLOW,

            accent_verify: PURPLE,

            accent_remember: GREEN,

            selection_border: BG_HOVER,
            prompt_border: BG_HIGHLIGHT,
            prompt_border_active: DEEPSEEK_BLUE,
            hover_border: rgb(44, 45, 39), // #2c2d27

            accent_model: CYAN,

            scrollbar_bg: BG_DARK,
            scrollbar_fg: BG_HIGHLIGHT,

            diff_delete_bg: RED_DARK,
            diff_delete_fg: PINK,
            diff_insert_bg: GREEN_DARK,
            diff_insert_fg: GREEN,
            diff_equal_fg: COMMENT,
            diff_gutter_fg: COMMENT,

            bg_visual: BG_HOVER,

            paste_bg: BG_DARK,
            paste_fg: FG_DARK,
            paste_dim: COMMENT,

            md_heading_h1: PINK,
            md_heading_h1_mod: Modifier::BOLD,
            md_heading_h2: GREEN,
            md_heading_h2_mod: Modifier::BOLD,
            md_heading_h3: CYAN,
            md_heading_h3_mod: Modifier::BOLD,
            md_heading_h4: ORANGE,
            md_heading_h4_mod: Modifier::BOLD,
            md_heading_h5: PURPLE,
            md_heading_h5_mod: Modifier::BOLD,
            md_heading_h6: COMMENT,
            md_heading_h6_mod: Modifier::empty(),
            md_code: YELLOW,
            md_task_checked: GREEN,
            md_task_unchecked: FG_DARK,
            md_muted: COMMENT,
            md_code_bg: BG_DARK,
            md_text: FG,
            link_fg: DEEPSEEK_BLUE,
        }
    }
}
