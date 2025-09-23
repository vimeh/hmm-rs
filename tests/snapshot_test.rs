use hmm_rs::app::{AppMode, AppState};
use hmm_rs::config::AppConfig;
use hmm_rs::model::Node;
use hmm_rs::ui;
use insta::assert_snapshot;
use ratatui::{backend::TestBackend, Terminal};

fn create_test_app_with_tree() -> AppState {
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Create a sample tree
    let root = app.tree.new_node(Node::new("Mind Map Root".to_string()));
    let feature = app.tree.new_node(Node::new("Features".to_string()));
    let task1 = app.tree.new_node(Node::new("✓ Completed Task".to_string()));
    let task2 = app.tree.new_node(Node::new("✗ Failed Task".to_string()));
    let architecture = app.tree.new_node(Node::new("Architecture".to_string()));
    let module1 = app.tree.new_node(Node::new("model.rs".to_string()));
    let module2 = app.tree.new_node(Node::new("ui.rs".to_string()));

    root.append(feature, &mut app.tree);
    root.append(architecture, &mut app.tree);
    feature.append(task1, &mut app.tree);
    feature.append(task2, &mut app.tree);
    architecture.append(module1, &mut app.tree);
    architecture.append(module2, &mut app.tree);

    app.root_id = Some(root);
    app.active_node_id = Some(root);

    app
}

#[test]
fn test_render_empty_mindmap() {
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    let root = app.tree.new_node(Node::new("Empty Map".to_string()));
    app.root_id = Some(root);
    app.active_node_id = Some(root);

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();

    assert_snapshot!(terminal.backend());
}

#[test]
fn test_render_simple_tree() {
    let mut app = create_test_app_with_tree();

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();

    assert_snapshot!(terminal.backend());
}

#[test]
fn test_render_with_collapsed_node() {
    let mut app = create_test_app_with_tree();

    // Collapse the Features node
    let features_id = app.root_id.unwrap().children(&app.tree).next().unwrap();
    app.tree
        .get_mut(features_id)
        .unwrap()
        .get_mut()
        .is_collapsed = true;

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();

    assert_snapshot!(terminal.backend());
}

#[test]
fn test_render_with_active_node() {
    let mut app = create_test_app_with_tree();

    // Set active node to a child
    let features_id = app.root_id.unwrap().children(&app.tree).next().unwrap();
    app.active_node_id = Some(features_id);

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();

    assert_snapshot!(terminal.backend());
}

#[test]
fn test_render_edit_mode() {
    let mut app = create_test_app_with_tree();

    app.mode = AppMode::Editing {
        buffer: "Editing this node".to_string(),
        cursor_pos: 17,
    };

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();

    assert_snapshot!(terminal.backend());
}

#[test]
fn test_render_search_mode() {
    let mut app = create_test_app_with_tree();

    app.mode = AppMode::Search {
        query: "test search".to_string(),
    };

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();

    assert_snapshot!(terminal.backend());
}

#[test]
fn test_render_help_screen() {
    let mut app = create_test_app_with_tree();

    app.mode = AppMode::Help;

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();

    assert_snapshot!(terminal.backend());
}

#[test]
fn test_render_with_message() {
    let mut app = create_test_app_with_tree();

    app.set_message("File saved successfully!");

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();

    assert_snapshot!(terminal.backend());
}

#[test]
fn test_render_with_hidden_node() {
    let mut app = create_test_app_with_tree();

    // Mark a node as hidden
    let features_id = app.root_id.unwrap().children(&app.tree).next().unwrap();
    let task1_id = features_id.children(&app.tree).next().unwrap();
    app.tree.get_mut(task1_id).unwrap().get_mut().title = "[HIDDEN] Secret Task".to_string();

    // Hide hidden nodes
    app.config.show_hidden = false;

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();

    assert_snapshot!(terminal.backend());
}

#[test]
fn test_render_deep_tree() {
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Create a deeper tree
    let root = app.tree.new_node(Node::new("Root".to_string()));
    let l1 = app.tree.new_node(Node::new("Level 1".to_string()));
    let l2 = app.tree.new_node(Node::new("Level 2".to_string()));
    let l3 = app.tree.new_node(Node::new("Level 3".to_string()));
    let l4 = app.tree.new_node(Node::new("Level 4".to_string()));

    root.append(l1, &mut app.tree);
    l1.append(l2, &mut app.tree);
    l2.append(l3, &mut app.tree);
    l3.append(l4, &mut app.tree);

    app.root_id = Some(root);
    app.active_node_id = Some(l3);

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();

    assert_snapshot!(terminal.backend());
}

#[test]
fn test_render_with_symbols() {
    let mut app = create_test_app_with_tree();

    // Add symbols to some nodes
    let root = app.root_id.unwrap();
    let features_id = root.children(&app.tree).next().unwrap();
    let task1 = features_id.children(&app.tree).next().unwrap();
    let task2 = features_id.children(&app.tree).nth(1).unwrap();

    // Mark first task as done (symbol1)
    app.tree.get_mut(task1).unwrap().get_mut().title =
        format!("{} Completed Task", app.config.symbol1);

    // Mark second task as failed (symbol2)
    app.tree.get_mut(task2).unwrap().get_mut().title =
        format!("{} Failed Task", app.config.symbol2);

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();

    assert_snapshot!(terminal.backend());
}

#[test]
fn test_render_wide_tree() {
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Create tree with very long node titles
    let root = app.tree.new_node(Node::new(
        "This is a very long root node title that should wrap when displayed in the terminal".to_string()
    ));

    let child1 = app.tree.new_node(Node::new(
        "Another extremely long child node title that exceeds normal width constraints and needs proper text wrapping to display correctly".to_string()
    ));

    let child2 = app.tree.new_node(Node::new(
        "Short child".to_string()
    ));

    let grandchild = app.tree.new_node(Node::new(
        "This grandchild also has a considerably long title that will test the wrapping behavior at deeper levels of the tree structure".to_string()
    ));

    root.append(child1, &mut app.tree);
    root.append(child2, &mut app.tree);
    child1.append(grandchild, &mut app.tree);

    app.root_id = Some(root);
    app.active_node_id = Some(root);

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();

    assert_snapshot!(terminal.backend());
}

#[test]
fn test_render_very_deep_tree() {
    let config = AppConfig::default();
    let mut app = AppState::new(config);

    // Create a very deep tree (10+ levels)
    let mut current = app.tree.new_node(Node::new("Level 0 (Root)".to_string()));
    app.root_id = Some(current);
    app.active_node_id = Some(current);

    for i in 1..=10 {
        let child = app.tree.new_node(Node::new(format!("Level {}", i)));
        current.append(child, &mut app.tree);
        current = child;
    }

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();

    assert_snapshot!(terminal.backend());
}

#[test]
fn test_render_with_viewport_offset() {
    let mut app = create_test_app_with_tree();

    // Simulate viewport scrolled right
    app.viewport_left = 10.0;

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();

    assert_snapshot!(terminal.backend());
}
