use crate::app::{AppMode, AppState};
use crate::ui::constants::{CURSOR_INDICATOR, STATUS_EDIT_PREFIX, STATUS_SEARCH_PREFIX};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Paragraph, Wrap},
    Frame,
};

// Status line renderer
pub struct StatusLineRenderer;

impl StatusLineRenderer {
    pub fn render(frame: &mut Frame, app: &AppState, area: Rect) {
        let (content, style) = Self::get_content_and_style(app, area);

        let paragraph = Paragraph::new(content)
            .style(style)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    fn get_content_and_style(app: &AppState, area: Rect) -> (String, Style) {
        match &app.mode {
            AppMode::Normal => Self::render_normal_mode(app),
            AppMode::Editing { buffer, cursor_pos } => {
                Self::render_edit_mode(buffer, *cursor_pos, area.width)
            }
            AppMode::Search { query } => Self::render_search_mode(query),
            AppMode::Help => Self::render_help_mode(),
        }
    }

    fn render_normal_mode(app: &AppState) -> (String, Style) {
        let content = if let Some(ref msg) = app.message {
            msg.clone()
        } else {
            format!("h-m-m | {} nodes", app.tree.count())
        };

        let style = if app.message.is_some() {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Magenta)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray).bg(Color::Black)
        };

        (content, style)
    }

    fn render_edit_mode(buffer: &str, cursor_pos: usize, width: u16) -> (String, Style) {
        let mut display = String::from(STATUS_EDIT_PREFIX);

        // Calculate visible portion if text is too long
        let available_width = width.saturating_sub(STATUS_EDIT_PREFIX.len() as u16 + 1) as usize;
        let text_start = if cursor_pos > available_width.saturating_sub(10) {
            cursor_pos.saturating_sub(available_width / 2)
        } else {
            0
        };

        let visible_buffer = if buffer.len() > available_width {
            let end = (text_start + available_width).min(buffer.len());
            &buffer[text_start..end]
        } else {
            buffer
        };

        // Adjust cursor position for visible portion
        let visible_cursor = cursor_pos.saturating_sub(text_start);

        // Insert cursor indicator
        if visible_cursor <= visible_buffer.len() {
            display.push_str(&visible_buffer[..visible_cursor]);
            display.push(CURSOR_INDICATOR);
            display.push_str(&visible_buffer[visible_cursor..]);
        } else {
            display.push_str(visible_buffer);
            display.push(CURSOR_INDICATOR);
        }

        let style = Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        (display, style)
    }

    fn render_search_mode(query: &str) -> (String, Style) {
        let content = format!("{}{}", STATUS_SEARCH_PREFIX, query);
        let style = Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        (content, style)
    }

    fn render_help_mode() -> (String, Style) {
        let content = String::from("Press ESC or q to close help");
        let style = Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        (content, style)
    }
}
