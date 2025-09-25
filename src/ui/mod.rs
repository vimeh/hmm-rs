mod canvas;
mod connections;
mod constants;
mod help;
mod mindmap;
mod status_line;
mod view;
pub mod text;

#[cfg(test)]
mod tests;

use crate::app::{AppMode, AppState};
use crate::layout::LayoutEngine;
use help::HelpRenderer;
use mindmap::MindMapRenderer;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};
use status_line::StatusLineRenderer;
use view::ViewCalculator;

// Main render function - the only public API
pub fn render(frame: &mut Frame, app: &mut AppState) {
    // Update terminal size
    let size = frame.area();
    app.terminal_width = size.width;
    app.terminal_height = size.height;

    // Create main layout chunks
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(size);
    let mindmap_area = chunks[0];

    // --- Refactored Rendering Flow ---
    // 1. Calculate ideal layout
    let layout = LayoutEngine::calculate_layout(app);

    // 2. Calculate final screen positions (the new step)
    let view_map = ViewCalculator::calculate(app, &layout, mindmap_area);

    // 3. Render based on mode
    match &app.mode {
        AppMode::Help => HelpRenderer::render(frame, mindmap_area),
        _ => {
            let renderer = MindMapRenderer::new(app, &layout, &view_map);
            renderer.render(frame, mindmap_area);
        }
    }

    // Render status line
    StatusLineRenderer::render(frame, app, chunks[1]);
}
