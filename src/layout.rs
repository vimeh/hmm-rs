use crate::app::AppState;
use crate::model::NodeId;
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

const WIDTH_TOLERANCE: f32 = 1.3;
const LEFT_PADDING: usize = 1;
/// Space allocated for connection lines between parent and child nodes
pub const NODE_CONNECTION_SPACING: f64 = 6.0;

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub lh: f64, // Line height
    pub yo: f64, // Y offset for centering
    pub xo: f64, // X offset for unicode width compensation
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
            // First pass: calculate widths and line heights
            engine.calculate_x_and_lh(app, root_id, 0.0);

            // Second pass: calculate heights
            engine.calculate_h(app, root_id);

            // Third pass: calculate y positions
            engine.calculate_y(app, root_id, 0.0);

            // Fourth pass: calculate x offsets for unicode width
            engine.calculate_xo(app);
        }

        engine
    }

    fn calculate_x_and_lh(&mut self, app: &AppState, node_id: NodeId, parent_x: f64) {
        let node = app.tree.get(node_id).unwrap().get();

        // Calculate x position
        let x = if node_id == app.root_id.unwrap() {
            LEFT_PADDING as f64
        } else {
            parent_x
                + self
                    .nodes
                    .get(&node_id.ancestors(&app.tree).nth(1).unwrap())
                    .map(|p| p.w)
                    .unwrap_or(0.0)
                + NODE_CONNECTION_SPACING
        };

        // Determine if this is a leaf or collapsed node
        let children: Vec<NodeId> = if !app.config.show_hidden {
            node_id
                .children(&app.tree)
                .filter(|cid| {
                    let child = app.tree.get(*cid).unwrap().get();
                    !child.is_hidden()
                })
                .collect()
        } else {
            node_id.children(&app.tree).collect()
        };

        let at_the_end = children.is_empty() || node.is_collapsed;

        // Get max width for this node type
        let max_width = if at_the_end {
            app.config.max_leaf_node_width
        } else {
            app.config.max_parent_node_width
        };

        // Calculate width and line height
        let title_width = node.title.width();
        let (w, lh) = if title_width as f32 > WIDTH_TOLERANCE * max_width as f32 {
            // Need to wrap text
            let lines = wrap_text(&node.title, max_width);
            let max_line_width = lines.iter().map(|l| l.width()).max().unwrap_or(0);
            (max_line_width as f64, lines.len() as f64)
        } else {
            (title_width as f64, 1.0)
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
                xo: 0.0, // Will be calculated later
            },
        );

        // Update map width
        self.map_width = self.map_width.max(x + w);

        // Recurse for children
        for child_id in children {
            self.calculate_x_and_lh(app, child_id, x);
        }
    }

    fn calculate_h(&mut self, app: &AppState, node_id: NodeId) -> f64 {
        let node = app.tree.get(node_id).unwrap().get();

        let children: Vec<NodeId> = if !app.config.show_hidden {
            node_id
                .children(&app.tree)
                .filter(|cid| {
                    let child = app.tree.get(*cid).unwrap().get();
                    !child.is_hidden()
                })
                .collect()
        } else {
            node_id.children(&app.tree).collect()
        };

        let at_the_end = children.is_empty() || node.is_collapsed;

        let h = if at_the_end {
            // Leaf node: height is line height plus spacing
            let layout = self.nodes.get(&node_id).unwrap();
            app.config.line_spacing as f64 + layout.lh
        } else {
            // Parent node: height is sum of children or own line height
            let mut children_height = 0.0;
            for child_id in &children {
                children_height += self.calculate_h(app, *child_id);
            }

            let layout = self.nodes.get(&node_id).unwrap();
            children_height.max(layout.lh + app.config.line_spacing as f64)
        };

        // Update the layout node with calculated height
        if let Some(layout) = self.nodes.get_mut(&node_id) {
            layout.h = h;
        }

        h
    }

    fn calculate_y(&mut self, app: &AppState, node_id: NodeId, current_y: f64) {
        let node = app.tree.get(node_id).unwrap().get();

        // Set this node's y position
        if let Some(layout) = self.nodes.get_mut(&node_id) {
            layout.y = current_y;

            // Calculate y offset for vertical centering
            layout.yo = ((layout.h - layout.lh) / 2.0).round();
        }

        // Update map boundaries
        let layout = self.nodes.get(&node_id).unwrap();
        self.map_bottom = self
            .map_bottom
            .max(current_y + layout.lh + app.config.line_spacing as f64);
        self.map_top = self.map_top.min(current_y);

        // Process children
        if !node.is_collapsed {
            let children: Vec<NodeId> = if !app.config.show_hidden {
                node_id
                    .children(&app.tree)
                    .filter(|cid| {
                        let child = app.tree.get(*cid).unwrap().get();
                        !child.is_hidden()
                    })
                    .collect()
            } else {
                node_id.children(&app.tree).collect()
            };

            let mut child_y = current_y;
            for child_id in children {
                self.calculate_y(app, child_id, child_y);
                let child_layout = self.nodes.get(&child_id).unwrap();
                child_y += child_layout.h;
            }
        }

        self.map_height = self.map_bottom - self.map_top;
    }

    fn calculate_xo(&mut self, app: &AppState) {
        // Calculate x offset to compensate for unicode width differences
        for (node_id, layout) in self.nodes.iter_mut() {
            let node = app.tree.get(*node_id).unwrap().get();
            let title_len = node.title.len();
            let title_width = node.title.width();
            layout.xo = (title_len - title_width) as f64;
        }
    }

    pub fn get_visible_nodes(&self, viewport: (f64, f64, f64, f64)) -> Vec<NodeId> {
        let (vp_left, vp_top, vp_right, vp_bottom) = viewport;

        self.nodes
            .iter()
            .filter_map(|(id, layout)| {
                if layout.x + layout.w >= vp_left
                    && layout.x <= vp_right
                    && layout.y + layout.lh >= vp_top
                    && layout.y <= vp_bottom
                {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_width = word.width();

        if current_width > 0 && current_width + 1 + word_width > max_width {
            // Need to start a new line
            lines.push(current_line);
            current_line = word.to_string();
            current_width = word_width;
        } else {
            // Add to current line
            if !current_line.is_empty() {
                current_line.push(' ');
                current_width += 1;
            }
            current_line.push_str(word);
            current_width += word_width;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(text.to_string());
    }

    lines
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
        let root_layout = layout.nodes.get(&app.root_id.unwrap()).unwrap();
        assert_eq!(root_layout.x, LEFT_PADDING as f64);

        // Map dimensions should be positive
        assert!(layout.map_width > 0.0);
        assert!(layout.map_height >= 0.0);
    }

    #[test]
    fn test_calculate_layout_with_collapsed_node() {
        let mut app = create_test_app();

        // Collapse child2
        let child2_id = app.root_id.unwrap().children(&app.tree).nth(1).unwrap();
        app.tree.get_mut(child2_id).unwrap().get_mut().is_collapsed = true;

        let layout = LayoutEngine::calculate_layout(&app);

        // Should still have layout for all nodes
        assert_eq!(layout.nodes.len(), 4);
    }

    #[test]
    fn test_wrap_text() {
        let text = "This is a very long line that should be wrapped";
        let lines = wrap_text(text, 15);

        assert!(lines.len() > 1);
        for line in &lines {
            assert!(line.len() <= 15);
        }
    }

    #[test]
    fn test_wrap_text_single_word() {
        let text = "SingleWord";
        let lines = wrap_text(text, 20);

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "SingleWord");
    }

    #[test]
    fn test_wrap_text_unicode() {
        let text = "这是一段中文文本 with mixed 内容";
        let lines = wrap_text(text, 20);

        assert!(!lines.is_empty());
    }

    #[test]
    fn test_layout_with_hidden_nodes() {
        let mut app = create_test_app();

        // Mark a child as hidden
        let child1_id = app.root_id.unwrap().children(&app.tree).next().unwrap();
        app.tree.get_mut(child1_id).unwrap().get_mut().title = "[HIDDEN] Child 1".to_string();

        // Hide hidden nodes
        app.config.show_hidden = false;

        let layout = LayoutEngine::calculate_layout(&app);

        // When show_hidden is false, hidden nodes are filtered out during layout calculation
        // So we check that the layout was calculated (has nodes) but the hidden node might not be included
        assert!(!layout.nodes.is_empty());
        assert!(layout.nodes.contains_key(&app.root_id.unwrap()));
    }

    #[test]
    fn test_node_spacing_consistency() {
        let app = create_test_app();
        let layout = LayoutEngine::calculate_layout(&app);

        // Get root and its first child
        let root_id = app.root_id.unwrap();
        let child1_id = root_id.children(&app.tree).next().unwrap();

        let root_layout = layout.nodes.get(&root_id).unwrap();
        let child_layout = layout.nodes.get(&child1_id).unwrap();

        // Child should be positioned at parent_x + parent_width + NODE_CONNECTION_SPACING
        let expected_child_x = root_layout.x + root_layout.w + NODE_CONNECTION_SPACING;
        assert_eq!(
            child_layout.x, expected_child_x,
            "Child node should be positioned with {} units spacing from parent",
            NODE_CONNECTION_SPACING
        );
    }

    #[test]
    fn test_multiple_children_spacing() {
        let app = create_test_app();
        let layout = LayoutEngine::calculate_layout(&app);

        // Get root's children
        let root_id = app.root_id.unwrap();
        let children: Vec<_> = root_id.children(&app.tree).collect();
        assert!(children.len() >= 2, "Test requires at least 2 children");

        let child1_layout = layout.nodes.get(&children[0]).unwrap();
        let child2_layout = layout.nodes.get(&children[1]).unwrap();

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

        let root_layout = layout.nodes.get(&root).unwrap();
        let child_layout = layout.nodes.get(&child).unwrap();
        let grandchild_layout = layout.nodes.get(&grandchild).unwrap();

        // Check consistent spacing at each level
        let spacing1 = child_layout.x - (root_layout.x + root_layout.w);
        let spacing2 = grandchild_layout.x - (child_layout.x + child_layout.w);

        assert_eq!(
            spacing1, NODE_CONNECTION_SPACING,
            "Spacing between root and child should be {} units",
            NODE_CONNECTION_SPACING
        );
        assert_eq!(
            spacing2, NODE_CONNECTION_SPACING,
            "Spacing between child and grandchild should be {} units",
            NODE_CONNECTION_SPACING
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
                xo: 0.0,
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
                xo: 0.0,
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
}
