use crate::app::AppState;
use crate::layout::LayoutEngine;
use crate::model::{Node, NodeId};
use indextree::Arena;

pub fn toggle_collapse(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        if let Some(node) = app.tree.get_mut(active_id) {
            node.get_mut().is_collapsed = !node.get().is_collapsed;
        }
    }
}

pub fn collapse_all(app: &mut AppState) {
    for node in app.tree.iter_mut() {
        node.get_mut().is_collapsed = true;
    }
}

pub fn expand_all(app: &mut AppState) {
    for node in app.tree.iter_mut() {
        node.get_mut().is_collapsed = false;
    }
}

pub fn collapse_children(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        let children: Vec<NodeId> = active_id.children(&app.tree).collect();
        for child_id in children {
            if let Some(node) = app.tree.get_mut(child_id) {
                node.get_mut().is_collapsed = true;
            }
        }
    }
}

pub fn collapse_other_branches(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        // Collapse all nodes
        for node in app.tree.iter_mut() {
            node.get_mut().is_collapsed = true;
        }

        // Expand path to active node
        let ancestors: Vec<NodeId> = active_id.ancestors(&app.tree).collect();
        for ancestor_id in ancestors {
            if let Some(node) = app.tree.get_mut(ancestor_id) {
                node.get_mut().is_collapsed = false;
            }
        }
    }
}

pub fn collapse_to_level(app: &mut AppState, target_level: usize) {
    fn set_collapse_at_depth(
        tree: &mut Arena<Node>,
        node_id: NodeId,
        current_level: usize,
        target_level: usize,
    ) {
        if let Some(node) = tree.get_mut(node_id) {
            node.get_mut().is_collapsed = current_level >= target_level;
        }

        let children: Vec<NodeId> = node_id.children(tree).collect();
        for child_id in children {
            set_collapse_at_depth(tree, child_id, current_level + 1, target_level);
        }
    }

    if let Some(root_id) = app.root_id {
        set_collapse_at_depth(&mut app.tree, root_id, 0, target_level);
    }
}

pub fn center_active_node(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        // Get the layout to find the active node's position
        let layout = LayoutEngine::calculate_layout(app);

        if let Some(node_layout) = layout.nodes.get(&active_id) {
            // Calculate center position
            let node_center_x = node_layout.x + node_layout.w / 2.0;
            let node_center_y = node_layout.y + node_layout.yo + node_layout.lh / 2.0;

            // Center the viewport on the active node
            // Allow negative viewport values for proper centering of nodes near edges
            app.viewport_left = node_center_x - app.terminal_width as f64 / 2.0;
            app.viewport_top = node_center_y - app.terminal_height as f64 / 2.0;
        }
    }
}

pub fn toggle_center_lock(app: &mut AppState) {
    app.config.center_lock = !app.config.center_lock;
    app.set_message(format!(
        "Center lock: {}",
        if app.config.center_lock { "ON" } else { "OFF" }
    ));
}

pub fn focus(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        // Focus mode: collapse all except ancestors and descendants of active node
        // This matches the PHP implementation's focus_vh function

        // Collapse siblings recursively up the tree
        collapse_siblings_recursive(&mut app.tree, active_id);

        // Expand all descendants of the active node
        expand_descendants(&mut app.tree, active_id);

        app.set_message("Focus mode applied");
    }
}

pub fn toggle_focus_lock(app: &mut AppState) {
    app.config.focus_lock = !app.config.focus_lock;
    app.set_message(format!(
        "Focus lock: {}",
        if app.config.focus_lock { "ON" } else { "OFF" }
    ));
}

/// Helper function to recursively collapse all siblings of a node up the tree
fn collapse_siblings_recursive(tree: &mut Arena<Node>, node_id: NodeId) {
    // Get the parent of the current node
    let parent_id = match tree.get(node_id) {
        Some(node_ref) => match node_ref.parent() {
            Some(parent) => parent,
            None => return, // Root node, no parent
        },
        None => return,
    };

    // Collapse all siblings (children of parent except current node)
    let children: Vec<NodeId> = parent_id.children(tree).collect();
    for child_id in children {
        if child_id != node_id {
            if let Some(child_node) = tree.get_mut(child_id) {
                child_node.get_mut().is_collapsed = true;
            }
        }
    }

    // Recursively apply to parent
    collapse_siblings_recursive(tree, parent_id);
}

/// Helper function to recursively expand all descendants of a node
fn expand_descendants(tree: &mut Arena<Node>, node_id: NodeId) {
    // Expand the current node
    if let Some(node) = tree.get_mut(node_id) {
        node.get_mut().is_collapsed = false;
    }

    // Recursively expand all children
    let children: Vec<NodeId> = node_id.children(tree).collect();
    for child_id in children {
        expand_descendants(tree, child_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::model::Node;

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
    fn test_toggle_collapse() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        let initial_state = app.tree.get(root).unwrap().get().is_collapsed;
        toggle_collapse(&mut app);
        let new_state = app.tree.get(root).unwrap().get().is_collapsed;

        assert_ne!(initial_state, new_state);
    }

    #[test]
    fn test_collapse_all() {
        let mut app = create_test_app();

        collapse_all(&mut app);

        for node in app.tree.iter() {
            assert!(node.get().is_collapsed);
        }
    }

    #[test]
    fn test_expand_all() {
        let mut app = create_test_app();

        // First collapse all
        collapse_all(&mut app);
        // Then expand all
        expand_all(&mut app);

        for node in app.tree.iter() {
            assert!(!node.get().is_collapsed);
        }
    }

    #[test]
    fn test_collapse_children() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        // Ensure children are expanded first
        let children: Vec<_> = root.children(&app.tree).collect();
        for child_id in &children {
            app.tree.get_mut(*child_id).unwrap().get_mut().is_collapsed = false;
        }

        collapse_children(&mut app);

        // All direct children should be collapsed
        for child_id in root.children(&app.tree) {
            let child = app.tree.get(child_id).unwrap().get();
            assert!(child.is_collapsed);
        }

        // Root itself should not be collapsed
        assert!(!app.tree.get(root).unwrap().get().is_collapsed);
    }

    #[test]
    fn test_collapse_other_branches() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();

        // Set active node to child1
        app.active_node_id = Some(child1);

        // Expand all first
        expand_all(&mut app);

        collapse_other_branches(&mut app);

        // Child1 and its ancestors should be expanded
        assert!(!app.tree.get(child1).unwrap().get().is_collapsed);
        assert!(!app.tree.get(root).unwrap().get().is_collapsed);

        // Child2 should be collapsed (it's a sibling, not in the active path)
        let child2 = root.children(&app.tree).nth(1).unwrap();
        assert!(app.tree.get(child2).unwrap().get().is_collapsed);
    }

    #[test]
    fn test_toggle_settings() {
        let mut app = create_test_app();

        let initial_center_lock = app.config.center_lock;
        toggle_center_lock(&mut app);
        assert_ne!(app.config.center_lock, initial_center_lock);

        let initial_focus_lock = app.config.focus_lock;
        toggle_focus_lock(&mut app);
        assert_ne!(app.config.focus_lock, initial_focus_lock);
    }

    #[test]
    fn test_center_active_node() {
        let mut app = create_test_app();

        // Set terminal dimensions
        app.terminal_width = 80;
        app.terminal_height = 24;

        // Test centering the root node
        center_active_node(&mut app);

        // The viewport should be adjusted to center the active node
        // Since we don't know exact layout positions without running the layout engine,
        // we just verify the function doesn't panic and modifies viewport
        let initial_viewport_top = app.viewport_top;
        let initial_viewport_left = app.viewport_left;

        // Change active node and center again
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();
        app.active_node_id = Some(child1);
        center_active_node(&mut app);

        // Viewport should have changed (unless nodes are at exact same position)
        // This is a basic test - more comprehensive tests would need mock layouts
        assert!(
            app.viewport_top != initial_viewport_top || app.viewport_left != initial_viewport_left
        );
    }

    #[test]
    fn test_center_active_node_allows_negative_viewport() {
        let mut app = create_test_app();

        // Set small terminal dimensions to force negative viewport
        app.terminal_width = 10;
        app.terminal_height = 10;

        // Center on root node which is typically at (0,0) or small positive coordinates
        center_active_node(&mut app);

        // With small terminal and node near origin, viewport should be negative
        // to center the node in the terminal
        // This test verifies we don't clamp to 0
        assert!(app.viewport_top < 0.0 || app.viewport_left < 0.0);
    }

    #[test]
    fn test_focus_mode() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        // Get the structure: Root -> Child1, Child2 -> Grandchild
        let children: Vec<_> = root.children(&app.tree).collect();
        let child1 = children[0];
        let child2 = children[1];
        let grandchild = child2.children(&app.tree).next().unwrap();

        // Set active node to grandchild
        app.active_node_id = Some(grandchild);

        // First expand all to ensure initial state
        expand_all(&mut app);

        // Apply focus mode
        focus(&mut app);

        // Check that:
        // - Grandchild (active) should be expanded
        assert!(!app.tree.get(grandchild).unwrap().get().is_collapsed);

        // - Child2 (parent of active) should be expanded
        assert!(!app.tree.get(child2).unwrap().get().is_collapsed);

        // - Root (ancestor) should be expanded
        assert!(!app.tree.get(root).unwrap().get().is_collapsed);

        // - Child1 (sibling of parent) should be collapsed
        assert!(app.tree.get(child1).unwrap().get().is_collapsed);
    }

    #[test]
    fn test_focus_on_root() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        // Set active node to root
        app.active_node_id = Some(root);

        // First collapse all
        collapse_all(&mut app);

        // Apply focus mode
        focus(&mut app);

        // Root and all its descendants should be expanded
        assert!(!app.tree.get(root).unwrap().get().is_collapsed);

        // All children should be expanded when root is focused
        for child_id in root.children(&app.tree) {
            assert!(!app.tree.get(child_id).unwrap().get().is_collapsed);
        }
    }

    #[test]
    fn test_helper_collapse_siblings_recursive() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        // Get children
        let children: Vec<_> = root.children(&app.tree).collect();
        let child1 = children[0];
        let child2 = children[1];

        // First expand all
        expand_all(&mut app);

        // Call helper on child2
        collapse_siblings_recursive(&mut app.tree, child2);

        // Child1 should be collapsed (sibling)
        assert!(app.tree.get(child1).unwrap().get().is_collapsed);

        // Child2 should still be expanded (not a sibling of itself)
        assert!(!app.tree.get(child2).unwrap().get().is_collapsed);
    }

    #[test]
    fn test_helper_expand_descendants() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        // First collapse all
        collapse_all(&mut app);

        // Expand descendants of root
        expand_descendants(&mut app.tree, root);

        // All nodes should be expanded
        for node in app.tree.iter() {
            assert!(!node.get().is_collapsed);
        }
    }
}
