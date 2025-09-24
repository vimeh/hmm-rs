use anyhow::Result;
use clap::Parser;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "h-m-m")]
#[command(version, about = "A simple, fast, keyboard-centric terminal-based tool for working with mind maps", long_about = None)]
pub struct CliArgs {
    /// The mind map file to open
    pub filename: Option<PathBuf>,

    /// Custom configuration file path
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Debug configuration
    #[arg(long)]
    pub debug_config: bool,

    /// Initial depth to expand nodes
    #[arg(long)]
    pub initial_depth: Option<usize>,

    /// Show hidden nodes
    #[arg(long)]
    pub show_hidden: Option<bool>,

    /// Auto-save mode
    #[arg(long)]
    pub auto_save: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_max_parent_node_width")]
    pub max_parent_node_width: usize,

    #[serde(default = "default_max_leaf_node_width")]
    pub max_leaf_node_width: usize,

    #[serde(default = "default_line_spacing")]
    pub line_spacing: usize,

    #[serde(default = "default_symbol1")]
    pub symbol1: String,

    #[serde(default = "default_symbol2")]
    pub symbol2: String,

    #[serde(default = "default_show_hidden")]
    pub show_hidden: bool,

    #[serde(default = "default_initial_depth")]
    pub initial_depth: usize,

    #[serde(default = "default_center_lock")]
    pub center_lock: bool,

    #[serde(default = "default_focus_lock")]
    pub focus_lock: bool,

    #[serde(default = "default_max_undo_steps")]
    pub max_undo_steps: usize,

    #[serde(default = "default_auto_save")]
    pub auto_save: bool,

    #[serde(default = "default_auto_save_interval")]
    pub auto_save_interval: usize,

    #[serde(default = "default_echo_keys")]
    pub echo_keys: bool,

    #[serde(default = "default_post_export_command")]
    pub post_export_command: String,

    #[serde(default = "default_clipboard")]
    pub clipboard: ClipboardType,

    #[serde(default = "default_clipboard_file")]
    pub clipboard_file: PathBuf,

    #[serde(default)]
    pub clipboard_in_command: String,

    #[serde(default)]
    pub clipboard_out_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardType {
    Os,
    File,
    Command,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            max_parent_node_width: default_max_parent_node_width(),
            max_leaf_node_width: default_max_leaf_node_width(),
            line_spacing: default_line_spacing(),
            symbol1: default_symbol1(),
            symbol2: default_symbol2(),
            show_hidden: default_show_hidden(),
            initial_depth: default_initial_depth(),
            center_lock: default_center_lock(),
            focus_lock: default_focus_lock(),
            max_undo_steps: default_max_undo_steps(),
            auto_save: default_auto_save(),
            auto_save_interval: default_auto_save_interval(),
            echo_keys: default_echo_keys(),
            post_export_command: default_post_export_command(),
            clipboard: default_clipboard(),
            clipboard_file: default_clipboard_file(),
            clipboard_in_command: String::new(),
            clipboard_out_command: String::new(),
        }
    }
}

fn default_max_parent_node_width() -> usize {
    25
}
fn default_max_leaf_node_width() -> usize {
    55
}
fn default_line_spacing() -> usize {
    1
}
fn default_symbol1() -> String {
    "✓".to_string()
}
fn default_symbol2() -> String {
    "✗".to_string()
}
fn default_show_hidden() -> bool {
    false
}
fn default_initial_depth() -> usize {
    1
}
fn default_center_lock() -> bool {
    false
}
fn default_focus_lock() -> bool {
    false
}
fn default_max_undo_steps() -> usize {
    24
}
fn default_auto_save() -> bool {
    false
}

fn default_auto_save_interval() -> usize {
    30 // 30 seconds default
}
fn default_echo_keys() -> bool {
    false
}
fn default_post_export_command() -> String {
    String::new()
}
fn default_clipboard() -> ClipboardType {
    ClipboardType::Os
}
fn default_clipboard_file() -> PathBuf {
    PathBuf::from("/tmp/h-m-m")
}

pub fn load_config(args: &CliArgs) -> Result<AppConfig> {
    let mut config = config::Config::builder();

    // Start with default values
    config = config.add_source(config::Config::try_from(&AppConfig::default())?);

    // Try to load from config file
    let config_path = if let Some(ref path) = args.config {
        path.clone()
    } else {
        get_default_config_path()
    };

    if config_path.exists() {
        config = config.add_source(config::File::from(config_path));
    }

    // Apply environment variables prefixed with HMM_
    config = config.add_source(config::Environment::with_prefix("HMM").separator("_"));

    // Apply command line arguments
    if let Some(depth) = args.initial_depth {
        config = config.set_override("initial_depth", depth as i64)?;
    }
    if let Some(show) = args.show_hidden {
        config = config.set_override("show_hidden", show)?;
    }
    if let Some(auto) = args.auto_save {
        config = config.set_override("auto_save", auto)?;
    }

    let config = config.build()?;
    Ok(config.try_deserialize()?)
}

fn get_default_config_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "h-m-m") {
        let config_dir = proj_dirs.config_dir();
        config_dir.join("h-m-m.conf")
    } else {
        // Fallback to home directory
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("h-m-m")
            .join("h-m-m.conf")
    }
}
