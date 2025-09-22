mod actions;
mod app;
mod config;
mod event;
mod layout;
mod model;
mod parser;
mod ui;

use anyhow::Result;
use app::AppState;
use clap::Parser;
use config::{CliArgs, load_config};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

fn main() -> Result<()> {
    // Parse command line arguments
    let args = CliArgs::parse();

    // Load configuration
    let config = load_config(&args)?;

    if args.debug_config {
        println!("Configuration:");
        println!("{:#?}", config);
        return Ok(());
    }

    // Create application state
    let mut app = AppState::new(config);

    // Load file if provided
    if let Some(ref filename) = args.filename {
        let (tree, root_id) = parser::load_file(filename)?;
        app.tree = tree;
        app.root_id = Some(root_id);
        app.active_node_id = Some(root_id);
        app.filename = Some(filename.clone());
    } else {
        // Create a new empty map
        let root = app
            .tree
            .new_node(model::Node::new("New Mind Map".to_string()));
        app.root_id = Some(root);
        app.active_node_id = Some(root);
    }

    // Initialize the first history entry
    app.push_history();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Clear the terminal
    terminal.clear()?;

    // Run the main loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Handle any errors from the main loop
    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
) -> Result<()> {
    while app.running {
        // Draw the UI
        terminal.draw(|frame| ui::render(frame, app))?;

        // Handle events
        if let Some(action) = event::handle_events(app)? {
            actions::execute_action(action, app)?;
        }

        // Auto-save if enabled
        if app.config.auto_save && app.filename.is_some() {
            // TODO: Implement auto-save timer
        }
    }

    Ok(())
}
