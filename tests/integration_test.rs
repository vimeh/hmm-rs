use hmm_rs::{AppConfig, AppState, Node, NodeId, parser};
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