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
            app.viewport_left = (node_center_x - app.terminal_width as f64 / 2.0).max(0.0);
            app.viewport_top = (node_center_y - app.terminal_height as f64 / 2.0).max(0.0);
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
    // TODO: Implement focus mode
    app.set_message("Focus mode not yet implemented");
}

pub fn toggle_focus_lock(app: &mut AppState) {
    app.config.focus_lock = !app.config.focus_lock;
    app.set_message(format!(
        "Focus lock: {}",
        if app.config.focus_lock { "ON" } else { "OFF" }
    ));
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
}
