use hmm_rs::{parser, AppConfig, AppState, Node, NodeId};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_load_and_save_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.hmm");

    // Create test content
    let content = "Root\n\tChild 1\n\t\tGrandchild 1\n\tChild 2\n\t\tGrandchild 2";
    fs::write(&file_path, content).unwrap();

    // Load the file
    let (tree, root_id) = parser::load_file(&file_path).unwrap();

    // Verify structure
    assert!(tree.count() > 1);
    assert_eq!(tree.get(root_id).unwrap().get().title, "Root");

    // Save the file
    parser::save_file(&tree, root_id, &file_path).unwrap();

    // Read it back
    let saved_content = fs::read_to_string(&file_path).unwrap();
    assert!(saved_content.contains("Root"));
    assert!(saved_content.contains("Child 1"));
    assert!(saved_content.contains("Grandchild 1"));
}

#[test]
fn test_app_state_initialization() {
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Create a simple tree
    let root = app.tree.new_node(Node::new("Root".to_string()));
    app.root_id = Some(root);
    app.active_node_id = Some(root);

    assert!(app.running);
    assert_eq!(app.root_id, Some(root));
    assert_eq!(app.active_node_id, Some(root));
}

#[test]
fn test_tree_manipulation() {
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Create a tree
    let root = app.tree.new_node(Node::new("Root".to_string()));
    let child1 = app.tree.new_node(Node::new("Child 1".to_string()));
    let child2 = app.tree.new_node(Node::new("Child 2".to_string()));

    root.append(child1, &mut app.tree);
    root.append(child2, &mut app.tree);

    app.root_id = Some(root);

    // Test tree structure
    let children: Vec<NodeId> = root.children(&app.tree).collect();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0], child1);
    assert_eq!(children[1], child2);
}

#[test]
fn test_history_management() {
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Create initial state
    let root = app.tree.new_node(Node::new("Root".to_string()));
    app.root_id = Some(root);
    app.push_history();

    // Make a change
    app.tree.get_mut(root).unwrap().get_mut().title = "Modified".to_string();
    app.push_history();

    // Verify history was saved
    assert!(app.history.len() >= 2);
    assert!(app.history_index > 0);

    // History should support undo
    let initial_index = app.history_index;
    let undo_result = app.undo();
    assert!(undo_result);
    assert!(app.history_index < initial_index);
}

#[test]
fn test_message_system() {
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Set a message
    app.set_message("Test message");
    assert_eq!(app.message, Some("Test message".to_string()));

    // Message should persist
    assert!(app.message.is_some());
}

#[test]
fn test_config_defaults() {
    let config = AppConfig::default();

    // Check some important defaults
    assert_eq!(config.max_parent_node_width, 25);
    assert_eq!(config.max_leaf_node_width, 55);
    assert_eq!(config.line_spacing, 1);
    assert!(!config.show_hidden);
    assert!(!config.center_lock);
    assert!(!config.focus_lock);
    assert_eq!(config.max_undo_steps, 24); // Default is 24, not 100
}

#[test]
fn test_round_trip_with_special_characters() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("special.hmm");

    // Create content with special characters
    let content = "Root ✓\n\t子节点 🎯\n\t✗ Failed node\n\t\tNested [HIDDEN] node";
    fs::write(&file_path, content).unwrap();

    // Load and save
    let (tree, root_id) = parser::load_file(&file_path).unwrap();
    parser::save_file(&tree, root_id, &file_path).unwrap();

    // Verify content is preserved
    let saved_content = fs::read_to_string(&file_path).unwrap();
    assert!(saved_content.contains("Root ✓"));
    assert!(saved_content.contains("子节点 🎯"));
    assert!(saved_content.contains("✗ Failed node"));
}

#[test]
fn test_empty_file_handling() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("empty.hmm");

    // Create empty file
    fs::write(&file_path, "").unwrap();

    // Should create default map
    let (tree, root_id) = parser::load_file(&file_path).unwrap();
    assert_eq!(tree.count(), 1);
    assert_eq!(tree.get(root_id).unwrap().get().title, "New Mind Map");
}

#[test]
fn test_node_hidden_state() {
    let mut node = Node::new("Test".to_string());
    assert!(!node.is_hidden());

    node.title = "[HIDDEN] Test".to_string();
    assert!(node.is_hidden());

    node.title = "Test".to_string();
    node.is_hidden = true;
    assert!(node.is_hidden());
}

#[test]
fn test_clipboard_functionality() {
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Set clipboard content
    let clipboard_text = "Node 1\n\tChild 1\n\tChild 2";
    app.clipboard = Some(clipboard_text.to_string());

    assert_eq!(app.clipboard, Some(clipboard_text.to_string()));
}

#[test]
fn test_viewport_navigation() {
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Create a tree that extends beyond viewport
    let root = app.tree.new_node(Node::new("Root".to_string()));
    app.root_id = Some(root);
    app.active_node_id = Some(root);

    // Add many children to force scrolling
    for i in 0..30 {
        let child = app.tree.new_node(Node::new(format!("Child {}", i)));
        root.append(child, &mut app.tree);
    }

    // Set a small terminal height to test scrolling
    app.terminal_height = 10;
    app.terminal_width = 80;

    // Initial viewport should be at top
    assert_eq!(app.viewport_top, 0.0);

    // Move down beyond visible area (this would trigger scrolling in a real app)
    // The actual viewport adjustment happens in the render phase
    let last_child = root.children(&app.tree).next_back().unwrap();
    app.active_node_id = Some(last_child);

    // Test center lock functionality
    app.config.center_lock = true;
    assert!(app.config.center_lock);

    app.config.center_lock = false;
    assert!(!app.config.center_lock);
}

#[test]
fn test_complex_tree_operations() {
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Create a more complex tree
    let root = app.tree.new_node(Node::new("Project".to_string()));
    let frontend = app.tree.new_node(Node::new("Frontend".to_string()));
    let backend = app.tree.new_node(Node::new("Backend".to_string()));
    let database = app.tree.new_node(Node::new("Database".to_string()));

    let react = app.tree.new_node(Node::new("React".to_string()));
    let vue = app.tree.new_node(Node::new("Vue".to_string()));
    let node = app.tree.new_node(Node::new("Node.js".to_string()));
    let python = app.tree.new_node(Node::new("Python".to_string()));
    let postgres = app.tree.new_node(Node::new("PostgreSQL".to_string()));
    let mongodb = app.tree.new_node(Node::new("MongoDB".to_string()));

    // Build tree structure
    root.append(frontend, &mut app.tree);
    root.append(backend, &mut app.tree);
    root.append(database, &mut app.tree);

    frontend.append(react, &mut app.tree);
    frontend.append(vue, &mut app.tree);
    backend.append(node, &mut app.tree);
    backend.append(python, &mut app.tree);
    database.append(postgres, &mut app.tree);
    database.append(mongodb, &mut app.tree);

    app.root_id = Some(root);
    app.active_node_id = Some(root);

    // Test tree has correct structure
    assert_eq!(app.tree.count(), 10);
    assert_eq!(root.children(&app.tree).count(), 3);
    assert_eq!(frontend.children(&app.tree).count(), 2);

    // Test collapsing a branch
    app.tree.get_mut(frontend).unwrap().get_mut().is_collapsed = true;

    // Frontend's children should still exist but not be visible when collapsed
    assert_eq!(frontend.children(&app.tree).count(), 2);
    assert!(app.tree.get(frontend).unwrap().get().is_collapsed);
}

#[test]
fn test_paste_operations() {
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Create initial tree
    let root = app.tree.new_node(Node::new("Root".to_string()));
    let child1 = app.tree.new_node(Node::new("Child1".to_string()));
    let child2 = app.tree.new_node(Node::new("Child2".to_string()));

    root.append(child1, &mut app.tree);
    root.append(child2, &mut app.tree);

    app.root_id = Some(root);
    app.active_node_id = Some(child1);

    // Copy content to clipboard
    app.clipboard = Some("NewNode1\n\tSubNode1\nNewNode2".to_string());

    // Test paste as children - would add to child1
    let _initial_count = app.tree.count();
    // Note: paste_as_children would be called here via actions
    // We're testing the setup for it

    assert!(app.clipboard.is_some());
    assert_eq!(app.active_node_id, Some(child1));

    // Test paste as siblings - would add as siblings to child1
    app.clipboard = Some("Sibling1\nSibling2".to_string());

    // Verify clipboard is ready for paste operations
    assert!(app.clipboard.is_some());
}
