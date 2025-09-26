use crate::app::AppState;
use crate::model::NodeId;
use crate::ui::text::TextWrapper;
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

/// Ratio threshold for when text should wrap (1.3 = 130% of max width)
const WRAP_THRESHOLD_RATIO: f32 = 1.3;

/// Left padding for the root node
const LEFT_PADDING: usize = 1;

/// Space allocated for connection lines between parent and child nodes
pub const NODE_CONNECTION_SPACING: f64 = 6.0;

#[derive(Debug, Clone)]
pub struct LayoutNode {
    // Position
    pub x: f64,
    pub y: f64,
    // Dimensions
    pub w: f64,  // Width
    pub h: f64,  // Height
    pub lh: f64, // Line height (number of text lines)
    // Offsets
    pub yo: f64, // Y offset for vertical centering

    // Explicit Connection Geometry
    /// Point where a parent's connection line enters this node
    pub entry_point: (f64, f64),
    /// Point where this node's connection line to its children exits
    pub exit_point: (f64, f64),
    /// The X-coordinate of the vertical spine for this node's children
    pub child_spine_x: Option<f64>,
}

pub struct LayoutEngine {
    pub nodes: HashMap<NodeId, LayoutNode>,
    pub map_width: f64,
    pub map_height: f64,
    pub map_top: f64,
    pub map_bottom: f64,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            map_width: 0.0,
            map_height: 0.0,
            map_top: 0.0,
            map_bottom: 0.0,
        }
    }

    pub fn calculate_layout(app: &AppState) -> Self {
        let mut engine = Self::new();

        if let Some(root_id) = app.root_id {
            // First pass: calculate positions and connection points
            engine.calculate_positions(app, root_id, LEFT_PADDING as f64);

            // Second pass: calculate heights and y coordinates
            let total_height = engine.calculate_heights_and_y_coords(app, root_id, 0.0);

            // Update map dimensions
            engine.map_height = total_height;
            engine.map_bottom = total_height;
        }

        engine
    }

    /// Get children of a node that should be displayed (respecting hidden nodes)
    fn get_filtered_children(app: &AppState, node_id: NodeId) -> Vec<NodeId> {
        node_id
            .children(&app.tree)
            .filter(|child_id| {
                if !app.config.show_hidden {
                    app.tree
                        .get(*child_id)
                        .map(|n| !n.get().is_hidden())
                        .unwrap_or(false)
                } else {
                    true
                }
            })
            .collect()
    }

    /// Check if a node should be treated as a leaf (collapsed or no children)
    fn is_leaf_like(app: &AppState, node_id: NodeId, children: &[NodeId]) -> bool {
        let node = match app.tree.get(node_id) {
            Some(n) => n.get(),
            None => return true,
        };

        children.is_empty() || node.is_collapsed
    }

    /// Calculate all X-related geometry and connection points
    fn calculate_positions(&mut self, app: &AppState, node_id: NodeId, entry_x: f64) {
        let node = match app.tree.get(node_id) {
            Some(n) => n.get(),
            None => return,
        };

        let children = Self::get_filtered_children(app, node_id);
        let at_the_end = Self::is_leaf_like(app, node_id, &children);

        // Get max width for this node type
        let max_width = if at_the_end {
            app.config.max_leaf_node_width
        } else {
            app.config.max_parent_node_width
        };

        // Calculate width and line height
        let title_width = node.title.width();
        let (w, lh) = if title_width as f32 > WRAP_THRESHOLD_RATIO * max_width as f32 {
            let lines = TextWrapper::wrap(&node.title, max_width);
            let max_line_width = lines.iter().map(|l| l.width()).max().unwrap_or(0);
            (max_line_width as f64, lines.len() as f64)
        } else {
            (title_width as f64, 1.0)
        };

        // Calculate geometry with predictable positions
        let x = entry_x;
        let entry_point = (x, 0.0); // Y will be calculated in next pass
        let exit_point = (x + w, 0.0);

        // Determine spine position for children
        let child_spine_x = if at_the_end || node.is_collapsed || children.len() == 1 {
            None // No spine needed for leaf nodes, collapsed nodes, or single children
        } else {
            Some(exit_point.0 + 4.0) // Predictable spine position for multiple children
        };

        // Store the layout node
        self.nodes.insert(
            node_id,
            LayoutNode {
                x,
                y: 0.0, // Will be calculated later
                w,
                h: 0.0, // Will be calculated later
                lh,
                yo: 0.0, // Will be calculated later
                entry_point,
                exit_point,
                child_spine_x,
            },
        );

        // Update map width
        self.map_width = self.map_width.max(x + w);

        // Recurse for children only if node is not collapsed
        if !node.is_collapsed {
            // Calculate child entry X based on whether there's a spine
            let child_entry_x = if let Some(spine_x) = child_spine_x {
                spine_x + 3.0 // Children start 3 units after spine
            } else {
                exit_point.0 + 7.0 // Single child or leaf gets standard spacing
            };

            for child_id in children {
                self.calculate_positions(app, child_id, child_entry_x);
            }
        }
    }

    /// Calculate heights and Y coordinates using post-order traversal
    fn calculate_heights_and_y_coords(
        &mut self,
        app: &AppState,
        node_id: NodeId,
        start_y: f64,
    ) -> f64 {
        let node = match app.tree.get(node_id) {
            Some(n) => n.get(),
            None => return 0.0,
        };

        let children = Self::get_filtered_children(app, node_id);
        let own_lh = self.nodes.get(&node_id).map(|n| n.lh).unwrap_or(1.0);
        let line_spacing = app.config.line_spacing as f64;

        // Post-order traversal: Process children first to get their total height
        let mut children_total_height = 0.0;
        if !node.is_collapsed && !children.is_empty() {
            for child_id in &children {
                let child_height = self.calculate_heights_and_y_coords(
                    app,
                    *child_id,
                    start_y + children_total_height,
                );
                children_total_height += child_height;
            }
        }

        // Calculate this node's height based on children or its own line height
        let h = if children_total_height > 0.0 {
            children_total_height.max(own_lh + line_spacing)
        } else {
            own_lh + line_spacing
        };

        let yo = ((h - own_lh) / 2.0).round();

        // Update this node with its final geometry
        if let Some(layout) = self.nodes.get_mut(&node_id) {
            layout.y = start_y;
            layout.h = h;
            layout.yo = yo;
            layout.entry_point.1 = start_y + yo;
            layout.exit_point.1 = start_y + yo;
        }

        // Return this node's height for parent's calculation
        h
    }

    pub fn get_visible_nodes(&self, viewport: (f64, f64, f64, f64)) -> Vec<NodeId> {
        let (vp_left, vp_top, vp_right, vp_bottom) = viewport;

        self.nodes
            .iter()
            .filter_map(|(id, layout)| {
                let is_visible = layout.x + layout.w >= vp_left
                    && layout.x <= vp_right
                    && layout.y + layout.lh >= vp_top
                    && layout.y <= vp_bottom;

                is_visible.then_some(*id)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppState;
    use crate::config::AppConfig;
    use crate::model::Node;
    use indextree::Arena;

    fn create_test_app() -> AppState {
        let config = AppConfig::default();
        let mut app = AppState::new(config);

        // Create a simple tree
        let root = app.tree.new_node(Node::new("Root".to_string()));
        let child1 = app.tree.new_node(Node::new("Child 1".to_string()));
        let child2 = app.tree.new_node(Node::new("Child 2".to_string()));
        let grandchild = app.tree.new_node(Node::new("Grandchild".to_string()));

        root.append(child1, &mut app.tree);
        root.append(child2, &mut app.tree);
        child2.append(grandchild, &mut app.tree);

        app.root_id = Some(root);
        app.active_node_id = Some(root);

        app
    }

    #[test]
    fn test_layout_engine_creation() {
        let engine = LayoutEngine::new();
        assert_eq!(engine.map_width, 0.0);
        assert_eq!(engine.map_height, 0.0);
        assert_eq!(engine.map_top, 0.0);
        assert_eq!(engine.map_bottom, 0.0);
        assert!(engine.nodes.is_empty());
    }

    #[test]
    fn test_calculate_layout_simple_tree() {
        let app = create_test_app();
        let layout = LayoutEngine::calculate_layout(&app);

        // Should have layout for all nodes
        assert_eq!(layout.nodes.len(), 4);

        // Root should be at leftmost position
        if let Some(root_id) = app.root_id {
            let root_layout = layout
                .nodes
                .get(&root_id)
                .expect("Root node should have a layout");
            assert_eq!(root_layout.x, LEFT_PADDING as f64);
        }

        // Map dimensions should be positive
        assert!(layout.map_width > 0.0);
        assert!(layout.map_height >= 0.0);
    }

    #[test]
    fn test_calculate_layout_with_collapsed_node() {
        let mut app = create_test_app();

        // Collapse child2
        if let Some(root_id) = app.root_id
            && let Some(child2_id) = root_id.children(&app.tree).nth(1)
            && let Some(node) = app.tree.get_mut(child2_id)
        {
            node.get_mut().is_collapsed = true;
        }

        let layout = LayoutEngine::calculate_layout(&app);

        // Should have layout for root, child1, and child2 (but not grandchild of collapsed child2)
        assert_eq!(layout.nodes.len(), 3);
    }

    #[test]
    fn test_text_wrapping_via_textwrapper() {
        // Test that TextWrapper works correctly for our use case
        let text = "This is a very long line that should be wrapped";
        let lines = TextWrapper::wrap(text, 15);

        assert!(lines.len() > 1);
        for line in &lines {
            // Note: TextWrapper uses unicode width, not byte length
            assert!(unicode_width::UnicodeWidthStr::width(line.as_str()) <= 15);
        }
    }

    #[test]
    fn test_layout_with_hidden_nodes() {
        let mut app = create_test_app();

        // Mark a child as hidden
        if let Some(root_id) = app.root_id
            && let Some(child1_id) = root_id.children(&app.tree).next()
            && let Some(node) = app.tree.get_mut(child1_id)
        {
            node.get_mut().title = "[HIDDEN] Child 1".to_string();
        }

        // Hide hidden nodes
        app.config.show_hidden = false;

        let layout = LayoutEngine::calculate_layout(&app);

        // When show_hidden is false, hidden nodes are filtered out during layout calculation
        // So we check that the layout was calculated (has nodes) but the hidden node might not be included
        assert!(!layout.nodes.is_empty());
        if let Some(root_id) = app.root_id {
            assert!(layout.nodes.contains_key(&root_id));
        }
    }

    #[test]
    fn test_node_spacing_consistency() {
        let app = create_test_app();
        let layout = LayoutEngine::calculate_layout(&app);

        // Get root and its first child
        let root_id = app.root_id.expect("Test app should have a root");
        let child1_id = root_id
            .children(&app.tree)
            .next()
            .expect("Root should have at least one child");

        let root_layout = layout
            .nodes
            .get(&root_id)
            .expect("Root should have a layout");
        let child_layout = layout
            .nodes
            .get(&child1_id)
            .expect("Child should have a layout");

        // With the new simplified geometry, children position is based on spine or direct spacing
        // Multiple children get spine + 3, single child gets exit + 7
        // This test has multiple children, so spine position is exit + 4, child is spine + 3
        let expected_child_x = root_layout.exit_point.0 + 4.0 + 3.0;
        assert_eq!(
            child_layout.x, expected_child_x,
            "Child node should be positioned based on spine geometry"
        );
    }

    #[test]
    fn test_multiple_children_spacing() {
        let app = create_test_app();
        let layout = LayoutEngine::calculate_layout(&app);

        // Get root's children
        let root_id = app.root_id.expect("Test app should have a root");
        let children: Vec<_> = root_id.children(&app.tree).collect();
        assert!(children.len() >= 2, "Test requires at least 2 children");

        let child1_layout = layout
            .nodes
            .get(&children[0])
            .expect("First child should have a layout");
        let child2_layout = layout
            .nodes
            .get(&children[1])
            .expect("Second child should have a layout");

        // Both children should have the same x position
        assert_eq!(
            child1_layout.x, child2_layout.x,
            "Sibling nodes should be aligned at the same x position"
        );
    }

    #[test]
    fn test_deep_tree_spacing() {
        let config = AppConfig::default();
        let mut app = AppState::new(config);

        // Create a linear chain
        let root = app.tree.new_node(Node::new("Root".to_string()));
        let child = app.tree.new_node(Node::new("Child".to_string()));
        let grandchild = app.tree.new_node(Node::new("Grandchild".to_string()));

        root.append(child, &mut app.tree);
        child.append(grandchild, &mut app.tree);

        app.root_id = Some(root);

        let layout = LayoutEngine::calculate_layout(&app);

        let root_layout = layout.nodes.get(&root).expect("Root should have a layout");
        let child_layout = layout
            .nodes
            .get(&child)
            .expect("Child should have a layout");
        let grandchild_layout = layout
            .nodes
            .get(&grandchild)
            .expect("Grandchild should have a layout");

        // Check consistent spacing at each level
        let spacing1 = child_layout.x - (root_layout.x + root_layout.w);
        let spacing2 = grandchild_layout.x - (child_layout.x + child_layout.w);

        // Single-child chains now have extra spacing (7 instead of 6)
        let expected_spacing = 7.0;
        assert_eq!(
            spacing1, expected_spacing,
            "Spacing between root and child should be {} units",
            expected_spacing
        );
        assert_eq!(
            spacing2, expected_spacing,
            "Spacing between child and grandchild should be {} units",
            expected_spacing
        );
    }

    #[test]
    fn test_single_child_spacing_with_hidden_sibling() {
        let mut app = create_test_app();

        // Hide the first child so only the second remains visible
        let root_id = app.root_id.expect("Test app should have a root");
        let mut children_iter = root_id.children(&app.tree);
        let hidden_child = children_iter
            .next()
            .expect("Root should have a first child");
        let visible_child = children_iter
            .next()
            .expect("Root should have a second child");

        if let Some(node) = app.tree.get_mut(hidden_child) {
            node.get_mut().title = "[HIDDEN] Child 1".to_string();
        }

        app.config.show_hidden = false;

        let layout = LayoutEngine::calculate_layout(&app);

        let root_layout = layout
            .nodes
            .get(&root_id)
            .expect("Root should have a layout entry");
        let visible_child_layout = layout
            .nodes
            .get(&visible_child)
            .expect("Visible child should have a layout entry");

        // With new geometry, single child gets exit + 7 spacing
        let expected_x = root_layout.exit_point.0 + 7.0;
        assert_eq!(
            visible_child_layout.x, expected_x,
            "Single visible child should be positioned with standard single-child spacing"
        );
    }

    #[test]
    fn test_grandchild_spacing_with_hidden_sibling() {
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

        let parent_layout = layout
            .nodes
            .get(&parent)
            .expect("Parent should have a layout entry");
        let visible_child_layout = layout
            .nodes
            .get(&visible_child)
            .expect("Visible child should have a layout entry");

        // Single child gets exit + 7 spacing in the new system
        let expected_x = parent_layout.exit_point.0 + 7.0;
        assert_eq!(
            visible_child_layout.x, expected_x,
            "Nested single visible child should be positioned with standard single-child spacing"
        );
    }

    #[test]
    fn test_get_visible_nodes() {
        let mut engine = LayoutEngine::new();

        // Create nodes from a shared arena
        let mut arena = Arena::<Node>::new();
        let node1 = arena.new_node(Node::new("test1".to_string()));
        let node2 = arena.new_node(Node::new("test2".to_string()));

        engine.nodes.insert(
            node1,
            LayoutNode {
                x: 10.0,
                y: 10.0,
                w: 20.0,
                h: 10.0,
                lh: 1.0,
                yo: 0.0,
                entry_point: (10.0, 10.0),
                exit_point: (30.0, 10.0),
                child_spine_x: None,
            },
        );

        engine.nodes.insert(
            node2,
            LayoutNode {
                x: 50.0,
                y: 50.0,
                w: 20.0,
                h: 10.0,
                lh: 1.0,
                yo: 0.0,
                entry_point: (50.0, 50.0),
                exit_point: (70.0, 50.0),
                child_spine_x: None,
            },
        );

        // Test viewport that includes first node only
        let viewport = (0.0, 0.0, 40.0, 40.0);
        let visible = engine.get_visible_nodes(viewport);
        assert_eq!(visible.len(), 1);

        // Test viewport that includes both nodes
        let viewport = (0.0, 0.0, 100.0, 100.0);
        let visible = engine.get_visible_nodes(viewport);
        assert_eq!(visible.len(), 2);
    }

    fn create_test_app_with_tree() -> AppState {
        let config = AppConfig::default();
        let mut app = AppState::new(config);

        // Create a tree structure
        let root = app.tree.new_node(Node::new("Root".to_string()));
        let child1 = app.tree.new_node(Node::new("Child 1".to_string()));
        let child2 = app.tree.new_node(Node::new("Child 2".to_string()));
        let grandchild1 = app.tree.new_node(Node::new("Grandchild 1".to_string()));
        let grandchild2 = app.tree.new_node(Node::new("Grandchild 2".to_string()));

        root.append(child1, &mut app.tree);
        root.append(child2, &mut app.tree);
        child1.append(grandchild1, &mut app.tree);
        child2.append(grandchild2, &mut app.tree);

        app.root_id = Some(root);
        app.active_node_id = Some(root);

        app
    }

    #[test]
    fn test_collapsed_nodes_not_in_layout() {
        let mut app = create_test_app_with_tree();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();

        // Collapse child1
        if let Some(node) = app.tree.get_mut(child1) {
            node.get_mut().is_collapsed = true;
        }

        // Calculate layout
        let layout = LayoutEngine::calculate_layout(&app);

        // Root and both children should be in layout
        assert!(layout.nodes.contains_key(&root));
        assert!(layout.nodes.contains_key(&child1));

        let child2 = root.children(&app.tree).nth(1).unwrap();
        assert!(layout.nodes.contains_key(&child2));

        // Grandchildren of collapsed node should NOT be in layout
        let grandchild1 = child1.children(&app.tree).next().unwrap();
        assert!(!layout.nodes.contains_key(&grandchild1));

        // Grandchild of non-collapsed node SHOULD be in layout
        let grandchild2 = child2.children(&app.tree).next().unwrap();
        assert!(layout.nodes.contains_key(&grandchild2));
    }

    #[test]
    fn test_collapsed_node_height() {
        let mut app = create_test_app_with_tree();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();

        // Get layout with expanded node
        let layout_expanded = LayoutEngine::calculate_layout(&app);
        let root_expanded_height = layout_expanded.nodes.get(&root).unwrap().h;

        // Collapse child1
        if let Some(node) = app.tree.get_mut(child1) {
            node.get_mut().is_collapsed = true;
        }

        // Get layout with collapsed node
        let layout_collapsed = LayoutEngine::calculate_layout(&app);
        let root_collapsed_height = layout_collapsed.nodes.get(&root).unwrap().h;

        // Root should have smaller height when child is collapsed (fewer total descendants)
        assert!(root_collapsed_height <= root_expanded_height);

        // Child1 itself should have the same or smaller height when collapsed
        let child1_collapsed_h = layout_collapsed.nodes.get(&child1).unwrap().h;
        let child1_expanded_h = layout_expanded.nodes.get(&child1).unwrap().h;
        assert!(child1_collapsed_h <= child1_expanded_h);
    }

    #[test]
    fn test_all_collapsed_layout() {
        let mut app = create_test_app_with_tree();

        // Collapse all non-root nodes
        for node in app.tree.iter_mut() {
            if node.get().title != "Root" {
                node.get_mut().is_collapsed = true;
            }
        }

        // Calculate layout
        let layout = LayoutEngine::calculate_layout(&app);

        // Only root and its direct children should be in layout
        let root = app.root_id.unwrap();
        assert!(layout.nodes.contains_key(&root));

        // Direct children should be in layout (even if collapsed)
        for child_id in root.children(&app.tree) {
            assert!(layout.nodes.contains_key(&child_id));

            // But their children should NOT be
            for grandchild_id in child_id.children(&app.tree) {
                assert!(!layout.nodes.contains_key(&grandchild_id));
            }
        }
    }
}
