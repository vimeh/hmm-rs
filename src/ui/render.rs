use crate::config::Config;
use crate::core::{MindMap, NodeId};
use ratatui::prelude::{Frame, Rect, Size};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::collections::HashMap;
use textwrap::wrap;
use unicode_width::UnicodeWidthStr;

// Intermediate structure during layout calculation
#[derive(Debug, Clone)]
struct LayoutCalcNode {
    id: NodeId,
    text: String,
    children: Vec<NodeId>,
    visible_children: Vec<NodeId>,
    w: u16,
    h: u16,
    subtree_h: u16,
    x: u16,
    y: u16,
    wrapped_text: String,
    is_leaf: bool,
    is_visible: bool,
}

/// Represents a node with calculated layout information for rendering.
#[derive(Debug, Clone)]
pub struct RenderNode {
    pub id: NodeId,
    pub display_title: String,
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
    pub is_active: bool,
    // Add more fields later, like children, collapsed status, etc.
}

/// Calculates the layout for all visible nodes in the mind map.
/// Returns a map from NodeId to its calculated RenderNode.
pub fn calculate_layout(
    map: &MindMap,
    config: &Config,
    _area: Size,            // Parameter currently unused
    active_node_id: NodeId, // Add active_node_id parameter
) -> HashMap<NodeId, RenderNode> {
    let mut calc_nodes: HashMap<NodeId, LayoutCalcNode> = HashMap::new();
    let root_id = match map.root {
        Some(id) => id,
        None => return HashMap::new(), // Return empty layout if no root
    };

    // --- Pass 1: Calculate node dimensions (w, h) ---
    calculate_dimensions_recursive(map, config, root_id, &mut calc_nodes);

    // --- Pass 2: Calculate subtree heights (TODO) ---
    calculate_subtree_height_recursive(map, config, root_id, &mut calc_nodes);

    // --- Pass 3: Calculate X and Y positions (TODO) ---
    // For now, use placeholder positions
    let mut current_y = 0;
    for (id, calc_node) in calc_nodes.iter_mut() {
        calc_node.x = 1; // Dummy X
        calc_node.y = current_y; // Dummy Y, stacking vertically
        current_y += calc_node.h + config.line_spacing as u16;
    }

    // --- Convert LayoutCalcNode to RenderNode ---
    let mut render_map = HashMap::new();
    for (id, calc_node) in calc_nodes {
        // if calc_node.is_visible { // TODO: Add visibility check later
        render_map.insert(
            id,
            RenderNode {
                id: calc_node.id,
                display_title: calc_node.wrapped_text.clone(),
                x: calc_node.x,
                y: calc_node.y,
                w: calc_node.w,
                h: calc_node.h,
                is_active: id == active_node_id,
            },
        );
        // }
    }

    render_map
}

// Pass 1: Calculate w, h for each node
fn calculate_dimensions_recursive(
    map: &MindMap,
    config: &Config,
    node_id: NodeId,
    calc_nodes: &mut HashMap<NodeId, LayoutCalcNode>,
    // parent_is_visible: bool, // Add later for visibility
) {
    if calc_nodes.contains_key(&node_id) {
        // Avoid redundant calculations
        return;
    }

    if let Some(node) = map.get_node(node_id) {
        // let is_visible = parent_is_visible && !node.hidden; // Add later

        let is_leaf = node.children.is_empty();
        let max_width = if is_leaf {
            config.max_leaf_node_width
        } else {
            config.max_parent_node_width
        } as usize;

        // Perform text wrapping
        let wrapped_lines: Vec<String> = wrap(&node.text, max_width)
            .iter()
            .map(|s| s.to_string())
            .collect();

        let wrapped_text = wrapped_lines.join("\n");
        // Calculate width based on longest wrapped line + padding/border
        let w = wrapped_lines
            .iter()
            .map(|line| UnicodeWidthStr::width(line.as_str()))
            .max()
            .unwrap_or(0) as u16
            + 2; // +2 for padding/border
        // Calculate height based on number of lines
        let h = wrapped_lines.len().max(1) as u16; // Ensure height is at least 1

        calc_nodes.insert(
            node_id,
            LayoutCalcNode {
                id: node_id,
                text: node.text.clone(),
                children: node.children.clone(),
                visible_children: Vec::new(), // Placeholder, will be filled later
                w,
                h,
                subtree_h: 0, // Placeholder, will be filled later
                x: 0,         // Placeholder
                y: 0,         // Placeholder
                wrapped_text,
                is_leaf,
                is_visible: true, // Placeholder, will be filled later
            },
        );

        // Recurse for all children
        for child_id in &node.children {
            calculate_dimensions_recursive(map, config, *child_id, calc_nodes);
        }
    }
}

// Pass 2: Calculate subtree height (bottom-up)
fn calculate_subtree_height_recursive(
    map: &MindMap,
    config: &Config,
    node_id: NodeId,
    calc_nodes: &mut HashMap<NodeId, LayoutCalcNode>,
) -> u16 {
    // Check if height already calculated (memoization)
    if let Some(calc_node) = calc_nodes.get(&node_id) {
        if calc_node.subtree_h > 0 {
            return calc_node.subtree_h;
        }
        // If node is not visible, its subtree height for layout purposes is 0
        if !calc_node.is_visible {
            return 0;
        }
    } else {
        return 0; // Node doesn't exist
    }

    // Calculate based on children (post-order traversal)
    let children_ids = calc_nodes.get(&node_id).unwrap().visible_children.clone(); // Use visible children
    let mut children_total_h = 0u16;

    for child_id in children_ids {
        children_total_h += calculate_subtree_height_recursive(map, config, child_id, calc_nodes);
    }

    // Update the current node's subtree_h in the map
    if let Some(calc_node) = calc_nodes.get_mut(&node_id) {
        let node_h_with_spacing = calc_node.h + config.line_spacing as u16;
        // The subtree height is the max of its own height or the sum of its children's subtree heights
        calc_node.subtree_h = node_h_with_spacing.max(children_total_h);
        calc_node.subtree_h // Return the calculated height
    } else {
        0 // Should not happen if node exists
    }
}

// Pass 3: Calculate X and Y positions
fn calculate_position_recursive(
    map: &MindMap,
    config: &Config,
    node_id: NodeId,
    calc_nodes: &mut HashMap<NodeId, LayoutCalcNode>,
) {
    let root_id = map.root.expect("Cannot calculate position without root");

    // Node's own position should be set by its parent before this call (except root)
    // Since node.parent doesn't exist, we use a placeholder for non-root nodes.
    // Finding the actual parent requires traversing the map or storing parent links.
    let (parent_x, _parent_y, parent_w) = if node_id != root_id {
        // Placeholder values - this will result in incorrect layout for children
        (0, 0, 0)
    } else {
        // Root position is set before this call
        let root_node = calc_nodes.get(&root_id).unwrap();
        (root_node.x, root_node.y, root_node.w)
    };

    // Constants for connector lengths
    const CONN_LEFT_LEN: u16 = 6;
    const CONN_RIGHT_LEN: u16 = 4;

    // Use unwrapped root_id for comparison
    let calculated_x = if root_id == node_id {
        1 // Root starts at 1
    } else {
        // Placeholder X calculation - depends on correct parent info
        parent_x + parent_w + CONN_LEFT_LEN + CONN_RIGHT_LEN - 1
    };

    // Calculate Y for children
    // ... (rest of the Y calculation logic remains the same)
    let mut children_start_y = calc_nodes.get(&node_id).map_or(0, |n| n.y);
    let children_ids = match calc_nodes.get(&node_id) {
        Some(n) => n.visible_children.clone(),
        None => return,
    };
    let total_children_subtree_h = children_ids.iter().fold(0, |acc, &child_id| {
        acc + calc_nodes.get(&child_id).map_or(0, |cn| cn.subtree_h)
    });
    if total_children_subtree_h > 0 {
        if let Some(parent_calc_node) = calc_nodes.get(&node_id) {
            let parent_center_y = parent_calc_node.y + parent_calc_node.h / 2;
            children_start_y = parent_center_y.saturating_sub(total_children_subtree_h / 2);
        }
    }
    let mut current_y = children_start_y;
    for child_id in children_ids {
        if let Some(child_calc_node) = calc_nodes.get_mut(&child_id) {
            // Child X calculation needs parent info
            child_calc_node.x = calculated_x + CONN_LEFT_LEN; // Placeholder
            child_calc_node.y = current_y;
            let subtree_h = child_calc_node.subtree_h;
            // Recurse AFTER setting position, pass root_id down if needed by children
            calculate_position_recursive(map, config, child_id, calc_nodes);
            current_y += subtree_h; // Increment y based on child's subtree height
        }
    }
}

/// Draws the mind map onto the frame.
pub fn draw_map(
    frame: &mut Frame,
    render_nodes: &HashMap<NodeId, RenderNode>,
    _config: &Config,
    area: Rect,
    viewport_y: u16,
    viewport_x: u16,
    _map: &MindMap, // Keep map in case needed later, mark unused for now
) {
    let viewport_block = Block::default().borders(Borders::ALL).title("Mind Map");
    let inner_area = viewport_block.inner(area);
    frame.render_widget(viewport_block, area);

    for (_id, render_node) in render_nodes {
        // Calculate node bounds relative to viewport
        let node_rect = Rect {
            x: render_node.x.saturating_sub(viewport_x),
            y: render_node.y.saturating_sub(viewport_y),
            width: render_node.w,
            height: render_node.h,
        };

        // Ensure the calculated coordinates are within the inner area before drawing
        if inner_area.intersects(node_rect) {
            // Clip the node area to the drawable inner area
            let clipped_node_area = inner_area.intersection(node_rect);
            if clipped_node_area.area() == 0 {
                continue; // Don't draw if completely outside
            }

            // --- Draw Node Widget directly here ---
            // Apply styling (Placeholder)
            let is_active = false; // Placeholder - need active node info from TuiState
            let node_style = if is_active {
                // config.theme.active_node_style.to_ratatui_style()
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                // config.theme.inactive_node_style.to_ratatui_style()
                Style::default().fg(Color::White).bg(Color::Blue)
            };

            let paragraph = Paragraph::new(render_node.display_title.clone())
                .style(node_style)
                .block(Block::default().borders(Borders::ALL))
                .wrap(ratatui::widgets::Wrap { trim: true });

            frame.render_widget(paragraph, clipped_node_area);

            // Connection lines removed
        }
    }
}

// Helper function to draw a character at a specific map coordinate, considering viewport
fn draw_char_at(
    frame: &mut Frame,
    area: Rect,
    viewport_x: u16,
    viewport_y: u16,
    x: u16,
    y: u16,
    ch: char,
) {
    if x >= viewport_x
        && x < viewport_x + area.width
        && y >= viewport_y
        && y < viewport_y + area.height
    {
        let draw_x = area.x + x - viewport_x;
        let draw_y = area.y + y - viewport_y;
        // Check bounds again after viewport calculation
        if draw_x < area.right() && draw_y < area.bottom() {
            frame.buffer_mut().set_string(
                draw_x,
                draw_y,
                String::from(ch),
                Style::default().fg(Color::DarkGray),
            );
        }
    }
}

// Helper function to draw a horizontal line
fn draw_horizontal_line(
    frame: &mut Frame,
    area: Rect,
    viewport_x: u16,
    viewport_y: u16,
    x1: u16,
    x2: u16,
    y: u16,
    ch: char,
) {
    if y >= viewport_y && y < viewport_y + area.height {
        let draw_y = area.y + y - viewport_y;
        let start_map_x = x1.min(x2);
        let end_map_x = x1.max(x2);
        for map_x in start_map_x..end_map_x {
            if map_x >= viewport_x && map_x < viewport_x + area.width {
                let draw_x = area.x + map_x - viewport_x;
                if draw_x < area.right() {
                    // Ensure not drawing past right edge
                    frame.buffer_mut().set_string(
                        draw_x,
                        draw_y,
                        String::from(ch),
                        Style::default().fg(Color::DarkGray),
                    );
                }
            }
        }
    }
}

// Helper function to draw a vertical line
fn draw_vertical_line(
    frame: &mut Frame,
    area: Rect,
    viewport_x: u16,
    viewport_y: u16,
    x: u16,
    y1: u16,
    y2: u16,
    ch: char,
) {
    if x >= viewport_x && x < viewport_x + area.width {
        let draw_x = area.x + x - viewport_x;
        if draw_x >= area.right() {
            return;
        }

        let start_map_y = y1.min(y2);
        let end_map_y = y1.max(y2);
        for map_y in start_map_y..=end_map_y {
            // Include end_y
            if map_y >= viewport_y && map_y < viewport_y + area.height {
                let draw_y = area.y + map_y - viewport_y;
                if draw_y < area.bottom() {
                    // Ensure not drawing past bottom edge
                    frame.buffer_mut().set_string(
                        draw_x,
                        draw_y,
                        String::from(ch),
                        Style::default().fg(Color::DarkGray),
                    );
                }
            }
        }
    }
}
