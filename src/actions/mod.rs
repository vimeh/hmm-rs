mod clipboard;
mod editing;
mod file;
mod formatting;
mod help;
mod history;
mod movement;
mod node;
mod search;
mod view;

use crate::app::AppState;
use anyhow::Result;

// Re-export all public functions from submodules
pub use clipboard::*;
pub use editing::*;
pub use file::*;
pub use formatting::*;
pub use help::*;
pub use history::*;
pub use movement::*;
pub use node::*;
pub use search::*;
pub use view::*;

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
    MoveCursorWordLeft,
    MoveCursorWordRight,
    DeleteWordBackward,
    DeleteWordForward,
    DeleteToEnd,
    DeleteToStart,
    PasteAtCursor,
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

    // Help
    ShowHelp,
    CloseHelp,
}

pub fn execute_action(action: Action, app: &mut AppState) -> Result<()> {
    match action {
        Action::Quit => {
            if app.is_dirty {
                app.set_message("Unsaved changes! Press Shift+Q to force quit or 's' to save");
            } else {
                app.running = false;
            }
        }
        Action::ForceQuit => {
            app.running = false;
        }

        // Movement actions
        Action::GoUp => movement::go_up(app),
        Action::GoDown => movement::go_down(app),
        Action::GoLeft => movement::go_left(app),
        Action::GoRight => movement::go_right(app),
        Action::GoToRoot => movement::go_to_root(app),
        Action::GoToTop => movement::go_to_top(app),
        Action::GoToBottom => movement::go_to_bottom(app),

        // Node manipulation
        Action::InsertSibling => node::insert_sibling(app),
        Action::InsertChild => node::insert_child(app),
        Action::DeleteNode => node::delete_node(app),
        Action::DeleteChildren => node::delete_children(app),
        Action::MoveNodeUp => node::move_node_up(app),
        Action::MoveNodeDown => node::move_node_down(app),

        // Editing
        Action::EditNodeAppend => editing::start_editing(app, false),
        Action::EditNodeReplace => editing::start_editing(app, true),
        Action::TypeChar(c) => editing::type_char(app, c),
        Action::Backspace => editing::backspace(app),
        Action::Delete => editing::delete_char(app),
        Action::MoveCursorLeft => editing::move_cursor_left(app),
        Action::MoveCursorRight => editing::move_cursor_right(app),
        Action::MoveCursorHome => editing::move_cursor_home(app),
        Action::MoveCursorEnd => editing::move_cursor_end(app),
        Action::MoveCursorWordLeft => editing::move_cursor_word_left(app),
        Action::MoveCursorWordRight => editing::move_cursor_word_right(app),
        Action::DeleteWordBackward => editing::delete_word_backward(app),
        Action::DeleteWordForward => editing::delete_word_forward(app),
        Action::DeleteToEnd => editing::delete_to_end(app),
        Action::DeleteToStart => editing::delete_to_start(app),
        Action::PasteAtCursor => editing::paste_at_cursor(app),
        Action::ConfirmEdit => editing::confirm_edit(app),
        Action::CancelEdit => editing::cancel_edit(app),

        // View control
        Action::ToggleCollapse => view::toggle_collapse(app),
        Action::CollapseAll => view::collapse_all(app),
        Action::ExpandAll => view::expand_all(app),
        Action::CollapseChildren => view::collapse_children(app),
        Action::CollapseOtherBranches => view::collapse_other_branches(app),
        Action::CollapseToLevel(level) => view::collapse_to_level(app, level),
        Action::CenterActiveNode => view::center_active_node(app),
        Action::ToggleCenterLock => view::toggle_center_lock(app),
        Action::Focus => view::focus(app),
        Action::ToggleFocusLock => view::toggle_focus_lock(app),

        // File operations
        Action::Save => file::save(app)?,
        Action::SaveAs => file::save_as(app)?,
        Action::ExportText => file::export_text(app)?,

        // Clipboard
        Action::YankNode => clipboard::yank_node(app)?,
        Action::YankChildren => clipboard::yank_children(app)?,
        Action::PasteAsChildren => clipboard::paste_as_children(app)?,
        Action::PasteAsSiblings => clipboard::paste_as_siblings(app)?,

        // Undo/Redo
        Action::Undo => history::undo(app),
        Action::Redo => history::redo(app),

        // Search
        Action::Search => search::start_search(app),
        Action::TypeSearchChar(c) => search::type_search_char(app, c),
        Action::BackspaceSearch => search::backspace_search(app),
        Action::ConfirmSearch => search::confirm_search(app),
        Action::CancelSearch => search::cancel_search(app),
        Action::NextSearchResult => search::next_search_result(app),
        Action::PreviousSearchResult => search::previous_search_result(app),

        // Symbols
        Action::ToggleSymbol => formatting::toggle_symbol(app),
        Action::SortSiblings => formatting::sort_siblings(app),
        Action::ToggleNumbers => formatting::toggle_numbers(app),
        Action::ToggleHide => formatting::toggle_hide(app),
        Action::ToggleShowHidden => formatting::toggle_show_hidden(app),

        // Layout
        Action::IncreaseTextWidth => formatting::increase_text_width(app),
        Action::DecreaseTextWidth => formatting::decrease_text_width(app),
        Action::IncreaseLineSpacing => formatting::increase_line_spacing(app),
        Action::DecreaseLineSpacing => formatting::decrease_line_spacing(app),

        // Help
        Action::ShowHelp => help::show_help(app),
        Action::CloseHelp => help::close_help(app),
    }
    Ok(())
}
