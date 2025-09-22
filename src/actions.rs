use crate::app::{AppMode, AppState};
use crate::model::{Node, NodeId};
use crate::parser;
use anyhow::Result;
use clipboard::{ClipboardContext, ClipboardProvider};

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

        // Help
        Action::ShowHelp => show_help(app),
        Action::CloseHelp => close_help(app),
    }
    Ok(())
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
                } else if parent_id != app.root_id.unwrap() {
                    app.active_node_id = Some(parent_id);
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
                }
            }
        }
    }
}

fn go_left(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        if let Some(parent_id) = active_id.ancestors(&app.tree).nth(1) {
            if parent_id != app.root_id.unwrap() {
                app.active_node_id = Some(parent_id);
            }
        }
    }
}

fn go_right(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        let node = app.tree.get(active_id).unwrap().get();
        if !node.is_collapsed {
            if let Some(first_child) = active_id.children(&app.tree).next() {
                app.active_node_id = Some(first_child);
            }
        }
    }
}

fn go_to_root(app: &mut AppState) {
    app.active_node_id = app.root_id;
}

fn go_to_top(app: &mut AppState) {
    if let Some(root_id) = app.root_id {
        app.active_node_id = Some(root_id);
    }
}

fn go_to_bottom(app: &mut AppState) {
    if let Some(root_id) = app.root_id {
        fn find_last_visible(tree: &indextree::Arena<Node>, node_id: NodeId) -> NodeId {
            let node = tree.get(node_id).unwrap().get();
            if node.is_collapsed {
                return node_id;
            }

            if let Some(last_child) = node_id.children(tree).last() {
                return find_last_visible(tree, last_child);
            }

            node_id
        }

        app.active_node_id = Some(find_last_visible(&app.tree, root_id));
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
    // TODO: Implement viewport centering
    app.set_message("Centering not yet implemented");
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
    // TODO: Implement HTML export
    app.set_message("HTML export not yet implemented");
    Ok(())
}

fn export_text(app: &mut AppState) -> Result<()> {
    // TODO: Implement text export
    app.set_message("Text export not yet implemented");
    Ok(())
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
    if let Some(ref _clipboard_text) = app.clipboard {
        if let Some(_active_id) = app.active_node_id {
            app.push_history();

            // TODO: Parse clipboard text and add as children
            app.set_message("Paste as children not yet fully implemented");
        }
    } else {
        app.set_message("Clipboard is empty");
    }
    Ok(())
}

fn paste_as_siblings(app: &mut AppState) -> Result<()> {
    if let Some(ref _clipboard_text) = app.clipboard {
        if let Some(_active_id) = app.active_node_id {
            app.push_history();

            // TODO: Parse clipboard text and add as siblings
            app.set_message("Paste as siblings not yet fully implemented");
        }
    } else {
        app.set_message("Clipboard is empty");
    }
    Ok(())
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

// Help functions
fn show_help(app: &mut AppState) {
    app.mode = AppMode::Help;
}

fn close_help(app: &mut AppState) {
    app.mode = AppMode::Normal;
}
