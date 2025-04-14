// use crate::commands::{self, InsertMode}; // Import commands
use crate::config::Config; // Add Config import back
use crate::core::{MindMap, NodeId};
// use crate::errors::AppResult;
use crossterm::{
    event::{
        self,
        DisableMouseCapture,
        EnableMouseCapture,
        Event as CEvent,
        KeyCode,
        KeyEvent,
        // Remove unused KeyModifiers
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    prelude::{CrosstermBackend, Size, Terminal},
    // Remove unused Style, Borders, Paragraph
    // style::Style,
    // widgets::{Borders, Paragraph},
};
// use std::collections::HashMap;
use std::io::{self, Stdout, stdout};
use std::time::Duration; // Add Duration import back
// use std::sync::mpsc;
// use std::thread;
// use std::time::{Duration, Instant};
use std::collections::HashMap;

pub mod render;
// pub mod state;

use render::{RenderNode, calculate_layout, draw_map}; // Use items from render

// Define potential errors during UI operations.
#[derive(thiserror::Error, Debug)]
pub enum UiError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    // Add other UI-specific errors later if needed
}

// State specific to the TUI
struct TuiState {
    active_node_id: NodeId, // Add active_node_id to TuiState
    viewport_y: u16,        // Vertical scroll offset
    viewport_x: u16,        // Horizontal scroll offset
                            // Add messages, input mode, etc. later
}

// Main function to run the TUI application loop.
pub fn run(mut map: MindMap, config: Config) -> Result<(), UiError> {
    let mut terminal = setup_terminal()?;
    // Handle Option for map.root
    let initial_active_node = map.root.expect("MindMap must have a root node to run TUI");
    let mut app_state = TuiState {
        active_node_id: initial_active_node,
        viewport_y: 0,
        viewport_x: 0,
    };

    loop {
        let terminal_rect: Size = match terminal.size() {
            Ok(size) => size,
            Err(e) => {
                eprintln!("Failed to get terminal size: {}", e);
                // Provide a default size or handle the error appropriately
                // For now, let's return early or use a default.
                // Returning early might be problematic if draw must complete.
                // Using a default Size:
                Size::new(80, 24) // Example default size
            }
        };
        // We have terminal_rect which is a Size.
        // We don't need terminal_area anymore as calculate_layout expects Size.
        // let terminal_area = Rect::new(0, 0, terminal_rect.width, terminal_rect.height);

        // Pass the Size directly to calculate_layout
        let layout: HashMap<NodeId, RenderNode> = calculate_layout(&map, &config, terminal_rect);

        terminal.draw(|frame| draw_ui(frame, &map, &config, &layout, &app_state))?;

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                CEvent::Key(key_event) => {
                    handle_key_event(key_event, &mut map, &config, &mut app_state);
                    if key_event.code == KeyCode::Char('q') {
                        break;
                    }
                    // Pass the Size to adjust_viewport (already correctly typed)
                    adjust_viewport(&map, &layout, &mut app_state, terminal_rect);
                }
                _ => {} // Ignore other events
            }
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}

// Function to handle key events - takes KeyEvent now
fn handle_key_event(
    key_event: KeyEvent, // Changed from CEvent
    map: &mut MindMap,
    _config: &Config,
    state: &mut TuiState,
) {
    match key_event.code {
        KeyCode::Char('q') => { /* Quit handled in main loop */ }

        // Navigation
        KeyCode::Char('k') | KeyCode::Up => navigate(map, state, 0, -1),
        KeyCode::Char('j') | KeyCode::Down => navigate(map, state, 0, 1),
        KeyCode::Char('h') | KeyCode::Left => navigate(map, state, -1, 0),
        KeyCode::Char('l') | KeyCode::Right => navigate(map, state, 1, 0),

        // Node Creation - Commented out
        /*
        KeyCode::Char('o') => {
            // TODO: Handle potential error (e.g., display message)
            let _ = commands::insert_new_node(map, InsertMode::Sibling);
            // TODO: Trigger inline edit after creation
        }
        KeyCode::Char('O') | KeyCode::Tab => {
            // TODO: Handle potential error
            let _ = commands::insert_new_node(map, InsertMode::Child);
            // TODO: Trigger inline edit after creation
        }
        */

        // Node Deletion - Commented out
        /*
        KeyCode::Char('d') => {
            if key_event.modifiers == KeyModifiers::SHIFT {
                // Handle 'D' (Shift+d)
                // TODO: Handle potential error
                let _ = commands::delete_active_node_children(map);
            } else {
                // TODO: Handle potential error
                let _ = commands::delete_active_node(map);
                // TODO: Yank to clipboard before deleting
            }
        }
        KeyCode::Delete => {
            // Handle Delete key
            // TODO: Handle potential error
            let _ = commands::delete_active_node(map);
            // NOTE: This version doesn't yank to clipboard, matching PHP behavior
        }
        */
        KeyCode::Char(' ') | KeyCode::Enter => {
            if let Some(node) = map.get_node_mut(state.active_node_id) {
                if !node.children.is_empty() {
                    println!("Toggle collapse for node {}", node.id); // Placeholder action
                } else {
                    // Cannot navigate to parent as node.parent doesn't exist
                    // if let Some(parent_id) = node.parent {
                    //     state.active_node_id = parent_id;
                    // }
                    println!("Enter on leaf node {} (no action)", node.id);
                }
            }
        }

        // TODO: Implement other commands (Edit, Yank, Paste, etc.)
        _ => {}
    }
}

// Prefix unused _map parameter
fn navigate(_map: &mut MindMap, state: &mut TuiState, dx: i8, dy: i8) {
    let current_node_id = state.active_node_id;
    let current_node = match _map.get_node(current_node_id) {
        Some(n) => n,
        None => return,
    };

    let next_node_id = if dx > 0 {
        // Go Right (Child)
        current_node.children.first().cloned()
    } else if dx < 0 {
        // Go Left (Parent) - Cannot use node.parent
        // current_node.parent
        None // Disabled parent navigation for now
    } else if dy < 0 {
        // Go Up - Cannot use node.parent
        /*
        current_node.parent.and_then(|p_id| {
            _map.get_node(p_id).and_then(|p_node| {
                p_node
                    .children
                    .iter()
                    .position(|&id| id == current_node_id)
                    .and_then(|idx| idx.checked_sub(1))
                    .map(|prev_idx| p_node.children[prev_idx])
            })
        })
        */
        None // Disabled sibling navigation for now
    } else if dy > 0 {
        // Go Down - Cannot use node.parent
        /*
        current_node.parent.and_then(|p_id| {
            _map.get_node(p_id).and_then(|p_node| {
                p_node
                    .children
                    .iter()
                    .position(|&id| id == current_node_id)
                    .and_then(|idx| p_node.children.get(idx + 1))
                    .cloned()
            })
        })
        */
        None // Disabled sibling navigation for now
    } else {
        None // No movement
    };

    if let Some(next_id) = next_node_id {
        if _map.nodes.contains_key(&next_id) {
            state.active_node_id = next_id;
        }
    }
}

// Update adjust_viewport to use TuiState
fn adjust_viewport(
    _map: &MindMap, // Prefixed unused variable
    layout: &HashMap<NodeId, RenderNode>,
    state: &mut TuiState,
    term_size: ratatui::layout::Size, // Change Rect to Size
) {
    // Use state.active_node_id
    if let Some(active_render_node) = layout.get(&state.active_node_id) {
        let node_y = active_render_node.y;
        let node_h = active_render_node.h;
        let node_x = active_render_node.x;
        let node_w = active_render_node.w;

        // Adjust vertical viewport
        let margin_y = 2;
        if node_y < state.viewport_y + margin_y {
            state.viewport_y = node_y.saturating_sub(margin_y);
        } else if node_y + node_h > state.viewport_y + term_size.height.saturating_sub(margin_y) {
            state.viewport_y = (node_y + node_h + margin_y).saturating_sub(term_size.height);
        }

        // Adjust horizontal viewport
        let margin_x = 4;
        if node_x < state.viewport_x + margin_x {
            state.viewport_x = node_x.saturating_sub(margin_x);
        } else if node_x + node_w > state.viewport_x + term_size.width.saturating_sub(margin_x) {
            state.viewport_x = (node_x + node_w + margin_x).saturating_sub(term_size.width);
        }
    }
}

// Setup the terminal environment.
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, UiError> {
    enable_raw_mode()?; // Put terminal in raw mode
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

// Restore the terminal environment.
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), UiError> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

// Draw the complete UI
fn draw_ui(
    frame: &mut ratatui::Frame,
    map: &MindMap,
    config: &Config,
    layout: &HashMap<NodeId, RenderNode>,
    state: &TuiState,
) {
    // Use frame.area() which returns Rect
    let area = frame.area();
    // Draw the map content using the area (Rect)
    draw_map(
        frame,
        layout,
        config,
        area,
        state.viewport_y,
        state.viewport_x,
        map,
    );
    // TODO: Draw other UI elements like status bar, help message, etc.
}
