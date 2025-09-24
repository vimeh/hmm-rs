use crate::app::AppState;
use crate::layout::LayoutEngine;
use crate::model::{Node, NodeId};

// Helper function to ensure active node is visible
pub fn ensure_node_visible(app: &mut AppState) {
    if app.config.center_lock {
        center_active_node(app);
    } else if let Some(active_id) = app.active_node_id {
        let layout = LayoutEngine::calculate_layout(app);

        if let Some(node_layout) = layout.nodes.get(&active_id) {
            let node_x = node_layout.x;
            let node_y = node_layout.y + node_layout.yo;
            let node_right = node_x + node_layout.w;
            let node_bottom = node_y + node_layout.lh;

            // Adjust viewport to ensure node is visible
            let margin = 2.0; // Small margin around the node

            // Horizontal adjustment
            if node_x < app.viewport_left + margin {
                app.viewport_left = (node_x - margin).max(0.0);
            } else if node_right > app.viewport_left + app.terminal_width as f64 - margin {
                app.viewport_left = node_right - app.terminal_width as f64 + margin;
            }

            // Vertical adjustment
            if node_y < app.viewport_top + margin {
                app.viewport_top = (node_y - margin).max(0.0);
            } else if node_bottom > app.viewport_top + app.terminal_height as f64 - margin {
                app.viewport_top = node_bottom - app.terminal_height as f64 + margin;
            }
        }
    }
}

/// Find the previous node in visual tree order (depth-first traversal)
fn find_prev_visible_node(tree: &indextree::Arena<Node>, current: NodeId) -> Option<NodeId> {
    // Try to find previous sibling
    if let Some(parent_id) = current.ancestors(tree).nth(1) {
        let siblings: Vec<NodeId> = parent_id.children(tree).collect();
        if let Some(current_idx) = siblings.iter().position(|&id| id == current) {
            if current_idx > 0 {
                // Go to previous sibling's last descendant
                let prev_sibling = siblings[current_idx - 1];
                return Some(find_last_visible_descendant(tree, prev_sibling));
            } else {
                // No previous sibling, go to parent
                return Some(parent_id);
            }
        }
    }
    None
}

/// Find the last visible descendant of a node (or the node itself)
fn find_last_visible_descendant(tree: &indextree::Arena<Node>, node_id: NodeId) -> NodeId {
    let node = tree.get(node_id).unwrap().get();
    if node.is_collapsed {
        return node_id;
    }

    if let Some(last_child) = node_id.children(tree).next_back() {
        return find_last_visible_descendant(tree, last_child);
    }

    node_id
}

pub fn go_up(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        // Don't move up from root
        if Some(active_id) == app.root_id {
            return;
        }

        if let Some(prev_node) = find_prev_visible_node(&app.tree, active_id) {
            app.active_node_id = Some(prev_node);
            ensure_node_visible(app);
        }
    }
}

/// Find the next node in visual tree order (depth-first traversal)
fn find_next_visible_node(tree: &indextree::Arena<Node>, current: NodeId) -> Option<NodeId> {
    let node = tree.get(current).unwrap().get();

    // If not collapsed and has children, go to first child
    if !node.is_collapsed {
        if let Some(first_child) = current.children(tree).next() {
            return Some(first_child);
        }
    }

    // Otherwise, find next sibling or ancestor's next sibling
    let mut node_id = current;
    while let Some(parent_id) = node_id.ancestors(tree).nth(1) {
        let siblings: Vec<NodeId> = parent_id.children(tree).collect();
        if let Some(current_idx) = siblings.iter().position(|&id| id == node_id) {
            if current_idx < siblings.len() - 1 {
                // Found a next sibling
                return Some(siblings[current_idx + 1]);
            }
        }
        // No next sibling, try parent's next sibling
        node_id = parent_id;
    }

    None
}

pub fn go_down(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        if let Some(next_node) = find_next_visible_node(&app.tree, active_id) {
            app.active_node_id = Some(next_node);
            ensure_node_visible(app);
        }
    }
}

pub fn go_left(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        if let Some(parent_id) = active_id.ancestors(&app.tree).nth(1) {
            // Allow moving to parent even if it's the root
            app.active_node_id = Some(parent_id);
            ensure_node_visible(app);
        }
    }
}

pub fn go_right(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        let node = app.tree.get(active_id).unwrap().get();
        if !node.is_collapsed {
            if let Some(first_child) = active_id.children(&app.tree).next() {
                app.active_node_id = Some(first_child);
                ensure_node_visible(app);
            }
        }
    }
}

pub fn go_to_root(app: &mut AppState) {
    app.active_node_id = app.root_id;
    ensure_node_visible(app);
}

pub fn go_to_top(app: &mut AppState) {
    if let Some(root_id) = app.root_id {
        app.active_node_id = Some(root_id);
        app.viewport_top = 0.0;
        app.viewport_left = 0.0;
    }
}

pub fn go_to_bottom(app: &mut AppState) {
    if let Some(root_id) = app.root_id {
        fn find_last_visible(tree: &indextree::Arena<Node>, node_id: NodeId) -> NodeId {
            let node = tree.get(node_id).unwrap().get();
            if node.is_collapsed {
                return node_id;
            }

            if let Some(last_child) = node_id.children(tree).next_back() {
                return find_last_visible(tree, last_child);
            }

            node_id
        }

        app.active_node_id = Some(find_last_visible(&app.tree, root_id));
        ensure_node_visible(app);
    }
}

// Import from view module to avoid circular dependency
use super::view::center_active_node;

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
    fn test_movement_go_down() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();
        let child2 = root.children(&app.tree).nth(1).unwrap();
        let grandchild = child2.children(&app.tree).next().unwrap();

        // Test vertical navigation: Root -> Child1 -> Child2 -> Grandchild
        app.active_node_id = Some(root);

        go_down(&mut app);
        assert_eq!(
            app.active_node_id,
            Some(child1),
            "Should go from Root to Child1"
        );

        go_down(&mut app);
        assert_eq!(
            app.active_node_id,
            Some(child2),
            "Should go from Child1 to Child2"
        );

        go_down(&mut app);
        assert_eq!(
            app.active_node_id,
            Some(grandchild),
            "Should go from Child2 to Grandchild"
        );

        // At the end, go_down should not change position
        go_down(&mut app);
        assert_eq!(
            app.active_node_id,
            Some(grandchild),
            "Should stay at Grandchild"
        );
    }

    #[test]
    fn test_movement_go_up() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();
        let child2 = root.children(&app.tree).nth(1).unwrap();
        let grandchild = child2.children(&app.tree).next().unwrap();

        // Test vertical navigation: Grandchild -> Child2 -> Child1 -> Root
        app.active_node_id = Some(grandchild);

        go_up(&mut app);
        assert_eq!(
            app.active_node_id,
            Some(child2),
            "Should go from Grandchild to Child2"
        );

        go_up(&mut app);
        assert_eq!(
            app.active_node_id,
            Some(child1),
            "Should go from Child2 to Child1"
        );

        go_up(&mut app);
        assert_eq!(
            app.active_node_id,
            Some(root),
            "Should go from Child1 to Root"
        );

        // At the root, go_up should not change position
        go_up(&mut app);
        assert_eq!(app.active_node_id, Some(root), "Should stay at Root");
    }

    #[test]
    fn test_movement_with_collapsed_node() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();
        let child2 = root.children(&app.tree).nth(1).unwrap();

        // Collapse Child2 (which has a grandchild)
        app.tree.get_mut(child2).unwrap().get_mut().is_collapsed = true;

        // Test that collapsed node's children are skipped
        app.active_node_id = Some(root);

        go_down(&mut app);
        assert_eq!(app.active_node_id, Some(child1), "Should go to Child1");

        go_down(&mut app);
        assert_eq!(app.active_node_id, Some(child2), "Should go to Child2");

        go_down(&mut app);
        assert_eq!(
            app.active_node_id,
            Some(child2),
            "Should stay at Child2 (grandchild is hidden)"
        );
    }

    #[test]
    fn test_movement_go_left() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();

        app.active_node_id = Some(child1);
        go_left(&mut app);
        assert_eq!(app.active_node_id, Some(root));
    }

    #[test]
    fn test_movement_go_right() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();

        // Ensure node is not collapsed
        app.tree.get_mut(root).unwrap().get_mut().is_collapsed = false;

        go_right(&mut app);
        assert_eq!(app.active_node_id, Some(child1));
    }

    #[test]
    fn test_movement_go_to_root() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();

        app.active_node_id = Some(child1);
        go_to_root(&mut app);
        assert_eq!(app.active_node_id, Some(root));
    }

    #[test]
    fn test_go_to_top() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child2 = root.children(&app.tree).nth(1).unwrap();

        // Start at child2
        app.active_node_id = Some(child2);

        go_to_top(&mut app);

        // Should be at the root (first visible node)
        assert_eq!(app.active_node_id, Some(root));
    }

    #[test]
    fn test_go_to_bottom() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        // Expand all to make grandchild visible
        for node in app.tree.iter_mut() {
            node.get_mut().is_collapsed = false;
        }

        go_to_bottom(&mut app);

        // Should be at the last visible node (grandchild)
        // Get the grandchild through Child2
        let child2 = root.children(&app.tree).nth(1).unwrap();
        let grandchild = child2.children(&app.tree).next().unwrap();
        assert_eq!(app.active_node_id, Some(grandchild));
    }
}
