use crate::app::AppState;
use crate::layout::LayoutEngine;
use crate::model::NodeId;
use crate::ui::canvas::BufferCanvas;
use crate::ui::connections::ConnectionRenderer;
use crate::ui::text::TextWrapper;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Paragraph,
    Frame,
};

// Mind map renderer
pub struct MindMapRenderer<'a> {
    app: &'a AppState,
    layout: &'a LayoutEngine,
}

impl<'a> MindMapRenderer<'a> {
    pub fn new(app: &'a AppState, layout: &'a LayoutEngine) -> Self {
        Self { app, layout }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut canvas = BufferCanvas::new(area.width as usize, area.height as usize);

        // Draw connections first (behind nodes)
        if let Some(root_id) = self.app.root_id {
            let mut conn_renderer =
                ConnectionRenderer::new(&mut canvas, self.app, self.layout, area);
            conn_renderer.draw_node_connections(root_id);
        }

        // Draw nodes on top
        if let Some(root_id) = self.app.root_id {
            self.draw_node_content(&mut canvas, root_id, area);
        }

        // Convert buffer to paragraph and render
        let lines = canvas.to_lines();
        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, area);
    }

    fn draw_node_content(&self, canvas: &mut BufferCanvas, node_id: NodeId, area: Rect) {
        let Some(node_ref) = self.app.tree.get(node_id) else {
            return;
        };
        let node = node_ref.get();

        let Some(node_layout) = self.layout.nodes.get(&node_id) else {
            return;
        };

        let x = (node_layout.x - self.app.viewport_left) as usize;
        let y = (node_layout.y + node_layout.yo - self.app.viewport_top) as usize;

        // Determine node style
        let style = self.get_node_style(node_id, node);

        // Draw node text
        if x < area.width as usize && y < area.height as usize {
            let lines = TextWrapper::wrap(&node.title, node_layout.w as usize);
            for (i, line) in lines.iter().enumerate() {
                let line_y = y + i;
                if line_y < area.height as usize {
                    canvas.draw_styled_text(x, line_y, line, style);
                }
            }
        }

        // Draw children if not collapsed
        if !node.is_collapsed {
            let children = self.get_visible_children(node_id);
            for child_id in children {
                self.draw_node_content(canvas, child_id, area);
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

    fn get_visible_children(&self, node_id: NodeId) -> Vec<NodeId> {
        if !self.app.config.show_hidden {
            node_id
                .children(&self.app.tree)
                .filter(|cid| {
                    self.app
                        .tree
                        .get(*cid)
                        .map(|n| !n.get().is_hidden())
                        .unwrap_or(false)
                })
                .collect()
        } else {
            node_id.children(&self.app.tree).collect()
        }
    }
}
