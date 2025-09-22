use crate::actions::Action;
use crate::app::{AppMode, AppState};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

pub fn handle_events(app: &mut AppState) -> Result<Option<Action>> {
    if event::poll(Duration::from_millis(10))? {
        if let Event::Key(key) = event::read()? {
            return Ok(handle_key_event(app, key));
        }
    }
    Ok(None)
}

fn handle_key_event(app: &AppState, key: KeyEvent) -> Option<Action> {
    match &app.mode {
        AppMode::Normal => handle_normal_mode(key),
        AppMode::Editing { .. } => handle_editing_mode(key),
        AppMode::Search { .. } => handle_search_mode(key),
        AppMode::Help => handle_help_mode(key),
    }
}

fn handle_normal_mode(key: KeyEvent) -> Option<Action> {
    use KeyCode::*;

    match (key.code, key.modifiers) {
        // Quit
        (Char('q'), KeyModifiers::NONE) => Some(Action::Quit),
        (Char('Q'), KeyModifiers::SHIFT) => Some(Action::ForceQuit),
        (Char('c'), KeyModifiers::CONTROL) => Some(Action::Quit),

        // Movement
        (Char('h'), KeyModifiers::NONE) | (Left, _) => Some(Action::GoLeft),
        (Char('j'), KeyModifiers::NONE) | (Down, _) => Some(Action::GoDown),
        (Char('k'), KeyModifiers::NONE) | (Up, _) => Some(Action::GoUp),
        (Char('l'), KeyModifiers::NONE) | (Right, _) => Some(Action::GoRight),

        // Node manipulation
        (Char('o'), KeyModifiers::NONE) | (Enter, KeyModifiers::NONE) => {
            Some(Action::InsertSibling)
        }
        (Char('O'), KeyModifiers::SHIFT) | (Tab, KeyModifiers::NONE) => Some(Action::InsertChild),
        (Char(' '), KeyModifiers::NONE) => Some(Action::ToggleCollapse),
        (Char('d'), KeyModifiers::NONE) => Some(Action::DeleteNode),
        (Char('D'), KeyModifiers::SHIFT) => Some(Action::DeleteChildren),

        // Editing
        (Char('e'), KeyModifiers::NONE) | (Char('i'), KeyModifiers::NONE) => {
            Some(Action::EditNodeAppend)
        }
        (Char('E'), KeyModifiers::SHIFT) | (Char('I'), KeyModifiers::SHIFT) => {
            Some(Action::EditNodeReplace)
        }
        (Char('a'), KeyModifiers::NONE) => Some(Action::EditNodeAppend),
        (Char('A'), KeyModifiers::SHIFT) => Some(Action::EditNodeReplace),

        // View control
        (Char('c'), KeyModifiers::NONE) => Some(Action::CenterActiveNode),
        (Char('C'), KeyModifiers::SHIFT) => Some(Action::ToggleCenterLock),
        (Char('f'), KeyModifiers::NONE) => Some(Action::Focus),
        (Char('F'), KeyModifiers::SHIFT) => Some(Action::ToggleFocusLock),

        // Collapsing
        (Char('v'), KeyModifiers::NONE) => Some(Action::CollapseAll),
        (Char('b'), KeyModifiers::NONE) => Some(Action::ExpandAll),
        (Char('V'), KeyModifiers::SHIFT) => Some(Action::CollapseChildren),
        (Char('r'), KeyModifiers::NONE) => Some(Action::CollapseOtherBranches),
        (Char('1'), KeyModifiers::NONE) => Some(Action::CollapseToLevel(1)),
        (Char('2'), KeyModifiers::NONE) => Some(Action::CollapseToLevel(2)),
        (Char('3'), KeyModifiers::NONE) => Some(Action::CollapseToLevel(3)),
        (Char('4'), KeyModifiers::NONE) => Some(Action::CollapseToLevel(4)),
        (Char('5'), KeyModifiers::NONE) => Some(Action::CollapseToLevel(5)),

        // Navigation
        (Char('g'), KeyModifiers::NONE) => Some(Action::GoToTop),
        (Char('G'), KeyModifiers::SHIFT) => Some(Action::GoToBottom),
        (Char('m'), KeyModifiers::NONE) | (Char('~'), KeyModifiers::NONE) => Some(Action::GoToRoot),

        // File operations
        (Char('s'), KeyModifiers::NONE) => Some(Action::Save),
        (Char('S'), KeyModifiers::SHIFT) => Some(Action::SaveAs),

        // Export
        (Char('x'), KeyModifiers::NONE) => Some(Action::ExportHtml),
        (Char('X'), KeyModifiers::SHIFT) => Some(Action::ExportText),

        // Clipboard
        (Char('y'), KeyModifiers::NONE) => Some(Action::YankNode),
        (Char('Y'), KeyModifiers::SHIFT) => Some(Action::YankChildren),
        (Char('p'), KeyModifiers::NONE) => Some(Action::PasteAsChildren),
        (Char('P'), KeyModifiers::SHIFT) => Some(Action::PasteAsSiblings),

        // Node movement
        (Char('J'), KeyModifiers::SHIFT) => Some(Action::MoveNodeDown),
        (Char('K'), KeyModifiers::SHIFT) => Some(Action::MoveNodeUp),

        // Undo/Redo
        (Char('u'), KeyModifiers::NONE) => Some(Action::Undo),
        (Char('r'), KeyModifiers::CONTROL) => Some(Action::Redo),

        // Search
        (Char('/'), KeyModifiers::NONE) | (Char('f'), KeyModifiers::CONTROL) => {
            Some(Action::Search)
        }
        (Char('n'), KeyModifiers::NONE) => Some(Action::NextSearchResult),
        (Char('N'), KeyModifiers::SHIFT) => Some(Action::PreviousSearchResult),

        // Symbols
        (Char('t'), KeyModifiers::NONE) => Some(Action::ToggleSymbol),
        (Char('T'), KeyModifiers::SHIFT) => Some(Action::SortSiblings),
        (Char('#'), KeyModifiers::NONE) => Some(Action::ToggleNumbers),

        // Layout
        (Char('w'), KeyModifiers::NONE) => Some(Action::IncreaseTextWidth),
        (Char('W'), KeyModifiers::SHIFT) => Some(Action::DecreaseTextWidth),
        (Char('z'), KeyModifiers::NONE) => Some(Action::DecreaseLineSpacing),
        (Char('Z'), KeyModifiers::SHIFT) => Some(Action::IncreaseLineSpacing),
        (Char('|'), KeyModifiers::NONE) => Some(Action::ToggleAlign),

        // Hidden nodes
        (Char('H'), KeyModifiers::SHIFT) => Some(Action::ToggleHide),
        (Char('h'), KeyModifiers::CONTROL) => Some(Action::ToggleShowHidden),

        // Help
        (Char('?'), KeyModifiers::NONE) => Some(Action::ShowHelp),

        _ => None,
    }
}

fn handle_editing_mode(key: KeyEvent) -> Option<Action> {
    use KeyCode::*;

    match key.code {
        Esc => Some(Action::CancelEdit),
        Enter => Some(Action::ConfirmEdit),
        Char(c) => Some(Action::TypeChar(c)),
        Backspace => Some(Action::Backspace),
        Delete => Some(Action::Delete),
        Left => Some(Action::MoveCursorLeft),
        Right => Some(Action::MoveCursorRight),
        Home => Some(Action::MoveCursorHome),
        End => Some(Action::MoveCursorEnd),
        _ => None,
    }
}

fn handle_search_mode(key: KeyEvent) -> Option<Action> {
    use KeyCode::*;

    match key.code {
        Esc => Some(Action::CancelSearch),
        Enter => Some(Action::ConfirmSearch),
        Char(c) => Some(Action::TypeSearchChar(c)),
        Backspace => Some(Action::BackspaceSearch),
        _ => None,
    }
}

fn handle_help_mode(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => Some(Action::CloseHelp),
        _ => None,
    }
}
