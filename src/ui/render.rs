#![allow(dead_code)]
use crate::config::Config;
use crate::core::{MindMap, NodeId};
use ratatui::prelude::{Frame, Rect, Size};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use textwrap::wrap;
use unicode_width::UnicodeWidthStr;

// Add horizontal spacing constant
const H_SPACING: u16 = 6;

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
#[derive(Debug, Clone, Serialize)]
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

    // --- Pass 3: Calculate X and Y positions ---
    // Set initial root position before recursive calculation
    if let Some(root_calc_node) = calc_nodes.get_mut(&root_id) {
        root_calc_node.x = 1; // Start root X at 1
        root_calc_node.y = 1; // Start root Y at 1 (or calculate based on centering later)
    }
    // Calculate positions for all nodes starting from the root
    calculate_position_recursive(map, config, root_id, &mut calc_nodes, 0, 0, 0);

    // --- Convert LayoutCalcNode to RenderNode ---
    let mut render_map = HashMap::new();
    for (id, calc_node) in calc_nodes {
        // if calc_node.is_visible { // TODO: Add visibility check later
        render_map.insert(
            id,
            RenderNode {
                id: calc_node.id,
                display_title: calc_node.text.clone(),
                x: calc_node.x,
                y: calc_node.y,
                w: calc_node.w,
                h: calc_node.h,
                is_active: id == active_node_id,
            },
        );
        // }
    }

    // Debug: Write the final render map to a file
    match serde_json::to_string_pretty(&render_map) {
        Ok(json) => {
            // Ensure target directory exists (create if not)
            if std::fs::create_dir_all("target").is_ok() {
                match File::create("target/render_map_debug.json") {
                    Ok(mut file) => {
                        if let Err(e) = file.write_all(json.as_bytes()) {
                            eprintln!("Error writing debug info: {}", e); // Log error
                        }
                    }
                    Err(e) => eprintln!("Error creating debug file: {}", e), // Log error
                }
            } else {
                eprintln!("Error creating target directory for debug file."); // Log error
            }
        }
        Err(e) => eprintln!("Error serializing render_map: {}", e), // Log error
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
        let max_width_for_wrap = if is_leaf {
            config.max_leaf_node_width
        } else {
            config.max_parent_node_width
        } as usize;

        // Perform text wrapping for height calculation
        let wrapped_lines: Vec<String> = wrap(&node.text, max_width_for_wrap)
            .iter()
            .map(|s| s.to_string())
            .collect();
        let wrapped_text = wrapped_lines.join("\n");

        // Calculate width based on ORIGINAL text width capped by max_width + borders
        let text_width = node.text.width() as u16;
        let max_allowed_text_width = max_width_for_wrap as u16;
        let w = text_width.min(max_allowed_text_width) + 2;

        // Calculate height based on number of wrapped lines + borders
        let h = wrapped_lines.len().max(1) as u16 + 2; // Ensure height is at least 1, add borders

        // Populate visible_children (assuming all are visible for now)
        let visible_children = node.children.clone();

        calc_nodes.insert(
            node_id,
            LayoutCalcNode {
                id: node_id,
                text: node.text.clone(),
                children: node.children.clone(),
                visible_children, // Use the populated list
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
    // --- Get immutable borrow first for checks ---
    let memoized_height = {
        let calc_node_opt = calc_nodes.get(&node_id);
        if calc_node_opt.is_none() {
            return 0; // Node doesn't exist
        }
        let calc_node = calc_node_opt.unwrap();
        // Check memoization
        if calc_node.subtree_h > 0 {
            return calc_node.subtree_h;
        }
        // Check visibility
        if !calc_node.is_visible {
            return 0;
        }
        // If not memoized and visible, proceed (no height to return yet)
        None
    };
    // Return memoized height if it was found
    if let Some(height) = memoized_height {
        return height;
    }

    // --- Get necessary data (need to re-borrow) ---
    let (node_h, visible_children_ids) = {
        // Node must exist based on the check above
        let calc_node = calc_nodes.get(&node_id).unwrap();
        (calc_node.h, calc_node.visible_children.clone())
    };

    // --- Recursively calculate for visible children ---
    let mut children_total_subtree_h = 0u16;
    let num_visible_children = visible_children_ids.len();

    for child_id in &visible_children_ids {
        // Ensure child exists before recursing (should always be true if map is consistent)
        if calc_nodes.contains_key(child_id) {
            children_total_subtree_h +=
                calculate_subtree_height_recursive(map, config, *child_id, calc_nodes);
        } else {
            // Log or handle inconsistency if necessary
            // eprintln!("Warning: Child node {} not found in calc_nodes during subtree height calculation.", child_id);
        }
    }

    // --- Calculate total spacing between children ---
    let total_spacing = if num_visible_children > 0 {
        (num_visible_children as u16).saturating_sub(1) * config.line_spacing as u16
    } else {
        0
    };

    // --- Determine final subtree height ---
    let calculated_subtree_h = if num_visible_children == 0 {
        // Leaf node (or node with no visible children): height is its own height
        node_h
    } else {
        // Non-leaf: sum of children subtree heights + spacing between them
        children_total_subtree_h + total_spacing
    };

    // --- Update the current node's subtree_h in the map ---
    // Need mutable borrow now
    if let Some(calc_node) = calc_nodes.get_mut(&node_id) {
        calc_node.subtree_h = calculated_subtree_h;
    }

    calculated_subtree_h // Return the calculated height
}

// Pass 3: Calculate X and Y positions
fn calculate_position_recursive(
    map: &MindMap,
    config: &Config,
    node_id: NodeId,
    calc_nodes: &mut HashMap<NodeId, LayoutCalcNode>,
    // Add parent coordinates as arguments
    parent_x: u16,
    _parent_y: u16, // Keep parent_y for potential future use (vertical centering?), prefix with _
    parent_w: u16,
) {
    let root_id = map.root.expect("Cannot calculate position without root");

    // --- Calculate current node's X position and read its W/H ---
    let node_w;
    // let node_h; // Removed unused variable
    let node_x; // This node's final X

    if node_id == root_id {
        // Root position is already set in calculate_layout. Read its values.
        let root_node = calc_nodes.get(&root_id).unwrap(); // Immutable borrow
        node_x = root_node.x;
        node_w = root_node.w;
        // node_h = root_node.h; // Removed unused assignment
        // Y will be read later just before centering
    } else {
        // Calculate X based on parent's position and width
        node_x = parent_x + parent_w + H_SPACING;
        // Set node's X. Y coordinate is set by the PARENT's loop.
        // Read W and H.
        let node = calc_nodes.get_mut(&node_id).unwrap(); // Mutable borrow
        node.x = node_x;
        // node.y = parent_y; // DO NOT set Y here; it's set by the parent's centering loop.
        node_w = node.w; // Read width
        // node_h = node.h; // Removed unused assignment
        // Mutable borrow dropped here
    }

    // --- Calculate Y position for children (center vertically) ---
    // Read the current node's *potentially updated* Y value just before centering its children
    let (current_node_y_for_centering, current_node_h, visible_children_ids) = {
        let node = calc_nodes.get(&node_id).unwrap(); // Immutable borrow
        (node.y, node.h, node.visible_children.clone())
    };

    let total_children_subtree_h = visible_children_ids.iter().fold(0, |acc, &child_id| {
        // Use get() defensively here, although children should exist
        acc + calc_nodes.get(&child_id).map_or(0, |cn| cn.subtree_h)
    });

    let children_start_y;
    if total_children_subtree_h > 0 {
        let parent_center_y = current_node_y_for_centering + current_node_h / 2;
        // Calculate total height including spacing between children for centering
        let num_visible_children = visible_children_ids.len() as u16;
        let total_spacing_between_children = if num_visible_children > 0 {
            num_visible_children.saturating_sub(1) * config.line_spacing as u16
        } else {
            0
        };
        let total_height_with_spacing = total_children_subtree_h + total_spacing_between_children;

        children_start_y = parent_center_y.saturating_sub(total_height_with_spacing / 2);
    } else {
        children_start_y = current_node_y_for_centering; // Align with parent if no children
    }

    // --- Position children and recurse ---
    let mut current_y_offset = children_start_y; // Start Y for the first child
    // Collect necessary info first to avoid borrow issues in recursion
    let mut child_info_for_recursion = Vec::new();
    for child_id in &visible_children_ids {
        if let Some(child_node) = calc_nodes.get(child_id) {
            // Immutable borrow here
            child_info_for_recursion.push((*child_id, child_node.subtree_h));
        }
    }

    // Now iterate through the collected info
    for (child_id, subtree_h) in child_info_for_recursion {
        // Scope the mutable borrow to set child position
        {
            // Get mutable access to the child node
            if let Some(child_calc_node) = calc_nodes.get_mut(&child_id) {
                // Child X depends on the *current* node's X and W (calculated/read above)
                child_calc_node.x = node_x + node_w + H_SPACING;
                // Set child's Y position based on vertical centering offset
                child_calc_node.y = current_y_offset;
            } else {
                continue; // Skip if node somehow disappeared (shouldn't happen)
            }
        } // Mutable borrow of child_calc_node ends here

        // Recurse AFTER the mutable borrow is released.
        // Pass the CURRENT node's calculated X, its finalized Y, and its W.
        calculate_position_recursive(
            map,
            config,
            child_id,
            calc_nodes,
            node_x,                       // Pass CURRENT node's final x
            current_node_y_for_centering, // Pass CURRENT node's final y
            node_w,                       // Pass CURRENT node's final w
        );

        // Increment the offset for the next sibling
        current_y_offset += subtree_h + config.line_spacing as u16;
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
    frame.render_widget(viewport_block, area); // Restore border drawing

    for (_id, render_node) in render_nodes {
        // 1. Define node boundaries in world coordinates
        let node_world_rect = Rect::new(render_node.x, render_node.y, render_node.w, render_node.h);

        // 2. Define the viewport boundaries in world coordinates
        let viewport_world_rect =
            Rect::new(viewport_x, viewport_y, inner_area.width, inner_area.height);

        // 3. Check if the node intersects the viewport *at all*
        if !viewport_world_rect.intersects(node_world_rect) {
            continue; // Skip drawing if node is entirely outside viewport
        }

        // 4. Calculate the node's top-left corner position in terminal coordinates,
        //    relative to the inner_area's top-left.
        let draw_x = inner_area.x + render_node.x.saturating_sub(viewport_x);
        let draw_y = inner_area.y + render_node.y.saturating_sub(viewport_y);

        // 5. Define the node's potential drawing rectangle in terminal coordinates.
        let node_terminal_rect = Rect {
            x: draw_x,
            y: draw_y,
            width: render_node.w,
            height: render_node.h,
        };

        // 6. Clip this terminal rectangle against the drawable inner_area.
        //    The widget will be rendered within this clipped area.
        let clipped_node_area = inner_area.intersection(node_terminal_rect);

        // 7. Skip drawing if clipping results in zero area (e.g., node is just off-screen).
        if clipped_node_area.area() == 0 {
            continue;
        }

        // --- Draw Node Widget directly here ---
        // Use the is_active flag from RenderNode
        let node_style = if render_node.is_active {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else {
            Style::default().fg(Color::White).bg(Color::Blue)
        };

        // --- Restore Paragraph drawing ---
        let paragraph = Paragraph::new(render_node.display_title.clone()).style(node_style);

        frame.render_widget(paragraph, clipped_node_area);

        // Connection lines removed
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::core::MindMap;
    use ratatui::prelude::Size;

    // Helper to create a simple map for testing
    fn create_test_map() -> (MindMap, NodeId, NodeId, NodeId, NodeId) {
        let mut map = MindMap::new();
        let root_id = map.add_node("Root".to_string(), None).unwrap();
        let child1_id = map.add_node("Child 1".to_string(), Some(root_id)).unwrap();
        let child2_id = map
            .add_node("Child 2 has longer text".to_string(), Some(root_id))
            .unwrap();
        let grandchild1_id = map
            .add_node("Grandchild 1.1".to_string(), Some(child1_id))
            .unwrap();
        (map, root_id, child1_id, child2_id, grandchild1_id)
    }

    #[test]
    fn test_basic_layout_calculation() {
        let (map, root_id, child1_id, child2_id, grandchild1_id) = create_test_map();
        let config = Config::default(); // Use default config
        let area = Size::new(100, 40); // Mock area, currently unused but good practice

        let layout = calculate_layout(&map, &config, area, root_id);

        assert_eq!(layout.len(), 4, "Should have layout for 4 nodes");

        // Root Node Assertions
        let root_node = layout.get(&root_id).expect("Root node not found in layout");
        assert_eq!(root_node.display_title, "Root");
        assert_eq!(root_node.x, 1, "Root X mismatch");
        // Y depends on centering, let's check relative positions later if needed
        assert_eq!(root_node.w, 4 + 2, "Root W mismatch (4 text + 2 border)"); // "Root".len() = 4
        assert_eq!(root_node.h, 1 + 2, "Root H mismatch (1 line + 2 border)"); // 1 line
        assert!(root_node.is_active, "Root should be active");

        // Child 1 Assertions
        let child1_node = layout.get(&child1_id).expect("Child 1 not found");
        assert_eq!(child1_node.display_title, "Child 1");
        assert_eq!(
            child1_node.x,
            root_node.x + root_node.w + H_SPACING,
            "Child 1 X mismatch"
        );
        // Y check might be complex, focus on width/height for now
        assert_eq!(child1_node.w, 7 + 2, "Child 1 W mismatch"); // "Child 1".len() = 7
        assert_eq!(child1_node.h, 1 + 2, "Child 1 H mismatch"); // 1 line
        assert!(!child1_node.is_active, "Child 1 should not be active");

        // Child 2 Assertions
        let child2_node = layout.get(&child2_id).expect("Child 2 not found");
        assert_eq!(child2_node.display_title, "Child 2 has longer text");
        assert_eq!(
            child2_node.x,
            root_node.x + root_node.w + H_SPACING,
            "Child 2 X mismatch"
        );
        // Y relative to child1
        assert_eq!(
            child2_node.y,
            child1_node.y + child1_node.h + config.line_spacing as u16,
            "Child 2 Y mismatch"
        );
        assert_eq!(child2_node.w, 22 + 2, "Child 2 W mismatch"); // "Child 2 has longer text".len() = 22
        assert_eq!(child2_node.h, 1 + 2, "Child 2 H mismatch"); // 1 line
        assert!(!child2_node.is_active, "Child 2 should not be active");

        // Grandchild 1.1 Assertions
        let grandchild1_node = layout.get(&grandchild1_id).expect("Grandchild 1 not found");
        assert_eq!(grandchild1_node.display_title, "Grandchild 1.1");
        assert_eq!(
            grandchild1_node.x,
            child1_node.x + child1_node.w + H_SPACING,
            "Grandchild 1 X mismatch"
        );
        // Check Y relative to parent (Child 1)
        assert_eq!(
            grandchild1_node.y, child1_node.y,
            "Grandchild 1 Y should align with Child 1 Y (approx)"
        );
        assert_eq!(grandchild1_node.w, 14 + 2, "Grandchild 1 W mismatch"); // "Grandchild 1.1".len() = 14
        assert_eq!(grandchild1_node.h, 1 + 2, "Grandchild 1 H mismatch"); // 1 line
        assert!(
            !grandchild1_node.is_active,
            "Grandchild 1 should not be active"
        );
    }

    // Add more tests here later for:
    // - Text wrapping
    // - Different active nodes
    // - Large maps requiring scrolling (if viewport affects layout)
    // - Different config values (spacing, max width)
}
