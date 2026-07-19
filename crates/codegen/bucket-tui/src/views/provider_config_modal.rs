//! Provider configuration modal for BYOK.
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::theme::Theme;

#[derive(Debug, Default)]
pub struct ProviderConfigModalState {
    pub provider_input: String,
    pub api_key_input: String,
    pub focus: usize, // 0 = provider, 1 = api_key
}

pub enum ProviderConfigModalOutcome {
    Unchanged,
    Changed,
    Confirmed,
    Cancelled,
}

impl ProviderConfigModalState {
    pub fn new() -> Self {
        Self {
            provider_input: String::new(),
            api_key_input: String::new(),
            focus: 0,
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
            KeyCode::Char(c) => {
                if self.focus == 0 {
                    self.provider_input.push(c);
                } else {
                    self.api_key_input.push(c);
                }
                ProviderConfigModalOutcome::Changed
            }
            KeyCode::Backspace => {
                if self.focus == 0 {
                    self.provider_input.pop();
                } else {
                    self.api_key_input.pop();
                }
                ProviderConfigModalOutcome::Changed
            }
            _ => ProviderConfigModalOutcome::Unchanged,
        }
    }
}

pub fn render_provider_config_modal(area: Rect, buf: &mut Buffer, state: &ProviderConfigModalState) {
    let theme = Theme::current();
    let dialog_width = 60;
    let dialog_height = 8;
    
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
    
    let title = Line::from(Span::styled(
        "Configure BYOK Provider",
        Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD),
    ));
    title.render(Rect::new(inner_x, dialog.y + 1, inner_width, 1), buf);

    let p_prefix = "Provider: ";
    let mut p_line = vec![Span::styled(p_prefix, Style::default().fg(theme.gray_bright))];
    p_line.push(Span::styled(&state.provider_input, Style::default().fg(theme.text_primary)));
    if state.focus == 0 {
        p_line.push(Span::styled("\u{2588}", Style::default().fg(theme.accent_user)));
    }
    Line::from(p_line).render(Rect::new(inner_x, dialog.y + 3, inner_width, 1), buf);

    let k_prefix = "API Key:  ";
    let mut k_line = vec![Span::styled(k_prefix, Style::default().fg(theme.gray_bright))];
    let masked_key = "*".repeat(state.api_key_input.len());
    k_line.push(Span::styled(&masked_key, Style::default().fg(theme.text_primary)));
    if state.focus == 1 {
        k_line.push(Span::styled("\u{2588}", Style::default().fg(theme.accent_user)));
    }
    Line::from(k_line).render(Rect::new(inner_x, dialog.y + 5, inner_width, 1), buf);

    let hints = Line::from(vec![
        Span::styled("tab", Style::default().fg(theme.accent_user).add_modifier(Modifier::BOLD)),
        Span::styled(" = next   ", Style::default().fg(theme.gray)),
        Span::styled("enter", Style::default().fg(theme.accent_user).add_modifier(Modifier::BOLD)),
        Span::styled(" = save   ", Style::default().fg(theme.gray)),
        Span::styled("esc", Style::default().fg(theme.accent_user).add_modifier(Modifier::BOLD)),
        Span::styled(" = cancel", Style::default().fg(theme.gray)),
    ]);
    hints.render(Rect::new(inner_x, dialog.y + 7, inner_width, 1), buf);
}
