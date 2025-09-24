use crate::app::AppState;
use crate::layout::LayoutEngine;
use crate::model::NodeId;
use crate::ui::canvas::BufferCanvas;
use crate::ui::constants::{
    connections, junction, MIDDLE_CONNECTOR_Y_OFFSET, NODE_MIDDLE_Y_OFFSET,
    VERTICAL_CONNECTOR_OFFSET,
};
use ratatui::layout::Rect;

// Connection renderer
pub struct ConnectionRenderer<'a> {
    canvas: &'a mut BufferCanvas,
    app: &'a AppState,
    layout: &'a LayoutEngine,
    area: Rect,
}

impl<'a> ConnectionRenderer<'a> {
    pub fn new(
        canvas: &'a mut BufferCanvas,
        app: &'a AppState,
        layout: &'a LayoutEngine,
        area: Rect,
    ) -> Self {
        Self {
            canvas,
            app,
            layout,
            area,
        }
    }

    pub fn draw_node_connections(&mut self, node_id: NodeId) {
        let Some(node_ref) = self.app.tree.get(node_id) else {
            return;
        };
        let node = node_ref.get();

        let Some(node_layout) = self.layout.nodes.get(&node_id) else {
            return;
        };

        // Get children information
        let all_children: Vec<NodeId> = node_id.children(&self.app.tree).collect();
        let visible_children = self.get_visible_children(node_id);
        let has_hidden = all_children.len() != visible_children.len();

        // Calculate node middle Y position
        let node_middle_y = self.calculate_middle_y(node_layout);

        // Handle different cases
        if node.is_collapsed && !all_children.is_empty() {
            self.draw_collapsed_indicator(node_layout, has_hidden);
        } else if visible_children.is_empty() && !all_children.is_empty() {
            self.draw_hidden_only_indicator(node_layout, node_middle_y);
        } else if visible_children.len() == 1 {
            self.draw_single_child_connection(
                node_layout,
                node_middle_y,
                visible_children[0],
                has_hidden,
            );
        } else if visible_children.len() > 1 {
            self.draw_multi_child_connections(
                node_layout,
                node_middle_y,
                &visible_children,
                has_hidden,
            );
        }

        // Recursively draw connections for visible children
        // Only recurse if the node is not collapsed
        if !node.is_collapsed {
            for child_id in visible_children {
                self.draw_node_connections(child_id);
            }
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

    fn calculate_middle_y(&self, node_layout: &crate::layout::LayoutNode) -> i32 {
        (node_layout.y + node_layout.yo + node_layout.lh / 2.0 - NODE_MIDDLE_Y_OFFSET).round()
            as i32
    }

    fn viewport_x(&self, x: f64) -> i32 {
        (x - self.app.viewport_left) as i32
    }

    fn viewport_y(&self, y: f64) -> i32 {
        (y - self.app.viewport_top) as i32
    }

    fn draw_collapsed_indicator(
        &mut self,
        node_layout: &crate::layout::LayoutNode,
        has_hidden: bool,
    ) {
        let x = self.viewport_x(node_layout.x + node_layout.w + 1.0);
        let y = self.viewport_y(node_layout.y + node_layout.yo);

        if self.is_in_bounds(x, y) {
            let text = if has_hidden {
                connections::COLLAPSED_HIDDEN
            } else {
                connections::COLLAPSED
            };
            self.canvas.draw_text(x as usize, y as usize, text);
        }
    }

    fn draw_hidden_only_indicator(
        &mut self,
        node_layout: &crate::layout::LayoutNode,
        middle_y: i32,
    ) {
        let x = self.viewport_x(node_layout.x + node_layout.w + 1.0);
        let y = self.viewport_y(middle_y as f64);

        if self.is_in_bounds(x, y) {
            self.canvas
                .draw_text(x as usize, y as usize, connections::HIDDEN_ONLY);
        }
    }

    fn draw_single_child_connection(
        &mut self,
        node_layout: &crate::layout::LayoutNode,
        parent_middle_y: i32,
        child_id: NodeId,
        has_hidden: bool,
    ) {
        let Some(child_layout) = self.layout.nodes.get(&child_id) else {
            return;
        };

        let child_middle_y = self.calculate_middle_y(child_layout);
        let x = self.viewport_x(node_layout.x + node_layout.w + 1.0);

        // Draw horizontal line
        let line = if has_hidden {
            connections::SINGLE_HIDDEN
        } else {
            connections::SINGLE
        };

        let y = parent_middle_y.min(child_middle_y);
        if self.is_in_bounds(x, self.viewport_y(y as f64)) {
            self.canvas
                .draw_text(x as usize, self.viewport_y(y as f64) as usize, line);
        }

        // Draw vertical connection if needed
        if (parent_middle_y - child_middle_y).abs() > 0 {
            self.draw_vertical_connection(child_layout, parent_middle_y, child_middle_y);
        }
    }

    fn draw_multi_child_connections(
        &mut self,
        node_layout: &crate::layout::LayoutNode,
        middle_y: i32,
        children: &[NodeId],
        has_hidden: bool,
    ) {
        // Find top and bottom children
        let (top_child, top_y, bottom_child, bottom_y) = self.find_extremes(children);

        let Some(top_child_layout) = self.layout.nodes.get(&top_child) else {
            return;
        };

        let x = self.viewport_x(node_layout.x + node_layout.w + 1.0);

        // Draw horizontal line from parent
        let line = if has_hidden {
            connections::MULTI_HIDDEN
        } else {
            connections::MULTI
        };

        let py = self.viewport_y(middle_y as f64);
        if self.is_in_bounds(x, py) {
            self.canvas.draw_text(x as usize, py as usize, line);
        }

        // Draw vertical spine
        let vert_x = self.viewport_x(top_child_layout.x - VERTICAL_CONNECTOR_OFFSET);
        self.draw_vertical_spine(vert_x, top_y, bottom_y);

        // Draw child connectors
        self.draw_child_connectors(vert_x, children, top_child, bottom_child);

        // Fix junction at parent level
        self.fix_junction(vert_x, self.viewport_y(middle_y as f64));
    }

    fn draw_vertical_connection(
        &mut self,
        child_layout: &crate::layout::LayoutNode,
        y1: i32,
        y2: i32,
    ) {
        let vert_x = self.viewport_x(child_layout.x - VERTICAL_CONNECTOR_OFFSET);

        // Draw vertical line
        for y in y1.min(y2)..y1.max(y2) {
            let py = self.viewport_y(y as f64);
            if self.is_in_bounds(vert_x, py) {
                self.canvas
                    .set_char(vert_x as usize, py as usize, junction::VERTICAL);
            }
        }

        // Draw corners
        let py2 = self.viewport_y(y2 as f64);
        if self.is_in_bounds(vert_x, py2) {
            let corner = if y2 > y1 {
                junction::BOTTOM_CORNER
            } else {
                junction::TOP_CORNER
            };
            self.canvas.set_char(vert_x as usize, py2 as usize, corner);
        }

        let py_min = self.viewport_y(y1.min(y2) as f64);
        if self.is_in_bounds(vert_x, py_min) {
            let corner = if y2 > y1 {
                junction::TOP_RIGHT
            } else {
                junction::BOTTOM_RIGHT
            };
            self.canvas
                .set_char(vert_x as usize, py_min as usize, corner);
        }
    }

    fn draw_vertical_spine(&mut self, x: i32, top_y: i32, bottom_y: i32) {
        for y in top_y..bottom_y {
            let py = self.viewport_y(y as f64);
            if self.is_in_bounds(x, py) {
                self.canvas
                    .set_char(x as usize, py as usize, junction::VERTICAL);
            }
        }
    }

    fn draw_child_connectors(
        &mut self,
        vert_x: i32,
        children: &[NodeId],
        top_child: NodeId,
        bottom_child: NodeId,
    ) {
        // Draw top corner
        if let Some(top_layout) = self.layout.nodes.get(&top_child) {
            let top_py = self.viewport_y(top_layout.y + top_layout.yo);
            if self.is_in_bounds(vert_x, top_py) {
                self.canvas
                    .draw_text(vert_x as usize, top_py as usize, "╭──");
            }
        }

        // Draw bottom corner
        if let Some(bottom_layout) = self.layout.nodes.get(&bottom_child) {
            let bot_py = self.viewport_y(bottom_layout.y + bottom_layout.yo);
            if self.is_in_bounds(vert_x, bot_py) {
                self.canvas
                    .draw_text(vert_x as usize, bot_py as usize, "╰──");
            }
        }

        // Draw middle connectors
        for &child_id in children {
            if child_id != top_child && child_id != bottom_child {
                if let Some(child_layout) = self.layout.nodes.get(&child_id) {
                    let cy = (child_layout.y + child_layout.yo + child_layout.lh / 2.0
                        - MIDDLE_CONNECTOR_Y_OFFSET) as i32;
                    let py = self.viewport_y(cy as f64);
                    if self.is_in_bounds(vert_x, py) {
                        self.canvas.draw_text(vert_x as usize, py as usize, "├──");
                    }
                }
            }
        }
    }

    fn fix_junction(&mut self, x: i32, y: i32) {
        if !self.is_in_bounds(x, y) {
            return;
        }

        let existing = self.canvas.char_buffer[y as usize][x as usize];
        let replacement = match existing {
            '│' => junction::MIDDLE_RIGHT,
            '╭' => junction::TOP_TEE,
            '├' => junction::CROSS,
            _ => existing,
        };
        self.canvas.set_char(x as usize, y as usize, replacement);
    }

    fn find_extremes(&self, children: &[NodeId]) -> (NodeId, i32, NodeId, i32) {
        let mut top_y = i32::MAX;
        let mut bottom_y = i32::MIN;
        let mut top_child = children[0];
        let mut bottom_child = children[0];

        for &child_id in children {
            if let Some(child_layout) = self.layout.nodes.get(&child_id) {
                let child_y = (child_layout.y + child_layout.yo) as i32;
                if child_y < top_y {
                    top_y = child_y;
                    top_child = child_id;
                }
                if child_y > bottom_y {
                    bottom_y = child_y;
                    bottom_child = child_id;
                }
            }
        }

        (top_child, top_y, bottom_child, bottom_y)
    }

    fn is_in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.area.width as i32 && y < self.area.height as i32
    }
}
