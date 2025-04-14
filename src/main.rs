mod commands;
mod config;
mod core;
mod errors;
mod io;
mod ui;

use crate::commands::{Cli, Commands};
use crate::config::{get_mind_map_path, load_config};
use crate::core::{MindMap, NodeId};
use crate::errors::{AppError, AppResult};
use crate::io::{load_map_json, save_map_json};
use crate::ui::{/* UiError, */ run as run_tui};
use clap::Parser;
use std::path::PathBuf;

fn main() -> AppResult<()> {
    let cli = Cli::parse();

    let config = load_config()?;

    let map_path = get_mind_map_path(cli.file.clone(), &config)
        .unwrap_or_else(|| PathBuf::from("mind_map.json"));

    let mut map = load_map_json(&map_path)?;
    let mut modified = false;

    // Ensure the map has a root node before starting TUI or performing operations
    if map.root.is_none() {
        println!("Mind map file not found or empty, creating default root node.");
        // Add a default root node if the map is empty
        map.add_node("Root".to_string(), None)?;
        modified = true; // Mark as modified since we added the root
    }

    match cli.command {
        Some(command) => match command {
            Commands::Add { parent_id, text } => {
                handle_add(&mut map, text, parent_id)?;
                modified = true;
            }
            Commands::Edit { id, text } => {
                handle_edit(&mut map, id, text)?;
                modified = true;
            }
            Commands::Delete { id } => {
                handle_delete(&mut map, id)?;
                modified = true;
            }
            Commands::Show { depth, format } => {
                handle_show(&map, depth, format)?;
            }
            Commands::Import { path, format } => {
                println!("Importing from '{}', format: {:?}", path, format);
            }
            Commands::Export { path, format } => {
                println!("Exporting to '{}', format: {:?}", path, format);
            }
        },
        None => {
            println!("No command specified, starting TUI...");
            run_tui(map.clone(), config.clone()).map_err(AppError::Ui)?;
            modified = false;
        }
    }

    if modified {
        save_map_json(&map, &map_path)?;
        println!("Mind map saved to {}", map_path.display());
    }

    Ok(())
}

fn handle_add(map: &mut MindMap, text: String, parent_id: Option<NodeId>) -> AppResult<()> {
    let new_id = map.add_node(text, parent_id)?;
    println!("Added node with ID: {}", new_id);
    Ok(())
}

fn handle_edit(map: &mut MindMap, id: NodeId, new_text: String) -> AppResult<()> {
    map.edit_node(id, new_text)?;
    println!("Edited node with ID: {}", id);
    Ok(())
}

fn handle_delete(map: &mut MindMap, id: NodeId) -> AppResult<()> {
    map.delete_node(id)?;
    println!("Deleted node with ID: {}", id);
    Ok(())
}

fn handle_show(map: &MindMap, depth: Option<usize>, format: Option<String>) -> AppResult<()> {
    println!("Showing map (Depth: {:?}, Format: {:?})", depth, format);
    if let Some(root_id) = map.root {
        print_node_recursive(map, root_id, 0, depth.unwrap_or(usize::MAX));
    } else {
        println!("(Empty map)");
    }
    Ok(())
}

fn print_node_recursive(map: &MindMap, node_id: NodeId, level: usize, max_depth: usize) {
    if level >= max_depth {
        return;
    }
    if let Some(node) = map.get_node(node_id) {
        let indent = "  ".repeat(level);
        println!(
            "{}{}- {} ({})",
            indent,
            if node.children.is_empty() { "-" } else { "+" },
            node.text,
            node.id
        );
        for &child_id in &node.children {
            print_node_recursive(map, child_id, level + 1, max_depth);
        }
    }
}
