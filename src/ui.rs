use crate::app::{AppMode, AppState};
use crate::layout::LayoutEngine;
use crate::model::NodeId;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

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
        AppMode::Help => render_help(frame, chunks[0]),
        _ => render_mind_map(frame, app, &layout, chunks[0]),
    }

    // Render status/input line
    render_status_line(frame, app, chunks[1]);
}

fn render_mind_map(frame: &mut Frame, app: &AppState, layout: &LayoutEngine, area: Rect) {
    // Create a buffer for the mind map
    let mut map_buffer: Vec<Vec<char>> = vec![vec![' '; area.width as usize]; area.height as usize];
    let mut style_buffer: Vec<Vec<Style>> =
        vec![vec![Style::default(); area.width as usize]; area.height as usize];

    // Draw connections
    draw_connections(&mut map_buffer, app, layout, area);

    // Draw nodes
    draw_nodes(&mut map_buffer, &mut style_buffer, app, layout, area);

    // Convert buffer to text
    let mut lines = Vec::new();
    for (y, row) in map_buffer.iter().enumerate() {
        let mut spans = Vec::new();
        let mut current_style = Style::default();
        let mut current_text = String::new();

        for (x, &ch) in row.iter().enumerate() {
            let style = style_buffer[y][x];
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

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn draw_connections(
    buffer: &mut Vec<Vec<char>>,
    app: &AppState,
    layout: &LayoutEngine,
    area: Rect,
) {
    if let Some(root_id) = app.root_id {
        draw_node_connections(buffer, app, layout, root_id, area);
    }
}

fn draw_node_connections(
    buffer: &mut Vec<Vec<char>>,
    app: &AppState,
    layout: &LayoutEngine,
    node_id: NodeId,
    area: Rect,
) {
    let node = app.tree.get(node_id).unwrap().get();

    if let Some(node_layout) = layout.nodes.get(&node_id) {
        let children: Vec<NodeId> = if !app.config.show_hidden {
            node_id
                .children(&app.tree)
                .filter(|cid| {
                    let child = app.tree.get(*cid).unwrap().get();
                    !child.is_hidden()
                })
                .collect()
        } else {
            node_id.children(&app.tree).collect()
        };

        if node.is_collapsed && !children.is_empty() {
            // Draw collapsed indicator
            let x = (node_layout.x + node_layout.w + 1.0 - app.viewport_left) as usize;
            let y = (node_layout.y + node_layout.yo - app.viewport_top) as usize;

            if x < area.width as usize && y < area.height as usize {
                draw_text(buffer, x, y, " [+]");
            }
        } else if !children.is_empty() {
            // Draw connections to children
            let parent_x = (node_layout.x + node_layout.w - app.viewport_left) as usize;
            let parent_y = (node_layout.y + node_layout.yo + node_layout.lh / 2.0
                - app.viewport_top) as usize;

            // Draw horizontal line from parent node text
            if parent_x < area.width as usize && parent_y < area.height as usize {
                for x in parent_x..(parent_x + 4).min(area.width as usize) {
                    set_char(buffer, x, parent_y, '─');
                }
            }

            // Calculate vertical connector position (right after parent's horizontal line)
            let conn_x = parent_x + 4;

                // Get first and last child positions
                if let (Some(first_layout), Some(last_layout)) =
                    (layout.nodes.get(&children[0]), layout.nodes.get(&children[children.len() - 1])) {

                    let first_y = (first_layout.y + first_layout.yo + first_layout.lh / 2.0 - app.viewport_top) as usize;
                    let last_y = (last_layout.y + last_layout.yo + last_layout.lh / 2.0 - app.viewport_top) as usize;

                    // Draw continuous vertical line from parent level to last child
                    if conn_x < area.width as usize {
                        // Connect from parent y to first child
                        if parent_y < first_y {
                            for y in parent_y..first_y {
                                if y < area.height as usize {
                                    set_char(buffer, conn_x, y, '│');
                                }
                            }
                        }
                        // Draw through all children
                        for y in first_y..=last_y.min(area.height as usize - 1) {
                            if y < area.height as usize {
                                set_char(buffer, conn_x, y, '│');
                            }
                        }
                    }
                }

            // Now draw horizontal connectors for each child
            for (i, child_id) in children.iter().enumerate() {
                if let Some(child_layout) = layout.nodes.get(child_id) {
                    let child_x = (child_layout.x - app.viewport_left) as usize;
                    let child_y = (child_layout.y + child_layout.yo + child_layout.lh / 2.0
                        - app.viewport_top) as usize;

                    if child_y < area.height as usize && conn_x < area.width as usize {
                        // Draw horizontal line from vertical connector to just before child text
                        // Leave 2 spaces before the child text for the "──" prefix
                        if child_x > 2 {
                            for x in conn_x..(child_x - 2).min(area.width as usize) {
                                set_char(buffer, x, child_y, '─');
                            }
                            // Add the "──" prefix right before the child text
                            if child_x >= 2 {
                                set_char(buffer, child_x - 2, child_y, '─');
                                set_char(buffer, child_x - 1, child_y, '─');
                            }
                        }

                        // Place junction character at vertical line
                        if children.len() == 1 {
                            // Single child - horizontal line continues
                            set_char(buffer, conn_x, child_y, '─');
                        } else if i == 0 {
                            // First child of multiple
                            set_char(buffer, conn_x, child_y, '├');
                        } else if i == children.len() - 1 {
                            // Last child
                            set_char(buffer, conn_x, child_y, '╰');
                        } else {
                            // Middle children
                            set_char(buffer, conn_x, child_y, '├');
                        }
                    }

                    // Recursively draw connections for children
                    if !node.is_collapsed {
                        draw_node_connections(buffer, app, layout, *child_id, area);
                    }
                }
            }
        }
    }
}

fn draw_nodes(
    buffer: &mut Vec<Vec<char>>,
    style_buffer: &mut Vec<Vec<Style>>,
    app: &AppState,
    layout: &LayoutEngine,
    area: Rect,
) {
    if let Some(root_id) = app.root_id {
        draw_node_content(buffer, style_buffer, app, layout, root_id, area);
    }
}

fn draw_node_content(
    buffer: &mut Vec<Vec<char>>,
    style_buffer: &mut Vec<Vec<Style>>,
    app: &AppState,
    layout: &LayoutEngine,
    node_id: NodeId,
    area: Rect,
) {
    let node = app.tree.get(node_id).unwrap().get();

    if let Some(node_layout) = layout.nodes.get(&node_id) {
        let x = (node_layout.x - app.viewport_left) as usize;
        let y = (node_layout.y + node_layout.yo - app.viewport_top) as usize;

        // Determine node style
        let style = if Some(node_id) == app.active_node_id {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if node.title.starts_with(&app.config.symbol1) {
            Style::default().fg(Color::Green)
        } else if node.title.starts_with(&app.config.symbol2) {
            Style::default().fg(Color::Red)
        } else if node.is_hidden() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        // Draw node text
        if x < area.width as usize && y < area.height as usize {
            let lines = wrap_text(&node.title, node_layout.w as usize);
            for (i, line) in lines.iter().enumerate() {
                let line_y = y + i;
                if line_y < area.height as usize {
                    draw_styled_text(buffer, style_buffer, x, line_y, line, style);
                }
            }
        }

        // Draw children if not collapsed
        if !node.is_collapsed {
            let children: Vec<NodeId> = if !app.config.show_hidden {
                node_id
                    .children(&app.tree)
                    .filter(|cid| {
                        let child = app.tree.get(*cid).unwrap().get();
                        !child.is_hidden()
                    })
                    .collect()
            } else {
                node_id.children(&app.tree).collect()
            };

            for child_id in children {
                draw_node_content(buffer, style_buffer, app, layout, child_id, area);
            }
        }
    }
}

fn render_status_line(frame: &mut Frame, app: &AppState, area: Rect) {
    let content = match &app.mode {
        AppMode::Normal => {
            if let Some(ref msg) = app.message {
                msg.clone()
            } else {
                format!("h-m-m | {} nodes", app.tree.count())
            }
        }
        AppMode::Editing {
            buffer,
            cursor_pos: _,
        } => {
            format!("Edit: {}", buffer)
        }
        AppMode::Search { query } => {
            format!("Search: {}", query)
        }
        AppMode::Help => "Press ESC or q to close help".to_string(),
    };

    let style = match &app.mode {
        AppMode::Editing { .. } | AppMode::Search { .. } => Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        _ => {
            if app.message.is_some() {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Magenta)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray).bg(Color::Black)
            }
        }
    };

    let paragraph = Paragraph::new(content)
        .style(style)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(vec![Span::styled(
            "h-m-m Help",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  h/←  Move left (parent)"),
        Line::from("  j/↓  Move down"),
        Line::from("  k/↑  Move up"),
        Line::from("  l/→  Move right (child)"),
        Line::from("  g    Go to top"),
        Line::from("  G    Go to bottom"),
        Line::from("  m/~  Go to root"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Editing:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  e/i  Edit node (append)"),
        Line::from("  E/I  Edit node (replace)"),
        Line::from("  o/⏎  Insert sibling"),
        Line::from("  O/⇥  Insert child"),
        Line::from("  d    Delete node"),
        Line::from("  D    Delete children"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "View:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ␣    Toggle collapse"),
        Line::from("  v    Collapse all"),
        Line::from("  b    Expand all"),
        Line::from("  1-5  Collapse to level"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "File:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  s    Save"),
        Line::from("  S    Save as"),
        Line::from("  q    Quit"),
        Line::from(""),
        Line::from("Press ESC or q to close help"),
    ];

    let block = Block::default().borders(Borders::ALL).title(" Help ");

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

// Helper functions
fn set_char(buffer: &mut Vec<Vec<char>>, x: usize, y: usize, ch: char) {
    if y < buffer.len() && x < buffer[y].len() {
        buffer[y][x] = ch;
    }
}

fn draw_text(buffer: &mut Vec<Vec<char>>, x: usize, y: usize, text: &str) {
    for (i, ch) in text.chars().enumerate() {
        set_char(buffer, x + i, y, ch);
    }
}

fn draw_styled_text(
    buffer: &mut Vec<Vec<char>>,
    style_buffer: &mut Vec<Vec<Style>>,
    x: usize,
    y: usize,
    text: &str,
    style: Style,
) {
    for (i, ch) in text.chars().enumerate() {
        if y < buffer.len() && x + i < buffer[y].len() {
            buffer[y][x + i] = ch;
            style_buffer[y][x + i] = style;
        }
    }
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
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
