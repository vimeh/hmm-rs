use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

// Bring in specific errors from other crates we want to wrap
use crate::config::ConfigError;
use crate::io::IoError;
use crate::ui::UiError;
// Add other specific errors like serde_json::Error when needed

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("I/O error: {0}")]
    Io(#[from] IoError),

    #[error("Serialization/Deserialization error (JSON): {0}")]
    Json(#[from] serde_json::Error),

    #[error("Node with ID {0} not found")]
    NodeNotFound(Uuid),

    #[error("Cannot add node: Parent node with ID {0} not found")]
    ParentNodeNotFound(Uuid),

    #[error("Invalid operation: Cannot delete the root node")]
    CannotDeleteRoot,

    #[error("Invalid file path: {0}")]
    InvalidPath(PathBuf),

    #[error("Import/Export error: {0}")]
    FormatError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error("MindMap logic error: {0}")]
    MindMapLogic(String),

    #[error("UI error: {0}")]
    Ui(#[from] UiError),

    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

// You might want type aliases for Results using this AppError
pub type AppResult<T> = Result<T, AppError>;
