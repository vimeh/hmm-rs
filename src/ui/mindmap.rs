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

        // Calculate viewport coordinates as signed integers
        let x = (node_layout.x - self.app.viewport_left) as i32;
        let original_y = (node_layout.y + node_layout.yo - self.app.viewport_top) as i32;

        // Adjust Y position for parent nodes with visible children
        let y = self.get_adjusted_parent_y(node_id, original_y, area);

        // Determine node style
        let style = self.get_node_style(node_id, node);

        // Skip drawing if the node AND its children are completely off-screen
        let node_height = TextWrapper::wrap(&node.title, node_layout.w as usize).len() as i32;
        let is_node_visible = y + node_height > 0 && y < area.height as i32;
        let has_visible_children = !node.is_collapsed && self.has_visible_children_in_viewport(node_id, area);

        // Check if node is within viewport bounds
        // Skip nodes that are completely above or to the left of viewport
        if x >= area.width as i32 || (!is_node_visible && !has_visible_children) {
            // Node and its children are completely off-screen
            return; // No need to process children if parent and all children are off-screen
        } else if x >= 0 && y >= 0 && is_node_visible {
            // Node is at least partially visible
            let lines = TextWrapper::wrap(&node.title, node_layout.w as usize);
            let num_lines = lines.len() as i32;

            // Only draw if at least part of the node is visible
            if y + num_lines > 0 {
                for (i, line) in lines.iter().enumerate() {
                    let line_y = y + i as i32;
                    // Only draw lines that are within the viewport
                    if line_y >= 0 && line_y < area.height as i32 {
                        canvas.draw_styled_text(x as usize, line_y as usize, line, style);
                    }
                }
            }
        }
        // If x < 0, the node starts off-screen from the left but might be partially visible
        else if x < 0 && x + node_layout.w as i32 > 0 && y >= 0 && y < area.height as i32 {
            // Node is partially visible from the left
            let lines = TextWrapper::wrap(&node.title, node_layout.w as usize);
            for (i, line) in lines.iter().enumerate() {
                let line_y = y + i as i32;
                if line_y >= 0 && line_y < area.height as i32 {
                    // Calculate how many characters to skip
                    let skip_count = (-x) as usize;
                    // Use character-based skipping, not byte-based
                    let visible_part: String = line.chars().skip(skip_count).collect();
                    if !visible_part.is_empty() {
                        // The visible width is the total width minus what we skipped
                        let visible_width = (node_layout.w as i32 + x).max(visible_part.len() as i32) as usize;
                        // Pad the visible part to ensure it overwrites any connections
                        let padded = format!("{:<width$}", visible_part, width = visible_width);
                        canvas.draw_styled_text(0, line_y as usize, &padded, style);
                    }
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

    /// Check if any children of a node are visible in the viewport
    fn has_visible_children_in_viewport(&self, node_id: NodeId, area: Rect) -> bool {
        let Some(node_ref) = self.app.tree.get(node_id) else {
            return false;
        };
        let node = node_ref.get();

        if node.is_collapsed {
            return false;
        }

        let children = self.get_visible_children(node_id);
        for child_id in children {
            if self.is_node_in_viewport(child_id, area) {
                return true;
            }
            // Recursively check children's children
            if self.has_visible_children_in_viewport(child_id, area) {
                return true;
            }
        }
        false
    }

    /// Check if a node is at least partially visible in the viewport
    fn is_node_in_viewport(&self, node_id: NodeId, area: Rect) -> bool {
        let Some(node_layout) = self.layout.nodes.get(&node_id) else {
            return false;
        };

        let y = (node_layout.y + node_layout.yo - self.app.viewport_top) as i32;
        let node_height = node_layout.lh as i32;

        // Check if node is vertically within viewport
        y + node_height > 0 && y < area.height as i32
    }

    /// Get the adjusted Y position for a parent node based on its visible children
    fn get_adjusted_parent_y(&self, node_id: NodeId, original_y: i32, area: Rect) -> i32 {
        let Some(node_layout) = self.layout.nodes.get(&node_id) else {
            return original_y;
        };

        let node_height = node_layout.lh as i32;

        // Only preserve parent visibility if it's JUST scrolling off the top
        // and has visible children. If it's far above (more than its height),
        // let it disappear completely.
        let threshold = node_height * 2; // Only preserve if within 2x height of viewport top

        if original_y < 0 && original_y > -threshold && original_y + node_height <= 0 {
            let Some(node_ref) = self.app.tree.get(node_id) else {
                return original_y;
            };
            let node = node_ref.get();

            if !node.is_collapsed && self.has_visible_children_in_viewport(node_id, area) {
                // Keep the parent at the top of the viewport
                // Position it so its bottom line is just visible
                return 1 - node_height;
            }
        }
        original_y
    }
}
