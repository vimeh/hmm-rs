use ratatui::style::Style;

// Type aliases for clarity
pub type CharBuffer = Vec<Vec<char>>;
pub type StyleBuffer = Vec<Vec<Style>>;

// Constants for rendering
pub const CURSOR_INDICATOR: char = '▌';
pub const STATUS_EDIT_PREFIX: &str = "Edit: ";
pub const STATUS_SEARCH_PREFIX: &str = "Search: ";

// Connection line constants
#[allow(dead_code)]
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
#[allow(dead_code)]
pub mod junction {
    pub const VERTICAL: char = '│';
    pub const TOP_CORNER: char = '╭';
    pub const BOTTOM_CORNER: char = '╰';
    pub const TOP_RIGHT: char = '╮';
    pub const BOTTOM_RIGHT: char = '╯';
    pub const MIDDLE_RIGHT: char = '┤';
    pub const MIDDLE_LEFT: char = '├';
    pub const CROSS: char = '┼';
    pub const TOP_TEE: char = '┬';
    pub const BOTTOM_TEE: char = '┴';
}
