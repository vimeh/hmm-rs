pub mod app;
pub mod config;
pub mod layout;
pub mod model;
pub mod parser;
pub mod ui;

// Internal modules
pub mod actions;
pub mod event;

// Re-export commonly used types
pub use app::{AppMode, AppState};
pub use config::AppConfig;
pub use model::{Node, NodeId};