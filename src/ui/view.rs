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
    /// Amount of text clipped from the left side (in characters).
    pub left_clip: usize,
    /// The original unclipped width of the node text.
    pub original_width: u16,
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
        // Calculate the node's position relative to viewport
        let world_x_with_offset = node_layout.x + x_offset;
        let relative_x = world_x_with_offset - self.app.viewport_left;

        // Calculate how much of the node is clipped on the left
        let left_clip = if relative_x < 0.0 {
            (-relative_x).min(node_layout.w) // Don't clip more than the width
        } else {
            0.0
        };

        // Adjust screen position and visible width
        let screen_x = relative_x.max(0.0) as u16;
        let visible_width = (node_layout.w - left_clip).max(0.0) as u16;

        let ideal_screen_y = (world_y - self.app.viewport_top) as i32;
        let height = node_layout.lh as u16;
        let final_screen_y = self.get_adjusted_y(node_id, ideal_screen_y, height);

        let screen_rect = Rect::new(screen_x, final_screen_y as u16, visible_width, height);

        // Only add nodes that are plausibly on screen to the map.
        if final_screen_y < self.area.height as i32 && (final_screen_y + height as i32) >= 0 {
            self.view_map.insert(
                node_id,
                ViewNode {
                    screen_rect,
                    connection_y: screen_rect.y + height / 2, // Middle of the node for connections
                    left_clip: left_clip as usize,
                    original_width: node_layout.w as u16,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppState;
    use crate::config::AppConfig;
    use crate::layout::{LayoutEngine, LayoutNode};
    use crate::model::Node;
    use ratatui::layout::Rect;

    #[test]
    fn test_viewport_offset_clipping() {
        // Create a simple app with a root node
        let config = AppConfig::default();
        let mut app = AppState::new(config);
        let root = app.tree.new_node(Node::new("Test Node".to_string()));
        app.root_id = Some(root);

        // Create layout engine with test data
        let mut layout = LayoutEngine::new();
        layout.nodes.insert(
            root,
            LayoutNode {
                x: 5.0,
                y: 0.0,
                w: 9.0, // "Test Node" is 9 chars
                h: 1.0,
                lh: 1.0,
                yo: 0.0,
                xo: 0.0,
            },
        );

        // Test 1: No viewport offset - full node visible
        app.viewport_left = 0.0;
        app.viewport_top = 0.0;
        let area = Rect::new(0, 0, 80, 20);
        let view_map = ViewCalculator::calculate(&app, &layout, area);

        let view_node = view_map.get(&root).expect("Root should be in view");
        assert_eq!(view_node.screen_rect.x, 5);
        assert_eq!(view_node.screen_rect.width, 9);
        assert_eq!(view_node.left_clip, 0);
        assert_eq!(view_node.original_width, 9);

        // Test 2: Viewport offset clips left side
        app.viewport_left = 7.0; // Clip first 2 chars
        let view_map = ViewCalculator::calculate(&app, &layout, area);

        let view_node = view_map.get(&root).expect("Root should be in view");
        assert_eq!(view_node.screen_rect.x, 0); // Starts at screen edge
        assert_eq!(view_node.screen_rect.width, 7); // Only 7 chars visible
        assert_eq!(view_node.left_clip, 2); // 2 chars clipped
        assert_eq!(view_node.original_width, 9); // Original width unchanged

        // Test 3: Large viewport offset - entire node clipped
        app.viewport_left = 20.0;
        let view_map = ViewCalculator::calculate(&app, &layout, area);

        let view_node = view_map.get(&root).expect("Root should be in view");
        assert_eq!(view_node.screen_rect.x, 0);
        assert_eq!(view_node.screen_rect.width, 0); // Completely clipped
        assert_eq!(view_node.left_clip, 9); // All chars clipped
        assert_eq!(view_node.original_width, 9);
    }

    #[test]
    fn test_connection_point_with_clipping() {
        // Create app with parent and child nodes
        let config = AppConfig::default();
        let mut app = AppState::new(config);
        let parent = app.tree.new_node(Node::new("Parent Node".to_string()));
        let child = app.tree.new_node(Node::new("Child".to_string()));
        parent.append(child, &mut app.tree);
        app.root_id = Some(parent);

        // Create layout
        let mut layout = LayoutEngine::new();
        layout.nodes.insert(
            parent,
            LayoutNode {
                x: 0.0,
                y: 0.0,
                w: 11.0, // "Parent Node"
                h: 1.0,
                lh: 1.0,
                yo: 0.0,
                xo: 0.0,
            },
        );
        layout.nodes.insert(
            child,
            LayoutNode {
                x: 18.0, // 11 + 7 spacing
                y: 0.0,
                w: 5.0, // "Child"
                h: 1.0,
                lh: 1.0,
                yo: 0.0,
                xo: 0.0,
            },
        );

        // Test with viewport offset
        app.viewport_left = 5.0;
        let area = Rect::new(0, 0, 80, 20);
        let view_map = ViewCalculator::calculate(&app, &layout, area);

        let parent_view = view_map.get(&parent).expect("Parent should be in view");
        // Connection exit point should be at original text end, accounting for clipping
        let connection_x =
            parent_view.screen_rect.x + parent_view.original_width - parent_view.left_clip as u16;
        assert_eq!(connection_x, 6); // screen_x(0) + original_width(11) - left_clip(5)
    }

    #[test]
    fn test_single_visible_child_spacing_with_hidden_sibling() {
        let config = AppConfig::default();
        let mut app = AppState::new(config);

        let root = app.tree.new_node(Node::new("Root".to_string()));
        let parent = app.tree.new_node(Node::new("Parent".to_string()));
        let hidden_child = app.tree.new_node(Node::new("Hidden".to_string()));
        let visible_child = app.tree.new_node(Node::new("Visible".to_string()));

        root.append(parent, &mut app.tree);
        parent.append(hidden_child, &mut app.tree);
        parent.append(visible_child, &mut app.tree);

        app.root_id = Some(root);

        if let Some(node) = app.tree.get_mut(hidden_child) {
            node.get_mut().title = "[HIDDEN] Hidden".to_string();
        }

        app.config.show_hidden = false;

        let layout = LayoutEngine::calculate_layout(&app);
        let area = Rect::new(0, 0, 80, 20);
        let view_map = ViewCalculator::calculate(&app, &layout, area);

        let parent_view = view_map
            .get(&parent)
            .expect("Parent should appear in view map");
        let child_view = view_map
            .get(&visible_child)
            .expect("Child should appear in view map");

        let parent_exit_x =
            parent_view.screen_rect.x + parent_view.original_width - parent_view.left_clip as u16;
        let spacing = child_view.screen_rect.x as i32 - parent_exit_x as i32;

        assert_eq!(
            spacing, 8,
            "Visible child should be positioned with extra spacing when siblings are hidden"
        );
    }

    #[test]
    fn test_spacing_with_hidden_child_in_sample_tree() {
        let config = AppConfig::default();
        let mut app = AppState::new(config);

        let root = app.tree.new_node(Node::new("Mind Map Root".to_string()));
        let features = app.tree.new_node(Node::new("Features".to_string()));
        let hidden_task = app
            .tree
            .new_node(Node::new("[HIDDEN] Secret Task".to_string()));
        let visible_task = app.tree.new_node(Node::new("✗ Failed Task".to_string()));
        let architecture = app.tree.new_node(Node::new("Architecture".to_string()));

        root.append(features, &mut app.tree);
        root.append(architecture, &mut app.tree);
        features.append(hidden_task, &mut app.tree);
        features.append(visible_task, &mut app.tree);

        app.root_id = Some(root);
        app.config.show_hidden = false;

        let layout = LayoutEngine::calculate_layout(&app);
        let area = Rect::new(0, 0, 80, 20);
        let view_map = ViewCalculator::calculate(&app, &layout, area);

        let features_view = view_map.get(&features).expect("Features should be visible");
        let task_view = view_map.get(&visible_task).expect("Task should be visible");

        let parent_exit_x = features_view.screen_rect.x + features_view.original_width
            - features_view.left_clip as u16;
        let spacing = task_view.screen_rect.x as i32 - parent_exit_x as i32;

        assert_eq!(spacing, 7);
    }
}
