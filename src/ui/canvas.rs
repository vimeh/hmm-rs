use crate::ui::constants::{CharBuffer, StyleBuffer};
use ratatui::{
    style::Style,
    text::{Line, Span},
};

// Buffer canvas for drawing characters and styles
pub struct BufferCanvas {
    pub char_buffer: CharBuffer,
    pub style_buffer: StyleBuffer,
    pub width: usize,
    pub height: usize,
}

impl BufferCanvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            char_buffer: vec![vec![' '; width]; height],
            style_buffer: vec![vec![Style::default(); width]; height],
            width,
            height,
        }
    }

    pub fn set_char(&mut self, x: usize, y: usize, ch: char) {
        if self.in_bounds(x, y) {
            self.char_buffer[y][x] = ch;
        }
    }

    pub fn draw_text(&mut self, x: usize, y: usize, text: &str) {
        for (i, ch) in text.chars().enumerate() {
            self.set_char(x + i, y, ch);
        }
    }

    pub fn draw_styled_text(&mut self, x: usize, y: usize, text: &str, style: Style) {
        for (i, ch) in text.chars().enumerate() {
            if self.in_bounds(x + i, y) {
                self.char_buffer[y][x + i] = ch;
                self.style_buffer[y][x + i] = style;
            }
        }
    }

    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        y < self.height && x < self.width
    }

    pub fn to_lines(&self) -> Vec<Line<'_>> {
        let mut lines = Vec::new();

        for (y, row) in self.char_buffer.iter().enumerate() {
            let mut spans = Vec::new();
            let mut current_style = Style::default();
            let mut current_text = String::new();

            for (x, &ch) in row.iter().enumerate() {
                let style = self.style_buffer[y][x];
                if style != current_style {
                    if !current_text.is_empty() {
                        spans.push(Span::styled(current_text.clone(), current_style));
                        current_text.clear();
                    }
                    current_style = style;
                }
                current_text.push(ch);
            }

            if !current_text.is_empty() {
                spans.push(Span::styled(current_text, current_style));
            }

            lines.push(Line::from(spans));
        }

        lines
    }
}
