use crate::app::{AppMode, AppState};
use crate::layout::LayoutEngine;
use crate::model::NodeId;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

// Type aliases for clarity
type CharBuffer = Vec<Vec<char>>;
type StyleBuffer = Vec<Vec<Style>>;

// Constants for rendering
const CURSOR_INDICATOR: char = '▌';
const NODE_MIDDLE_Y_OFFSET: f64 = 0.6;
const VERTICAL_CONNECTOR_OFFSET: f64 = 1.0;
const MIDDLE_CONNECTOR_Y_OFFSET: f64 = 0.2;
const STATUS_EDIT_PREFIX: &str = "Edit: ";
const STATUS_SEARCH_PREFIX: &str = "Search: ";

// Connection line constants module
mod connections {
    pub const SINGLE: &str = "─────";
    pub const SINGLE_HIDDEN: &str = "─╫───";
    pub const MULTI: &str = "────";
    pub const MULTI_HIDDEN: &str = "─╫──";
    pub const COLLAPSED: &str = " [+]";
    pub const COLLAPSED_HIDDEN: &str = "─╫─ [+]";
    pub const HIDDEN_ONLY: &str = "─╫─";
}

// Junction characters
mod junction {
    pub const VERTICAL: char = '│';
    pub const TOP_CORNER: char = '╭';
    pub const BOTTOM_CORNER: char = '╰';
    pub const TOP_RIGHT: char = '╮';
    pub const BOTTOM_RIGHT: char = '╯';
    pub const MIDDLE_RIGHT: char = '┤';
    pub const CROSS: char = '┼';
    pub const TOP_TEE: char = '┬';
}

// Help text organization
mod help {
    pub struct HelpSection {
        pub title: &'static str,
        pub items: &'static [(&'static str, &'static str)],
    }

    pub const SECTIONS: &[HelpSection] = &[
        HelpSection {
            title: "Navigation:",
            items: &[
                ("h/←", "Move left (parent)"),
                ("j/↓", "Move down"),
                ("k/↑", "Move up"),
                ("l/→", "Move right (child)"),
                ("g  ", "Go to top"),
                ("G  ", "Go to bottom"),
                ("m/~", "Go to root"),
            ],
        },
        HelpSection {
            title: "Editing:",
            items: &[
                ("e/i", "Edit node (append)"),
                ("E/I", "Edit node (replace)"),
                ("o/⏎", "Insert sibling"),
                ("O/⇥", "Insert child"),
                ("d  ", "Delete node"),
                ("D  ", "Delete children"),
            ],
        },
        HelpSection {
            title: "View:",
            items: &[
                ("␣  ", "Toggle collapse"),
                ("v  ", "Collapse all"),
                ("b  ", "Expand all"),
                ("1-5", "Collapse to level"),
            ],
        },
        HelpSection {
            title: "File:",
            items: &[("s  ", "Save"), ("S  ", "Save as"), ("q  ", "Quit")],
        },
    ];
}

// Main render function
pub fn render(frame: &mut Frame, app: &mut AppState) {
    // Update terminal size
    let size = frame.area();
    app.terminal_width = size.width;
    app.terminal_height = size.height;

    // Calculate layout
    let layout = LayoutEngine::calculate_layout(app);

    // Create main layout chunks
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(size);

    // Render based on mode
    match &app.mode {
        AppMode::Help => HelpRenderer::render(frame, chunks[0]),
        _ => {
            let renderer = MindMapRenderer::new(app, &layout);
            renderer.render(frame, chunks[0]);
        }
    }

    // Render status line
    StatusLineRenderer::render(frame, app, chunks[1]);
}

// Buffer canvas for drawing characters and styles
struct BufferCanvas {
    char_buffer: CharBuffer,
    style_buffer: StyleBuffer,
    width: usize,
    height: usize,
}

impl BufferCanvas {
    fn new(width: usize, height: usize) -> Self {
        Self {
            char_buffer: vec![vec![' '; width]; height],
            style_buffer: vec![vec![Style::default(); width]; height],
            width,
            height,
        }
    }

    fn set_char(&mut self, x: usize, y: usize, ch: char) {
        if self.in_bounds(x, y) {
            self.char_buffer[y][x] = ch;
        }
    }

    fn draw_text(&mut self, x: usize, y: usize, text: &str) {
        for (i, ch) in text.chars().enumerate() {
            self.set_char(x + i, y, ch);
        }
    }

    fn draw_styled_text(&mut self, x: usize, y: usize, text: &str, style: Style) {
        for (i, ch) in text.chars().enumerate() {
            if self.in_bounds(x + i, y) {
                self.char_buffer[y][x + i] = ch;
                self.style_buffer[y][x + i] = style;
            }
        }
    }

    fn in_bounds(&self, x: usize, y: usize) -> bool {
        y < self.height && x < self.width
    }

    fn to_lines(&self) -> Vec<Line<'_>> {
        let mut lines = Vec::new();

        for (y, row) in self.char_buffer.iter().enumerate() {
            let mut spans = Vec::new();
            let mut current_style = Style::default();
            let mut current_text = String::new();

            for (x, &ch) in row.iter().enumerate() {
                let style = self.style_buffer[y][x];
                if style != current_style {
                    if !current_text.is_empty() {
                        spans.push(Span::styled(current_text.clone(), current_style));
                        current_text.clear();
                    }
                    current_style = style;
                }
                current_text.push(ch);
            }

            if !current_text.is_empty() {
                spans.push(Span::styled(current_text, current_style));
            }

            lines.push(Line::from(spans));
        }

        lines
    }
}

// Mind map renderer
struct MindMapRenderer<'a> {
    app: &'a AppState,
    layout: &'a LayoutEngine,
}

impl<'a> MindMapRenderer<'a> {
    fn new(app: &'a AppState, layout: &'a LayoutEngine) -> Self {
        Self { app, layout }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let mut canvas = BufferCanvas::new(area.width as usize, area.height as usize);

        // Draw connections first (behind nodes)
        if let Some(root_id) = self.app.root_id {
            let mut conn_renderer =
                ConnectionRenderer::new(&mut canvas, self.app, self.layout, area);
            conn_renderer.draw_node_connections(root_id);
        }

        // Draw nodes on top
        if let Some(root_id) = self.app.root_id {
            self.draw_node_content(&mut canvas, root_id, area);
        }

        // Convert buffer to paragraph and render
        let lines = canvas.to_lines();
        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, area);
    }

    fn draw_node_content(&self, canvas: &mut BufferCanvas, node_id: NodeId, area: Rect) {
        let Some(node_ref) = self.app.tree.get(node_id) else {
            return;
        };
        let node = node_ref.get();

        let Some(node_layout) = self.layout.nodes.get(&node_id) else {
            return;
        };

        let x = (node_layout.x - self.app.viewport_left) as usize;
        let y = (node_layout.y + node_layout.yo - self.app.viewport_top) as usize;

        // Determine node style
        let style = self.get_node_style(node_id, node);

        // Draw node text
        if x < area.width as usize && y < area.height as usize {
            let lines = TextWrapper::wrap(&node.title, node_layout.w as usize);
            for (i, line) in lines.iter().enumerate() {
                let line_y = y + i;
                if line_y < area.height as usize {
                    canvas.draw_styled_text(x, line_y, line, style);
                }
            }
        }

        // Draw children if not collapsed
        if !node.is_collapsed {
            let children = self.get_visible_children(node_id);
            for child_id in children {
                self.draw_node_content(canvas, child_id, area);
            }
        }
    }

    fn get_node_style(&self, node_id: NodeId, node: &crate::model::Node) -> Style {
        if Some(node_id) == self.app.active_node_id {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if node.title.starts_with(&self.app.config.symbol1) {
            Style::default().fg(Color::Green)
        } else if node.title.starts_with(&self.app.config.symbol2) {
            Style::default().fg(Color::Red)
        } else if node.is_hidden() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        }
    }

    fn get_visible_children(&self, node_id: NodeId) -> Vec<NodeId> {
        if !self.app.config.show_hidden {
            node_id
                .children(&self.app.tree)
                .filter(|cid| {
                    self.app
                        .tree
                        .get(*cid)
                        .map(|n| !n.get().is_hidden())
                        .unwrap_or(false)
                })
                .collect()
        } else {
            node_id.children(&self.app.tree).collect()
        }
    }
}

// Connection renderer
struct ConnectionRenderer<'a> {
    canvas: &'a mut BufferCanvas,
    app: &'a AppState,
    layout: &'a LayoutEngine,
    area: Rect,
}

impl<'a> ConnectionRenderer<'a> {
    fn new(
        canvas: &'a mut BufferCanvas,
        app: &'a AppState,
        layout: &'a LayoutEngine,
        area: Rect,
    ) -> Self {
        Self {
            canvas,
            app,
            layout,
            area,
        }
    }

    fn draw_node_connections(&mut self, node_id: NodeId) {
        let Some(node_ref) = self.app.tree.get(node_id) else {
            return;
        };
        let node = node_ref.get();

        let Some(node_layout) = self.layout.nodes.get(&node_id) else {
            return;
        };

        // Get children information
        let all_children: Vec<NodeId> = node_id.children(&self.app.tree).collect();
        let visible_children = self.get_visible_children(node_id);
        let has_hidden = all_children.len() != visible_children.len();

        // Calculate node middle Y position
        let node_middle_y = self.calculate_middle_y(node_layout);

        // Handle different cases
        if node.is_collapsed && !all_children.is_empty() {
            self.draw_collapsed_indicator(node_layout, has_hidden);
        } else if visible_children.is_empty() && !all_children.is_empty() {
            self.draw_hidden_only_indicator(node_layout, node_middle_y);
        } else if visible_children.len() == 1 {
            self.draw_single_child_connection(
                node_layout,
                node_middle_y,
                visible_children[0],
                has_hidden,
            );
        } else if visible_children.len() > 1 {
            self.draw_multi_child_connections(
                node_layout,
                node_middle_y,
                &visible_children,
                has_hidden,
            );
        }

        // Recursively draw connections for visible children
        for child_id in visible_children {
            self.draw_node_connections(child_id);
        }
    }

    fn get_visible_children(&self, node_id: NodeId) -> Vec<NodeId> {
        if !self.app.config.show_hidden {
            node_id
                .children(&self.app.tree)
                .filter(|cid| {
                    self.app
                        .tree
                        .get(*cid)
                        .map(|n| !n.get().is_hidden())
                        .unwrap_or(false)
                })
                .collect()
        } else {
            node_id.children(&self.app.tree).collect()
        }
    }

    fn calculate_middle_y(&self, node_layout: &crate::layout::LayoutNode) -> i32 {
        (node_layout.y + node_layout.yo + node_layout.lh / 2.0 - NODE_MIDDLE_Y_OFFSET).round()
            as i32
    }

    fn viewport_x(&self, x: f64) -> i32 {
        (x - self.app.viewport_left) as i32
    }

    fn viewport_y(&self, y: f64) -> i32 {
        (y - self.app.viewport_top) as i32
    }

    fn draw_collapsed_indicator(
        &mut self,
        node_layout: &crate::layout::LayoutNode,
        has_hidden: bool,
    ) {
        let x = self.viewport_x(node_layout.x + node_layout.w + 1.0);
        let y = self.viewport_y(node_layout.y + node_layout.yo);

        if self.is_in_bounds(x, y) {
            let text = if has_hidden {
                connections::COLLAPSED_HIDDEN
            } else {
                connections::COLLAPSED
            };
            self.canvas.draw_text(x as usize, y as usize, text);
        }
    }

    fn draw_hidden_only_indicator(
        &mut self,
        node_layout: &crate::layout::LayoutNode,
        middle_y: i32,
    ) {
        let x = self.viewport_x(node_layout.x + node_layout.w + 1.0);
        let y = self.viewport_y(middle_y as f64);

        if self.is_in_bounds(x, y) {
            self.canvas
                .draw_text(x as usize, y as usize, connections::HIDDEN_ONLY);
        }
    }

    fn draw_single_child_connection(
        &mut self,
        node_layout: &crate::layout::LayoutNode,
        parent_middle_y: i32,
        child_id: NodeId,
        has_hidden: bool,
    ) {
        let Some(child_layout) = self.layout.nodes.get(&child_id) else {
            return;
        };

        let child_middle_y = self.calculate_middle_y(child_layout);
        let x = self.viewport_x(node_layout.x + node_layout.w + 1.0);

        // Draw horizontal line
        let line = if has_hidden {
            connections::SINGLE_HIDDEN
        } else {
            connections::SINGLE
        };

        let y = parent_middle_y.min(child_middle_y);
        if self.is_in_bounds(x, self.viewport_y(y as f64)) {
            self.canvas
                .draw_text(x as usize, self.viewport_y(y as f64) as usize, line);
        }

        // Draw vertical connection if needed
        if (parent_middle_y - child_middle_y).abs() > 0 {
            self.draw_vertical_connection(child_layout, parent_middle_y, child_middle_y);
        }
    }

    fn draw_multi_child_connections(
        &mut self,
        node_layout: &crate::layout::LayoutNode,
        middle_y: i32,
        children: &[NodeId],
        has_hidden: bool,
    ) {
        // Find top and bottom children
        let (top_child, top_y, bottom_child, bottom_y) = self.find_extremes(children);

        let Some(top_child_layout) = self.layout.nodes.get(&top_child) else {
            return;
        };

        let x = self.viewport_x(node_layout.x + node_layout.w + 1.0);

        // Draw horizontal line from parent
        let line = if has_hidden {
            connections::MULTI_HIDDEN
        } else {
            connections::MULTI
        };

        let py = self.viewport_y(middle_y as f64);
        if self.is_in_bounds(x, py) {
            self.canvas.draw_text(x as usize, py as usize, line);
        }

        // Draw vertical spine
        let vert_x = self.viewport_x(top_child_layout.x - VERTICAL_CONNECTOR_OFFSET);
        self.draw_vertical_spine(vert_x, top_y, bottom_y);

        // Draw child connectors
        self.draw_child_connectors(vert_x, children, top_child, bottom_child);

        // Fix junction at parent level
        self.fix_junction(vert_x, self.viewport_y(middle_y as f64));
    }

    fn draw_vertical_connection(
        &mut self,
        child_layout: &crate::layout::LayoutNode,
        y1: i32,
        y2: i32,
    ) {
        let vert_x = self.viewport_x(child_layout.x - VERTICAL_CONNECTOR_OFFSET);

        // Draw vertical line
        for y in y1.min(y2)..y1.max(y2) {
            let py = self.viewport_y(y as f64);
            if self.is_in_bounds(vert_x, py) {
                self.canvas
                    .set_char(vert_x as usize, py as usize, junction::VERTICAL);
            }
        }

        // Draw corners
        let py2 = self.viewport_y(y2 as f64);
        if self.is_in_bounds(vert_x, py2) {
            let corner = if y2 > y1 {
                junction::BOTTOM_CORNER
            } else {
                junction::TOP_CORNER
            };
            self.canvas.set_char(vert_x as usize, py2 as usize, corner);
        }

        let py_min = self.viewport_y(y1.min(y2) as f64);
        if self.is_in_bounds(vert_x, py_min) {
            let corner = if y2 > y1 {
                junction::TOP_RIGHT
            } else {
                junction::BOTTOM_RIGHT
            };
            self.canvas
                .set_char(vert_x as usize, py_min as usize, corner);
        }
    }

    fn draw_vertical_spine(&mut self, x: i32, top_y: i32, bottom_y: i32) {
        for y in top_y..bottom_y {
            let py = self.viewport_y(y as f64);
            if self.is_in_bounds(x, py) {
                self.canvas
                    .set_char(x as usize, py as usize, junction::VERTICAL);
            }
        }
    }

    fn draw_child_connectors(
        &mut self,
        vert_x: i32,
        children: &[NodeId],
        top_child: NodeId,
        bottom_child: NodeId,
    ) {
        // Draw top corner
        if let Some(top_layout) = self.layout.nodes.get(&top_child) {
            let top_py = self.viewport_y(top_layout.y + top_layout.yo);
            if self.is_in_bounds(vert_x, top_py) {
                self.canvas
                    .draw_text(vert_x as usize, top_py as usize, "╭──");
            }
        }

        // Draw bottom corner
        if let Some(bottom_layout) = self.layout.nodes.get(&bottom_child) {
            let bot_py = self.viewport_y(bottom_layout.y + bottom_layout.yo);
            if self.is_in_bounds(vert_x, bot_py) {
                self.canvas
                    .draw_text(vert_x as usize, bot_py as usize, "╰──");
            }
        }

        // Draw middle connectors
        for &child_id in children {
            if child_id != top_child && child_id != bottom_child {
                if let Some(child_layout) = self.layout.nodes.get(&child_id) {
                    let cy = (child_layout.y + child_layout.yo + child_layout.lh / 2.0
                        - MIDDLE_CONNECTOR_Y_OFFSET) as i32;
                    let py = self.viewport_y(cy as f64);
                    if self.is_in_bounds(vert_x, py) {
                        self.canvas.draw_text(vert_x as usize, py as usize, "├──");
                    }
                }
            }
        }
    }

    fn fix_junction(&mut self, x: i32, y: i32) {
        if !self.is_in_bounds(x, y) {
            return;
        }

        let existing = self.canvas.char_buffer[y as usize][x as usize];
        let replacement = match existing {
            '│' => junction::MIDDLE_RIGHT,
            '╭' => junction::TOP_TEE,
            '├' => junction::CROSS,
            _ => existing,
        };
        self.canvas.set_char(x as usize, y as usize, replacement);
    }

    fn find_extremes(&self, children: &[NodeId]) -> (NodeId, i32, NodeId, i32) {
        let mut top_y = i32::MAX;
        let mut bottom_y = i32::MIN;
        let mut top_child = children[0];
        let mut bottom_child = children[0];

        for &child_id in children {
            if let Some(child_layout) = self.layout.nodes.get(&child_id) {
                let child_y = (child_layout.y + child_layout.yo) as i32;
                if child_y < top_y {
                    top_y = child_y;
                    top_child = child_id;
                }
                if child_y > bottom_y {
                    bottom_y = child_y;
                    bottom_child = child_id;
                }
            }
        }

        (top_child, top_y, bottom_child, bottom_y)
    }

    fn is_in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.area.width as i32 && y < self.area.height as i32
    }
}

// Status line renderer
struct StatusLineRenderer;

impl StatusLineRenderer {
    fn render(frame: &mut Frame, app: &AppState, area: Rect) {
        let (content, style) = Self::get_content_and_style(app, area);

        let paragraph = Paragraph::new(content)
            .style(style)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    fn get_content_and_style(app: &AppState, area: Rect) -> (String, Style) {
        match &app.mode {
            AppMode::Normal => Self::render_normal_mode(app),
            AppMode::Editing { buffer, cursor_pos } => {
                Self::render_edit_mode(buffer, *cursor_pos, area.width)
            }
            AppMode::Search { query } => Self::render_search_mode(query),
            AppMode::Help => Self::render_help_mode(),
        }
    }

    fn render_normal_mode(app: &AppState) -> (String, Style) {
        let content = if let Some(ref msg) = app.message {
            msg.clone()
        } else {
            format!("h-m-m | {} nodes", app.tree.count())
        };

        let style = if app.message.is_some() {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Magenta)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray).bg(Color::Black)
        };

        (content, style)
    }

    fn render_edit_mode(buffer: &str, cursor_pos: usize, width: u16) -> (String, Style) {
        let mut display = String::from(STATUS_EDIT_PREFIX);

        // Calculate visible portion if text is too long
        let available_width = width.saturating_sub(STATUS_EDIT_PREFIX.len() as u16 + 1) as usize;
        let text_start = if cursor_pos > available_width.saturating_sub(10) {
            cursor_pos.saturating_sub(available_width / 2)
        } else {
            0
        };

        let visible_buffer = if buffer.len() > available_width {
            let end = (text_start + available_width).min(buffer.len());
            &buffer[text_start..end]
        } else {
            buffer
        };

        // Adjust cursor position for visible portion
        let visible_cursor = cursor_pos.saturating_sub(text_start);

        // Insert cursor indicator
        if visible_cursor <= visible_buffer.len() {
            display.push_str(&visible_buffer[..visible_cursor]);
            display.push(CURSOR_INDICATOR);
            display.push_str(&visible_buffer[visible_cursor..]);
        } else {
            display.push_str(visible_buffer);
            display.push(CURSOR_INDICATOR);
        }

        let style = Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        (display, style)
    }

    fn render_search_mode(query: &str) -> (String, Style) {
        let content = format!("{}{}", STATUS_SEARCH_PREFIX, query);
        let style = Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        (content, style)
    }

    fn render_help_mode() -> (String, Style) {
        let content = String::from("Press ESC or q to close help");
        let style = Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        (content, style)
    }
}

// Help renderer
struct HelpRenderer;

impl HelpRenderer {
    fn render(frame: &mut Frame, area: Rect) {
        let help_text = Self::build_help_text();
        let block = Block::default().borders(Borders::ALL).title(" Help ");
        let paragraph = Paragraph::new(help_text)
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    fn build_help_text() -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(vec![Span::styled(
                "h-m-m Help",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
        ];

        for section in help::SECTIONS {
            lines.push(Line::from(vec![Span::styled(
                section.title,
                Style::default().add_modifier(Modifier::BOLD),
            )]));

            for (key, desc) in section.items {
                lines.push(Line::from(format!("  {}  {}", key, desc)));
            }

            lines.push(Line::from(""));
        }

        lines.push(Line::from("Press ESC or q to close help"));
        lines
    }
}

// Text wrapper utility
struct TextWrapper;

impl TextWrapper {
    fn wrap(text: &str, max_width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0;

        for word in text.split_whitespace() {
            let word_width = unicode_width::UnicodeWidthStr::width(word);

            if current_width > 0 && current_width + 1 + word_width > max_width {
                lines.push(current_line);
                current_line = word.to_string();
                current_width = word_width;
            } else {
                if !current_line.is_empty() {
                    current_line.push(' ');
                    current_width += 1;
                }
                current_line.push_str(word);
                current_width += word_width;
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            lines.push(text.to_string());
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_line_constants() {
        // Verify single child connection is 5 dashes
        assert_eq!(
            connections::SINGLE.chars().filter(|&c| c == '─').count(),
            5,
            "Single child connection should have exactly 5 dashes"
        );

        // Verify multi-child connection is 4 dashes
        assert_eq!(
            connections::MULTI.chars().filter(|&c| c == '─').count(),
            4,
            "Multi-child connection should have exactly 4 dashes"
        );

        // Verify hidden variants have the correct dash count
        assert_eq!(
            connections::SINGLE_HIDDEN
                .chars()
                .filter(|&c| c == '─')
                .count(),
            4,
            "Single child connection with hidden should have 4 dashes plus ╫"
        );

        assert_eq!(
            connections::MULTI_HIDDEN
                .chars()
                .filter(|&c| c == '─')
                .count(),
            3,
            "Multi-child connection with hidden should have 3 dashes plus ╫"
        );
    }

    #[test]
    fn test_no_spaces_in_connection_lines() {
        assert!(
            !connections::SINGLE.contains(' '),
            "Single child connection should not contain spaces"
        );
        assert!(
            !connections::MULTI.contains(' '),
            "Multi-child connection should not contain spaces"
        );
        assert!(
            !connections::SINGLE_HIDDEN.contains(' '),
            "Single child connection with hidden should not contain spaces"
        );
        assert!(
            !connections::MULTI_HIDDEN.contains(' '),
            "Multi-child connection with hidden should not contain spaces"
        );
    }

    #[test]
    fn test_buffer_canvas() {
        let mut canvas = BufferCanvas::new(20, 5);

        // Test set_char
        canvas.set_char(5, 2, 'X');
        assert_eq!(canvas.char_buffer[2][5], 'X');

        // Test draw_text
        canvas.draw_text(0, 0, "Hello");
        assert_eq!(&canvas.char_buffer[0][0..5], ['H', 'e', 'l', 'l', 'o']);

        // Test bounds checking
        canvas.set_char(25, 2, 'Y'); // Out of bounds - should not panic
        canvas.set_char(5, 10, 'Z'); // Out of bounds - should not panic

        // Test in_bounds
        assert!(canvas.in_bounds(5, 2));
        assert!(!canvas.in_bounds(20, 2));
        assert!(!canvas.in_bounds(5, 5));
    }

    #[test]
    fn test_text_wrapper() {
        let text = "The quick brown fox jumps over the lazy dog";
        let wrapped = TextWrapper::wrap(text, 10);

        assert!(
            wrapped.len() > 1,
            "Text should be wrapped into multiple lines"
        );
        for line in &wrapped {
            assert!(
                unicode_width::UnicodeWidthStr::width(line.as_str()) <= 10,
                "Line width should not exceed max width"
            );
        }

        // Test empty text
        let empty_wrapped = TextWrapper::wrap("", 10);
        assert_eq!(empty_wrapped.len(), 1);
        assert_eq!(empty_wrapped[0], "");

        // Test single word longer than max width
        let long_word = "verylongword";
        let single_wrapped = TextWrapper::wrap(long_word, 5);
        assert_eq!(single_wrapped.len(), 1);
        assert_eq!(single_wrapped[0], long_word);
    }

    #[test]
    fn test_connection_total_length() {
        use crate::layout::NODE_CONNECTION_SPACING;

        // Total spacing is 6 units
        // With 1 space before connection, we need 5 dashes
        let expected_connection_chars = NODE_CONNECTION_SPACING as usize - 1;

        // Count characters, not bytes
        let actual_chars = connections::SINGLE.chars().count();

        assert_eq!(
            actual_chars, expected_connection_chars,
            "Single child connection should have {} characters to fill spacing",
            expected_connection_chars
        );
    }
}
