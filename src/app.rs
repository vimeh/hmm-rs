use crate::config::AppConfig;
use crate::model::{Node, NodeId};
use indextree::Arena;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Editing { buffer: String, cursor_pos: usize },
    Search { query: String },
    Help,
}

pub struct AppState {
    pub running: bool,
    pub mode: AppMode,
    pub tree: Arena<Node>,
    pub root_id: Option<NodeId>,
    pub active_node_id: Option<NodeId>,
    pub config: AppConfig,
    pub filename: Option<PathBuf>,

    // Viewport state
    pub viewport_top: f64,
    pub viewport_left: f64,
    pub terminal_width: u16,
    pub terminal_height: u16,

    // Undo/Redo history
    pub history: Vec<Arena<Node>>,
    pub history_index: usize,

    // Message for status line
    pub message: Option<String>,

    // Search state
    pub search_results: Vec<NodeId>,
    pub search_index: usize,

    // Clipboard
    pub clipboard: Option<String>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let tree = Arena::new();

        Self {
            running: true,
            mode: AppMode::Normal,
            tree,
            root_id: None,
            active_node_id: None,
            config,
            filename: None,
            viewport_top: 0.0,
            viewport_left: 0.0,
            terminal_width: 80,
            terminal_height: 24,
            history: Vec::new(),
            history_index: 0,
            message: None,
            search_results: Vec::new(),
            search_index: 0,
            clipboard: None,
        }
    }

    pub fn push_history(&mut self) {
        // Remove any redo history
        self.history.truncate(self.history_index);

        // Add current state to history
        self.history.push(self.tree.clone());
        self.history_index += 1;

        // Limit history size
        if self.history.len() > self.config.max_undo_steps {
            self.history.remove(0);
            self.history_index -= 1;
        }
    }

    pub fn undo(&mut self) -> bool {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.tree = self.history[self.history_index].clone();
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            self.tree = self.history[self.history_index].clone();
            true
        } else {
            false
        }
    }

    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.message = Some(msg.into());
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }
}
