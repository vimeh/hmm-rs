use crate::app::AppState;
use crate::model::{Node, NodeId};
use crate::parser;

use super::editing::start_editing;

pub fn insert_sibling(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        app.push_history();

        let new_node = app.tree.new_node(Node::new("NEW".to_string()));

        if let Some(_parent_id) = active_id.ancestors(&app.tree).nth(1) {
            active_id.insert_after(new_node, &mut app.tree);
        }

        app.active_node_id = Some(new_node);
        start_editing(app, true);
    }
}

pub fn insert_child(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        app.push_history();

        let new_node = app.tree.new_node(Node::new("NEW".to_string()));
        active_id.append(new_node, &mut app.tree);

        // Expand parent node
        if let Some(node) = app.tree.get_mut(active_id) {
            node.get_mut().is_collapsed = false;
        }

        app.active_node_id = Some(new_node);
        start_editing(app, true);
    }
}

pub fn delete_node(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        if active_id == app.root_id.unwrap() {
            app.set_message("Cannot delete root node");
            return;
        }

        app.push_history();

        // Save to clipboard
        let subtree_text = parser::map_to_list(&app.tree, active_id, false, 0);
        app.clipboard = Some(subtree_text);

        // Move to sibling or parent
        if let Some(parent_id) = active_id.ancestors(&app.tree).nth(1) {
            let siblings: Vec<NodeId> = parent_id.children(&app.tree).collect();
            let current_index = siblings.iter().position(|&id| id == active_id);

            if let Some(idx) = current_index {
                if idx > 0 {
                    app.active_node_id = Some(siblings[idx - 1]);
                } else if siblings.len() > 1 {
                    app.active_node_id = Some(siblings[1]);
                } else {
                    app.active_node_id = Some(parent_id);
                }
            }
        }

        active_id.remove(&mut app.tree);
    }
}

pub fn delete_children(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        app.push_history();

        let children: Vec<NodeId> = active_id.children(&app.tree).collect();
        for child_id in children {
            child_id.remove(&mut app.tree);
        }
    }
}

pub fn move_node_up(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        if let Some(prev_sibling) = active_id.preceding_siblings(&app.tree).nth(1) {
            app.push_history();
            prev_sibling.insert_before(active_id, &mut app.tree);
        }
    }
}

pub fn move_node_down(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        if let Some(next_sibling) = active_id.following_siblings(&app.tree).nth(1) {
            app.push_history();
            next_sibling.insert_after(active_id, &mut app.tree);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppMode;
    use crate::config::AppConfig;

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
    fn test_insert_child() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let initial_children_count = root.children(&app.tree).count();

        insert_child(&mut app);

        let new_children_count = root.children(&app.tree).count();
        assert_eq!(new_children_count, initial_children_count + 1);

        // Should be in editing mode
        assert!(matches!(app.mode, AppMode::Editing { .. }));
    }

    #[test]
    fn test_insert_sibling() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();

        app.active_node_id = Some(child1);
        let initial_children_count = root.children(&app.tree).count();

        insert_sibling(&mut app);

        let new_children_count = root.children(&app.tree).count();
        assert_eq!(new_children_count, initial_children_count + 1);

        // Should be in editing mode
        assert!(matches!(app.mode, AppMode::Editing { .. }));
    }

    #[test]
    fn test_delete_node() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();

        app.active_node_id = Some(child1);

        // Store the NodeId of child1 before deletion
        let child1_id = child1;

        delete_node(&mut app);

        // Check that the node is marked as removed
        if let Some(node_ref) = app.tree.get(child1_id) {
            assert!(node_ref.is_removed(), "Node should be marked as removed");
        }

        // Should have moved to another node (sibling or parent)
        assert_ne!(app.active_node_id, Some(child1_id));

        // The active node should be valid and not removed
        if let Some(active_id) = app.active_node_id {
            let active_node = app.tree.get(active_id).expect("Active node should exist");
            assert!(
                !active_node.is_removed(),
                "Active node should not be removed"
            );
        }

        // Verify clipboard has the deleted content
        assert!(app.clipboard.is_some());

        // Verify that the node is no longer a child of root
        let remaining_children: Vec<_> = root.children(&app.tree).collect();
        assert!(
            !remaining_children.contains(&child1_id),
            "Child1 should not be in root's children"
        );

        // Should only have one child left (Child2 with its Grandchild)
        assert_eq!(remaining_children.len(), 1);
    }

    #[test]
    fn test_delete_root_node_fails() {
        let mut app = create_test_app();
        let initial_count = app.tree.count();

        delete_node(&mut app);

        // Root should not be deleted
        assert_eq!(app.tree.count(), initial_count);
        assert!(app.message.is_some());
    }

    #[test]
    fn test_delete_children() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        // Root initially has 2 children (Child1, Child2)
        let initial_children: Vec<_> = root.children(&app.tree).collect();
        assert_eq!(initial_children.len(), 2);

        // Ensure root is the active node
        app.active_node_id = Some(root);

        // Call delete_children
        delete_children(&mut app);

        // Children should be marked as removed
        for child_id in initial_children {
            if let Some(node) = app.tree.get(child_id) {
                assert!(
                    node.is_removed(),
                    "Child {:?} should be marked as removed",
                    child_id
                );
            }
        }

        // Root itself should still exist and not be removed
        assert!(app.tree.get(root).is_some());
        assert!(!app.tree.get(root).unwrap().is_removed());
        assert_eq!(app.active_node_id, Some(root));
    }

    #[test]
    fn test_move_node_up() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let children: Vec<_> = root.children(&app.tree).collect();
        let child2 = children[1]; // Second child

        app.active_node_id = Some(child2);

        move_node_up(&mut app);

        // Child2 should now be the first child
        let new_children: Vec<_> = root.children(&app.tree).collect();
        assert_eq!(new_children[0], child2);
        assert_eq!(new_children[1], children[0]);
    }

    #[test]
    fn test_move_node_down() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let children: Vec<_> = root.children(&app.tree).collect();
        let child1 = children[0]; // First child

        app.active_node_id = Some(child1);

        move_node_down(&mut app);

        // Child1 should now be the second child
        let new_children: Vec<_> = root.children(&app.tree).collect();
        assert_eq!(new_children[0], children[1]);
        assert_eq!(new_children[1], child1);
    }
}
