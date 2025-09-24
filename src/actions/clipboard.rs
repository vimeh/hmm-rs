use crate::app::AppState;
use crate::model::{Node, NodeId};
use crate::parser;
use anyhow::Result;
use clipboard::{ClipboardContext, ClipboardProvider};
use indextree::Arena;

pub fn yank_node(app: &mut AppState) -> Result<()> {
    if let Some(active_id) = app.active_node_id {
        let text = parser::map_to_list(&app.tree, active_id, false, 0);
        app.clipboard = Some(text.clone());

        // Try to copy to system clipboard
        if let Ok(mut ctx) = ClipboardContext::new() {
            let _ = ctx.set_contents(text);
        }

        app.set_message("Node yanked");
    }
    Ok(())
}

pub fn yank_children(app: &mut AppState) -> Result<()> {
    if let Some(active_id) = app.active_node_id {
        let text = parser::map_to_list(&app.tree, active_id, true, 0);
        app.clipboard = Some(text.clone());

        // Try to copy to system clipboard
        if let Ok(mut ctx) = ClipboardContext::new() {
            let _ = ctx.set_contents(text);
        }

        app.set_message("Children yanked");
    }
    Ok(())
}

pub fn paste_as_children(app: &mut AppState) -> Result<()> {
    if let Some(clipboard_text) = app.clipboard.clone() {
        if let Some(active_id) = app.active_node_id {
            app.push_history();

            // Parse the clipboard text into a tree
            match parser::parse_hmm_content(&clipboard_text) {
                Ok((parsed_tree, parsed_root)) => {
                    // Add all nodes from the parsed tree as children of the active node
                    add_subtree_to_parent(&mut app.tree, &parsed_tree, parsed_root, active_id);
                    app.set_message("Pasted as children");
                }
                Err(_) => {
                    app.set_message("Failed to parse clipboard content");
                }
            }
        }
    } else {
        app.set_message("Clipboard is empty");
    }
    Ok(())
}

pub fn paste_as_siblings(app: &mut AppState) -> Result<()> {
    if let Some(clipboard_text) = app.clipboard.clone() {
        if let Some(active_id) = app.active_node_id {
            app.push_history();

            // Get the parent of the active node
            if let Some(parent_id) = app.tree.get(active_id).and_then(|n| n.parent()) {
                // Parse the clipboard text into a tree
                match parser::parse_hmm_content(&clipboard_text) {
                    Ok((parsed_tree, parsed_root)) => {
                        // Add all nodes from the parsed tree as siblings after the active node
                        add_subtree_as_sibling(
                            &mut app.tree,
                            &parsed_tree,
                            parsed_root,
                            active_id,
                            parent_id,
                        );
                        app.set_message("Pasted as siblings");
                    }
                    Err(_) => {
                        app.set_message("Failed to parse clipboard content");
                    }
                }
            } else {
                app.set_message("Cannot paste siblings at root level");
            }
        }
    } else {
        app.set_message("Clipboard is empty");
    }
    Ok(())
}

// Helper functions for paste operations
pub fn add_subtree_to_parent(
    target_tree: &mut Arena<Node>,
    source_tree: &Arena<Node>,
    source_root: NodeId,
    parent_id: NodeId,
) {
    // Recursively copy nodes from source tree to target tree
    fn copy_subtree(
        target_tree: &mut Arena<Node>,
        source_tree: &Arena<Node>,
        source_id: NodeId,
        target_parent_id: NodeId,
    ) {
        // Copy the node
        let source_node = source_tree.get(source_id).unwrap().get();
        let new_node_id = target_tree.new_node(source_node.clone());
        target_parent_id.append(new_node_id, target_tree);

        // Recursively copy children
        for child in source_id.children(source_tree) {
            copy_subtree(target_tree, source_tree, child, new_node_id);
        }
    }

    // If the parsed root is a synthetic root, add its children
    // Otherwise, add the root itself
    let source_node = source_tree.get(source_root).unwrap().get();
    if source_node.title == "root" && source_root.children(source_tree).count() > 0 {
        // Skip the synthetic root and add its children directly
        for child in source_root.children(source_tree) {
            copy_subtree(target_tree, source_tree, child, parent_id);
        }
    } else {
        // Add the root and all its descendants
        copy_subtree(target_tree, source_tree, source_root, parent_id);
    }
}

pub fn add_subtree_as_sibling(
    target_tree: &mut Arena<Node>,
    source_tree: &Arena<Node>,
    source_root: NodeId,
    after_node: NodeId,
    parent_id: NodeId,
) {
    // Recursively copy nodes from source tree to target tree
    fn copy_subtree(
        target_tree: &mut Arena<Node>,
        source_tree: &Arena<Node>,
        source_id: NodeId,
        target_parent_id: NodeId,
    ) -> NodeId {
        // Copy the node
        let source_node = source_tree.get(source_id).unwrap().get();
        let new_node_id = target_tree.new_node(source_node.clone());
        target_parent_id.append(new_node_id, target_tree);

        // Recursively copy children
        for child in source_id.children(source_tree) {
            copy_subtree(target_tree, source_tree, child, new_node_id);
        }

        new_node_id
    }

    // Collect all nodes to add
    let mut nodes_to_add = Vec::new();

    let source_node = source_tree.get(source_root).unwrap().get();
    if source_node.title == "root" && source_root.children(source_tree).count() > 0 {
        // Skip the synthetic root and add its children
        for child in source_root.children(source_tree) {
            let new_node = copy_subtree(target_tree, source_tree, child, parent_id);
            nodes_to_add.push(new_node);
        }
    } else {
        // Add the root itself
        let new_node = copy_subtree(target_tree, source_tree, source_root, parent_id);
        nodes_to_add.push(new_node);
    }

    // Move the new nodes to be after the specified node
    // This requires detaching and re-attaching in the right order
    for new_node in nodes_to_add {
        new_node.detach(target_tree);
        after_node.insert_after(new_node, target_tree);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_yank_node() {
        let mut app = create_test_app();

        yank_node(&mut app).unwrap();
        assert!(app.clipboard.is_some());
        assert!(app.message.is_some());
    }

    #[test]
    fn test_yank_children() {
        let mut app = create_test_app();

        yank_children(&mut app).unwrap();

        // Clipboard should contain the children
        assert!(app.clipboard.is_some());
        let clipboard = app.clipboard.as_ref().unwrap();
        assert!(clipboard.contains("Child 1"));
        assert!(clipboard.contains("Child 2"));
        assert!(!clipboard.contains("Root")); // Should not include the parent
    }

    #[test]
    fn test_paste_as_children() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        // Prepare clipboard with some content
        app.clipboard = Some("New Node 1\n\tSubnode 1\n\tSubnode 2\nNew Node 2".to_string());

        // Paste as children to root
        paste_as_children(&mut app).unwrap();

        // Check that new nodes were added as children
        let children: Vec<_> = root.children(&app.tree).collect();
        assert!(children.len() > 2); // Original 2 children + new nodes

        // Verify the new nodes exist
        let mut found_new_node1 = false;
        let mut found_new_node2 = false;
        for child in root.children(&app.tree) {
            let node = app.tree.get(child).unwrap().get();
            if node.title == "New Node 1" {
                found_new_node1 = true;
                // Check it has subnodes
                let subnodes: Vec<_> = child.children(&app.tree).collect();
                assert_eq!(subnodes.len(), 2);
            }
            if node.title == "New Node 2" {
                found_new_node2 = true;
            }
        }
        assert!(found_new_node1);
        assert!(found_new_node2);
    }

    #[test]
    fn test_paste_as_siblings() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();

        // Set active node to child1
        app.active_node_id = Some(child1);

        // Prepare clipboard with some content
        app.clipboard = Some("Sibling 1\nSibling 2".to_string());

        // Paste as siblings
        paste_as_siblings(&mut app).unwrap();

        // Check that new nodes were added as siblings
        let children: Vec<_> = root.children(&app.tree).collect();
        assert!(children.len() >= 4); // Original 2 children + 2 new siblings

        // Verify the new siblings exist
        let mut found_sibling1 = false;
        let mut found_sibling2 = false;
        for child in root.children(&app.tree) {
            let node = app.tree.get(child).unwrap().get();
            if node.title == "Sibling 1" {
                found_sibling1 = true;
            }
            if node.title == "Sibling 2" {
                found_sibling2 = true;
            }
        }
        assert!(found_sibling1);
        assert!(found_sibling2);
    }
}
