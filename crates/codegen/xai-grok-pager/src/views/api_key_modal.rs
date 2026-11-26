//! Masked DeepSeek API-key prompt shown at startup when no key is available.
//!
//! The input is a single-line [`LineEditor`] rendered as `*` per character. The
//! value is handed to `Action::SubmitApiKey`, which owns persistence; this view
//! never logs or displays it.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::input::line_editor::{LineEditOutcome, LineEditor};
use crate::theme::Theme;
use crate::views::modal_window::{
    self, ModalContentArea, ModalSizing, ModalWindowConfig, ModalWindowState, Shortcut,
};

/// Whether the startup prompt should open. Only an interactive startup that
/// needs a login, with neither env nor a usable saved key, prompts. An unusable
/// file (insecure permissions) still prompts so the user can replace it.
pub(crate) fn should_show_prompt(
    needs_login: bool,
    force_login: bool,
    resolution: &Result<Option<String>, xai_grok_login::credentials_store::CredentialsError>,
) -> bool {
    needs_login && !force_login && matches!(resolution, Ok(None) | Err(_))
}

/// Cap pasted keys so a stray huge clipboard cannot freeze the UI.
const MAX_KEY_BYTES: usize = 4096;
const LABEL: &str = "API key: ";
const TITLE: &str = "DeepSeek API key";
const HINT_STORAGE: &str = "Stored locally in ~/.deepseek-build/credentials.json (0600).";
const HINT_ENV: &str = "Set DEEPSEEK_API_KEY later to replace it; the environment wins.";

/// Result of an input event for the caller to act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiKeyModalOutcome {
    /// User pressed Enter with a valid key.
    Submitted(String),
    /// User pressed Esc / Ctrl-C.
    Cancelled,
    /// State changed; caller should redraw.
    Changed,
    /// Key not consumed.
    Unchanged,
}

/// State for the masked API-key prompt.
pub struct ApiKeyModalState {
    pub window: ModalWindowState,
    input: LineEditor,
    error: Option<String>,
}

impl ApiKeyModalState {
    pub fn new(error: Option<String>) -> Self {
        Self {
            window: ModalWindowState::new(),
            input: LineEditor::default(),
            error,
        }
    }

    /// Current (unmasked) input, for tests only.
    #[cfg(test)]
    pub(crate) fn text(&self) -> &str {
        self.input.text()
    }

    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error = Some(error.into());
    }

    fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn insert_paste(&mut self, text: &str) -> ApiKeyModalOutcome {
        self.error = None;
        Self::from_line_edit(self.input.insert_paste_with_policy(
            text,
            |character| !character.is_control(),
            MAX_KEY_BYTES,
        ))
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ApiKeyModalOutcome {
        if crate::input::key::is_paste_key(key) {
            return crate::clipboard::system_clipboard_get()
                .map_or(ApiKeyModalOutcome::Unchanged, |text| {
                    self.insert_paste(&text)
                });
        }
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && !crate::input::key::is_altgr(key.modifiers)
            && matches!(key.code, KeyCode::Char('c' | 'd' | 'q'))
        {
            return ApiKeyModalOutcome::Cancelled;
        }
        if key.code == KeyCode::Esc {
            return ApiKeyModalOutcome::Cancelled;
        }
        if key.code == KeyCode::Enter && key.modifiers.is_empty() {
            let key = self.input.text();
            if key.is_empty() || key.trim() != key {
                self.error =
                    Some("Enter a non-empty key without leading or trailing whitespace.".into());
                return ApiKeyModalOutcome::Changed;
            }
            return ApiKeyModalOutcome::Submitted(key.to_owned());
        }

        self.error = None;
        Self::from_line_edit(
            self.input
                .handle_key_with_insert_policy(key, |character| !character.is_control()),
        )
    }

    fn from_line_edit(outcome: LineEditOutcome) -> ApiKeyModalOutcome {
        match outcome {
            LineEditOutcome::TextChanged
            | LineEditOutcome::CursorChanged
            | LineEditOutcome::HandledNoChange => ApiKeyModalOutcome::Changed,
            LineEditOutcome::Unhandled => ApiKeyModalOutcome::Unchanged,
        }
    }
}

/// Render the prompt with the shared modal chrome. The key is always masked.
pub fn render_api_key_modal(
    buf: &mut Buffer,
    full_area: Rect,
    state: &mut ApiKeyModalState,
    _compact: bool,
) {
    let theme = Theme::current();
    let shortcuts = [
        Shortcut {
            label: "Enter save",
            clickable: false,
            id: 0,
        },
        Shortcut {
            label: "Esc cancel",
            clickable: false,
            id: 1,
        },
    ];
    let config = ModalWindowConfig {
        title: TITLE,
        tabs: None,
        shortcuts: &shortcuts,
        sizing: ModalSizing {
            width_pct: 0.70,
            max_width: 96,
            min_width: 44,
            v_margin: 7,
            h_pad: 2,
            v_pad: 1,
            footer_lines: 2,
        },
        fold_info: None,
    };
    let Some(ModalContentArea { content, .. }) =
        modal_window::render_modal_window(buf, full_area, &mut state.window, &config, &theme)
    else {
        return;
    };
    if content.height == 0 || content.width == 0 {
        return;
    }

    let hint_style = Style::default().fg(theme.gray_bright);
    let mut row = content.y;
    let width = content.width;

    // One-line hint that explains where the key goes. Long lines are clipped by
    // `set_line`; the modal is sized to fit them on normal terminals.
    for hint in [HINT_STORAGE, HINT_ENV] {
        if row >= content.y + content.height.saturating_sub(3) {
            break;
        }
        let line = Line::from(Span::styled(hint, hint_style));
        buf.set_line(content.x, row, &line, width);
        row = row.saturating_add(1);
    }
    row = row.saturating_add(1);
    if row >= content.y + content.height {
        return;
    }

    let label_style = Style::default()
        .fg(theme.text_primary)
        .add_modifier(Modifier::BOLD);
    let label_width = LABEL.len();
    let input_width = (width as usize).saturating_sub(label_width);
    let viewport = state.input.viewport(input_width);
    let visible = state
        .input
        .text()
        .get(viewport.visible_byte_range.clone())
        .unwrap_or("");
    let mask: String = visible.chars().map(|_| '*').collect();
    let input_line = Line::from(vec![
        Span::styled(LABEL, label_style),
        Span::styled(mask, Style::default().fg(theme.text_primary)),
    ]);
    buf.set_line(content.x, row, &input_line, width);

    if input_width > 0 {
        let cursor_column = viewport.cursor_display_column.min(input_width - 1);
        let cursor_x = content.x + label_width as u16 + cursor_column as u16;
        if let Some(cell) = buf.cell_mut((cursor_x, row)) {
            cell.set_style(theme.block_cursor_over(theme.bg_base));
        }
    }
    row = row.saturating_add(1);

    if let Some(error) = state.error()
        && row < content.y + content.height
    {
        let line = Line::from(Span::styled(error, Style::default().fg(theme.accent_error)));
        buf.set_line(content.x, row, &line, width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_grok_login::credentials_store::CredentialsError;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn type_text(state: &mut ApiKeyModalState, text: &str) {
        for character in text.chars() {
            state.handle_key(&key(KeyCode::Char(character)));
        }
    }

    fn render_text(state: &mut ApiKeyModalState) -> String {
        let area = Rect::new(0, 0, 100, 30);
        let mut buffer = Buffer::empty(area);
        render_api_key_modal(&mut buffer, area, state, false);
        let mut rows = Vec::new();
        for y in 0..area.height {
            let mut row = String::new();
            for x in 0..area.width {
                row.push_str(buffer.cell((x, y)).map(|cell| cell.symbol()).unwrap_or(" "));
            }
            rows.push(row);
        }
        rows.join("\n")
    }

    #[test]
    fn prompt_opens_only_when_login_is_needed_and_no_key_is_available() {
        assert!(should_show_prompt(true, false, &Ok(None)));
        assert!(should_show_prompt(
            true,
            false,
            &Err(CredentialsError::EmptyKey)
        ));
        assert!(!should_show_prompt(true, false, &Ok(Some("key".into()))));
        assert!(!should_show_prompt(false, false, &Ok(None)));
        assert!(!should_show_prompt(true, true, &Ok(None)));
    }

    #[test]
    fn rendering_masks_every_character() {
        let mut state = ApiKeyModalState::new(None);
        assert_eq!(state.insert_paste("sk-abc"), ApiKeyModalOutcome::Changed);

        let rendered = render_text(&mut state);
        assert!(rendered.contains("DeepSeek API key"), "title: {rendered}");
        assert!(
            rendered.contains("credentials.json"),
            "storage hint: {rendered}"
        );
        assert_eq!(rendered.matches('*').count(), 6, "one star per character");
        assert!(!rendered.contains("sk-abc"), "plaintext must not render");
    }

    #[test]
    fn submit_returns_the_entered_key() {
        let mut state = ApiKeyModalState::new(None);
        type_text(&mut state, "sk-abc");
        assert_eq!(
            state.handle_key(&key(KeyCode::Enter)),
            ApiKeyModalOutcome::Submitted("sk-abc".into())
        );
    }

    #[test]
    fn submit_rejects_empty_or_surrounding_whitespace() {
        let mut state = ApiKeyModalState::new(None);
        assert_eq!(
            state.handle_key(&key(KeyCode::Enter)),
            ApiKeyModalOutcome::Changed
        );
        assert!(state.error().is_some());

        state.insert_paste(" sk-abc ");
        assert_eq!(
            state.handle_key(&key(KeyCode::Enter)),
            ApiKeyModalOutcome::Changed
        );
        assert!(state.error().is_some());
    }

    #[test]
    fn escape_and_ctrl_c_cancel() {
        let mut state = ApiKeyModalState::new(None);
        assert_eq!(
            state.handle_key(&key(KeyCode::Esc)),
            ApiKeyModalOutcome::Cancelled
        );

        let mut state = ApiKeyModalState::new(None);
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(state.handle_key(&ctrl_c), ApiKeyModalOutcome::Cancelled);
    }

    #[test]
    fn paste_and_backspace_edit_the_masked_value() {
        let mut state = ApiKeyModalState::new(None);
        state.insert_paste("sk-abc");
        assert_eq!(state.text(), "sk-abc");

        state.handle_key(&key(KeyCode::Backspace));
        assert_eq!(state.text(), "sk-ab");
        let rendered = render_text(&mut state);
        assert_eq!(rendered.matches('*').count(), 5);
    }

    #[test]
    fn error_from_save_is_shown_and_value_stays_masked() {
        let mut state = ApiKeyModalState::new(None);
        state.insert_paste("sk-abc");
        state.set_error("refusing to use credentials file: run `chmod 600`");

        let rendered = render_text(&mut state);
        assert!(rendered.contains("chmod 600"));
        assert!(!rendered.contains("sk-abc"));
    }
}
