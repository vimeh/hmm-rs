use clap::{Parser, Subcommand};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE", help = "Path to the mind map file")]
    pub file: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Adds a new node to the mind map
    Add {
        #[arg(short, long, help = "ID of the parent node")]
        parent_id: Option<Uuid>,

        #[arg(help = "Text content of the new node")]
        text: String,
    },
    /// Edits the text of an existing node
    Edit {
        #[arg(help = "ID of the node to edit")]
        id: Uuid,

        #[arg(help = "New text content for the node")]
        text: String,
    },
    /// Deletes a node and its children
    Delete {
        #[arg(help = "ID of the node to delete")]
        id: Uuid,
    },
    /// Shows the mind map structure
    Show {
        #[arg(short, long, help = "Maximum depth to display")]
        depth: Option<usize>,

        #[arg(short, long, help = "Display format (e.g., tree, json)")]
        format: Option<String>, // Consider using an enum later
    },
    /// Imports a mind map from a file
    Import {
        #[arg(help = "Path to the file to import from")]
        path: String,

        #[arg(short, long, help = "Format of the input file (e.g., json, markdown)")]
        format: Option<String>, // Consider using an enum later
    },
    /// Exports the mind map to a file
    Export {
        #[arg(help = "Path to the file to export to")]
        path: String,

        #[arg(
            short,
            long,
            help = "Format of the output file (e.g., json, markdown, dot)"
        )]
        format: Option<String>, // Consider using an enum later
    },
}
