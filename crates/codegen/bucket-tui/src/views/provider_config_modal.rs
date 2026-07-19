//! Provider configuration modal for BYOK.
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreconfiguredProvider {
    pub id: &'static str,
    pub name: &'static str,
    pub key_example: &'static str,
}

pub const PRECONFIGURED_PROVIDERS: [PreconfiguredProvider; 5] = [
    PreconfiguredProvider {
        id: "openai",
        name: "OpenAI",
        key_example: "sk-proj-... (e.g. sk-proj-1234567890abcdef...)",
    },
    PreconfiguredProvider {
        id: "anthropic",
        name: "Anthropic",
        key_example: "sk-ant-api03-... (e.g. sk-ant-api03-1234567890abcdef...)",
    },
    PreconfiguredProvider {
        id: "nvidia_nim",
        name: "NVIDIA NIM",
        key_example: "nvapi-... (e.g. nvapi-1234567890abcdef...)",
    },
    PreconfiguredProvider {
        id: "openrouter",
        name: "OpenRouter",
        key_example: "sk-or-v1-... (e.g. sk-or-v1-1234567890abcdef...)",
    },
    PreconfiguredProvider {
        id: "groq",
        name: "Groq",
        key_example: "gsk_... (e.g. gsk_1234567890abcdef...)",
    },
];

#[derive(Debug)]
pub struct ProviderConfigModalState {
    pub selected_provider_idx: usize, // 0..=4 = preconfigured, 5 = Type it
    pub custom_provider_input: String,
    pub provider_input: String,
    pub api_key_input: String,
    pub focus: usize, // 0 = provider selection / custom input, 1 = api_key input
}

pub enum ProviderConfigModalOutcome {
    Unchanged,
    Changed,
    Confirmed,
    Cancelled,
}

impl Default for ProviderConfigModalState {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderConfigModalState {
    pub fn new() -> Self {
        Self {
            selected_provider_idx: 0,
            custom_provider_input: String::new(),
            provider_input: PRECONFIGURED_PROVIDERS[0].id.to_string(),
            api_key_input: String::new(),
            focus: 0,
        }
    }

    pub fn update_provider_input(&mut self) {
        if self.selected_provider_idx < PRECONFIGURED_PROVIDERS.len() {
            self.provider_input = PRECONFIGURED_PROVIDERS[self.selected_provider_idx]
                .id
                .to_string();
        } else {
            self.provider_input = self.custom_provider_input.clone();
        }
    }

    pub fn provider_has_env_key(&self) -> bool {
        let p = self.provider_input.trim().to_lowercase();
        match p.as_str() {
            "openai" => std::env::var("OPENAI_API_KEY")
                .map(|k| !k.trim().is_empty())
                .unwrap_or(false),
            "anthropic" => std::env::var("ANTHROPIC_API_KEY")
                .or_else(|_| std::env::var("ANTHROPIC_AUTH_TOKEN"))
                .map(|k| !k.trim().is_empty())
                .unwrap_or(false),
            "nvidia_nim" | "nvidia" => std::env::var("NVIDIA_API_KEY")
                .or_else(|_| std::env::var("NIM_API_KEY"))
                .or_else(|_| std::env::var("NVAPI_KEY"))
                .map(|k| !k.trim().is_empty())
                .unwrap_or(false),
            "openrouter" => std::env::var("OPENROUTER_API_KEY")
                .map(|k| !k.trim().is_empty())
                .unwrap_or(false),
            "groq" => std::env::var("GROQ_API_KEY")
                .map(|k| !k.trim().is_empty())
                .unwrap_or(false),
            "gemini" | "google" => std::env::var("GEMINI_API_KEY")
                .or_else(|_| std::env::var("GOOGLE_API_KEY"))
                .map(|k| !k.trim().is_empty())
                .unwrap_or(false),
            "ollama" => true,
            _ => std::env::var("BUCKET_API_KEY")
                .map(|k| !k.trim().is_empty())
                .unwrap_or(false),
        }
    }

    pub fn current_key_example(&self) -> &'static str {
        if self.selected_provider_idx < PRECONFIGURED_PROVIDERS.len() {
            PRECONFIGURED_PROVIDERS[self.selected_provider_idx].key_example
        } else {
            "your-api-key-here (e.g. sk-...)"
        }
    }

    pub fn handle_key(&mut self, key: &crossterm::event::KeyEvent) -> ProviderConfigModalOutcome {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Esc => ProviderConfigModalOutcome::Cancelled,
            KeyCode::Enter => ProviderConfigModalOutcome::Confirmed,
            KeyCode::Tab => {
                self.focus = (self.focus + 1) % 2;
                ProviderConfigModalOutcome::Changed
            }
            KeyCode::BackTab => {
                self.focus = if self.focus == 0 { 1 } else { 0 };
                ProviderConfigModalOutcome::Changed
            }
            KeyCode::Up => {
                if self.focus == 1 {
                    self.focus = 0;
                    ProviderConfigModalOutcome::Changed
                } else if self.selected_provider_idx >= 3 {
                    self.selected_provider_idx -= 3;
                    self.update_provider_input();
                    ProviderConfigModalOutcome::Changed
                } else {
                    ProviderConfigModalOutcome::Unchanged
                }
            }
            KeyCode::Down => {
                if self.focus == 0 {
                    if self.selected_provider_idx < 3 {
                        self.selected_provider_idx += 3;
                        self.update_provider_input();
                        ProviderConfigModalOutcome::Changed
                    } else {
                        self.focus = 1;
                        ProviderConfigModalOutcome::Changed
                    }
                } else {
                    ProviderConfigModalOutcome::Unchanged
                }
            }
            KeyCode::Left => {
                if self.focus == 0 {
                    if self.selected_provider_idx > 0 {
                        self.selected_provider_idx -= 1;
                    } else {
                        self.selected_provider_idx = 5;
                    }
                    self.update_provider_input();
                    ProviderConfigModalOutcome::Changed
                } else {
                    ProviderConfigModalOutcome::Unchanged
                }
            }
            KeyCode::Right => {
                if self.focus == 0 {
                    if self.selected_provider_idx < 5 {
                        self.selected_provider_idx += 1;
                    } else {
                        self.selected_provider_idx = 0;
                    }
                    self.update_provider_input();
                    ProviderConfigModalOutcome::Changed
                } else {
                    ProviderConfigModalOutcome::Unchanged
                }
            }
            KeyCode::Char(c) => {
                if self.focus == 0 {
                    if self.selected_provider_idx < 5 && c >= '1' && c <= '6' {
                        let idx = (c as usize) - ('1' as usize);
                        self.selected_provider_idx = idx;
                        self.update_provider_input();
                    } else if self.selected_provider_idx == 5 {
                        if c >= '1' && c <= '6' && self.custom_provider_input.is_empty() {
                            let idx = (c as usize) - ('1' as usize);
                            self.selected_provider_idx = idx;
                            self.update_provider_input();
                        } else {
                            self.custom_provider_input.push(c);
                            self.update_provider_input();
                        }
                    }
                } else {
                    self.api_key_input.push(c);
                }
                ProviderConfigModalOutcome::Changed
            }
            KeyCode::Backspace => {
                if self.focus == 0 {
                    if self.selected_provider_idx == 5 {
                        self.custom_provider_input.pop();
                        self.update_provider_input();
                    }
                } else {
                    self.api_key_input.pop();
                }
                ProviderConfigModalOutcome::Changed
            }
            _ => ProviderConfigModalOutcome::Unchanged,
        }
    }
}

pub fn render_provider_config_modal(
    area: Rect,
    buf: &mut Buffer,
    state: &ProviderConfigModalState,
) {
    let theme = Theme::current();
    let dialog_width = 72;
    let dialog_height = 14;

    if area.height < dialog_height || area.width < dialog_width {
        return;
    }

    let [_, dialog_h, _] = Layout::horizontal([
        Constraint::Min(0),
        Constraint::Length(dialog_width),
        Constraint::Min(0),
    ])
    .flex(Flex::Center)
    .areas(area);

    let [_, dialog, _] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(dialog_height),
        Constraint::Min(0),
    ])
    .flex(Flex::Center)
    .areas(dialog_h);

    let bg_style = Style::default().bg(theme.bg_dark);
    for y in dialog.y..dialog.y + dialog.height {
        for x in dialog.x..dialog.x + dialog.width {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ');
                cell.set_style(bg_style);
            }
        }
    }

    let inner_x = dialog.x + 2;
    let inner_width = dialog.width.saturating_sub(4);

    // Title
    let title = Line::from(Span::styled(
        "Configure BYOK Provider",
        Style::default()
            .fg(theme.text_primary)
            .add_modifier(Modifier::BOLD),
    ));
    title.render(Rect::new(inner_x, dialog.y + 1, inner_width, 1), buf);

    // Provider selection menu header
    let p_header = Line::from(vec![
        Span::styled(
            "Provider: ",
            Style::default()
                .fg(theme.gray_bright)
                .add_modifier(Modifier::BOLD),
        ),
        if state.focus == 0 {
            Span::styled(
                "(use ←/→ or 1-6 to select)",
                Style::default().fg(theme.accent_user),
            )
        } else {
            Span::styled("(press Tab to edit)", Style::default().fg(theme.gray))
        },
    ]);
    p_header.render(Rect::new(inner_x, dialog.y + 3, inner_width, 1), buf);

    // Grid row 1 (1. OpenAI, 2. Anthropic, 3. NVIDIA NIM)
    let provider_names = [
        "1. OpenAI",
        "2. Anthropic",
        "3. NVIDIA NIM",
        "4. OpenRouter",
        "5. Groq",
        "6. Type it",
    ];

    let render_pill = |idx: usize, label: &str, x: u16, y: u16, buf: &mut Buffer| {
        let is_selected = state.selected_provider_idx == idx;
        let is_focused = state.focus == 0;
        let prefix = if is_selected { "(•) " } else { "( ) " };
        let full_text = format!("{}{}", prefix, label);

        let style = if is_selected && is_focused {
            Style::default()
                .fg(theme.text_primary)
                .bg(theme.bg_highlight)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default()
                .fg(theme.accent_user)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.gray_bright)
        };

        let line = Line::from(Span::styled(full_text, style));
        line.render(Rect::new(x, y, 21, 1), buf);
    };

    // Row 0 pills
    render_pill(0, provider_names[0], inner_x + 1, dialog.y + 4, buf);
    render_pill(1, provider_names[1], inner_x + 23, dialog.y + 4, buf);
    render_pill(2, provider_names[2], inner_x + 45, dialog.y + 4, buf);

    // Row 1 pills
    render_pill(3, provider_names[3], inner_x + 1, dialog.y + 5, buf);
    render_pill(4, provider_names[4], inner_x + 23, dialog.y + 5, buf);
    render_pill(5, provider_names[5], inner_x + 45, dialog.y + 5, buf);

    // Selected Provider Details / Custom text input line
    if state.selected_provider_idx == 5 {
        let mut custom_line = vec![
            Span::styled(
                "  Custom Provider Name: ",
                Style::default().fg(theme.gray_bright),
            ),
            Span::styled(
                &state.custom_provider_input,
                Style::default().fg(theme.text_primary),
            ),
        ];
        if state.focus == 0 {
            custom_line.push(Span::styled(
                "\u{2588}",
                Style::default().fg(theme.accent_user),
            ));
        }
        Line::from(custom_line).render(Rect::new(inner_x, dialog.y + 6, inner_width, 1), buf);
    } else {
        let active_p = &PRECONFIGURED_PROVIDERS[state.selected_provider_idx];
        let p_info = Line::from(vec![
            Span::styled("  Selected ID: ", Style::default().fg(theme.gray)),
            Span::styled(
                active_p.id,
                Style::default()
                    .fg(theme.accent_user)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        p_info.render(Rect::new(inner_x, dialog.y + 6, inner_width, 1), buf);
    }

    // API Key input section
    let k_prefix = "API Key:  ";
    let mut k_line = vec![Span::styled(
        k_prefix,
        Style::default()
            .fg(theme.gray_bright)
            .add_modifier(Modifier::BOLD),
    )];
    let masked_key = "*".repeat(state.api_key_input.len());
    if state.api_key_input.is_empty() && state.provider_has_env_key() {
        k_line.push(Span::styled(
            "(api key configured)",
            Style::default()
                .fg(theme.accent_user)
                .add_modifier(Modifier::BOLD),
        ));
    } else {
        k_line.push(Span::styled(
            &masked_key,
            Style::default().fg(theme.text_primary),
        ));
    }
    if state.focus == 1 {
        k_line.push(Span::styled(
            "\u{2588}",
            Style::default().fg(theme.accent_user),
        ));
    }
    Line::from(k_line).render(Rect::new(inner_x, dialog.y + 8, inner_width, 1), buf);

    // Guide / Example format line
    let key_guide = state.current_key_example();
    let guide_line = Line::from(vec![
        Span::styled(
            "  Guide:  ",
            Style::default()
                .fg(theme.accent_user)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(key_guide, Style::default().fg(theme.gray_bright)),
    ]);
    guide_line.render(Rect::new(inner_x, dialog.y + 9, inner_width, 1), buf);

    // Key hints
    let hints = Line::from(vec![
        Span::styled(
            "tab",
            Style::default()
                .fg(theme.accent_user)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" = next field   ", Style::default().fg(theme.gray)),
        Span::styled(
            "←/→/1-6",
            Style::default()
                .fg(theme.accent_user)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" = select provider   ", Style::default().fg(theme.gray)),
        Span::styled(
            "enter",
            Style::default()
                .fg(theme.accent_user)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" = save   ", Style::default().fg(theme.gray)),
        Span::styled(
            "esc",
            Style::default()
                .fg(theme.accent_user)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" = cancel", Style::default().fg(theme.gray)),
    ]);
    hints.render(Rect::new(inner_x, dialog.y + 12, inner_width, 1), buf);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn make_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_default_provider_selection() {
        let state = ProviderConfigModalState::new();
        assert_eq!(state.selected_provider_idx, 0);
        assert_eq!(state.provider_input, "openai");
        assert!(state.current_key_example().contains("sk-proj-"));
    }

    #[test]
    fn test_switching_providers_updates_key_example() {
        let mut state = ProviderConfigModalState::new();

        // Press 2 for Anthropic
        state.handle_key(&make_key(KeyCode::Char('2')));
        assert_eq!(state.selected_provider_idx, 1);
        assert_eq!(state.provider_input, "anthropic");
        assert!(state.current_key_example().contains("sk-ant-api03-"));

        // Press 3 for NVIDIA NIM
        state.handle_key(&make_key(KeyCode::Char('3')));
        assert_eq!(state.selected_provider_idx, 2);
        assert_eq!(state.provider_input, "nvidia_nim");
        assert!(state.current_key_example().contains("nvapi-"));

        // Press 4 for OpenRouter
        state.handle_key(&make_key(KeyCode::Char('4')));
        assert_eq!(state.selected_provider_idx, 3);
        assert_eq!(state.provider_input, "openrouter");
        assert!(state.current_key_example().contains("sk-or-v1-"));

        // Press 5 for Groq
        state.handle_key(&make_key(KeyCode::Char('5')));
        assert_eq!(state.selected_provider_idx, 4);
        assert_eq!(state.provider_input, "groq");
        assert!(state.current_key_example().contains("gsk_"));
    }

    #[test]
    fn test_type_it_custom_provider() {
        let mut state = ProviderConfigModalState::new();

        // Press 6 for "Type it"
        state.handle_key(&make_key(KeyCode::Char('6')));
        assert_eq!(state.selected_provider_idx, 5);
        assert_eq!(state.provider_input, "");

        // Type custom provider name: "my_local_llm"
        for c in "my_local_llm".chars() {
            state.handle_key(&make_key(KeyCode::Char(c)));
        }
        assert_eq!(state.provider_input, "my_local_llm");
        assert_eq!(
            state.current_key_example(),
            "your-api-key-here (e.g. sk-...)"
        );
    }

    #[test]
    fn test_provider_has_env_key() {
        let state = ProviderConfigModalState::new();
        unsafe {
            std::env::set_var("OPENAI_API_KEY", "sk-test12345");
        }
        assert!(state.provider_has_env_key());
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
        }
    }
}
