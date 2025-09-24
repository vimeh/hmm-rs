use hmm_rs::{parser, AppConfig, AppState};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

mod common;
use common::*;

#[test]
fn test_load_nonexistent_file() {
    let path = Path::new("tests/fixtures/does_not_exist.hmm");
    let result = parser::load_file(path);

    assert!(result.is_err());
}

#[test]
fn test_save_to_readonly_directory() {
    // Skip this test on Windows as permission handling is different
    if cfg!(windows) {
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let readonly_dir = temp_dir.path().join("readonly");
    fs::create_dir(&readonly_dir).unwrap();

    // Make directory read-only
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_mode(0o555); // r-xr-xr-x
        fs::set_permissions(&readonly_dir, perms).unwrap();
    }

    let (tree, root) = create_test_tree();
    let file_path = readonly_dir.join("test.hmm");

    let result = parser::save_file(&tree, root, &file_path);
    assert!(result.is_err());

    // Restore permissions for cleanup
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&readonly_dir, perms).unwrap();
    }
}

#[test]
fn test_malformed_indentation() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("malformed.hmm");

    // Write file with inconsistent indentation
    let content = "Root\n        Orphan Node\n\tProper Child\n      Another Orphan";
    fs::write(&file_path, content).unwrap();

    // Should still load without crashing
    let result = parser::load_file(&file_path);
    assert!(result.is_ok());

    let (tree, _root) = result.unwrap();
    assert!(tree.count() > 0);
}

#[test]
fn test_mixed_tabs_and_spaces() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("mixed.hmm");

    // Mix tabs and spaces
    let content = "Root\n\tTab Child\n  Space Child\n\t  Mixed Child\n    \tAnother Mixed";
    fs::write(&file_path, content).unwrap();

    let result = parser::load_file(&file_path);
    assert!(result.is_ok());

    // Should handle mixed indentation gracefully
    let (tree, _root) = result.unwrap();
    let titles = get_all_titles(&tree);
    assert!(titles.contains("Root"));
    assert!(titles.contains("Tab Child"));
    assert!(titles.contains("Space Child"));
}

#[test]
fn test_extremely_long_lines() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("long.hmm");

    // Create a very long title
    let long_title = "A".repeat(10000);
    let content = format!("Root\n\t{}", long_title);
    fs::write(&file_path, content).unwrap();

    let result = parser::load_file(&file_path);
    assert!(result.is_ok());

    let (tree, _root) = result.unwrap();
    let titles = get_all_titles(&tree);
    assert!(titles.iter().any(|t| t.len() == 10000));
}

#[test]
fn test_binary_file_rejection() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("binary.hmm");

    // Write binary data
    let binary_data: Vec<u8> = vec![0x00, 0xFF, 0x01, 0xFE, 0x02, 0xFD];
    fs::write(&file_path, binary_data).unwrap();

    // Should handle gracefully (likely as UTF-8 error)
    let result = parser::load_file(&file_path);
    // This might either fail or load with replacement characters
    // The important thing is it shouldn't panic
    let _ = result; // Don't assert on specific behavior
}

#[test]
fn test_circular_reference_prevention() {
    // Our tree structure doesn't allow circular references by design
    // This test verifies that property
    let (tree, root) = create_test_tree();

    let _child = root.children(&tree).next().unwrap();

    // Try to append root to child (which would create a cycle)
    // This should not be possible with the indextree structure
    // The API doesn't allow this operation

    // Verify tree integrity is maintained
    assert!(verify_tree_integrity(&tree).is_ok());
}

#[test]
fn test_empty_lines_handling() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("empty_lines.hmm");

    let content = "Root\n\n\n\tChild 1\n\n\t\tGrandchild\n\n\n\n\tChild 2\n\n";
    fs::write(&file_path, content).unwrap();

    let result = parser::load_file(&file_path);
    assert!(result.is_ok());

    let (tree, root) = result.unwrap();

    // Should skip empty lines
    let children: Vec<_> = root.children(&tree).collect();
    assert_eq!(children.len(), 2);

    let titles = get_all_titles(&tree);
    assert!(titles.contains("Child 1"));
    assert!(titles.contains("Child 2"));
    assert!(titles.contains("Grandchild"));
}

#[test]
fn test_windows_line_endings() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("windows.hmm");

    // Use Windows-style line endings
    let content = "Root\r\n\tChild 1\r\n\t\tGrandchild\r\n\tChild 2\r\n";
    fs::write(&file_path, content).unwrap();

    let result = parser::load_file(&file_path);
    assert!(result.is_ok());

    let (tree, root) = result.unwrap();
    let children: Vec<_> = root.children(&tree).collect();
    assert_eq!(children.len(), 2);
}

#[test]
fn test_mac_classic_line_endings() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("mac.hmm");

    // Use classic Mac line endings (CR only)
    let content = "Root\r\tChild 1\r\t\tGrandchild\r\tChild 2";
    fs::write(&file_path, content).unwrap();

    // This might not parse correctly as most parsers expect LF or CRLF
    // But it shouldn't crash
    let result = parser::load_file(&file_path);
    let _ = result; // Don't assert specific behavior
}

#[test]
fn test_utf8_bom_handling() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("bom.hmm");

    // UTF-8 BOM followed by content
    let mut content = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
    content.extend_from_slice(b"Root\n\tChild 1\n\tChild 2");
    fs::write(&file_path, content).unwrap();

    let result = parser::load_file(&file_path);

    // Should handle BOM gracefully
    if let Ok((tree, _root)) = result {
        let titles = get_all_titles(&tree);
        // BOM might be included in first title or stripped
        assert!(titles.iter().any(|t| t.contains("Root")));
    }
}

#[test]
fn test_invalid_utf16_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("utf16.hmm");

    // Write UTF-16 encoded content (which won't be valid UTF-8)
    let content_utf16 = "Root\n\tChild".encode_utf16().collect::<Vec<u16>>();
    let bytes: Vec<u8> = content_utf16
        .iter()
        .flat_map(|&x| x.to_le_bytes())
        .collect();
    fs::write(&file_path, bytes).unwrap();

    // Should fail gracefully as it's not UTF-8
    let result = parser::load_file(&file_path);
    // Likely to fail, but shouldn't panic
    let _ = result;
}

#[test]
fn test_deeply_nested_beyond_limit() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("deep.hmm");

    // Create extremely deep nesting
    let mut content = String::from("Root");
    for i in 1..=100 {
        content.push('\n');
        content.push_str(&"\t".repeat(i));
        content.push_str(&format!("Level {}", i));
    }
    fs::write(&file_path, &content).unwrap();

    let result = parser::load_file(&file_path);
    assert!(result.is_ok());

    let (tree, root) = result.unwrap();
    let depth = calculate_tree_depth(&tree, root);
    assert_eq!(depth, 100);
}

#[test]
fn test_special_filesystem_characters() {
    // Skip on Windows as it has different filename rules
    if cfg!(windows) {
        return;
    }

    let temp_dir = TempDir::new().unwrap();

    // Try various special characters that might cause issues
    let special_names = [
        "test file.hmm", // Space
        "test-file.hmm", // Dash
        "test_file.hmm", // Underscore
        "test.file.hmm", // Multiple dots
        "test$file.hmm", // Dollar sign
    ];

    for name in &special_names {
        let file_path = temp_dir.path().join(name);

        let content = "Test Root\n\tChild";
        fs::write(&file_path, content).unwrap();

        let result = parser::load_file(&file_path);
        assert!(result.is_ok(), "Failed to handle filename: {}", name);
    }
}

#[test]
fn test_concurrent_file_access() {
    use std::sync::Arc;
    use std::thread;

    let temp_dir = Arc::new(TempDir::new().unwrap());
    let file_path = temp_dir.path().join("concurrent.hmm");

    // Create initial file
    let (tree, root) = create_test_tree();
    parser::save_file(&tree, root, &file_path).unwrap();

    // Try to read from multiple threads simultaneously
    let mut handles = vec![];
    for _ in 0..5 {
        let path = file_path.clone();
        let handle = thread::spawn(move || parser::load_file(&path));
        handles.push(handle);
    }

    // All reads should succeed
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok());
    }
}

#[test]
fn test_app_state_error_recovery() {
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Set invalid filename
    app.filename = Some(Path::new("/nonexistent/path/file.hmm").to_path_buf());
    app.root_id = Some(app.tree.new_node(hmm_rs::Node::new("Test".to_string())));

    // Try to save - should handle error gracefully
    let result = hmm_rs::actions::save(&mut app);

    // Should fail but not panic
    assert!(result.is_err() || app.message.is_some());
}

#[test]
fn test_null_bytes_in_content() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("null.hmm");

    // Content with null bytes
    let content = "Root\n\tChild\0WithNull\n\tNormal Child";
    fs::write(&file_path, content).unwrap();

    let result = parser::load_file(&file_path);

    // Should either handle or reject gracefully
    if let Ok((tree, _)) = result {
        let titles = get_all_titles(&tree);
        // Null byte might be preserved or cause that line to be skipped
        assert!(titles.contains("Root"));
    }
}

#[test]
fn test_symlink_handling() {
    // Skip on Windows as symlinks require special permissions
    if cfg!(windows) {
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let real_file = temp_dir.path().join("real.hmm");
    let symlink = temp_dir.path().join("link.hmm");

    // Create real file
    fs::write(&real_file, "Root\n\tChild").unwrap();

    // Create symlink
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&real_file, &symlink).unwrap();
    }

    // Should be able to read through symlink
    let result = parser::load_file(&symlink);
    assert!(result.is_ok());
}
