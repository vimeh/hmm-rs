## Plan: Rewriting `h-m-m` from PHP to Idiomatic Rust

**STATUS: ~75% Complete - Core architecture done, finishing features**

______________________________________________________________________

### 1. Project Goal

The objective is to create a new version of `h-m-m` in Rust that is more performant, robust, and maintainable. We will migrate from a single-file, procedural PHP script with global state to a structured, modular, and safe Rust application, while preserving all original features and keyboard-centric design.

### 2. Core Philosophy & Architectural Shift

The new architecture will be based on modern TUI (Terminal User Interface) application design principles, moving away from the global mutable array (`$mm`) of the PHP version.

- **State Management:** All application state (nodes, viewport, configuration, UI mode, etc.) will be encapsulated in a single `AppState` struct. Functions will operate on this struct, either immutably or mutably, making data flow explicit.
- **Modularity:** The codebase will be split into logical modules (e.g., `config`, `model`, `layout`, `tui`, `event`, `actions`).
- **Data Structures:** The mind map will be represented using a more robust data structure. While a `HashMap<NodeId, Node>` is a direct translation, we will use the `indextree` crate, which is perfect for representing tree structures in Rust without fighting the borrow checker.
- **Error Handling:** We will use Rust's `Result<T, E>` for all operations that can fail (file I/O, parsing). The `anyhow` crate will be used for ergonomic error handling.
- **Event-Driven Loop:** Instead of a busy-wait loop, we will use a blocking event loop that responds to user input, terminal resizes, and other events.

### 3. Recommended Crates

This is the foundational toolkit for the project.

- **`clap`**: For parsing command-line arguments (replaces manual `$argv` parsing).
- **`serde`**: For serialization and deserialization, primarily for handling the config file.
- **`config`**: To manage configuration from files, environment variables, and defaults (replaces manual config loading).
- **`crossterm`**: For low-level terminal manipulation: entering/leaving alternate screen, enabling raw mode, handling keyboard and mouse events.
- **`ratatui`**: A TUI library for drawing the UI in a declarative, buffer-based way. This is a significant improvement over printing ANSI codes directly.
- **`indextree`**: For storing the mind map tree structure efficiently and safely.
- **`anyhow`**: For flexible and easy-to-use error handling.
- **`clipboard`**: A cross-platform library to interact with the system clipboard.
- **`directories`**: To find platform-specific configuration directories.

### 4. Proposed Project Structure

```
h-m-m/
├── Cargo.toml
└── src/
    ├── main.rs         # Entry point, sets up terminal, main loop
    ├── app.rs          # AppState struct and core application logic
    ├── model.rs        # Data structures (Node, etc.)
    ├── config.rs       # Configuration loading and management
    ├── parser.rs       # Logic for parsing .hmm files (list_to_map)
    ├── layout.rs       # Mind map layout calculation engine
    ├── ui.rs           # Rendering logic using ratatui
    ├── event.rs        # Event handling (keyboard input)
    └── actions.rs      # Business logic for all user commands (move, edit, save, etc.)
```

### 5. Detailed Implementation Steps

#### Step 1: Project Setup and Core Data Models ✅ COMPLETE

1. **Initialize Project:** Run `cargo new h-m-m` and add the recommended crates to `Cargo.toml`.
1. **Define Models (`src/model.rs`):**
   - Create a `NodeId` type alias: `pub type NodeId = indextree::NodeId;`.
   - Define the `Node` struct. It will contain data intrinsic to a node, but *not* layout information.
     ```rust
     // src/model.rs
     pub struct Node {
         pub title: String,
         pub is_collapsed: bool,
         // Other properties like symbols, rank, etc. can be added here
     }
     ```
1. **Define App State (`src/app.rs`):**
   - Create an `enum AppMode { Normal, Editing, ... }` to manage UI states.
   - Create the main `AppState` struct.
     ```rust
     // src/app.rs
     use crate::model::{Node, NodeId};
     use indextree::Arena;

     pub struct AppState {
         pub running: bool,
         pub mode: AppMode,
         pub tree: Arena<Node>,
         pub root_id: Option<NodeId>,
         pub active_node_id: Option<NodeId>,
         pub config: AppConfig, // From src/config.rs
         // Viewport state
         pub viewport_top: f64,
         pub viewport_left: f64,
         // Undo/Redo history
         pub history: Vec<Arena<Node>>,
         pub history_index: usize,
     }
     ```

#### Step 2: Configuration Loading ✅ COMPLETE

1. **Implement (`src/config.rs`):**
   - Use `clap::Parser` to define a struct for command-line arguments.
   - Use `serde::Deserialize` on a struct for the `.conf` file settings.
   - Create a function `load_config()` that uses the `directories` crate to find the config file path, `clap` to parse args, and the `config` crate to merge defaults, file settings, environment variables (`HMM_*`), and arguments into a single `AppConfig` struct.

#### Step 3: Parsing the Mind Map File ✅ COMPLETE

1. **Implement (`src/parser.rs`):**
   - Create a function `parse_hmm_file(file_contents: &str) -> (Arena<Node>, NodeId)`.
   - This function will replicate the logic of `list_to_map` from the PHP script.
   - It will calculate indentation, track the parent for the current level, and build the tree by adding nodes to the `indextree::Arena`.
   - Handle the case of multiple root nodes by creating a synthetic "root" node, just like the original script.

#### Step 4: Terminal Setup and Main Event Loop ✅ COMPLETE

1. **Implement (`src/main.rs`):**
   - Create a `Tui` struct to manage the `crossterm` terminal state (alternate screen, raw mode). It should implement `Drop` to restore the terminal on exit.
   - The `main` function will:
     - Load the configuration.
     - Parse the input file into the `AppState.tree`.
     - Initialize the `Tui`.
     - Start the main loop: `while app.running { ... }`.
1. **Implement (`src/event.rs`):**
   - Create a function `handle_events(app: &mut AppState)`.
   - Inside the main loop in `main.rs`, call this handler.
   - Use `crossterm::event::read()` to get the next event.
   - Match on the `KeyEvent` and, based on `app.mode`, map it to an `Action` enum.
   - Dispatch the action to be handled in `src/actions.rs`.
     ```rust
     // A simplified event loop structure in main.rs
     loop {
         // 1. Draw the UI
         terminal.draw(|frame| ui::render(frame, &mut app))?;

         // 2. Handle events
         if let Some(action) = event::handle_key_events(&app)? {
             // 3. Update state based on action
             actions::execute_action(action, &mut app);
         }

         if !app.running {
             break;
         }
     }
     ```

#### Step 5: Layout Calculation Engine ⚠️ PARTIAL (70%)

1. **Implement (`src/layout.rs`):**
   - This is the most complex part. It translates all the coordinate calculation logic.
   - Create a `LayoutNode` struct that contains computed positions. The layout engine will produce a `HashMap<NodeId, LayoutNode>`.
     ```rust
     // src/layout.rs
     pub struct LayoutNode {
         pub x: f64, pub y: f64, // Position
         pub w: f64, pub h: f64, // Dimensions
         // ... and other layout-specific data like 'yo', 'xo'
     }
     ```
   - Create a main function `calculate_layout(app: &AppState) -> HashMap<NodeId, LayoutNode>`.
   - Port the PHP functions step-by-step into pure Rust functions that operate on the `AppState.tree`:
     - `calculate_x_and_lh`: A recursive function to determine node widths, heights (`lh`), and initial x-positions.
     - `calculate_h`: A function to determine the total height of sub-trees (`h`).
     - `calculate_y`: A recursive function to set the final y-positions.
     - These functions should traverse the `indextree` and return the calculated layout map.

#### Step 6: UI Rendering ⚠️ PARTIAL (60%)

1. **Implement (`src/ui.rs`):**
   - Create the main `render(frame: &mut Frame, app: &mut AppState)` function.
   - Inside `render`:
     1. Call `layout::calculate_layout(app)` to get the positions of all nodes.
     1. Use `ratatui` widgets and direct buffer manipulation to draw the mind map.
     1. Draw connection lines (`draw_connections`) using box-drawing characters.
     1. Draw the text for each node (`add_content_to_the_map`).
     1. Highlight the `active_node_id` with a different style.
     1. Render the status/message line at the bottom of the screen.
     1. If in `Editing` mode, render the text input line.

#### Step 7: Implementing User Actions ✅ MOSTLY COMPLETE (90%)

1. **Implement (`src/actions.rs`):**
   - Define a comprehensive `Action` enum covering all keybindings.
     ```rust
     // src/actions.rs
     pub enum Action {
         Quit,
         GoUp, GoDown, GoLeft, GoRight,
         EditNode,
         Save,
         ToggleNodeCollapse,
         // ... etc
     }
     ```
   - Create the `execute_action(action: Action, app: &mut AppState)` function.
   - Implement the logic for each action as a separate private function within the module. For example:
     - `fn move_up(app: &mut AppState)`: Implements the logic from `go_up`. Find the visually closest sibling or node above the current one.
     - `fn delete_node(app: &mut AppState)`: Implements `delete_node`. Removes the active node and its descendants from the `Arena`, copies the subtree to the clipboard, and updates `active_node_id`.
     - `fn save_file(app: &mut AppState)`: Implements `save`. This will require a `map_to_list` equivalent that traverses the `Arena` and generates the indented text format.
     - `fn enter_edit_mode(app: &mut AppState)`: Switches `app.mode` to `Editing` and prepares a text buffer.

#### Step 8: The "Magic Readline" Implementation ⚠️ PARTIAL (50%)

1. **State:** Add fields to `AppState` for the editor state, e.g., `editor_buffer: String`, `editor_cursor_pos: usize`.
1. **Mode:** When an "edit" action is triggered, switch `app.mode` to `Editing`.
1. **Event Handling:** In `event.rs`, when the mode is `Editing`, key presses are not mapped to `Action`s. Instead, they directly modify `editor_buffer` and `editor_cursor_pos` (e.g., character input, backspace, moving cursor left/right). The `Enter` key finalizes the edit, updates the node's title in the `tree`, and switches the mode back to `Normal`. `Escape` cancels.
1. **Rendering:** In `ui.rs`, if the mode is `Editing`, render the `editor_buffer` as a text input line at the bottom of the screen, with a styled block representing the cursor.

#### Step 9: Final Features and Polish 🚧 IN PROGRESS (40%)

1. **Clipboard:** Use the `clipboard` crate for all yank/paste/cut operations.
1. **Exporting:** Implement `export_html` and `export_text` by traversing the layout map and tree data.
1. **Undo/Redo:** Before any action that modifies the `app.tree`, push a clone of the tree onto the `app.history` vector. The `undo` action simply pops from this history and replaces the current tree.

This structured plan breaks the monolithic script into manageable, testable components and leverages the strengths of the Rust ecosystem to produce a superior final product.

---

## Implementation Status Summary (as of 2025-09-22)

### ✅ Completed Components:
- **Project Setup**: All dependencies installed, project structure in place
- **Core Data Models**: Node, NodeId, AppState fully implemented
- **Configuration**: Complete config loading with CLI args, config files, and env vars
- **File Parsing**: Full .hmm format parser and save functionality
- **Terminal Management**: Crossterm integration for alternate screen and raw mode
- **Event Loop**: Main application loop with event handling
- **Basic Actions**: Movement, node manipulation, editing, collapsing

### ⚠️ Partially Complete:
- **Layout Engine (70%)**: Basic layout calculations work, missing some edge cases
- **UI Rendering (60%)**: Basic rendering works, but connection drawing needs work
- **Magic Readline (50%)**: Basic editing works, missing advanced features
- **Export Functions (20%)**: Stubs exist, implementation needed

### 🔴 TODO - High Priority:
1. **Connection Drawing**: Port the complete box-drawing algorithm from PHP
2. **Viewport Management**: Implement proper scrolling and centering
3. **Export Functions**: Complete HTML and text export
4. **Missing Keybindings**: Add rank adjustment, star operations, link opening

### 🟡 TODO - Medium Priority:
5. **Visual Polish**: Add colors, symbols, and visual feedback
6. **Auto-save**: Implement timer-based saving
7. **Advanced Editing**: Word jumping, clipboard in edit mode
8. **Help Screen**: Complete help documentation

### 🟢 TODO - Low Priority:
9. **Tests**: Add comprehensive test coverage
10. **Performance**: Optimize for large mind maps
11. **Documentation**: User guide and API docs
12. **Packaging**: Distribution and installation scripts

**Overall Progress: ~75% Complete**

The core architecture is solid and most basic functionality works. The remaining work is primarily feature completion and polish rather than fundamental changes.
