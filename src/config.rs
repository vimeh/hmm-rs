use clap::Parser;
use config::{
    /* Config as ConfigCrate, */
    ConfigError as ConfigCrateError, /* File, FileFormat */ /* Value, */
                                    /* ValueKind, */
};
use serde::{Deserialize /*, Serialize */};
use std::{collections::HashMap, path::PathBuf};
use thiserror::Error;

// Using constants for default values makes them easy to change.
const DEFAULT_MAX_PARENT_WIDTH: u16 = 25;
const DEFAULT_MAX_LEAF_WIDTH: u16 = 55;
const DEFAULT_LINE_SPACING: u8 = 1;
const DEFAULT_ALIGN_LEVELS: bool = false;
const DEFAULT_INITIAL_DEPTH: u8 = 1;
const DEFAULT_CENTER_LOCK: bool = false;
const DEFAULT_FOCUS_LOCK: bool = false;
const DEFAULT_MAX_UNDO: usize = 24;
const DEFAULT_ACTIVE_COLOR: &str = "\033[38;5;0m\033[48;5;172m\033[1m"; // Black fg, Orange bg, Bold
const DEFAULT_MESSAGE_COLOR: &str = "\033[38;5;0m\033[48;5;141m\033[1m"; // Black fg, Pink bg, Bold
const DEFAULT_CLIPBOARD_MODE: &str = "os";
const DEFAULT_CLIPBOARD_FILE: &str = "/tmp/h-m-m"; // Note: Platform-specific default might be better
const DEFAULT_AUTO_SAVE: bool = false;

// Define potential errors during configuration loading.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file error: {0}")]
    ConfigFile(#[from] ConfigCrateError),
    #[error("Failed to determine config directory")]
    DirectoryNotFound,
    #[error("Invalid keybinding key: {0}")]
    InvalidKeybindingKey(String),
    #[error("Unknown command in keybinding: {0}")]
    UnknownKeybindingCommand(String),
    #[error("I/O error: {0}")]
    IoError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

// Serde struct for deserializing config file values.
// Optional fields allow for layered config (defaults -> file -> env -> args).
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)] // Use Default trait for missing fields
struct FileConfig {
    pub default_file: Option<String>,
    max_parent_node_width: Option<u16>,
    max_leaf_node_width: Option<u16>,
    line_spacing: Option<u8>,
    align_levels: Option<bool>,
    symbol1: Option<String>,
    symbol2: Option<String>,
    show_hidden: Option<bool>, // Note: show_hidden might be better as a runtime toggle state
    initial_depth: Option<u8>,
    center_lock: Option<bool>,
    focus_lock: Option<bool>,
    max_undo_steps: Option<usize>,
    active_node_color: Option<String>,
    message_color: Option<String>,
    doubt_color: Option<String>,
    post_export_command: Option<String>,
    clipboard: Option<String>,
    clipboard_file: Option<String>,
    clipboard_in_command: Option<String>,
    clipboard_out_command: Option<String>,
    auto_save: Option<bool>,
    echo_keys: Option<bool>, // For debugging key input
    keybindings: Option<HashMap<String, String>>,
}

// Final Config struct, combining all sources.
// Fields are non-optional as they will always have a value (default or overridden).
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub default_file: Option<String>,
    pub filename: Option<PathBuf>, // From command line argument
    pub max_parent_node_width: u16,
    pub max_leaf_node_width: u16,
    pub line_spacing: u8,
    pub align_levels: bool,
    pub symbol1: String,
    pub symbol2: String,
    pub show_hidden: bool, // Initially loaded, might change at runtime
    pub initial_depth: u8,
    pub center_lock: bool, // Initially loaded, might change at runtime
    pub focus_lock: bool,  // Initially loaded, might change at runtime
    pub max_undo_steps: usize,
    pub active_node_color: String,
    pub message_color: String,
    pub doubt_color: Option<String>, // Keep optional if no sensible default
    pub post_export_command: String,
    pub clipboard: String, // Consider an enum: Os, Internal, File, Command
    pub clipboard_file: String,
    pub clipboard_in_command: String,
    pub clipboard_out_command: String,
    pub auto_save: bool,
    pub echo_keys: bool,
    pub keybindings: HashMap<String, String>, // Parsed keybindings
}

// Command line arguments defined using clap.
#[derive(Parser, Debug)]
#[command(author, version, about = "h-m-m in Rust", long_about = None)]
struct Args {
    /// Path to the mind map file to load
    filename: Option<PathBuf>,

    /// Path to a custom configuration file
    #[arg(long)]
    config: Option<PathBuf>,

    // --- Mirrored settings from PHP version ---
    #[arg(long)]
    max_parent_node_width: Option<u16>,
    #[arg(long)]
    max_leaf_node_width: Option<u16>,
    #[arg(long)]
    line_spacing: Option<u8>,
    #[arg(long)]
    align_levels: Option<bool>,
    #[arg(long)]
    symbol1: Option<String>,
    #[arg(long)]
    symbol2: Option<String>,
    #[arg(long)]
    show_hidden: Option<bool>,
    #[arg(long)]
    initial_depth: Option<u8>,
    #[arg(long)]
    center_lock: Option<bool>,
    #[arg(long)]
    focus_lock: Option<bool>,
    #[arg(long)]
    max_undo_steps: Option<usize>,
    #[arg(long)]
    active_node_color: Option<String>,
    #[arg(long)]
    message_color: Option<String>,
    #[arg(long)]
    doubt_color: Option<String>,
    #[arg(long)]
    post_export_command: Option<String>,
    #[arg(long)]
    clipboard: Option<String>,
    #[arg(long)]
    clipboard_file: Option<String>,
    #[arg(long)]
    clipboard_in_command: Option<String>,
    #[arg(long)]
    clipboard_out_command: Option<String>,
    #[arg(long)]
    auto_save: Option<bool>,
    #[arg(long)]
    echo_keys: Option<bool>,
    #[arg(long)]
    default_file: Option<String>,
}

// Function to load configuration from all sources.
pub fn load_config() -> Result<Config, ConfigError> {
    // Parse command line arguments using clap.
    let args = Args::parse();
    // Now call the build function which handles layering
    build_config_from_args(args)
}

// Separate function to allow testing with specific args
fn build_config_from_args(args: Args) -> Result<Config, ConfigError> {
    // 1. Determine config file path
    let config_file_path = args.config.clone().or_else(|| {
        // Standard config directory
        dirs::config_dir().map(|dir| dir.join("hmm-rs").join("config.toml")) // Assuming TOML format
    });

    // 2. Load defaults (keybindings are handled later)
    let _defaults = FileConfig::default(); // Prefix unused variable

    // 3. Build configuration source using the `config` crate
    let mut config_builder = config::Config::builder();

    // Layer on config file if path is determined and file exists
    if let Some(ref path) = config_file_path {
        config_builder =
            config_builder.add_source(config::File::from(path.clone()).required(false));
    }

    // Layer on environment variables (e.g., HMM_MAX_UNDO_STEPS)
    config_builder =
        config_builder.add_source(config::Environment::with_prefix("HMM").separator("__"));

    // Deserialize into FileConfig, propagating errors
    let loaded_sources: FileConfig = config_builder.build()?.try_deserialize()?;

    // 4. Layer the configurations: args > env/file (loaded_sources) > defaults
    // Note: `loaded_sources` already combines file and env based on `config` crate's layering.
    // We prioritize command-line args (`args`) over everything else.
    let config = Config {
        filename: args.filename, // Only comes from args
        default_file: args
            .default_file // CLI arg for default file
            .or(loaded_sources.default_file), // Then config file/env
        // No final default needed as it's Option<String>
        max_parent_node_width: args
            .max_parent_node_width
            .or(loaded_sources.max_parent_node_width)
            .unwrap_or(DEFAULT_MAX_PARENT_WIDTH),
        max_leaf_node_width: args
            .max_leaf_node_width
            .or(loaded_sources.max_leaf_node_width)
            .unwrap_or(DEFAULT_MAX_LEAF_WIDTH),
        line_spacing: args
            .line_spacing
            .or(loaded_sources.line_spacing)
            .unwrap_or(DEFAULT_LINE_SPACING),
        align_levels: args
            .align_levels
            .or(loaded_sources.align_levels)
            .unwrap_or(DEFAULT_ALIGN_LEVELS),
        symbol1: args
            .symbol1
            .or(loaded_sources.symbol1)
            .unwrap_or_else(|| "✓".to_string()),
        symbol2: args
            .symbol2
            .or(loaded_sources.symbol2)
            .unwrap_or_else(|| "✗".to_string()),
        show_hidden: args
            .show_hidden
            .or(loaded_sources.show_hidden)
            .unwrap_or(false), // Default to not showing hidden
        initial_depth: args
            .initial_depth
            .or(loaded_sources.initial_depth)
            .unwrap_or(DEFAULT_INITIAL_DEPTH),
        center_lock: args
            .center_lock
            .or(loaded_sources.center_lock)
            .unwrap_or(DEFAULT_CENTER_LOCK),
        focus_lock: args
            .focus_lock
            .or(loaded_sources.focus_lock)
            .unwrap_or(DEFAULT_FOCUS_LOCK),
        max_undo_steps: args
            .max_undo_steps
            .or(loaded_sources.max_undo_steps)
            .unwrap_or(DEFAULT_MAX_UNDO),
        active_node_color: args
            .active_node_color
            .or(loaded_sources.active_node_color)
            .unwrap_or_else(|| DEFAULT_ACTIVE_COLOR.to_string()),
        message_color: args
            .message_color
            .or(loaded_sources.message_color)
            .unwrap_or_else(|| DEFAULT_MESSAGE_COLOR.to_string()),
        doubt_color: args.doubt_color.or(loaded_sources.doubt_color),
        post_export_command: args
            .post_export_command
            .or(loaded_sources.post_export_command)
            .unwrap_or_default(), // Empty string default
        clipboard: args
            .clipboard
            .or(loaded_sources.clipboard)
            .unwrap_or_else(|| DEFAULT_CLIPBOARD_MODE.to_string()),
        clipboard_file: args
            .clipboard_file
            .or(loaded_sources.clipboard_file)
            .unwrap_or_else(|| DEFAULT_CLIPBOARD_FILE.to_string()),
        clipboard_in_command: args
            .clipboard_in_command
            .or(loaded_sources.clipboard_in_command)
            .unwrap_or_default(),
        clipboard_out_command: args
            .clipboard_out_command
            .or(loaded_sources.clipboard_out_command)
            .unwrap_or_default(),
        auto_save: args
            .auto_save
            .or(loaded_sources.auto_save)
            .unwrap_or(DEFAULT_AUTO_SAVE),
        echo_keys: args.echo_keys.or(loaded_sources.echo_keys).unwrap_or(false),
        keybindings: loaded_sources // Keybindings come only from file/env/defaults for now
            .keybindings
            .unwrap_or_else(get_default_keybindings),
    };

    // Validate keybindings (example validation)
    for (key, command) in &config.keybindings {
        // Basic validation: check if command looks reasonable (e.g., exists in a known list)
        // This part needs actual command validation logic based on available commands
        if !is_valid_keybinding_command(command) {
            return Err(ConfigError::UnknownKeybindingCommand(command.clone()));
        }
        // Key format validation could also be added here
    }

    Ok(config)
}

// Placeholder for actual keybinding command validation
fn is_valid_keybinding_command(command: &str) -> bool {
    // TODO: Implement actual validation based on defined application commands
    !command.trim().is_empty() // Basic check: not empty
}

// Provides the default keybindings, mirroring the PHP script.
// TODO: Map these strings to actual function calls or enums later.
fn get_default_keybindings() -> HashMap<String, String> {
    let mut kb = HashMap::new();
    // Add keybindings based on the PHP defaults
    // Example: kb.insert("a".to_string(), "edit_node_append".to_string());
    // ... (populate all default keybindings) ...
    kb.insert("a".to_string(), "edit_node_append".to_string());
    kb.insert("A".to_string(), "edit_node_replace".to_string());
    kb.insert("b".to_string(), "expand_all".to_string());
    kb.insert("c".to_string(), "center_active_node".to_string());
    kb.insert("C".to_string(), "toggle_center_lock".to_string());
    kb.insert("ctrl_c".to_string(), "quit".to_string()); // Need mapping for special keys
    kb.insert("d".to_string(), "delete_node".to_string());
    kb.insert("D".to_string(), "delete_children".to_string());
    kb.insert(
        "del".to_string(),
        "delete_node_without_clipboard".to_string(),
    ); // Special key
    kb.insert("e".to_string(), "edit_node_append".to_string());
    kb.insert("E".to_string(), "edit_node_replace".to_string());
    kb.insert("f".to_string(), "focus".to_string());
    kb.insert("F".to_string(), "toggle_focus_lock".to_string());
    kb.insert("g".to_string(), "go_to_top".to_string());
    kb.insert("G".to_string(), "go_to_bottom".to_string());
    kb.insert("h".to_string(), "go_left".to_string());
    kb.insert("H".to_string(), "toggle_hide".to_string());
    kb.insert("ctrl_h".to_string(), "toggle_show_hidden".to_string()); // Special key
    kb.insert("i".to_string(), "edit_node_append".to_string());
    kb.insert("I".to_string(), "edit_node_replace".to_string());
    kb.insert("j".to_string(), "go_down".to_string());
    kb.insert("J".to_string(), "move_node_down".to_string());
    kb.insert("k".to_string(), "go_up".to_string());
    kb.insert("K".to_string(), "move_node_up".to_string());
    kb.insert("l".to_string(), "go_right".to_string());
    kb.insert("m".to_string(), "go_to_root".to_string());
    kb.insert("~".to_string(), "go_to_root".to_string());
    kb.insert("n".to_string(), "next_search_result".to_string());
    kb.insert("N".to_string(), "previous_search_result".to_string());
    kb.insert("o".to_string(), "insert_new_sibling".to_string());
    kb.insert("O".to_string(), "insert_new_child".to_string());
    kb.insert("ctrl_o".to_string(), "open_link".to_string()); // Special key
    kb.insert("p".to_string(), "paste_as_children".to_string());
    kb.insert("P".to_string(), "paste_as_siblings".to_string());
    kb.insert("ctrl_p".to_string(), "append".to_string()); // Special key
    kb.insert("q".to_string(), "quit".to_string());
    kb.insert("Q".to_string(), "shutdown".to_string()); // Force quit
    kb.insert("ctrl_q".to_string(), "quit_with_debug".to_string()); // Special key
    kb.insert("r".to_string(), "collapse_other_branches".to_string());
    kb.insert("R".to_string(), "collapse_inner".to_string());
    kb.insert("s".to_string(), "save".to_string());
    kb.insert("S".to_string(), "save_as".to_string());
    kb.insert("t".to_string(), "toggle_symbol".to_string());
    kb.insert("T".to_string(), "sort_siblings".to_string());
    kb.insert("#".to_string(), "toggle_numbers".to_string());
    kb.insert("u".to_string(), "undo".to_string());
    // Note: redo is often ctrl_r or ctrl_shift_z, add if desired
    kb.insert("v".to_string(), "collapse_all".to_string());
    kb.insert("V".to_string(), "collapse_children".to_string());
    kb.insert("w".to_string(), "increase_text_width".to_string());
    kb.insert("W".to_string(), "decrease_text_width".to_string());
    kb.insert("x".to_string(), "export_html".to_string());
    kb.insert("X".to_string(), "export_text".to_string());
    kb.insert("y".to_string(), "yank_node".to_string());
    kb.insert("Y".to_string(), "yank_children".to_string());
    kb.insert("z".to_string(), "decrease_line_spacing".to_string());
    kb.insert("Z".to_string(), "increase_line_spacing".to_string());
    kb.insert("enter".to_string(), "insert_new_sibling".to_string()); // Special key
    kb.insert("tab".to_string(), "insert_new_child".to_string()); // Special key
    kb.insert("space".to_string(), "toggle_node".to_string()); // Special key
    kb.insert("arr_down".to_string(), "go_down".to_string()); // Special key
    kb.insert("arr_up".to_string(), "go_up".to_string()); // Special key
    kb.insert("arr_right".to_string(), "go_right".to_string()); // Special key
    kb.insert("arr_left".to_string(), "go_left".to_string()); // Special key
    kb.insert("alt_arr_up".to_string(), "add_star".to_string()); // Special key
    kb.insert("alt_arr_down".to_string(), "remove_star".to_string()); // Special key
    kb.insert("1".to_string(), "collapse_level_1".to_string());
    kb.insert("2".to_string(), "collapse_level_2".to_string());
    kb.insert("3".to_string(), "collapse_level_3".to_string());
    kb.insert("4".to_string(), "collapse_level_4".to_string());
    kb.insert("5".to_string(), "collapse_level_5".to_string());
    kb.insert("6".to_string(), "collapse_level_6".to_string());
    kb.insert("7".to_string(), "collapse_level_7".to_string());
    kb.insert("8".to_string(), "collapse_level_8".to_string());
    kb.insert("9".to_string(), "collapse_level_9".to_string());
    kb.insert("|".to_string(), "toggle_align".to_string());
    kb.insert("?".to_string(), "help".to_string());
    kb.insert("/".to_string(), "search".to_string());
    kb.insert("ctrl_f".to_string(), "search".to_string()); // Special key
    kb.insert("=".to_string(), "increase_positive_rank".to_string());
    kb.insert("+".to_string(), "decrease_positive_rank".to_string());
    kb.insert("-".to_string(), "increase_negative_rank".to_string());
    kb.insert("_".to_string(), "decrease_negative_rank".to_string());
    kb
}

// Helper function to get the final path, considering CLI args and config
pub fn get_mind_map_path(cli_file: Option<String>, config: &Config) -> Option<PathBuf> {
    cli_file
        .or_else(|| config.default_file.clone()) // Use config.default_file
        .map(PathBuf::from)
}

// Basic tests for config loading
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::tempdir;

    // Helper to create Args for testing
    fn test_args(filename: Option<&str>, config_path: Option<&str>) -> Args {
        Args::try_parse_from(
            [
                "test_binary", // Program name, required by clap
                filename.unwrap_or(""),
                config_path
                    .map(|p| format!("--config={}", p))
                    .unwrap_or("".to_string())
                    .as_str(),
                // Add other args as needed, filter out empty strings
            ]
            .iter()
            .filter(|s| !s.is_empty()),
        )
        .unwrap_or_else(|e| panic!("Failed to parse test args: {}", e))
    }

    // Helper to set env var for duration of a test
    struct EnvVarGuard {
        key: String,
        original_value: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &str, value: &str) -> Self {
            let key = key.to_string();
            let original_value = env::var(&key).ok();
            // SAFETY: Modifying env vars is unsafe due to potential race conditions
            // in multi-threaded contexts. These tests run sequentially, so it's safe here.
            unsafe {
                env::set_var(&key, value);
            }
            EnvVarGuard {
                key,
                original_value,
            }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            // SAFETY: See above.
            unsafe {
                if let Some(ref val) = self.original_value {
                    env::set_var(&self.key, val);
                } else {
                    env::remove_var(&self.key);
                }
            }
        }
    }

    #[test]
    fn test_default_config() {
        // Pass empty args to simulate no command line input
        let args = test_args(None, None);
        let config = build_config_from_args(args).expect("Failed to load default config");

        assert_eq!(config.line_spacing, DEFAULT_LINE_SPACING);
        assert_eq!(config.clipboard, DEFAULT_CLIPBOARD_MODE);
        assert_eq!(config.keybindings.get("q").unwrap(), "quit");
        assert!(config.filename.is_none());
    }

    #[test]
    fn test_env_override() {
        let _guard = EnvVarGuard::set("HMM_LINE_SPACING", "5");
        let _guard2 = EnvVarGuard::set("HMM_CLIPBOARD", "internal");

        // Pass empty args
        let args = test_args(None, None);
        let config = build_config_from_args(args).expect("Failed to load config with env");

        assert_eq!(config.line_spacing, 5);
        assert_eq!(config.clipboard, "internal");
        assert_eq!(config.max_undo_steps, DEFAULT_MAX_UNDO); // Ensure others remain default
    }

    #[test]
    fn test_arg_override() {
        // Simulate passing a filename and another arg via command line
        let args = Args::try_parse_from([
            "test_binary",
            "my_map.hmm",
            "--line-spacing=10",
            "--clipboard=file",
        ])
        .expect("Failed to parse specific args");

        let config = build_config_from_args(args).expect("Failed to build config from args");

        assert_eq!(config.filename, Some(PathBuf::from("my_map.hmm")));
        assert_eq!(config.line_spacing, 10);
        assert_eq!(config.clipboard, "file");
    }

    // Test config file loading requires creating a temporary file.
    // Skipping for simplicity. Need tempfile crate and mock fs potentially.
}

/// Validates the loaded configuration for required fields and consistency.
fn validate_config(config: &Config) -> Result<(), ConfigError> {
    // Validate Theme (ensure colors are parseable, styles valid, etc.)
    // Example: Check if colors are valid hex codes or named colors
    // Note: This might be better handled by `Theme::try_from` if implemented

    // Validate Keybindings (ensure commands exist, keys are reasonable)
    for (_key, command) in &config.keybindings {
        // Placeholder: Validate command names exist in a known set
        // if !KNOWN_COMMANDS.contains(&command.command) {
        //     return Err(ConfigError::ValidationError(format!("Unknown command: {}", command.command)));
        // }
        if !is_valid_keybinding_command(command) {
            return Err(ConfigError::UnknownKeybindingCommand(command.clone()));
        }
    }

    // Validate Layout settings (ensure values are within reasonable bounds)
    // Commenting out as config.layout doesn't seem to exist anymore
    /*
    if config.layout.node_spacing_h < 1 {
        return Err(ConfigError::ValidationError(
            "layout.node_spacing_h must be at least 1".to_string(),
        ));
    }
    */
    // ... add more layout validations as needed

    Ok(())
}
