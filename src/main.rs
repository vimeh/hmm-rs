use hmm_rs::{actions, app, config, event, model, parser, ui};

use anyhow::Result;
use app::AppState;
use clap::Parser;
use config::{load_config, CliArgs};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};

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
        if app.config.auto_save && app.filename.is_some() && app.is_dirty {
            let should_save = if let Some(last_modify) = app.last_modify_time {
                // Check if enough time has passed since last modification
                let elapsed = Instant::now().duration_since(last_modify);
                elapsed >= Duration::from_secs(app.config.auto_save_interval as u64)
            } else {
                false
            };

            if should_save {
                if let Err(e) = actions::save(app) {
                    app.set_message(format!("Auto-save failed: {}", e));
                } else {
                    app.last_save_time = Some(Instant::now());
                }
            }
        }
    }

    Ok(())
}
