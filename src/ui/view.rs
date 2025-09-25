use crate::app::AppState;
use crate::layout::LayoutEngine;
use crate::model::NodeId;
use ratatui::layout::Rect;
use std::collections::HashMap;

/// Represents a node's final, calculated position on the screen.
#[derive(Debug, Clone)]
pub struct ViewNode {
    /// The final rectangle on the screen where the node should be drawn.
    pub screen_rect: Rect,
    /// The middle Y coordinate of the node's entry point for connections.
    pub connection_y: u16,
}

/// A map from a NodeId to its calculated view properties.
pub type ViewMap = HashMap<NodeId, ViewNode>;

/// Orchestrates the calculation of the view map.
pub struct ViewCalculator<'a> {
    app: &'a AppState,
    layout: &'a LayoutEngine,
    area: Rect,
    view_map: ViewMap,
}

impl<'a> ViewCalculator<'a> {
    /// Calculate the final screen positions for all visible nodes.
    pub fn calculate(app: &'a AppState, layout: &'a LayoutEngine, area: Rect) -> ViewMap {
        let mut calculator = Self {
            app,
            layout,
            area,
            view_map: ViewMap::new(),
        };

        if let Some(root_id) = app.root_id {
            calculator.compute_node_view(root_id, 0.0);
        }

        calculator.view_map
    }

    /// Recursively computes the view for a node and its children.
    fn compute_node_view(&mut self, node_id: NodeId, _parent_y_in_world: f64) {
        let Some(node_ref) = self.app.tree.get(node_id) else {
            return;
        };
        let Some(node_layout) = self.layout.nodes.get(&node_id) else {
            return;
        };
        let node = node_ref.get();

        // Use the absolute position from the layout
        let world_y = node_layout.y + node_layout.yo;

        // Determine offset based on node depth and sibling situation
        let x_offset = if let Some(parent_id) = node_id.ancestors(&self.app.tree).nth(1) {
            // This node has a parent
            let siblings = self.get_visible_children(parent_id);
            if siblings.len() > 1 {
                // Node is part of a multi-child group
                if self.app.root_id == Some(parent_id) {
                    // Direct child of root with siblings needs 1 char for space after junction
                    1.0
                } else {
                    // Grandchild or deeper with siblings needs 2 chars for junction + space
                    2.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };
        let screen_x = (node_layout.x - self.app.viewport_left + x_offset).max(0.0) as u16;
        let ideal_screen_y = (world_y - self.app.viewport_top) as i32;

        let height = node_layout.lh as u16;
        let final_screen_y = self.get_adjusted_y(node_id, ideal_screen_y, height);

        let screen_rect = Rect::new(
            screen_x,
            final_screen_y as u16,
            node_layout.w as u16,
            height,
        );

        // Only add nodes that are plausibly on screen to the map.
        if final_screen_y < self.area.height as i32 && (final_screen_y + height as i32) >= 0 {
            self.view_map.insert(
                node_id,
                ViewNode {
                    screen_rect,
                    connection_y: screen_rect.y + height / 2, // Middle of the node for connections
                },
            );
        }

        // Recurse for children if not collapsed
        if !node.is_collapsed {
            for child_id in self.get_visible_children(node_id) {
                self.compute_node_view(child_id, world_y);
            }
        }
    }

    /// Determines if a node should be "sticky" and adjusts its Y coordinate.
    fn get_adjusted_y(&self, node_id: NodeId, ideal_y: i32, height: u16) -> i32 {
        if ideal_y + (height as i32) <= 0 {
            let node = self.app.tree.get(node_id).unwrap().get();
            if !node.is_collapsed && self.has_visible_children_in_viewport(node_id) {
                return 0; // Stick to top
            }
        }
        ideal_y
    }

    /// Get visible (non-hidden) children of a node
    fn get_visible_children(&self, node_id: NodeId) -> Vec<NodeId> {
        if !self.app.config.show_hidden {
            node_id
                .children(&self.app.tree)
                .filter(|child_id| {
                    if let Some(child_ref) = self.app.tree.get(*child_id) {
                        let child = child_ref.get();
                        !child.is_hidden()
                    } else {
                        false
                    }
                })
                .collect()
        } else {
            node_id.children(&self.app.tree).collect()
        }
    }

    /// Check if a node has visible children in the viewport
    fn has_visible_children_in_viewport(&self, node_id: NodeId) -> bool {
        for child_id in self.get_visible_children(node_id) {
            if let Some(child_layout) = self.layout.nodes.get(&child_id) {
                let child_screen_y =
                    (child_layout.y + child_layout.yo - self.app.viewport_top) as i32;
                if child_screen_y >= 0 && child_screen_y < self.area.height as i32 {
                    return true;
                }
                // Recursively check grandchildren
                if let Some(child_ref) = self.app.tree.get(child_id) {
                    let child = child_ref.get();
                    if !child.is_collapsed && self.has_visible_children_in_viewport(child_id) {
                        return true;
                    }
                }
            }
        }
        false
    }
}
