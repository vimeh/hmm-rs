use hmm_rs::{actions, app::AppState, config::AppConfig, model::Node, parser};
use indextree::Arena;
use std::fs;
use std::path::{Path, PathBuf};

/// Helper function to get fixture path
fn fixture_path(name: &str) -> PathBuf {
    Path::new("tests/fixtures").join(name)
}

/// Helper function to count total nodes in tree
#[allow(dead_code)]
fn count_all_nodes(tree: &Arena<Node>) -> usize {
    tree.iter().filter(|n| !n.is_removed()).count()
}

/// Helper function to count nodes at a specific depth
fn count_nodes_at_depth(tree: &Arena<Node>, root_id: hmm_rs::NodeId, target_depth: usize) -> usize {
    fn traverse(
        tree: &Arena<Node>,
        node_id: hmm_rs::NodeId,
        current_depth: usize,
        target_depth: usize,
    ) -> usize {
        if current_depth == target_depth {
            return 1;
        }

        node_id
            .children(tree)
            .map(|child| traverse(tree, child, current_depth + 1, target_depth))
            .sum()
    }

    traverse(tree, root_id, 0, target_depth)
}

/// Helper to verify tree structure matches expected format
fn verify_tree_structure(tree: &Arena<Node>, root_id: hmm_rs::NodeId, expected_title: &str) {
    let root = tree.get(root_id).expect("Root should exist");
    assert_eq!(root.get().title, expected_title);
}

#[test]
fn test_load_all_fixtures() {
    let fixtures = [
        "simple.hmm",
        "complex.hmm",
        "unicode.hmm",
        "large.hmm",
        "edge_cases.hmm",
        "symbols.hmm",
        "hidden.hmm",
        "markdown.hmm",
        "project.hmm",
        "empty.hmm",
        "single_node.hmm",
        "test.hmm",
        "test_input.hmm",
    ];

    for fixture in fixtures {
        let path = fixture_path(fixture);
        let result = parser::load_file(&path);
        assert!(
            result.is_ok(),
            "Failed to load {}: {:?}",
            fixture,
            result.err()
        );

        let (tree, root_id) = result.unwrap();
        assert!(
            tree.count() > 0,
            "Tree should have at least one node for {}",
            fixture
        );
        assert!(
            tree.get(root_id).is_some(),
            "Root node should exist for {}",
            fixture
        );
    }
}

#[test]
fn test_round_trip_all_fixtures() {
    let fixtures = [
        "simple.hmm",
        "complex.hmm",
        "unicode.hmm",
        "symbols.hmm",
        "project.hmm",
    ];

    for fixture in fixtures {
        let path = fixture_path(fixture);

        // Load original
        let (tree1, root1) = parser::load_file(&path).unwrap();
        let original_content = fs::read_to_string(&path).unwrap();

        // Save to temp file
        let temp_path = path.with_extension("tmp");
        parser::save_file(&tree1, root1, &temp_path).unwrap();

        // Load saved version
        let (tree2, root2) = parser::load_file(&temp_path).unwrap();
        let saved_content = fs::read_to_string(&temp_path).unwrap();

        // Compare
        assert_eq!(
            tree1.count(),
            tree2.count(),
            "Node count mismatch for {} after round-trip",
            fixture
        );
        assert_eq!(
            tree1.get(root1).unwrap().get().title,
            tree2.get(root2).unwrap().get().title,
            "Root title mismatch for {} after round-trip",
            fixture
        );

        // For most files, content should be identical
        if !fixture.contains("edge_cases") {
            assert_eq!(
                original_content.trim(),
                saved_content.trim(),
                "Content mismatch for {} after round-trip",
                fixture
            );
        }

        // Clean up
        fs::remove_file(temp_path).ok();
    }
}

#[test]
fn test_simple_fixture_structure() {
    let path = fixture_path("simple.hmm");
    let (tree, root_id) = parser::load_file(&path).unwrap();

    verify_tree_structure(&tree, root_id, "Root Node");

    let children: Vec<_> = root_id.children(&tree).collect();
    assert_eq!(children.len(), 3);

    assert_eq!(tree.get(children[0]).unwrap().get().title, "Child 1");
    assert_eq!(tree.get(children[1]).unwrap().get().title, "Child 2");
    assert_eq!(tree.get(children[2]).unwrap().get().title, "Child 3");
}

#[test]
fn test_complex_fixture_structure() {
    let path = fixture_path("complex.hmm");
    let (tree, root_id) = parser::load_file(&path).unwrap();

    verify_tree_structure(&tree, root_id, "Project Management System");

    // Should have 4 main phases
    assert_eq!(count_nodes_at_depth(&tree, root_id, 1), 4);

    // Verify some specific nodes exist
    let content = parser::map_to_list(&tree, root_id, false, 0);
    assert!(content.contains("Planning Phase"));
    assert!(content.contains("Development Phase"));
    assert!(content.contains("Testing Phase"));
    assert!(content.contains("Deployment"));
    assert!(content.contains("Requirements Gathering"));
    assert!(content.contains("Unit Testing"));
}

#[test]
fn test_unicode_fixture() {
    let path = fixture_path("unicode.hmm");
    let (tree, root_id) = parser::load_file(&path).unwrap();

    verify_tree_structure(&tree, root_id, "International Project 🌍");

    // Verify Unicode content is preserved
    let content = parser::map_to_list(&tree, root_id, false, 0);
    assert!(content.contains("日本語 (Japanese)"));
    assert!(content.contains("中文 (Chinese)"));
    assert!(content.contains("한국어 (Korean)"));
    assert!(content.contains("العربية"));
    assert!(content.contains("עברית"));
    assert!(content.contains("Кириллица"));
    assert!(content.contains("✓ Complete"));
    assert!(content.contains("🔥 Hot Priority"));
}

#[test]
fn test_large_fixture_performance() {
    use std::time::Instant;

    let path = fixture_path("large.hmm");

    let start = Instant::now();
    let (tree, root_id) = parser::load_file(&path).unwrap();
    let load_time = start.elapsed();

    // Should load reasonably fast (under 100ms for this size)
    assert!(
        load_time.as_millis() < 100,
        "Load time too slow: {:?}",
        load_time
    );

    // Verify structure
    verify_tree_structure(&tree, root_id, "Large Dataset Test");
    assert!(tree.count() > 50, "Large fixture should have many nodes");

    // Test save performance
    let temp_path = path.with_extension("tmp");
    let start = Instant::now();
    parser::save_file(&tree, root_id, &temp_path).unwrap();
    let save_time = start.elapsed();

    assert!(
        save_time.as_millis() < 100,
        "Save time too slow: {:?}",
        save_time
    );

    fs::remove_file(temp_path).ok();
}

#[test]
fn test_edge_cases_fixture() {
    let path = fixture_path("edge_cases.hmm");
    let (tree, root_id) = parser::load_file(&path).unwrap();

    verify_tree_structure(&tree, root_id, "Edge Case Tests");

    // Check that empty titles are handled
    let content = parser::map_to_list(&tree, root_id, false, 0);

    // Verify various edge cases are preserved
    assert!(content.contains("Very Long Title"));
    assert!(content.contains("Single Character"));
    assert!(content.contains("123"));
    assert!(content.contains("Mixed Indentation"));
}

#[test]
fn test_hidden_nodes_fixture() {
    let path = fixture_path("hidden.hmm");
    let (tree, root_id) = parser::load_file(&path).unwrap();

    verify_tree_structure(&tree, root_id, "Visibility Test");

    // Count hidden nodes
    let hidden_count = tree
        .iter()
        .filter(|node| !node.is_removed() && node.get().is_hidden())
        .count();

    assert!(hidden_count > 0, "Should have hidden nodes");

    // Verify hidden nodes are saved correctly
    let content = parser::map_to_list(&tree, root_id, false, 0);
    assert!(content.contains("[HIDDEN] Secret Node"));
    assert!(content.contains("[HIDDEN] Private Section"));
}

#[test]
fn test_symbols_fixture() {
    let path = fixture_path("symbols.hmm");
    let (tree, root_id) = parser::load_file(&path).unwrap();

    verify_tree_structure(&tree, root_id, "Task Management");

    // Verify symbols are preserved
    let content = parser::map_to_list(&tree, root_id, false, 0);
    assert!(content.contains("✓ Completed Task"));
    assert!(content.contains("✗ Failed Task"));
    assert!(content.contains("→ In Progress"));
    assert!(content.contains("⚠️ Warning"));
    assert!(content.contains("🔴"));
    assert!(content.contains("∞ Infinite Loop"));
}

#[test]
fn test_markdown_fixture() {
    let path = fixture_path("markdown.hmm");
    let (tree, root_id) = parser::load_file(&path).unwrap();

    verify_tree_structure(&tree, root_id, "Documentation Project");

    // Verify markdown syntax is preserved as plain text
    let content = parser::map_to_list(&tree, root_id, false, 0);
    assert!(content.contains("# Main Heading"));
    assert!(content.contains("**Bold Text**"));
    assert!(content.contains("*Italic Text*"));
    assert!(content.contains("`inline code`"));
    assert!(content.contains("[Link Text](https://example.com)"));
}

#[test]
fn test_project_fixture_navigation() {
    let path = fixture_path("project.hmm");
    let (tree, root_id) = parser::load_file(&path).unwrap();

    verify_tree_structure(&tree, root_id, "E-Commerce Platform");

    // Test navigation through the tree
    let main_sections: Vec<_> = root_id.children(&tree).collect();
    assert_eq!(main_sections.len(), 5); // Frontend, Backend, Database, DevOps, Testing

    // Check Frontend structure
    let frontend = main_sections[0];
    assert_eq!(tree.get(frontend).unwrap().get().title, "Frontend");

    let frontend_children: Vec<_> = frontend.children(&tree).collect();
    assert_eq!(frontend_children[0].children(&tree).count(), 3); // React Application has 3 subsections
}

#[test]
fn test_empty_file_handling() {
    let path = fixture_path("empty.hmm");
    let (tree, root_id) = parser::load_file(&path).unwrap();

    // Should create default node for empty file
    assert_eq!(tree.count(), 1);
    assert_eq!(tree.get(root_id).unwrap().get().title, "New Mind Map");
}

#[test]
fn test_single_node_file() {
    let path = fixture_path("single_node.hmm");
    let (tree, root_id) = parser::load_file(&path).unwrap();

    // Should have exactly 2 nodes (root + synthetic root)
    assert_eq!(tree.count(), 2);
    assert_eq!(tree.get(root_id).unwrap().get().title, "Just One Node");
    assert_eq!(root_id.children(&tree).count(), 0);
}

#[test]
fn test_modify_and_save() {
    let path = fixture_path("simple.hmm");
    let (mut tree, root_id) = parser::load_file(&path).unwrap();

    // Add a new node
    let new_node = tree.new_node(Node::new("New Child".to_string()));
    root_id.append(new_node, &mut tree);

    // Save to temp file
    let temp_path = path.with_extension("modified");
    parser::save_file(&tree, root_id, &temp_path).unwrap();

    // Load and verify
    let (tree2, root2) = parser::load_file(&temp_path).unwrap();
    let children: Vec<_> = root2.children(&tree2).collect();
    assert_eq!(children.len(), 4); // Original 3 + 1 new

    // Clean up
    fs::remove_file(temp_path).ok();
}

#[test]
fn test_collapse_state_not_affecting_save() {
    let path = fixture_path("complex.hmm");
    let (mut tree, root_id) = parser::load_file(&path).unwrap();

    // Collapse some nodes
    for node in tree.iter_mut() {
        if !node.is_removed() && node.get().title.contains("Sprint") {
            node.get_mut().is_collapsed = true;
        }
    }

    // Save and reload
    let temp_path = path.with_extension("collapsed");
    parser::save_file(&tree, root_id, &temp_path).unwrap();
    let (tree2, _) = parser::load_file(&temp_path).unwrap();

    // All nodes should still be present
    assert_eq!(tree.count(), tree2.count());

    fs::remove_file(temp_path).ok();
}

#[test]
fn test_app_state_with_fixtures() {
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Load a fixture into app state
    let path = fixture_path("project.hmm");
    let (tree, root_id) = parser::load_file(&path).unwrap();

    app.tree = tree;
    app.root_id = Some(root_id);
    app.active_node_id = Some(root_id);
    app.filename = Some(path.clone());

    // Test that app state is properly initialized
    assert_eq!(app.root_id, Some(root_id));
    assert!(app.tree.count() > 10);

    // Test save through app actions
    app.is_dirty = true;
    actions::save(&mut app).unwrap();
    assert!(!app.is_dirty);
}

#[test]
fn test_tree_depth_calculation() {
    fn calculate_max_depth(tree: &Arena<Node>, node_id: hmm_rs::NodeId) -> usize {
        let children: Vec<_> = node_id.children(tree).collect();
        if children.is_empty() {
            0
        } else {
            1 + children
                .iter()
                .map(|&child| calculate_max_depth(tree, child))
                .max()
                .unwrap_or(0)
        }
    }

    // Test with different fixtures
    let fixtures = [
        ("simple.hmm", 1),      // Root -> Children
        ("single_node.hmm", 0), // Just root
        ("complex.hmm", 4),     // Deep nesting
    ];

    for (fixture, expected_min_depth) in fixtures {
        let path = fixture_path(fixture);
        let (tree, root_id) = parser::load_file(&path).unwrap();
        let depth = calculate_max_depth(&tree, root_id);
        assert!(
            depth >= expected_min_depth,
            "Depth for {} should be at least {}, got {}",
            fixture,
            expected_min_depth,
            depth
        );
    }
}
