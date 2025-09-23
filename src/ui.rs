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

fn draw_connections(buffer: &mut [Vec<char>], app: &AppState, layout: &LayoutEngine, area: Rect) {
    if let Some(root_id) = app.root_id {
        draw_node_connections(buffer, app, layout, root_id, area);
    }
}

fn draw_node_connections(
    buffer: &mut [Vec<char>],
    app: &AppState,
    layout: &LayoutEngine,
    node_id: NodeId,
    area: Rect,
) {
    let node = app.tree.get(node_id).unwrap().get();

    if let Some(node_layout) = layout.nodes.get(&node_id) {
        // Get visible children
        let all_children: Vec<NodeId> = node_id.children(&app.tree).collect();
        let visible_children: Vec<NodeId> = if !app.config.show_hidden {
            all_children
                .iter()
                .filter(|cid| {
                    let child = app.tree.get(**cid).unwrap().get();
                    !child.is_hidden()
                })
                .cloned()
                .collect()
        } else {
            all_children.clone()
        };

        let num_children = all_children.len();
        let num_visible_children = visible_children.len();
        let has_hidden_children = num_visible_children != num_children;

        // Constants from PHP version
        const CONN_LEFT_LEN: usize = 6;
        const CONN_RIGHT_LEN: usize = 4;

        // Calculate node middle Y position
        let node_middle_y =
            (node_layout.y + node_layout.yo + node_layout.lh / 2.0 - 0.6).round() as i32;

        // Case 1: Node is collapsed with children
        if node.is_collapsed && num_children > 0 {
            let x = (node_layout.x + node_layout.w + 1.0 - app.viewport_left) as i32;
            let y = (node_layout.y + node_layout.yo - app.viewport_top) as i32;

            if x >= 0 && y >= 0 && x < area.width as i32 && y < area.height as i32 {
                if has_hidden_children {
                    draw_text(buffer, x as usize, y as usize, "─╫─ [+]");
                } else {
                    draw_text(buffer, x as usize, y as usize, " [+]");
                }
            }
            return;
        }

        // Case 2: No visible children but has hidden children
        if num_visible_children == 0 {
            if num_children > 0 {
                let x = (node_layout.x + node_layout.w + 1.0 - app.viewport_left) as i32;
                let y = (node_middle_y as f64 - app.viewport_top) as i32;

                if x >= 0 && y >= 0 && x < area.width as i32 && y < area.height as i32 {
                    draw_text(buffer, x as usize, y as usize, "─╫─");
                }
            }
            return;
        }

        // Case 3: Single visible child
        if num_visible_children == 1 {
            let child_id = visible_children[0];
            if let Some(child_layout) = layout.nodes.get(&child_id) {
                let y1 = node_middle_y;
                let y2 =
                    (child_layout.y + child_layout.yo + child_layout.lh / 2.0 - 0.6).round() as i32;

                // Calculate x position based on alignment setting
                let x = if app.config.align_levels {
                    (node_layout.x + node_layout.w - 2.0 - app.viewport_left) as i32
                } else {
                    (child_layout.x
                        - CONN_LEFT_LEN as f64
                        - CONN_RIGHT_LEN as f64
                        - app.viewport_left) as i32
                };

                // Build the horizontal line
                let line_prefix = if has_hidden_children {
                    "─╫"
                } else {
                    "──"
                };
                let line_len = if app.config.align_levels {
                    (child_layout.x - node_layout.x - node_layout.w - 1.0).max(0.0) as usize
                } else {
                    CONN_LEFT_LEN + CONN_RIGHT_LEN - 3
                };

                // Draw horizontal line
                if x >= 0
                    && y1.min(y2) >= 0
                    && x < area.width as i32
                    && y1.min(y2) < area.height as i32
                {
                    draw_text(
                        buffer,
                        x as usize,
                        y1.min(y2) as usize - app.viewport_top as usize,
                        line_prefix,
                    );
                    for i in 0..line_len {
                        let px = x + line_prefix.len() as i32 + i as i32;
                        if px >= 0 && px < area.width as i32 {
                            set_char(
                                buffer,
                                px as usize,
                                y1.min(y2) as usize - app.viewport_top as usize,
                                '─',
                            );
                        }
                    }
                }

                // If child is at different Y level, draw vertical connection
                if (y1 - y2).abs() > 0 {
                    let vert_x = (child_layout.x - 2.0 - app.viewport_left) as i32;

                    // Draw vertical line
                    for y in y1.min(y2)..y1.max(y2) {
                        let py = y - app.viewport_top as i32;
                        if vert_x >= 0
                            && py >= 0
                            && vert_x < area.width as i32
                            && py < area.height as i32
                        {
                            set_char(buffer, vert_x as usize, py as usize, '│');
                        }
                    }

                    // Draw corner at child position
                    let py2 = y2 - app.viewport_top as i32;
                    if vert_x >= 0
                        && py2 >= 0
                        && vert_x < area.width as i32
                        && py2 < area.height as i32
                    {
                        set_char(
                            buffer,
                            vert_x as usize,
                            py2 as usize,
                            if y2 > y1 { '╰' } else { '╭' },
                        );
                    }

                    // Draw corner at parent level
                    let py_min = y1.min(y2) - app.viewport_top as i32;
                    if vert_x >= 0
                        && py_min >= 0
                        && vert_x < area.width as i32
                        && py_min < area.height as i32
                    {
                        set_char(
                            buffer,
                            vert_x as usize,
                            py_min as usize,
                            if y2 > y1 { '╮' } else { '╯' },
                        );
                    }
                }
            }

        // Case 4: Multiple visible children
        } else if num_visible_children > 1 {
            // Find top and bottom children
            let mut top_y = i32::MAX;
            let mut bottom_y = i32::MIN;
            let mut top_child = visible_children[0];
            let mut bottom_child = visible_children[0];

            for &child_id in &visible_children {
                if let Some(child_layout) = layout.nodes.get(&child_id) {
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

            if let Some(top_child_layout) = layout.nodes.get(&top_child) {
                let middle = node_middle_y;

                // Calculate x position based on alignment
                let x = if app.config.align_levels {
                    (node_layout.x + node_layout.w - 2.0 - app.viewport_left) as i32
                } else {
                    (top_child_layout.x
                        - CONN_LEFT_LEN as f64
                        - CONN_RIGHT_LEN as f64
                        - app.viewport_left) as i32
                };

                // Draw horizontal line from parent
                let line_prefix = if has_hidden_children {
                    "─╫"
                } else {
                    "──"
                };
                let line_len = if app.config.align_levels {
                    (top_child_layout.x - node_layout.x - node_layout.w - 3.0).max(0.0) as usize
                } else {
                    CONN_LEFT_LEN - 2
                };

                let py = middle - app.viewport_top as i32;
                if x >= 0 && py >= 0 && x < area.width as i32 && py < area.height as i32 {
                    draw_text(buffer, x as usize, py as usize, line_prefix);
                    for i in 0..line_len {
                        let px = x + line_prefix.len() as i32 + i as i32;
                        if px >= 0 && px < area.width as i32 {
                            set_char(buffer, px as usize, py as usize, '─');
                        }
                    }
                }

                // Vertical line position
                let vert_x =
                    (top_child_layout.x - CONN_RIGHT_LEN as f64 - app.viewport_left) as i32;

                // Draw vertical line spanning all children
                for y in top_y..bottom_y {
                    let py = y - app.viewport_top as i32;
                    if vert_x >= 0
                        && py >= 0
                        && vert_x < area.width as i32
                        && py < area.height as i32
                    {
                        set_char(buffer, vert_x as usize, py as usize, '│');
                    }
                }

                // Draw top corner
                let top_py = top_y - app.viewport_top as i32;
                if vert_x >= 0
                    && top_py >= 0
                    && vert_x < area.width as i32
                    && top_py < area.height as i32
                {
                    draw_text(buffer, vert_x as usize, top_py as usize, "╭──");
                }

                // Draw bottom corner
                let bot_py = bottom_y - app.viewport_top as i32;
                if vert_x >= 0
                    && bot_py >= 0
                    && vert_x < area.width as i32
                    && bot_py < area.height as i32
                {
                    draw_text(buffer, vert_x as usize, bot_py as usize, "╰──");
                }

                // Draw middle children connectors
                for &child_id in &visible_children {
                    if child_id != top_child && child_id != bottom_child {
                        if let Some(child_layout) = layout.nodes.get(&child_id) {
                            let cy = (child_layout.y + child_layout.yo + child_layout.lh / 2.0
                                - 0.2) as i32;
                            let py = cy - app.viewport_top as i32;
                            if vert_x >= 0
                                && py >= 0
                                && vert_x < area.width as i32
                                && py < area.height as i32
                            {
                                draw_text(buffer, vert_x as usize, py as usize, "├──");
                            }
                        }
                    }
                }

                // Fix junction points
                let middle_py = middle - app.viewport_top as i32;
                if vert_x >= 0
                    && middle_py >= 0
                    && vert_x < area.width as i32
                    && middle_py < area.height as i32
                {
                    let existing = buffer[middle_py as usize][vert_x as usize];
                    let replacement = match existing {
                        '│' => '┤',
                        '╭' => '┬',
                        '├' => '┼',
                        _ => existing,
                    };
                    set_char(buffer, vert_x as usize, middle_py as usize, replacement);
                }
            }
        }

        // Recursively draw connections for all visible children
        for child_id in visible_children {
            draw_node_connections(buffer, app, layout, child_id, area);
        }
    }
}

fn draw_nodes(
    buffer: &mut [Vec<char>],
    style_buffer: &mut [Vec<Style>],
    app: &AppState,
    layout: &LayoutEngine,
    area: Rect,
) {
    if let Some(root_id) = app.root_id {
        draw_node_content(buffer, style_buffer, app, layout, root_id, area);
    }
}

fn draw_node_content(
    buffer: &mut [Vec<char>],
    style_buffer: &mut [Vec<Style>],
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
fn set_char(buffer: &mut [Vec<char>], x: usize, y: usize, ch: char) {
    if y < buffer.len() && x < buffer[y].len() {
        buffer[y][x] = ch;
    }
}

fn draw_text(buffer: &mut [Vec<char>], x: usize, y: usize, text: &str) {
    for (i, ch) in text.chars().enumerate() {
        set_char(buffer, x + i, y, ch);
    }
}

fn draw_styled_text(
    buffer: &mut [Vec<char>],
    style_buffer: &mut [Vec<Style>],
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
