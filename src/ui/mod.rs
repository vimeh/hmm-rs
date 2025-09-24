mod canvas;
mod connections;
mod constants;
mod help;
mod mindmap;
mod status_line;
pub mod text;

#[cfg(test)]
mod tests;

use crate::app::{AppMode, AppState};
use crate::layout::LayoutEngine;
use help::HelpRenderer;
use mindmap::MindMapRenderer;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};
use status_line::StatusLineRenderer;

// Main render function - the only public API
pub fn render(frame: &mut Frame, app: &mut AppState) {
    // Update terminal size
    let size = frame.area();
    app.terminal_width = size.width;
    app.terminal_height = size.height;

    // Calculate layout
    let layout = LayoutEngine::calculate_layout(app);

    // Create main layout chunks
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(size);

    // Render based on mode
    match &app.mode {
        AppMode::Help => HelpRenderer::render(frame, chunks[0]),
        _ => {
            let renderer = MindMapRenderer::new(app, &layout);
            renderer.render(frame, chunks[0]);
        }
    }

    // Render status line
    StatusLineRenderer::render(frame, app, chunks[1]);
}
