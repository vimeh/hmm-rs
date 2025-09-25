use ratatui::style::Style;

// Type aliases for clarity
pub type CharBuffer = Vec<Vec<char>>;
pub type StyleBuffer = Vec<Vec<Style>>;

// Constants for rendering
pub const CURSOR_INDICATOR: char = '▌';
pub const NODE_MIDDLE_Y_OFFSET: f64 = 0.6;
pub const VERTICAL_CONNECTOR_OFFSET: f64 = 1.0;
pub const MIDDLE_CONNECTOR_Y_OFFSET: f64 = 0.2;
pub const STATUS_EDIT_PREFIX: &str = "Edit: ";
pub const STATUS_SEARCH_PREFIX: &str = "Search: ";

// Connection line constants
pub mod connections {
    pub const SINGLE: &str = "─────";
    pub const SINGLE_HIDDEN: &str = "─╫───";
    pub const MULTI: &str = "────";
    pub const MULTI_HIDDEN: &str = "─╫──";
    pub const COLLAPSED: &str = " [+]";
    pub const COLLAPSED_HIDDEN: &str = "─╫─ [+]";
    pub const HIDDEN_ONLY: &str = "─╫─";
}

// Junction characters
pub mod junction {
    pub const VERTICAL: char = '│';
    pub const TOP_CORNER: char = '╭';
    pub const BOTTOM_CORNER: char = '╰';
    pub const TOP_RIGHT: char = '╮';
    pub const BOTTOM_RIGHT: char = '╯';
    pub const MIDDLE_RIGHT: char = '┤';
    pub const CROSS: char = '┼';
    pub const TOP_TEE: char = '┬';
}
