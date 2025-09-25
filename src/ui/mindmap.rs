use crate::app::AppState;
use crate::layout::LayoutEngine;
use crate::model::NodeId;
use crate::ui::canvas::BufferCanvas;
use crate::ui::connections::ConnectionRenderer;
use crate::ui::text::TextWrapper;
use crate::ui::view::ViewMap;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Paragraph,
};

// Mind map renderer
pub struct MindMapRenderer<'a> {
    app: &'a AppState,
    view_map: &'a ViewMap,
}

impl<'a> MindMapRenderer<'a> {
    pub fn new(app: &'a AppState, _layout: &'a LayoutEngine, view_map: &'a ViewMap) -> Self {
        Self { app, view_map }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut canvas = BufferCanvas::new(area.width as usize, area.height as usize);

        if let Some(root_id) = self.app.root_id {
            // Draw connections first
            let mut conn_renderer =
                ConnectionRenderer::new(&mut canvas, self.app, self.view_map, area);
            conn_renderer.draw_connections_for_node(root_id);

            // Draw nodes on top
            self.draw_all_nodes(&mut canvas);
        }

        // Convert buffer to paragraph and render
        let lines = canvas.to_lines();
        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, area);
    }

    // The recursive draw function is replaced with a simple iteration
    fn draw_all_nodes(&self, canvas: &mut BufferCanvas) {
        // Iterate over the pre-calculated visible nodes
        for (node_id, view_node) in self.view_map {
            let Some(node_ref) = self.app.tree.get(*node_id) else {
                continue;
            };
            let node = node_ref.get();
            let style = self.get_node_style(*node_id, node);

            let x = view_node.screen_rect.x as usize;
            let y = view_node.screen_rect.y as usize;
            let width = view_node.screen_rect.width as usize;

            let lines = TextWrapper::wrap(&node.title, width);
            for (i, line) in lines.iter().enumerate() {
                canvas.draw_styled_text(x, y + i, line, style);
            }
        }
    }

    fn get_node_style(&self, node_id: NodeId, node: &crate::model::Node) -> Style {
        if Some(node_id) == self.app.active_node_id {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if node.title.starts_with(&self.app.config.symbol1) {
            Style::default().fg(Color::Green)
        } else if node.title.starts_with(&self.app.config.symbol2) {
            Style::default().fg(Color::Red)
        } else if node.is_hidden() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        }
    }
}
