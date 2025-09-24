use crate::app::AppState;
use crate::model::{Node, NodeId};
use crate::parser;
use anyhow::Result;
use clipboard::{ClipboardContext, ClipboardProvider};
use indextree::Arena;
use std::path::PathBuf;

pub fn save(app: &mut AppState) -> Result<()> {
    if let Some(ref path) = app.filename {
        if let Some(root_id) = app.root_id {
            match parser::save_file(&app.tree, root_id, path) {
                Ok(_) => {
                    app.set_message(format!("Saved to {}", path.display()));
                    app.is_dirty = false;
                }
                Err(e) => {
                    app.set_message(format!("Failed to save: {}", e));
                    return Err(e);
                }
            }
        } else {
            app.set_message("No content to save");
        }
    } else {
        app.set_message("No filename set - use Shift+S for Save As");
    }
    Ok(())
}

pub fn save_as(app: &mut AppState) -> Result<()> {
    // For now, we'll save with a default name
    // In a real implementation, this would open a file dialog
    let default_path = PathBuf::from("mindmap.hmm");

    if let Some(root_id) = app.root_id {
        match parser::save_file(&app.tree, root_id, &default_path) {
            Ok(_) => {
                app.filename = Some(default_path.clone());
                app.is_dirty = false;
                app.set_message(format!("Saved as {}", default_path.display()));
            }
            Err(e) => {
                app.set_message(format!("Failed to save: {}", e));
                return Err(e);
            }
        }
    } else {
        app.set_message("No content to save");
    }
    Ok(())
}

pub fn export_text(app: &mut AppState) -> Result<()> {
    if let Some(root_id) = app.root_id {
        // Export the entire visible tree to text format
        let mut output = String::new();
        export_text_node(&app.tree, root_id, &mut output, 0);

        // Copy to clipboard
        if let Ok(mut ctx) = ClipboardContext::new() {
            let _ = ctx.set_contents(output.clone());
        }
        app.clipboard = Some(output);

        app.set_message("Exported the map to clipboard.");
    }

    Ok(())
}

pub fn export_text_node(tree: &Arena<Node>, node_id: NodeId, output: &mut String, depth: usize) {
    let node = tree.get(node_id).unwrap().get();

    // Add the current node with proper indentation
    output.push_str(&"\t".repeat(depth));
    output.push_str(&node.title);
    output.push('\n');

    // Process children if node is not collapsed
    if !node.is_collapsed {
        for child_id in node_id.children(tree) {
            export_text_node(tree, child_id, output, depth + 1);
        }
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
    fn test_export_text() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        // Collapse Child 2 to test visible-only export
        let children: Vec<_> = root.children(&app.tree).collect();
        let child2 = children[1]; // Child 2 has the grandchild
        app.tree.get_mut(child2).unwrap().get_mut().is_collapsed = true;

        export_text(&mut app).unwrap();

        // Check clipboard contains exported text
        assert!(app.clipboard.is_some());
        let exported = app.clipboard.as_ref().unwrap();

        // Should contain root and both children
        assert!(exported.contains("Root"));
        assert!(exported.contains("Child 1"));
        assert!(exported.contains("Child 2"));

        // Should not contain grandchild of collapsed Child 2
        assert!(!exported.contains("Grandchild"));
    }
}
