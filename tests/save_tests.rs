use hmm_rs::{
    actions::{save, save_as},
    app::AppState,
    config::AppConfig,
    model::Node,
    parser,
};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_save_creates_file_with_correct_format() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_save.hmm");

    // Create app with tree
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Build tree structure
    let root = app.tree.new_node(Node::new("My Project".to_string()));
    let task1 = app.tree.new_node(Node::new("Task 1".to_string()));
    let task2 = app.tree.new_node(Node::new("Task 2".to_string()));
    let subtask1 = app.tree.new_node(Node::new("Subtask 1.1".to_string()));
    let subtask2 = app.tree.new_node(Node::new("Subtask 2.1".to_string()));
    let subtask3 = app.tree.new_node(Node::new("Subtask 2.2".to_string()));

    root.append(task1, &mut app.tree);
    root.append(task2, &mut app.tree);
    task1.append(subtask1, &mut app.tree);
    task2.append(subtask2, &mut app.tree);
    task2.append(subtask3, &mut app.tree);

    app.root_id = Some(root);
    app.filename = Some(file_path.clone());

    // Save the file
    save(&mut app).unwrap();

    // Verify file exists and has correct content
    assert!(file_path.exists());
    let content = fs::read_to_string(&file_path).unwrap();

    // Check exact format with tabs
    let expected =
        "My Project\n\tTask 1\n\t\tSubtask 1.1\n\tTask 2\n\t\tSubtask 2.1\n\t\tSubtask 2.2\n";
    assert_eq!(content, expected);
}

#[test]
fn test_save_preserves_collapsed_nodes() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("collapsed.hmm");

    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Create tree with collapsed node
    let root = app.tree.new_node(Node::new("Root".to_string()));
    let expanded = app.tree.new_node(Node::new("Expanded".to_string()));
    let collapsed = app.tree.new_node(Node::new("Collapsed".to_string()));
    let child1 = app.tree.new_node(Node::new("Child 1".to_string()));
    let child2 = app.tree.new_node(Node::new("Child 2".to_string()));

    root.append(expanded, &mut app.tree);
    root.append(collapsed, &mut app.tree);
    expanded.append(child1, &mut app.tree);
    collapsed.append(child2, &mut app.tree);

    // Collapse one node
    app.tree.get_mut(collapsed).unwrap().get_mut().is_collapsed = true;

    app.root_id = Some(root);
    app.filename = Some(file_path.clone());

    // Save
    save(&mut app).unwrap();

    // Load and verify all nodes are present
    let (loaded_tree, _loaded_root) = parser::load_file(&file_path).unwrap();

    // Count should include all nodes regardless of collapsed state
    assert_eq!(loaded_tree.count(), 6); // root + 2 branches + 2 children + synthetic root

    // Verify structure
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("Root"));
    assert!(content.contains("Expanded"));
    assert!(content.contains("Collapsed"));
    assert!(content.contains("Child 1"));
    assert!(content.contains("Child 2"));
}

#[test]
fn test_dirty_flag_tracking() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("dirty.hmm");

    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Create simple tree
    let root = app.tree.new_node(Node::new("Root".to_string()));
    app.root_id = Some(root);
    app.filename = Some(file_path.clone());

    // Initially not dirty
    assert!(!app.is_dirty);

    // Make a change by adding a child
    let child = app.tree.new_node(Node::new("Child".to_string()));
    root.append(child, &mut app.tree);
    app.is_dirty = true;

    // Should be dirty now
    assert!(app.is_dirty);

    // Save should clear dirty flag
    save(&mut app).unwrap();
    assert!(!app.is_dirty);
}

#[test]
fn test_save_without_filename() {
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    let root = app.tree.new_node(Node::new("Root".to_string()));
    app.root_id = Some(root);
    // No filename set

    // Save should handle gracefully
    let result = save(&mut app);
    assert!(result.is_ok());
    assert!(app.message.is_some());
    assert!(app.message.as_ref().unwrap().contains("No filename"));
}

#[test]
fn test_save_as_creates_new_file() {
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    let root = app.tree.new_node(Node::new("Test Map".to_string()));
    app.root_id = Some(root);

    // Initially no filename
    assert!(app.filename.is_none());

    // Save As should create default file
    save_as(&mut app).unwrap();

    // Should now have a filename
    assert!(app.filename.is_some());
    assert_eq!(
        app.filename.as_ref().unwrap(),
        &PathBuf::from("mindmap.hmm")
    );

    // File should exist
    assert!(PathBuf::from("mindmap.hmm").exists());

    // Clean up
    fs::remove_file("mindmap.hmm").ok();
}

#[test]
fn test_round_trip_preservation() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("roundtrip.hmm");

    // Create complex content
    let original_content = "Project Plan\n\tPhase 1\n\t\tDesign\n\t\t\tUI Mockups\n\t\t\tDatabase Schema\n\t\tImplementation\n\t\t\tFrontend\n\t\t\tBackend\n\tPhase 2\n\t\tTesting\n\t\t\tUnit Tests\n\t\t\tIntegration Tests\n\t\tDeployment\n\t\t\tStaging\n\t\t\tProduction\n";

    fs::write(&file_path, original_content).unwrap();

    // Load
    let (tree, root_id) = parser::load_file(&file_path).unwrap();

    // Save back
    parser::save_file(&tree, root_id, &file_path).unwrap();

    // Read saved content
    let saved_content = fs::read_to_string(&file_path).unwrap();

    // Should be identical
    assert_eq!(saved_content, original_content);
}

#[test]
fn test_save_with_special_characters() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("special.hmm");

    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Create tree with special characters
    let root = app.tree.new_node(Node::new("Root 根 🌳".to_string()));
    let child1 = app.tree.new_node(Node::new("✓ Complete".to_string()));
    let child2 = app.tree.new_node(Node::new("✗ Failed".to_string()));
    let child3 = app.tree.new_node(Node::new("→ In Progress".to_string()));

    root.append(child1, &mut app.tree);
    root.append(child2, &mut app.tree);
    root.append(child3, &mut app.tree);

    app.root_id = Some(root);
    app.filename = Some(file_path.clone());

    // Save
    save(&mut app).unwrap();

    // Verify special characters are preserved
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("Root 根 🌳"));
    assert!(content.contains("✓ Complete"));
    assert!(content.contains("✗ Failed"));
    assert!(content.contains("→ In Progress"));
}

#[test]
fn test_save_empty_tree() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("empty.hmm");

    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Just a root node
    let root = app.tree.new_node(Node::new("Solo Node".to_string()));
    app.root_id = Some(root);
    app.filename = Some(file_path.clone());

    // Save
    save(&mut app).unwrap();

    // Should create file with just the root
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "Solo Node\n");
}

#[test]
fn test_save_deeply_nested_tree() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("deep.hmm");

    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Create deeply nested structure
    let mut current = app.tree.new_node(Node::new("Level 0".to_string()));
    app.root_id = Some(current);

    for i in 1..10 {
        let next = app.tree.new_node(Node::new(format!("Level {}", i)));
        current.append(next, &mut app.tree);
        current = next;
    }

    app.filename = Some(file_path.clone());

    // Save
    save(&mut app).unwrap();

    // Verify deep nesting with proper indentation
    let content = fs::read_to_string(&file_path).unwrap();
    for i in 0..10 {
        let expected_line = format!("{}Level {}", "\t".repeat(i), i);
        assert!(content.contains(&expected_line));
    }
}
