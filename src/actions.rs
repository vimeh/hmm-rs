use crate::app::{AppMode, AppState};
use crate::layout::LayoutEngine;
use crate::model::{Node, NodeId};
use crate::parser;
use anyhow::Result;
use clipboard::{ClipboardContext, ClipboardProvider};
use indextree::Arena;

#[derive(Debug, Clone)]
pub enum Action {
    // Application control
    Quit,
    ForceQuit,

    // Movement
    GoUp,
    GoDown,
    GoLeft,
    GoRight,
    GoToTop,
    GoToBottom,
    GoToRoot,

    // Node manipulation
    InsertSibling,
    InsertChild,
    DeleteNode,
    DeleteChildren,
    MoveNodeUp,
    MoveNodeDown,

    // Editing
    EditNodeAppend,
    EditNodeReplace,
    TypeChar(char),
    Backspace,
    Delete,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorHome,
    MoveCursorEnd,
    ConfirmEdit,
    CancelEdit,

    // View control
    ToggleCollapse,
    CollapseAll,
    ExpandAll,
    CollapseChildren,
    CollapseOtherBranches,
    CollapseToLevel(usize),
    CenterActiveNode,
    ToggleCenterLock,
    Focus,
    ToggleFocusLock,

    // File operations
    Save,
    SaveAs,
    ExportHtml,
    ExportText,

    // Clipboard
    YankNode,
    YankChildren,
    PasteAsChildren,
    PasteAsSiblings,

    // Undo/Redo
    Undo,
    Redo,

    // Search
    Search,
    TypeSearchChar(char),
    BackspaceSearch,
    ConfirmSearch,
    CancelSearch,
    NextSearchResult,
    PreviousSearchResult,

    // Symbols and formatting
    ToggleSymbol,
    SortSiblings,
    ToggleNumbers,
    ToggleHide,
    ToggleShowHidden,

    // Rank operations
    IncreasePositiveRank,
    DecreasePositiveRank,
    IncreaseNegativeRank,
    DecreaseNegativeRank,

    // Star rating
    AddStar,
    RemoveStar,

    // Layout
    IncreaseTextWidth,
    DecreaseTextWidth,
    IncreaseLineSpacing,
    DecreaseLineSpacing,
    ToggleAlign,

    // Help
    ShowHelp,
    CloseHelp,
}

pub fn execute_action(action: Action, app: &mut AppState) -> Result<()> {
    match action {
        Action::Quit => {
            if app.filename.is_some() {
                // TODO: Check for unsaved changes
            }
            app.running = false;
        }
        Action::ForceQuit => {
            app.running = false;
        }

        // Movement actions
        Action::GoUp => go_up(app),
        Action::GoDown => go_down(app),
        Action::GoLeft => go_left(app),
        Action::GoRight => go_right(app),
        Action::GoToRoot => go_to_root(app),
        Action::GoToTop => go_to_top(app),
        Action::GoToBottom => go_to_bottom(app),

        // Node manipulation
        Action::InsertSibling => insert_sibling(app),
        Action::InsertChild => insert_child(app),
        Action::DeleteNode => delete_node(app),
        Action::DeleteChildren => delete_children(app),
        Action::MoveNodeUp => move_node_up(app),
        Action::MoveNodeDown => move_node_down(app),

        // Editing
        Action::EditNodeAppend => start_editing(app, false),
        Action::EditNodeReplace => start_editing(app, true),
        Action::TypeChar(c) => type_char(app, c),
        Action::Backspace => backspace(app),
        Action::Delete => delete_char(app),
        Action::MoveCursorLeft => move_cursor_left(app),
        Action::MoveCursorRight => move_cursor_right(app),
        Action::MoveCursorHome => move_cursor_home(app),
        Action::MoveCursorEnd => move_cursor_end(app),
        Action::ConfirmEdit => confirm_edit(app),
        Action::CancelEdit => cancel_edit(app),

        // View control
        Action::ToggleCollapse => toggle_collapse(app),
        Action::CollapseAll => collapse_all(app),
        Action::ExpandAll => expand_all(app),
        Action::CollapseChildren => collapse_children(app),
        Action::CollapseOtherBranches => collapse_other_branches(app),
        Action::CollapseToLevel(level) => collapse_to_level(app, level),
        Action::CenterActiveNode => center_active_node(app),
        Action::ToggleCenterLock => toggle_center_lock(app),
        Action::Focus => focus(app),
        Action::ToggleFocusLock => toggle_focus_lock(app),

        // File operations
        Action::Save => save(app)?,
        Action::SaveAs => save_as(app)?,
        Action::ExportHtml => export_html(app)?,
        Action::ExportText => export_text(app)?,

        // Clipboard
        Action::YankNode => yank_node(app)?,
        Action::YankChildren => yank_children(app)?,
        Action::PasteAsChildren => paste_as_children(app)?,
        Action::PasteAsSiblings => paste_as_siblings(app)?,

        // Undo/Redo
        Action::Undo => undo(app),
        Action::Redo => redo(app),

        // Search
        Action::Search => start_search(app),
        Action::TypeSearchChar(c) => type_search_char(app, c),
        Action::BackspaceSearch => backspace_search(app),
        Action::ConfirmSearch => confirm_search(app),
        Action::CancelSearch => cancel_search(app),
        Action::NextSearchResult => next_search_result(app),
        Action::PreviousSearchResult => previous_search_result(app),

        // Symbols
        Action::ToggleSymbol => toggle_symbol(app),
        Action::SortSiblings => sort_siblings(app),
        Action::ToggleNumbers => toggle_numbers(app),
        Action::ToggleHide => toggle_hide(app),
        Action::ToggleShowHidden => toggle_show_hidden(app),

        // Layout
        Action::IncreaseTextWidth => increase_text_width(app),
        Action::DecreaseTextWidth => decrease_text_width(app),
        Action::IncreaseLineSpacing => increase_line_spacing(app),
        Action::DecreaseLineSpacing => decrease_line_spacing(app),
        Action::ToggleAlign => toggle_align(app),

        // Rank operations
        Action::IncreasePositiveRank => increase_positive_rank(app),
        Action::DecreasePositiveRank => decrease_positive_rank(app),
        Action::IncreaseNegativeRank => increase_negative_rank(app),
        Action::DecreaseNegativeRank => decrease_negative_rank(app),
        // Star rating
        Action::AddStar => add_star(app),
        Action::RemoveStar => remove_star(app),
        // Help
        Action::ShowHelp => show_help(app),
        Action::CloseHelp => close_help(app),
    }
    Ok(())
}

// Helper function to ensure active node is visible
fn ensure_node_visible(app: &mut AppState) {
    if app.config.center_lock {
        center_active_node(app);
    } else if let Some(active_id) = app.active_node_id {
        let layout = LayoutEngine::calculate_layout(app);

        if let Some(node_layout) = layout.nodes.get(&active_id) {
            let node_x = node_layout.x;
            let node_y = node_layout.y + node_layout.yo;
            let node_right = node_x + node_layout.w;
            let node_bottom = node_y + node_layout.lh;

            // Adjust viewport to ensure node is visible
            let margin = 2.0; // Small margin around the node

            // Horizontal adjustment
            if node_x < app.viewport_left + margin {
                app.viewport_left = (node_x - margin).max(0.0);
            } else if node_right > app.viewport_left + app.terminal_width as f64 - margin {
                app.viewport_left = node_right - app.terminal_width as f64 + margin;
            }

            // Vertical adjustment
            if node_y < app.viewport_top + margin {
                app.viewport_top = (node_y - margin).max(0.0);
            } else if node_bottom > app.viewport_top + app.terminal_height as f64 - margin {
                app.viewport_top = node_bottom - app.terminal_height as f64 + margin;
            }
        }
    }
}

// Movement functions
fn go_up(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        // Find the previous visible sibling or parent's previous sibling
        if let Some(parent_id) = active_id.ancestors(&app.tree).nth(1) {
            let siblings: Vec<NodeId> = parent_id.children(&app.tree).collect();
            let current_index = siblings.iter().position(|&id| id == active_id);

            if let Some(idx) = current_index {
                if idx > 0 {
                    app.active_node_id = Some(siblings[idx - 1]);
                    ensure_node_visible(app);
                } else if parent_id != app.root_id.unwrap() {
                    app.active_node_id = Some(parent_id);
                    ensure_node_visible(app);
                }
            }
        }
    }
}

fn go_down(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        // Try to go to first child
        if let Some(first_child) = active_id.children(&app.tree).next() {
            let node = app.tree.get(active_id).unwrap().get();
            if !node.is_collapsed {
                app.active_node_id = Some(first_child);
                ensure_node_visible(app);
                return;
            }
        }

        // Otherwise, find next sibling
        if let Some(parent_id) = active_id.ancestors(&app.tree).nth(1) {
            let siblings: Vec<NodeId> = parent_id.children(&app.tree).collect();
            let current_index = siblings.iter().position(|&id| id == active_id);

            if let Some(idx) = current_index {
                if idx < siblings.len() - 1 {
                    app.active_node_id = Some(siblings[idx + 1]);
                    ensure_node_visible(app);
                }
            }
        }
    }
}

fn go_left(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        if let Some(parent_id) = active_id.ancestors(&app.tree).nth(1) {
            // Allow moving to parent even if it's the root
            app.active_node_id = Some(parent_id);
            ensure_node_visible(app);
        }
    }
}

fn go_right(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        let node = app.tree.get(active_id).unwrap().get();
        if !node.is_collapsed {
            if let Some(first_child) = active_id.children(&app.tree).next() {
                app.active_node_id = Some(first_child);
                ensure_node_visible(app);
            }
        }
    }
}

fn go_to_root(app: &mut AppState) {
    app.active_node_id = app.root_id;
    ensure_node_visible(app);
}

fn go_to_top(app: &mut AppState) {
    if let Some(root_id) = app.root_id {
        app.active_node_id = Some(root_id);
        app.viewport_top = 0.0;
        app.viewport_left = 0.0;
    }
}

fn go_to_bottom(app: &mut AppState) {
    if let Some(root_id) = app.root_id {
        fn find_last_visible(tree: &indextree::Arena<Node>, node_id: NodeId) -> NodeId {
            let node = tree.get(node_id).unwrap().get();
            if node.is_collapsed {
                return node_id;
            }

            if let Some(last_child) = node_id.children(tree).next_back() {
                return find_last_visible(tree, last_child);
            }

            node_id
        }

        app.active_node_id = Some(find_last_visible(&app.tree, root_id));
        ensure_node_visible(app);
    }
}

// Node manipulation functions
fn insert_sibling(app: &mut AppState) {
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

fn insert_child(app: &mut AppState) {
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

fn delete_node(app: &mut AppState) {
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

fn delete_children(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        app.push_history();

        let children: Vec<NodeId> = active_id.children(&app.tree).collect();
        for child_id in children {
            child_id.remove(&mut app.tree);
        }
    }
}

fn move_node_up(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        if let Some(prev_sibling) = active_id.preceding_siblings(&app.tree).nth(1) {
            app.push_history();
            prev_sibling.insert_before(active_id, &mut app.tree);
        }
    }
}

fn move_node_down(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        if let Some(next_sibling) = active_id.following_siblings(&app.tree).nth(1) {
            app.push_history();
            next_sibling.insert_after(active_id, &mut app.tree);
        }
    }
}

// Editing functions
fn start_editing(app: &mut AppState, replace: bool) {
    if let Some(active_id) = app.active_node_id {
        let node = app.tree.get(active_id).unwrap().get();
        let buffer = if replace {
            String::new()
        } else {
            node.title.clone()
        };
        let cursor_pos = buffer.len();

        app.mode = AppMode::Editing { buffer, cursor_pos };
    }
}

fn type_char(app: &mut AppState, c: char) {
    if let AppMode::Editing { buffer, cursor_pos } = &mut app.mode {
        buffer.insert(*cursor_pos, c);
        *cursor_pos += 1;
    }
}

fn backspace(app: &mut AppState) {
    if let AppMode::Editing { buffer, cursor_pos } = &mut app.mode {
        if *cursor_pos > 0 {
            *cursor_pos -= 1;
            buffer.remove(*cursor_pos);
        }
    }
}

fn delete_char(app: &mut AppState) {
    if let AppMode::Editing { buffer, cursor_pos } = &mut app.mode {
        if *cursor_pos < buffer.len() {
            buffer.remove(*cursor_pos);
        }
    }
}

fn move_cursor_left(app: &mut AppState) {
    if let AppMode::Editing { cursor_pos, .. } = &mut app.mode {
        if *cursor_pos > 0 {
            *cursor_pos -= 1;
        }
    }
}

fn move_cursor_right(app: &mut AppState) {
    if let AppMode::Editing { buffer, cursor_pos } = &mut app.mode {
        if *cursor_pos < buffer.len() {
            *cursor_pos += 1;
        }
    }
}

fn move_cursor_home(app: &mut AppState) {
    if let AppMode::Editing { cursor_pos, .. } = &mut app.mode {
        *cursor_pos = 0;
    }
}

fn move_cursor_end(app: &mut AppState) {
    if let AppMode::Editing { buffer, cursor_pos } = &mut app.mode {
        *cursor_pos = buffer.len();
    }
}

fn confirm_edit(app: &mut AppState) {
    let new_title = if let AppMode::Editing { buffer, .. } = &app.mode {
        buffer.clone()
    } else {
        return;
    };

    if let Some(active_id) = app.active_node_id {
        app.push_history();

        if let Some(node) = app.tree.get_mut(active_id) {
            node.get_mut().title = new_title;
        }
    }
    app.mode = AppMode::Normal;
}

fn cancel_edit(app: &mut AppState) {
    app.mode = AppMode::Normal;
}

// View control functions
fn toggle_collapse(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        if let Some(node) = app.tree.get_mut(active_id) {
            node.get_mut().is_collapsed = !node.get().is_collapsed;
        }
    }
}

fn collapse_all(app: &mut AppState) {
    for node in app.tree.iter_mut() {
        node.get_mut().is_collapsed = true;
    }
}

fn expand_all(app: &mut AppState) {
    for node in app.tree.iter_mut() {
        node.get_mut().is_collapsed = false;
    }
}

fn collapse_children(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        let children: Vec<NodeId> = active_id.children(&app.tree).collect();
        for child_id in children {
            if let Some(node) = app.tree.get_mut(child_id) {
                node.get_mut().is_collapsed = true;
            }
        }
    }
}

fn collapse_other_branches(app: &mut AppState) {
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

fn collapse_to_level(app: &mut AppState, target_level: usize) {
    fn set_collapse_at_depth(
        tree: &mut indextree::Arena<Node>,
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

fn center_active_node(app: &mut AppState) {
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

fn toggle_center_lock(app: &mut AppState) {
    app.config.center_lock = !app.config.center_lock;
    app.set_message(format!(
        "Center lock: {}",
        if app.config.center_lock { "ON" } else { "OFF" }
    ));
}

fn focus(app: &mut AppState) {
    // TODO: Implement focus mode
    app.set_message("Focus mode not yet implemented");
}

fn toggle_focus_lock(app: &mut AppState) {
    app.config.focus_lock = !app.config.focus_lock;
    app.set_message(format!(
        "Focus lock: {}",
        if app.config.focus_lock { "ON" } else { "OFF" }
    ));
}

// File operations
fn save(app: &mut AppState) -> Result<()> {
    if let Some(ref path) = app.filename {
        if let Some(root_id) = app.root_id {
            parser::save_file(&app.tree, root_id, path)?;
            app.set_message("File saved");
        }
    } else {
        app.set_message("No filename set");
    }
    Ok(())
}

fn save_as(app: &mut AppState) -> Result<()> {
    // TODO: Implement file dialog
    app.set_message("Save As not yet implemented");
    Ok(())
}

fn export_html(app: &mut AppState) -> Result<()> {
    if app.filename.is_none() {
        app.set_message(
            "Can't export the map when it doesn't have a file name yet. Save it first.",
        );
        return Ok(());
    }

    let filename = format!("{}.html", app.filename.as_ref().unwrap().display());

    if let Some(root_id) = app.root_id {
        let root_node = app.tree.get(root_id).unwrap().get();
        let root_title = &root_node.title;

        let mut html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <title>{}</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width,initial-scale=1,user-scalable=yes">
    <style>
        body {{
            background-color: #222;
            color: #ddd;
            padding: 20px;
            font-family: monospace;
            font-size: 16px;
            line-height: 1.6;
        }}
        #root {{
            font-size: 1.5em;
            font-weight: bold;
            margin-bottom: 20px;
            padding: 10px;
            border-bottom: 2px solid #444;
        }}
        details {{
            margin-left: 20px;
            margin-top: 10px;
        }}
        summary {{
            cursor: pointer;
            padding: 5px;
            border-radius: 3px;
        }}
        summary:hover {{
            background-color: #333;
        }}
        p {{
            margin: 5px 0 5px 20px;
            padding: 5px;
        }}
    </style>
</head>
<body>
"#,
            root_title
        );

        // Generate the HTML tree
        html.push_str(&export_html_node(&app.tree, root_id, true));

        html.push_str("</body>\n</html>\n");

        // Write to file
        std::fs::write(&filename, html)?;

        app.set_message(format!("Exported to {}", filename));
    }

    Ok(())
}

fn export_html_node(tree: &Arena<Node>, node_id: NodeId, is_root: bool) -> String {
    let node = tree.get(node_id).unwrap().get();
    let mut output = String::new();

    // Get visible children (not collapsed)
    let visible_children: Vec<NodeId> = if !node.is_collapsed {
        node_id.children(tree).collect()
    } else {
        vec![]
    };

    if is_root {
        // Root node
        output.push_str(&format!("<div id=\"root\">{}</div>\n", node.title));
        for child_id in visible_children {
            output.push_str(&export_html_node(tree, child_id, false));
        }
    } else if visible_children.is_empty() {
        // Leaf node or collapsed node
        output.push_str(&format!("<p>{}</p>\n", node.title));
    } else {
        // Node with children
        output.push_str("<details>\n");
        output.push_str(&format!("<summary>{}</summary>\n", node.title));
        for child_id in visible_children {
            output.push_str(&export_html_node(tree, child_id, false));
        }
        output.push_str("</details>\n");
    }

    output
}

fn export_text(app: &mut AppState) -> Result<()> {
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

fn export_text_node(tree: &Arena<Node>, node_id: NodeId, output: &mut String, depth: usize) {
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

// Clipboard functions
fn yank_node(app: &mut AppState) -> Result<()> {
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

fn yank_children(app: &mut AppState) -> Result<()> {
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

fn paste_as_children(app: &mut AppState) -> Result<()> {
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

fn paste_as_siblings(app: &mut AppState) -> Result<()> {
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
fn add_subtree_to_parent(
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

fn add_subtree_as_sibling(
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

// Undo/Redo
fn undo(app: &mut AppState) {
    if app.undo() {
        app.set_message("Undone");
    } else {
        app.set_message("Nothing to undo");
    }
}

fn redo(app: &mut AppState) {
    if app.redo() {
        app.set_message("Redone");
    } else {
        app.set_message("Nothing to redo");
    }
}

// Search functions
fn start_search(app: &mut AppState) {
    app.mode = AppMode::Search {
        query: String::new(),
    };
}

fn type_search_char(app: &mut AppState, c: char) {
    if let AppMode::Search { query } = &mut app.mode {
        query.push(c);
    }
}

fn backspace_search(app: &mut AppState) {
    if let AppMode::Search { query } = &mut app.mode {
        query.pop();
    }
}

fn confirm_search(app: &mut AppState) {
    if let AppMode::Search { query } = &app.mode {
        // Perform search
        let mut results = Vec::new();
        for node_ref in app.tree.iter() {
            if node_ref
                .get()
                .title
                .to_lowercase()
                .contains(&query.to_lowercase())
            {
                results.push(app.tree.get_node_id(node_ref).unwrap());
            }
        }

        app.search_results = results;
        app.search_index = 0;

        if !app.search_results.is_empty() {
            app.active_node_id = Some(app.search_results[0]);
            app.set_message(format!("Found {} results", app.search_results.len()));
        } else {
            app.set_message("No results found");
        }
    }

    app.mode = AppMode::Normal;
}

fn cancel_search(app: &mut AppState) {
    app.mode = AppMode::Normal;
}

fn next_search_result(app: &mut AppState) {
    if !app.search_results.is_empty() {
        app.search_index = (app.search_index + 1) % app.search_results.len();
        app.active_node_id = Some(app.search_results[app.search_index]);
        app.set_message(format!(
            "Result {}/{}",
            app.search_index + 1,
            app.search_results.len()
        ));
    }
}

fn previous_search_result(app: &mut AppState) {
    if !app.search_results.is_empty() {
        app.search_index = if app.search_index == 0 {
            app.search_results.len() - 1
        } else {
            app.search_index - 1
        };
        app.active_node_id = Some(app.search_results[app.search_index]);
        app.set_message(format!(
            "Result {}/{}",
            app.search_index + 1,
            app.search_results.len()
        ));
    }
}

// Symbol functions
fn toggle_symbol(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        app.push_history();

        if let Some(node) = app.tree.get_mut(active_id) {
            let title = &mut node.get_mut().title;
            let sym1 = format!("{} ", app.config.symbol1);
            let sym2 = format!("{} ", app.config.symbol2);

            if title.starts_with(&sym1) {
                *title = format!("{}{}", sym2, &title[sym1.len()..]);
            } else if title.starts_with(&sym2) {
                *title = title[sym2.len()..].to_string();
            } else {
                *title = format!("{}{}", sym1, title);
            }
        }
    }
}

fn sort_siblings(app: &mut AppState) {
    // TODO: Implement sibling sorting
    app.set_message("Sorting not yet implemented");
}

fn toggle_numbers(app: &mut AppState) {
    // TODO: Implement numbering
    app.set_message("Numbering not yet implemented");
}

fn toggle_hide(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        app.push_history();

        if let Some(node) = app.tree.get_mut(active_id) {
            let title = &mut node.get_mut().title;
            if title.starts_with("[HIDDEN] ") {
                *title = title[9..].to_string();
                app.set_message("Node unhidden");
            } else {
                *title = format!("[HIDDEN] {}", title);
                app.set_message("Node hidden");
            }
        }
    }
}

fn toggle_show_hidden(app: &mut AppState) {
    app.config.show_hidden = !app.config.show_hidden;
    app.set_message(format!(
        "Show hidden: {}",
        if app.config.show_hidden { "ON" } else { "OFF" }
    ));
}

// Layout functions
fn increase_text_width(app: &mut AppState) {
    app.config.max_parent_node_width = (app.config.max_parent_node_width as f32 * 1.2) as usize;
    app.config.max_leaf_node_width = (app.config.max_leaf_node_width as f32 * 1.2) as usize;
    app.set_message(format!(
        "Width: {} / {}",
        app.config.max_parent_node_width, app.config.max_leaf_node_width
    ));
}

fn decrease_text_width(app: &mut AppState) {
    app.config.max_parent_node_width =
        ((app.config.max_parent_node_width as f32 / 1.2).max(15.0)) as usize;
    app.config.max_leaf_node_width =
        ((app.config.max_leaf_node_width as f32 / 1.2).max(15.0)) as usize;
    app.set_message(format!(
        "Width: {} / {}",
        app.config.max_parent_node_width, app.config.max_leaf_node_width
    ));
}

fn increase_line_spacing(app: &mut AppState) {
    app.config.line_spacing += 1;
    app.set_message(format!("Line spacing: {}", app.config.line_spacing));
}

fn decrease_line_spacing(app: &mut AppState) {
    if app.config.line_spacing > 0 {
        app.config.line_spacing -= 1;
    }
    app.set_message(format!("Line spacing: {}", app.config.line_spacing));
}

fn toggle_align(app: &mut AppState) {
    app.config.align_levels = !app.config.align_levels;
    app.set_message(format!(
        "Align levels: {}",
        if app.config.align_levels { "ON" } else { "OFF" }
    ));
}

// Rank operations
fn increase_positive_rank(app: &mut AppState) {
    modify_rank(app, 1, 0);
}

fn decrease_positive_rank(app: &mut AppState) {
    modify_rank(app, -1, 0);
}

fn increase_negative_rank(app: &mut AppState) {
    modify_rank(app, 0, 1);
}

fn decrease_negative_rank(app: &mut AppState) {
    modify_rank(app, 0, -1);
}

fn modify_rank(app: &mut AppState, positive_change: i32, negative_change: i32) {
    if let Some(active_id) = app.active_node_id {
        app.push_history();

        let node = app.tree.get_mut(active_id).unwrap().get_mut();

        // Parse existing rank from title
        let mut positive = 0;
        let mut negative = 0;

        // Check if title starts with rank pattern (X+,Y-)
        if let Some(captures) = regex::Regex::new(r"^\((\d+)\+,(\d+)\-\) ")
            .unwrap()
            .captures(&node.title)
        {
            positive = captures[1].parse::<i32>().unwrap_or(0);
            negative = captures[2].parse::<i32>().unwrap_or(0);
            // Remove the existing rank from title
            node.title = regex::Regex::new(r"^\(\d+\+,\d+\-\) ")
                .unwrap()
                .replace(&node.title, "")
                .to_string();
        }

        // Apply changes
        positive = (positive + positive_change).max(0);
        negative = (negative + negative_change).max(0);

        // Add the new rank to the beginning of the title
        if positive > 0 || negative > 0 {
            node.title = format!("({}+,{}-) {}", positive, negative, node.title);
        }

        app.set_message(&format!("Rank: {}+, {}-", positive, negative));
    }
}

// Star rating operations
fn add_star(app: &mut AppState) {
    modify_stars(app, 1);
}

fn remove_star(app: &mut AppState) {
    modify_stars(app, -1);
}

fn modify_stars(app: &mut AppState, change: i32) {
    if let Some(active_id) = app.active_node_id {
        app.push_history();

        let node = app.tree.get_mut(active_id).unwrap().get_mut();

        // Count existing stars
        let current_stars = node.title.matches('★').count() as i32;
        let new_stars = (current_stars + change).max(0).min(5) as usize;

        // Remove existing stars and empty stars
        node.title = node
            .title
            .replace('★', "")
            .replace('☆', "")
            .trim()
            .to_string();

        // Add new stars at the end
        if new_stars > 0 {
            let stars_string = format!(" {}{}", "★".repeat(new_stars), "☆".repeat(5 - new_stars));
            node.title.push_str(&stars_string);
        }

        app.set_message(&format!("{} stars", new_stars));
    }
}

// Help functions
fn show_help(app: &mut AppState) {
    app.mode = AppMode::Help;
}

fn close_help(app: &mut AppState) {
    app.mode = AppMode::Normal;
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
    fn test_quit_action() {
        let mut app = create_test_app();
        assert!(app.running);

        execute_action(Action::Quit, &mut app).unwrap();
        assert!(!app.running);
    }

    #[test]
    fn test_force_quit_action() {
        let mut app = create_test_app();
        app.filename = Some(std::path::PathBuf::from("test.hmm"));
        assert!(app.running);

        execute_action(Action::ForceQuit, &mut app).unwrap();
        assert!(!app.running);
    }

    #[test]
    fn test_movement_go_down() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();

        // Go down from root to first child
        go_down(&mut app);
        assert_eq!(app.active_node_id, Some(child1));
    }

    #[test]
    fn test_movement_go_up() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();
        let child2 = root.children(&app.tree).nth(1).unwrap();

        app.active_node_id = Some(child2);
        go_up(&mut app);
        assert_eq!(app.active_node_id, Some(child1));
    }

    #[test]
    fn test_movement_go_left() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();

        app.active_node_id = Some(child1);
        go_left(&mut app);
        // go_left won't move to root if root is the only ancestor
        // In our test tree, child1's parent is root, and root's parent is itself
        // So go_left should make active_node = parent (which is root)
        assert_eq!(app.active_node_id, Some(root));
    }

    #[test]
    fn test_movement_go_right() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();

        // Ensure node is not collapsed
        app.tree.get_mut(root).unwrap().get_mut().is_collapsed = false;

        go_right(&mut app);
        assert_eq!(app.active_node_id, Some(child1));
    }

    #[test]
    fn test_movement_go_to_root() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();

        app.active_node_id = Some(child1);
        go_to_root(&mut app);
        assert_eq!(app.active_node_id, Some(root));
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

        // In indextree, removed nodes still exist in the arena but are marked as removed
        // The count() method includes removed nodes, so we need to check differently
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
    fn test_edit_mode_type_char() {
        let mut app = create_test_app();
        start_editing(&mut app, true);

        type_char(&mut app, 'T');
        type_char(&mut app, 'e');
        type_char(&mut app, 's');
        type_char(&mut app, 't');

        if let AppMode::Editing { buffer, .. } = &app.mode {
            assert_eq!(buffer, "Test");
        } else {
            panic!("Should be in editing mode");
        }
    }

    #[test]
    fn test_edit_mode_backspace() {
        let mut app = create_test_app();
        start_editing(&mut app, true);

        type_char(&mut app, 'T');
        type_char(&mut app, 'e');
        type_char(&mut app, 's');
        type_char(&mut app, 't');
        backspace(&mut app);

        if let AppMode::Editing { buffer, .. } = &app.mode {
            assert_eq!(buffer, "Tes");
        } else {
            panic!("Should be in editing mode");
        }
    }

    #[test]
    fn test_edit_mode_cursor_movement() {
        let mut app = create_test_app();
        start_editing(&mut app, true);

        type_char(&mut app, 'T');
        type_char(&mut app, 'e');
        type_char(&mut app, 's');
        type_char(&mut app, 't');

        move_cursor_home(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 0);
        }

        move_cursor_end(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 4);
        }

        move_cursor_left(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 3);
        }

        move_cursor_right(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 4);
        }
    }

    #[test]
    fn test_edit_confirm() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        start_editing(&mut app, true);
        type_char(&mut app, 'N');
        type_char(&mut app, 'e');
        type_char(&mut app, 'w');
        confirm_edit(&mut app);

        assert_eq!(app.tree.get(root).unwrap().get().title, "New");
        assert!(matches!(app.mode, AppMode::Normal));
    }

    #[test]
    fn test_edit_cancel() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let original_title = app.tree.get(root).unwrap().get().title.clone();

        start_editing(&mut app, true);
        type_char(&mut app, 'N');
        type_char(&mut app, 'e');
        type_char(&mut app, 'w');
        cancel_edit(&mut app);

        assert_eq!(app.tree.get(root).unwrap().get().title, original_title);
        assert!(matches!(app.mode, AppMode::Normal));
    }

    #[test]
    fn test_search_mode() {
        let mut app = create_test_app();

        start_search(&mut app);
        assert!(matches!(app.mode, AppMode::Search { .. }));

        type_search_char(&mut app, 'C');
        type_search_char(&mut app, 'h');
        type_search_char(&mut app, 'i');

        if let AppMode::Search { query } = &app.mode {
            assert_eq!(query, "Chi");
        }

        confirm_search(&mut app);
        assert!(matches!(app.mode, AppMode::Normal));
        assert!(!app.search_results.is_empty());
    }

    #[test]
    fn test_toggle_hide() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        toggle_hide(&mut app);
        assert!(app
            .tree
            .get(root)
            .unwrap()
            .get()
            .title
            .starts_with("[HIDDEN] "));

        toggle_hide(&mut app);
        assert!(!app
            .tree
            .get(root)
            .unwrap()
            .get()
            .title
            .starts_with("[HIDDEN] "));
    }

    #[test]
    fn test_toggle_symbol() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let original_title = app.tree.get(root).unwrap().get().title.clone();

        toggle_symbol(&mut app);
        let title_with_sym1 = app.tree.get(root).unwrap().get().title.clone();
        assert!(title_with_sym1.starts_with(&app.config.symbol1));

        toggle_symbol(&mut app);
        let title_with_sym2 = app.tree.get(root).unwrap().get().title.clone();
        assert!(title_with_sym2.starts_with(&app.config.symbol2));

        toggle_symbol(&mut app);
        let title_without_sym = app.tree.get(root).unwrap().get().title.clone();
        assert_eq!(title_without_sym, original_title);
    }

    #[test]
    fn test_undo_redo() {
        let mut app = create_test_app();

        // The initial tree should have "Root" as the title
        let initial_title = app
            .tree
            .get(app.root_id.unwrap())
            .unwrap()
            .get()
            .title
            .clone();
        assert_eq!(initial_title, "Root");

        // Save initial state to history
        app.push_history();

        // Make a change and save it
        let root = app.root_id.unwrap();
        app.tree.get_mut(root).unwrap().get_mut().title = "Modified".to_string();
        app.push_history();

        // Make another change (current state, not in history yet)
        app.tree.get_mut(root).unwrap().get_mut().title = "Modified2".to_string();

        // Verify we have the current state
        assert_eq!(
            app.tree.get(app.root_id.unwrap()).unwrap().get().title,
            "Modified2"
        );

        // Undo - should go back to "Modified" (the last saved state)
        undo(&mut app);
        assert_eq!(
            app.tree.get(app.root_id.unwrap()).unwrap().get().title,
            "Modified"
        );

        // Undo again - should go back to "Root"
        undo(&mut app);
        assert_eq!(
            app.tree.get(app.root_id.unwrap()).unwrap().get().title,
            "Root"
        );

        // Redo - should go forward to "Modified"
        redo(&mut app);
        assert_eq!(
            app.tree.get(app.root_id.unwrap()).unwrap().get().title,
            "Modified"
        );

        // Redo again - should not change since we lost "Modified2" when we did undo
        redo(&mut app);
        assert!(app.message.is_some()); // Should have "Nothing to redo" message
        assert_eq!(
            app.tree.get(app.root_id.unwrap()).unwrap().get().title,
            "Modified"
        );
    }

    #[test]
    fn test_help_mode() {
        let mut app = create_test_app();

        show_help(&mut app);
        assert!(matches!(app.mode, AppMode::Help));

        close_help(&mut app);
        assert!(matches!(app.mode, AppMode::Normal));
    }

    #[test]
    fn test_yank_node() {
        let mut app = create_test_app();

        yank_node(&mut app).unwrap();
        assert!(app.clipboard.is_some());
        assert!(app.message.is_some());
    }

    #[test]
    fn test_delete_children() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        // Root initially has 2 children (Child1, Child2)
        let initial_children: Vec<_> = root.children(&app.tree).collect();
        assert_eq!(initial_children.len(), 2);

        // Ensure root is the active node (delete_children deletes children of active node)
        app.active_node_id = Some(root);

        // Call delete_children
        delete_children(&mut app);

        // After calling delete_children on root:
        // Both children should be marked as removed
        // Note: They may still appear in children() iterator, but should be marked as removed
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

    #[test]
    fn test_go_to_top() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child2 = root.children(&app.tree).nth(1).unwrap();

        // Start at child2
        app.active_node_id = Some(child2);

        go_to_top(&mut app);

        // Should be at the root (first visible node)
        assert_eq!(app.active_node_id, Some(root));
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

    #[test]
    fn test_export_html() {
        use std::fs;
        use tempfile::TempDir;

        let mut app = create_test_app();

        // Set a filename so export can work
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.hmm");
        app.filename = Some(test_file.clone());

        export_html(&mut app).unwrap();

        // Check HTML file was created
        let html_file = temp_dir.path().join("test.hmm.html");
        assert!(html_file.exists());

        // Read and validate HTML content
        let html_content = fs::read_to_string(html_file).unwrap();
        assert!(html_content.contains("<!DOCTYPE html>"));
        assert!(html_content.contains("<title>Root</title>"));
        assert!(html_content.contains("<div id=\"root\">Root</div>"));
        assert!(html_content.contains("Child 1")); // Child 1 is a leaf, so it's a <p> not <summary>
        assert!(html_content.contains("<summary>Child 2</summary>")); // Child 2 has children
        assert!(html_content.contains("<p>Grandchild</p>"));
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

    #[test]
    fn test_rank_operations() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        app.active_node_id = Some(root);

        // Test increasing positive rank
        increase_positive_rank(&mut app);
        let node = app.tree.get(root).unwrap().get();
        assert!(node.title.starts_with("(1+,0-) "));

        // Test increasing negative rank
        increase_negative_rank(&mut app);
        let node = app.tree.get(root).unwrap().get();
        assert!(node.title.starts_with("(1+,1-) "));

        // Test decreasing positive rank
        decrease_positive_rank(&mut app);
        let node = app.tree.get(root).unwrap().get();
        assert!(node.title.starts_with("(0+,1-) "));

        // Test that rank is removed when both are 0
        decrease_negative_rank(&mut app);
        let node = app.tree.get(root).unwrap().get();
        assert!(!node.title.contains('+'));
        assert!(!node.title.contains('-'));
    }

    #[test]
    fn test_star_operations() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        app.active_node_id = Some(root);

        // Test adding stars
        add_star(&mut app);
        let node = app.tree.get(root).unwrap().get();
        assert!(node.title.contains('★'));
        assert!(node.title.contains("☆☆☆☆"));

        // Add more stars
        add_star(&mut app);
        add_star(&mut app);
        let node = app.tree.get(root).unwrap().get();
        assert!(node.title.contains("★★★"));
        assert!(node.title.contains("☆☆"));

        // Test max stars
        add_star(&mut app);
        add_star(&mut app);
        add_star(&mut app); // Should cap at 5
        let node = app.tree.get(root).unwrap().get();
        assert!(node.title.contains("★★★★★"));
        assert!(!node.title.contains('☆'));

        // Test removing stars
        remove_star(&mut app);
        let node = app.tree.get(root).unwrap().get();
        assert!(node.title.contains("★★★★"));
        assert!(node.title.contains('☆'));
    }

    #[test]
    fn test_go_to_bottom() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        // Expand all to make grandchild visible
        expand_all(&mut app);

        go_to_bottom(&mut app);

        // Should be at the last visible node (grandchild)
        // Get the grandchild through Child2
        let child2 = root.children(&app.tree).nth(1).unwrap();
        let grandchild = child2.children(&app.tree).next().unwrap();
        assert_eq!(app.active_node_id, Some(grandchild));
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
    fn test_layout_adjustments() {
        let mut app = create_test_app();
        let initial_width = app.config.max_parent_node_width;
        let initial_spacing = app.config.line_spacing;

        increase_text_width(&mut app);
        assert!(app.config.max_parent_node_width > initial_width);

        decrease_text_width(&mut app);
        assert!(app.config.max_parent_node_width <= initial_width);

        increase_line_spacing(&mut app);
        assert_eq!(app.config.line_spacing, initial_spacing + 1);

        decrease_line_spacing(&mut app);
        assert_eq!(app.config.line_spacing, initial_spacing);
    }

    #[test]
    fn test_toggle_settings() {
        let mut app = create_test_app();

        let initial_show_hidden = app.config.show_hidden;
        toggle_show_hidden(&mut app);
        assert_ne!(app.config.show_hidden, initial_show_hidden);

        let initial_center_lock = app.config.center_lock;
        toggle_center_lock(&mut app);
        assert_ne!(app.config.center_lock, initial_center_lock);

        let initial_focus_lock = app.config.focus_lock;
        toggle_focus_lock(&mut app);
        assert_ne!(app.config.focus_lock, initial_focus_lock);

        let initial_align = app.config.align_levels;
        toggle_align(&mut app);
        assert_ne!(app.config.align_levels, initial_align);
    }
}
