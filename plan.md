
Okay, here is a plan for rewriting the `h-m-m` PHP tool in Rust, designed for an LLM implementation assistant. This plan focuses on Rust best practices, modularity, and clear milestones.

**Goal:** Create a Rust version of `h-m-m` that mirrors the functionality of the original PHP script, leveraging Rust's strengths in performance, safety, and maintainability.

**Core Principles:**

* **Safety:** Leverage Rust's ownership and borrowing rules to prevent common errors. Avoid `unsafe` unless absolutely necessary and well-justified.
* **Modularity:** Structure the code logically into modules.
* **Error Handling:** Use `Result` and potentially custom error types (e.g., with `thiserror`) for robust error management.
* **Performance:** While prioritizing correctness, keep performance in mind, especially for rendering and large map operations.
* **Idiomatic Rust:** Use standard library features, iterators, pattern matching, and common crates where appropriate.
* **Testing:** Incorporate unit and potentially integration tests.

---

**Plan & Milestones:**

**Phase 1: Project Setup & Core Structures**

1. **Project Initialization:**
    * Create a new binary crate: `cargo new hmm-rs --bin`
    * Initialize git repository: `git init`
    * Add initial dependencies to `Cargo.toml`:
        * `clap` (with `derive` feature) for argument parsing.
        * `serde` (with `derive` feature) for serialization/deserialization.
        * `config` or `toml` for config file parsing.
        * `dirs` for finding default config paths.
        * `ratatui` (or another TUI library like `crossterm` directly if preferred) for terminal UI.
        * `arboard` for cross-platform clipboard access.
        * `unicode-width` for calculating terminal width of characters.
        * `thiserror` for custom error types.
    * **Milestone:** Project compiles and runs, printing a simple "Hello" message. Basic `Cargo.toml` is set up.

2. **Define Core Data Structures (`src/core/map.rs`):**
    * `NodeId`: Define a type alias or newtype for node identifiers (e.g., `usize`).
    * `Node`: Struct representing a mind map node.
        * Fields: `id: NodeId`, `title: String`, `parent: Option<NodeId>`, `children: Vec<NodeId>`, `collapsed: bool`, `hidden: bool`, `rank_pos: u32`, `rank_neg: u32`, `stars: u8`, `symbol: Option<String>`. *Note: Visual attributes like `x, y, w, h` will be handled separately during rendering state calculation.*
    * `MindMap`: Struct holding the overall map state.
        * Fields: `nodes: HashMap<NodeId, Node>`, `root_id: NodeId`, `active_node_id: NodeId`, `next_id: NodeId`, `filename: Option<PathBuf>`, `modified: bool`. Use `HashMap` for efficient node lookup by ID.
    * Implement basic methods for `MindMap`: `new()`, `add_node()`, `get_node()`, `get_node_mut()`.
    * **Milestone:** Core data structures defined and compile. Add basic unit tests for `MindMap::add_node` and parent/child relationship tracking.

3. **Configuration Handling (`src/config.rs`):**
    * Define a `Config` struct mirroring PHP settings (e.g., `max_parent_node_width`, `line_spacing`, `active_node_color`, `keybindings`, `clipboard_mode`, etc.). Use `serde::Deserialize`.
    * Implement loading logic:
        * Defaults.
        * Config file (e.g., `~/.config/hmm-rs/config.toml` or platform equivalent using `dirs`). Use `config` crate or `toml` + `serde`.
        * Environment variables (e.g., `HMM_LINE_SPACING`). Prefix them clearly.
        * Command-line arguments using `clap`. Define arguments mirroring PHP options (`--line-spacing`, `--config`, filename).
    * Implement precedence: Arguments > Environment > Config File > Defaults.
    * Define a `Keybindings` struct/map to parse and store key bindings.
    * **Milestone:** Implement configuration loading. Write unit tests verifying the loading sources and precedence rules. Parse command-line arguments including the optional filename.

**Phase 2: File Handling & Basic TUI**

4. **File I/O & Parsing (`src/io.rs`):**
    * Implement `load_map(path: &Path) -> Result<MindMap, AppError>`:
        * Read the file line by line.
        * Handle UTF-8 encoding.
        * Parse the tab indentation to build the `MindMap` structure (Refactor PHP `list_to_map`). Handle potential parsing errors.
        * Handle creating a default root if the file has multiple top-level items.
        * Handle empty or non-existent files (create a default map).
    * Implement `save_map(map: &MindMap) -> Result<(), AppError>`:
        * Requires a `filename` to be set on the `MindMap`.
        * Recursively traverse the map starting from the `root_id` and write nodes with correct indentation (Refactor PHP `map_to_list`).
    * Define a custom `AppError` enum using `thiserror` for I/O and parsing errors.
    * **Milestone:** Implement loading and saving `.hmm` files. Create test files (simple, nested, multiple roots) and verify that loading and saving preserves the structure and content. Handle file errors gracefully.

5. **Basic TUI Setup (`src/ui/mod.rs`, `src/main.rs`):**
    * Set up terminal: Enter alternate screen, enable raw mode, hide cursor on start. Restore terminal on exit (use drop guard or explicit cleanup).
    * Create a main loop in `main.rs`.
    * Integrate `ratatui`: Create a `Terminal` backend.
    * Define a `TuiState` struct to hold UI-specific state (viewport offset `top_left: (u16, u16)`, maybe active node position, screen dimensions).
    * Implement a basic `draw` function in `src/ui/mod.rs` that clears the screen and draws a static string (e.g., "h-m-m").
    * Implement basic event handling: Read keyboard input (`crossterm::event::read`). Quit the application on 'q'.
    * **Milestone:** Application starts, enters alternate screen, displays a static message, and exits cleanly on 'q'. Terminal state is properly restored.

**Phase 3: Rendering & Core Operations**

6. **Map Rendering (`src/ui/render.rs`):**
    * Define a `RenderNode` struct containing calculated layout info: `id: NodeId`, `display_title: String`, `x: u16`, `y: u16`, `w: u16`, `h: u16`, `children: Vec<NodeId>`, `is_leaf: bool`, `collapsed: bool`, etc.
    * Implement layout calculation logic (refactor PHP `calculate_x_and_lh`, `calculate_h`, `calculate_y`, `calculate_aligned_x`, etc.):
        * This is complex. Start with a recursive function that traverses the visible tree.
        * Calculate required width/height based on text wrapping (`unicode-width`, `textwrap` crate might help).
        * Calculate `x` based on parent position and indentation/alignment rules.
        * Calculate `y` based on siblings' heights and spacing.
        * Store calculated layout in a temporary structure (e.g., `HashMap<NodeId, RenderNode>`) during the render pass.
    * Implement drawing logic in the main `draw` function:
        * Iterate through the calculated `RenderNode`s.
        * Draw node text within its calculated bounds (`x`, `y`, `w`, `h`), considering the viewport offset.
        * Draw connecting lines using box-drawing characters.
        * Highlight the active node using `Config` colors.
        * Handle collapsed nodes (`[+]` symbol).
        * Handle hidden nodes (don't calculate/draw if `config.show_hidden` is false).
    * Handle terminal resizing events.
    * **Milestone 1:** Render a loaded mind map structure with basic text and node boxes (no lines yet).
    * **Milestone 2:** Implement connecting line drawing between parent/children.
    * **Milestone 3:** Implement text wrapping and correct node sizing.
    * **Milestone 4:** Implement viewport scrolling based on active node position (`ui/state.rs`) and centering (`c` key). Active node highlighting works.

7. **Core Map Operations (`src/core/commands.rs`, integrate into `main.rs` event loop):**
    * Map keys from `Config` to specific command functions.
    * Implement navigation: `go_up`, `go_down`, `go_left`, `go_right`, `go_to_root`, `go_to_top`, `go_to_bottom`. Update `MindMap::active_node_id` and potentially viewport.
    * Implement node creation: `insert_new_sibling`, `insert_new_child`. Update `MindMap::nodes` and relationships. Set `modified = true`.
    * Implement node deletion: `delete_node`, `delete_children`. Update `MindMap::nodes`, relationships, and handle selecting the next active node. Set `modified = true`.
    * Implement toggling: `toggle_node` (collapse/expand). Update `Node::collapsed`.
    * **Milestone 1:** Navigation keys (`h`, `j`, `k`, `l`, `m`, `g`, `G`) work correctly, updating the active node highlight. Viewport follows active node.
    * **Milestone 2:** Node creation (`o`, `O`/`Tab`) and deletion (`d`, `Delete`) work. The map structure updates correctly on screen.
    * **Milestone 3:** Toggling node collapse (`Space`) works.

**Phase 4: Advanced Features & Refinement**

8. **Editing & Undo/Redo:**
    * Implement inline node editing (`e`, `i`, `a`, `E`, `I`, `A`):
        * Requires a separate UI state/mode.
        * Capture text input, handle cursor movement, backspace, delete, paste (`Ctrl+V`).
        * Update `Node::title` on `Enter`, cancel on `Esc`.
    * Implement Undo/Redo (`u`):
        * Store history of `MindMap::nodes` (or diffs) in a `Vec`. Limit size using `config.max_undo_steps`.
        * Manage an index into the history `Vec`.
        * Save active node ID along with state.
        * Set `modified = true` when changes are made after an undo.
    * **Milestone 1:** Inline editing works for the active node title.
    * **Milestone 2:** Undo/Redo works for title edits and node creation/deletion.

9. **Clipboard Operations:**
    * Implement Yank (`y`, `Y`): Use `arboard` to copy the selected node/subtree (in `.hmm` format) to the OS clipboard (or handle other `clipboard_mode` options).
    * Implement Paste (`p`, `P`): Read text from the clipboard, parse it using `io::parse_subtree_fragment` (similar to `load_map` but takes a parent ID), and insert as children/siblings. Set `modified = true`.
    * Implement Append (`Ctrl+P`): Append clipboard text to the current node's title.
    * **Milestone:** Yank and Paste using the OS clipboard work correctly for single nodes and subtrees.

10. **Other Features:**
    * Implement Search (`/`, `n`, `N`): Prompt for query, find matching nodes, update active node, highlight matches.
    * Implement Marks/Ranks/Stars (`t`, `#`, `=`, `+`, `-`, `_`, `Alt+Up/Down`): Update corresponding `Node` fields.
    * Implement Hiding (`H`, `Ctrl+H`): Update `Node::hidden` and toggle `config.show_hidden`. Filter nodes during layout calculation based on `show_hidden`.
    * Implement Sorting (`T`): Sort siblings based on title or rank.
    * Implement Export (`x`, `X`): Generate HTML string or plain text map representation. Handle `post_export_command`.
    * Implement Open Link (`Ctrl+O`): Use `open` crate or platform commands.
    * Implement focus modes (`f`, `F`, `r`, `R`) by adjusting `collapsed` flags on nodes.
    * **Milestone:** Implement Search, Marks/Ranks/Stars, and Export to Text.

11. **Refinement & Release Prep:**
    * **Error Handling:** Ensure all fallible operations return `Result` and errors are handled gracefully (e.g., displayed as messages in the UI).
    * **Testing:** Add more unit tests for command logic and parsing edge cases. Consider integration tests if feasible (TUI makes this hard).
    * **Linting & Formatting:** Run `cargo fmt` and `cargo clippy -- -D warnings`. Address all lints.
    * **Documentation:** Add `///` doc comments to public items. Create/update `README.md` with Rust-specific build/install/usage instructions.
    * **Cross-Platform Testing:** Test basic functionality on Linux, macOS, and Windows (if possible).
    * **Milestone:** Code is well-formatted, lint-free, documented. README is updated. Basic cross-platform compatibility confirmed.

---

This plan provides a structured approach with verifiable milestones. The LLM should focus on implementing one milestone at a time, ensuring compilation and passing relevant tests before moving on. Remember to handle state updates (especially `modified` flag) and trigger redraws after commands modify the `MindMap`.
