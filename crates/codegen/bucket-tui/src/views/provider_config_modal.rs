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
    let dialog_height = 15;

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

    // Clear background
    let bg_style = Style::default().bg(theme.bg_dark);
    for y in dialog.y..dialog.y + dialog.height {
        for x in dialog.x..dialog.x + dialog.width {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ');
                cell.set_style(bg_style);
            }
        }
    }

    // Draw main outer border
    let border_style = Style::default().fg(theme.gray);
    for x in dialog.x + 1..dialog.x + dialog.width - 1 {
        if let Some(cell) = buf.cell_mut((x, dialog.y)) { cell.set_char('─').set_style(border_style); }
        if let Some(cell) = buf.cell_mut((x, dialog.y + dialog.height - 1)) { cell.set_char('─').set_style(border_style); }
    }
    for y in dialog.y + 1..dialog.y + dialog.height - 1 {
        if let Some(cell) = buf.cell_mut((dialog.x, y)) { cell.set_char('│').set_style(border_style); }
        if let Some(cell) = buf.cell_mut((dialog.x + dialog.width - 1, y)) { cell.set_char('│').set_style(border_style); }
    }
    if let Some(cell) = buf.cell_mut((dialog.x, dialog.y)) { cell.set_char('┌').set_style(border_style); }
    if let Some(cell) = buf.cell_mut((dialog.x + dialog.width - 1, dialog.y)) { cell.set_char('┐').set_style(border_style); }
    if let Some(cell) = buf.cell_mut((dialog.x, dialog.y + dialog.height - 1)) { cell.set_char('└').set_style(border_style); }
    if let Some(cell) = buf.cell_mut((dialog.x + dialog.width - 1, dialog.y + dialog.height - 1)) { cell.set_char('┘').set_style(border_style); }

    let inner_x = dialog.x + 3;
    let inner_width = dialog.width.saturating_sub(6);

    // Title on the top border (overlapping)
    let title_text = " CONFIGURE BYOK PROVIDERS ";
    let title_style = Style::default().fg(theme.accent_user).add_modifier(Modifier::BOLD);
    let title_len = title_text.len() as u16;
    let title_x = dialog.x + (dialog.width - title_len) / 2;
    for (i, c) in title_text.chars().enumerate() {
        if let Some(cell) = buf.cell_mut((title_x + i as u16, dialog.y)) {
            cell.set_char(c).set_style(title_style);
        }
    }

    // Section 1: Provider selection box
    let grid_y = dialog.y + 2;
    let grid_height = 4;
    let grid_border_style = if state.focus == 0 {
        Style::default().fg(theme.accent_user)
    } else {
        Style::default().fg(theme.gray)
    };

    // Draw inner grid border
    for x in inner_x..inner_x + inner_width {
        if let Some(cell) = buf.cell_mut((x, grid_y)) { cell.set_char('─').set_style(grid_border_style); }
        if let Some(cell) = buf.cell_mut((x, grid_y + grid_height - 1)) { cell.set_char('─').set_style(grid_border_style); }
    }
    for y in grid_y + 1..grid_y + grid_height - 1 {
        if let Some(cell) = buf.cell_mut((inner_x, y)) { cell.set_char('│').set_style(grid_border_style); }
        if let Some(cell) = buf.cell_mut((inner_x + inner_width - 1, y)) { cell.set_char('│').set_style(grid_border_style); }
    }
    if let Some(cell) = buf.cell_mut((inner_x, grid_y)) { cell.set_char('┌').set_style(grid_border_style); }
    if let Some(cell) = buf.cell_mut((inner_x + inner_width - 1, grid_y)) { cell.set_char('┐').set_style(grid_border_style); }
    if let Some(cell) = buf.cell_mut((inner_x, grid_y + grid_height - 1)) { cell.set_char('└').set_style(grid_border_style); }
    if let Some(cell) = buf.cell_mut((inner_x + inner_width - 1, grid_y + grid_height - 1)) { cell.set_char('┘').set_style(grid_border_style); }

    // Subtitle on inner grid top border
    let grid_title = " Select Provider (1-6 or ←/→) ";
    let grid_title_style = if state.focus == 0 {
        Style::default().fg(theme.accent_user).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.gray)
    };
    let grid_title_x = inner_x + (inner_width - grid_title.len() as u16) / 2;
    for (i, c) in grid_title.chars().enumerate() {
        if let Some(cell) = buf.cell_mut((grid_title_x + i as u16, grid_y)) {
            cell.set_char(c).set_style(grid_title_style);
        }
    }

    // Render provider options
    let provider_names = [
        "1. OpenAI",
        "2. Anthropic",
        "3. NVIDIA NIM",
        "4. OpenRouter",
        "5. Groq",
        "6. Custom URL",
    ];

    let render_pill = |idx: usize, label: &str, x: u16, y: u16, buf: &mut Buffer| {
        let is_selected = state.selected_provider_idx == idx;
        let is_focused = state.focus == 0;
        let prefix = if is_selected { "● " } else { "○ " };
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
        line.render(Rect::new(x, y, 20, 1), buf);
    };

    let col_w = (inner_width - 2) / 3;
    render_pill(0, provider_names[0], inner_x + 2, grid_y + 1, buf);
    render_pill(1, provider_names[1], inner_x + 2 + col_w, grid_y + 1, buf);
    render_pill(2, provider_names[2], inner_x + 2 + col_w * 2, grid_y + 1, buf);

    render_pill(3, provider_names[3], inner_x + 2, grid_y + 2, buf);
    render_pill(4, provider_names[4], inner_x + 2 + col_w, grid_y + 2, buf);
    render_pill(5, provider_names[5], inner_x + 2 + col_w * 2, grid_y + 2, buf);

    // Render custom url input line if custom is selected
    if state.selected_provider_idx == 5 {
        let input_y = dialog.y + 7;
        let mut custom_line = vec![
            Span::styled("   Custom Name/URL: ", Style::default().fg(theme.gray_bright)),
            Span::styled(&state.custom_provider_input, Style::default().fg(theme.text_primary)),
        ];
        if state.focus == 0 {
            custom_line.push(Span::styled("█", Style::default().fg(theme.accent_user)));
        }
        Line::from(custom_line).render(Rect::new(inner_x, input_y, inner_width, 1), buf);
    } else {
        let details_y = dialog.y + 7;
        let active_p = &PRECONFIGURED_PROVIDERS[state.selected_provider_idx];
        let p_info = Line::from(vec![
            Span::styled("   Provider ID: ", Style::default().fg(theme.gray)),
            Span::styled(active_p.id, Style::default().fg(theme.accent_user).add_modifier(Modifier::BOLD)),
        ]);
        p_info.render(Rect::new(inner_x, details_y, inner_width, 1), buf);
    }

    // Section 2: API Key input box
    let key_y = dialog.y + 9;
    let key_input_style = if state.focus == 1 {
        Style::default().fg(theme.accent_user)
    } else {
        Style::default().fg(theme.gray)
    };

    // Draw field bracket for API Key
    let input_prefix = " API Key: ";
    Line::from(Span::styled(input_prefix, Style::default().fg(theme.gray_bright).add_modifier(Modifier::BOLD)))
        .render(Rect::new(inner_x, key_y, inner_width, 1), buf);

    let val_x = inner_x + input_prefix.len() as u16;
    let val_w = inner_width.saturating_sub(input_prefix.len() as u16 + 2);

    // Draw input brackets `[ ... ]`
    if let Some(cell) = buf.cell_mut((val_x, key_y)) { cell.set_char('[').set_style(key_input_style); }
    if let Some(cell) = buf.cell_mut((val_x + val_w + 1, key_y)) { cell.set_char(']').set_style(key_input_style); }

    let masked_key = "*".repeat(state.api_key_input.len());
    let mut k_line = vec![];
    if state.api_key_input.is_empty() && state.provider_has_env_key() {
        k_line.push(Span::styled(
            " (api key configured from environment) ",
            Style::default().fg(theme.accent_user).add_modifier(Modifier::BOLD),
        ));
    } else {
        k_line.push(Span::styled(format!(" {} ", masked_key), Style::default().fg(theme.text_primary)));
    }
    if state.focus == 1 {
        k_line.push(Span::styled("█", Style::default().fg(theme.accent_user)));
    }
    Line::from(k_line).render(Rect::new(val_x + 1, key_y, val_w, 1), buf);

    // Format Example / Guide line
    let key_guide = state.current_key_example();
    let guide_line = Line::from(vec![
        Span::styled("   Example: ", Style::default().fg(theme.gray)),
        Span::styled(key_guide, Style::default().fg(theme.gray_bright)),
    ]);
    guide_line.render(Rect::new(inner_x, key_y + 1, inner_width, 1), buf);

    // Section 3: Hints in Footer
    let hints_y = dialog.y + dialog.height - 2;
    let hints = Line::from(vec![
        Span::styled(" tab ", Style::default().fg(theme.accent_user).add_modifier(Modifier::BOLD)),
        Span::styled("Next Field  •  ", Style::default().fg(theme.gray)),
        Span::styled(" enter ", Style::default().fg(theme.accent_user).add_modifier(Modifier::BOLD)),
        Span::styled("Save  •  ", Style::default().fg(theme.gray)),
        Span::styled(" esc ", Style::default().fg(theme.accent_user).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel", Style::default().fg(theme.gray)),
    ]).alignment(ratatui::layout::Alignment::Center);
    hints.render(Rect::new(inner_x, hints_y, inner_width, 1), buf);
}
