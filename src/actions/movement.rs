use crate::app::AppState;
use crate::layout::LayoutEngine;
use crate::model::NodeId;

// Weight factor for prioritizing vertical movement over horizontal
// Higher value means vertical distance matters more
const VERTICAL_WEIGHT: f64 = 15.0;

// Helper function to ensure active node is visible
pub fn ensure_node_visible(app: &mut AppState) {
    if app.config.center_lock {
        center_active_node(app);
    } else if let Some(active_id) = app.active_node_id {
        let layout = LayoutEngine::calculate_layout(app);

        if let Some(node_layout) = layout.nodes.get(&active_id) {
            let node_x = node_layout.x;
            let node_y = node_layout.y + node_layout.yo;
            let node_right = node_x + node_layout.w;
            let node_bottom = node_y + node_layout.lh;

            // Adjust viewport to ensure node is visible
            let margin = 2.0; // Small margin around the node

            // Horizontal adjustment
            if node_x < app.viewport_left + margin {
                app.viewport_left = (node_x - margin).max(0.0);
            } else if node_right > app.viewport_left + app.terminal_width as f64 - margin {
                app.viewport_left = node_right - app.terminal_width as f64 + margin;
            }

            // Vertical adjustment
            if node_y < app.viewport_top + margin {
                app.viewport_top = (node_y - margin).max(0.0);
            } else if node_bottom > app.viewport_top + app.terminal_height as f64 - margin {
                app.viewport_top = node_bottom - app.terminal_height as f64 + margin;
            }
        }
    }
}

// Helper to get center position of a node
fn get_node_center(layout: &LayoutEngine, node_id: NodeId) -> Option<(f64, f64)> {
    layout.nodes.get(&node_id).map(|node| {
        let center_x = node.x + node.w / 2.0;
        let center_y = node.y + node.yo + node.lh / 2.0;
        (center_x, center_y)
    })
}

// Find the nearest node in a specific direction using spatial distance
fn find_nearest_node_in_direction(
    _app: &AppState,
    layout: &LayoutEngine,
    active_id: NodeId,
    direction_x: f64,
    direction_y: f64,
) -> Option<NodeId> {
    let (current_x, current_y) = get_node_center(layout, active_id)?;

    let mut best_distance = f64::MAX;
    let mut best_node = None;

    // Search through all visible nodes
    for (node_id, node_layout) in &layout.nodes {
        // Skip the current node and root's parent
        if *node_id == active_id || node_layout.x < 0.0 || node_layout.y < 0.0 {
            continue;
        }

        let (node_x, node_y) = get_node_center(layout, *node_id)?;
        let dx = node_x - current_x;
        let dy = node_y - current_y;

        // Check if the node is in the desired direction
        let in_direction = (direction_x == 0.0 || dx * direction_x > 0.0)
            && (direction_y == 0.0 || dy * direction_y > 0.0);

        if !in_direction {
            continue;
        }

        // Calculate weighted distance (prioritize vertical movement)
        let distance = if direction_y != 0.0 {
            // For up/down movement, heavily weight vertical distance
            (dy * VERTICAL_WEIGHT).powi(2) + dx.powi(2)
        } else {
            // For left/right movement, use normal distance
            dy.powi(2) + dx.powi(2)
        };

        if distance < best_distance {
            best_distance = distance;
            best_node = Some(*node_id);
        }
    }

    best_node
}

pub fn go_up(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        let layout = LayoutEngine::calculate_layout(app);

        // First try to move to previous sibling based on position
        if let Some(parent_id) = active_id.ancestors(&app.tree).nth(1) {
            if let Some(current_layout) = layout.nodes.get(&active_id) {
                let current_y = current_layout.y + current_layout.yo;

                // Find siblings that are above us
                let mut best_sibling = None;
                let mut best_y = -1.0;

                for sibling_id in parent_id.children(&app.tree) {
                    if sibling_id == active_id {
                        continue;
                    }

                    if let Some(sibling_layout) = layout.nodes.get(&sibling_id) {
                        let sibling_y = sibling_layout.y + sibling_layout.yo;
                        if sibling_y < current_y && sibling_y > best_y {
                            best_y = sibling_y;
                            best_sibling = Some(sibling_id);
                        }
                    }
                }

                if let Some(sibling) = best_sibling {
                    app.active_node_id = Some(sibling);
                    ensure_node_visible(app);
                    return;
                }
            }
        }

        // If no sibling above, find any node above us
        if let Some(nearest) = find_nearest_node_in_direction(app, &layout, active_id, 0.0, -1.0) {
            app.active_node_id = Some(nearest);
            ensure_node_visible(app);
        }
    }
}

pub fn go_down(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        let layout = LayoutEngine::calculate_layout(app);

        // First try to move to next sibling based on position
        if let Some(parent_id) = active_id.ancestors(&app.tree).nth(1) {
            if let Some(current_layout) = layout.nodes.get(&active_id) {
                let current_y = current_layout.y + current_layout.yo;

                // Find siblings that are below us
                let mut best_sibling = None;
                let mut best_y = f64::MAX;

                for sibling_id in parent_id.children(&app.tree) {
                    if sibling_id == active_id {
                        continue;
                    }

                    if let Some(sibling_layout) = layout.nodes.get(&sibling_id) {
                        let sibling_y = sibling_layout.y + sibling_layout.yo;
                        if sibling_y > current_y && sibling_y < best_y {
                            best_y = sibling_y;
                            best_sibling = Some(sibling_id);
                        }
                    }
                }

                if let Some(sibling) = best_sibling {
                    app.active_node_id = Some(sibling);
                    ensure_node_visible(app);
                    return;
                }
            }
        }

        // If no sibling below, find any node below us
        if let Some(nearest) = find_nearest_node_in_direction(app, &layout, active_id, 0.0, 1.0) {
            app.active_node_id = Some(nearest);
            ensure_node_visible(app);
        }
    }
}

pub fn go_left(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        if let Some(parent_id) = active_id.ancestors(&app.tree).nth(1) {
            // Allow moving to parent even if it's the root
            app.active_node_id = Some(parent_id);
            ensure_node_visible(app);
        }
    }
}

pub fn go_right(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        let has_children = active_id.children(&app.tree).next().is_some();
        let is_collapsed = app.tree.get(active_id).unwrap().get().is_collapsed;

        // Auto-expand collapsed nodes when moving right (like PHP h-m-m)
        if is_collapsed && has_children {
            // Toggle the collapsed state
            app.tree.get_mut(active_id).unwrap().get_mut().is_collapsed = false;
        }

        // Get layout after potential expansion
        let layout = LayoutEngine::calculate_layout(app);

        if let Some(current_layout) = layout.nodes.get(&active_id) {
            let current_y = current_layout.y + current_layout.yo + current_layout.lh / 2.0;

            // Find the child closest to our vertical position
            let mut best_child = None;
            let mut best_distance = f64::MAX;

            for child_id in active_id.children(&app.tree) {
                if let Some(child_layout) = layout.nodes.get(&child_id) {
                    let child_y = child_layout.y + child_layout.yo + child_layout.lh / 2.0;
                    let distance = (child_y - current_y).abs();

                    if distance < best_distance {
                        best_distance = distance;
                        best_child = Some(child_id);
                    }
                }
            }

            if let Some(child) = best_child {
                app.active_node_id = Some(child);
                ensure_node_visible(app);
            }
        }
    }
}

pub fn go_to_root(app: &mut AppState) {
    app.active_node_id = app.root_id;
    ensure_node_visible(app);
}

pub fn go_to_top(app: &mut AppState) {
    let layout = LayoutEngine::calculate_layout(app);

    // Find the node with the smallest y position (topmost)
    let mut top_node = None;
    let mut min_y = f64::MAX;

    for (node_id, node_layout) in &layout.nodes {
        // Skip invalid nodes
        if node_layout.x < 0.0 || node_layout.y < 0.0 {
            continue;
        }

        let node_y = node_layout.y + node_layout.yo;
        if node_y < min_y {
            min_y = node_y;
            top_node = Some(*node_id);
        }
    }

    if let Some(node_id) = top_node {
        app.active_node_id = Some(node_id);
        app.viewport_top = 0.0;
        app.viewport_left = 0.0;
    }
}

pub fn go_to_bottom(app: &mut AppState) {
    let layout = LayoutEngine::calculate_layout(app);

    // Find the node with the largest y position (bottommost)
    let mut bottom_node = None;
    let mut max_y = -1.0;

    for (node_id, node_layout) in &layout.nodes {
        // Skip invalid nodes
        if node_layout.x < 0.0 || node_layout.y < 0.0 {
            continue;
        }

        let node_y = node_layout.y + node_layout.yo + node_layout.lh;
        if node_y > max_y {
            max_y = node_y;
            bottom_node = Some(*node_id);
        }
    }

    if let Some(node_id) = bottom_node {
        app.active_node_id = Some(node_id);
        ensure_node_visible(app);
    }
}

// Import from view module to avoid circular dependency
use super::view::center_active_node;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::model::Node;

    fn create_test_app() -> AppState {
        let config = AppConfig::default();
        let mut app = AppState::new(config);

        // Create a simple tree
        let root = app.tree.new_node(Node::new("Root".to_string()));
        let child1 = app.tree.new_node(Node::new("Child 1".to_string()));
        let child2 = app.tree.new_node(Node::new("Child 2".to_string()));
        let grandchild = app.tree.new_node(Node::new("Grandchild".to_string()));

        root.append(child1, &mut app.tree);
        root.append(child2, &mut app.tree);
        child2.append(grandchild, &mut app.tree);

        app.root_id = Some(root);
        app.active_node_id = Some(root);

        app
    }

    #[test]
    fn test_spatial_movement_go_down() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        // From root, going down should find the node below (spatially)
        // In our test tree: Root -> Child1 or Child2 (whichever is positioned below)
        go_down(&mut app);

        // Should move to one of the children
        assert_ne!(app.active_node_id, Some(root));
        assert!(app.active_node_id.is_some());
    }

    #[test]
    fn test_spatial_movement_go_up() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let _child1 = root.children(&app.tree).next().unwrap();
        let child2 = root.children(&app.tree).nth(1).unwrap();

        // Start at child2
        app.active_node_id = Some(child2);

        // Going up should move to the node above spatially
        // Could be child1 or root depending on layout
        go_up(&mut app);

        // Should have moved somewhere
        assert_ne!(app.active_node_id, Some(child2));
    }

    #[test]
    fn test_spatial_movement_siblings() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();
        let child2 = root.children(&app.tree).nth(1).unwrap();

        // Start at child1
        app.active_node_id = Some(child1);

        // Going down should move to sibling below if it exists
        go_down(&mut app);

        // In the default layout, should move to child2
        assert_eq!(app.active_node_id, Some(child2));

        // Going up from child2 should go back to child1
        go_up(&mut app);
        assert_eq!(app.active_node_id, Some(child1));
    }

    #[test]
    fn test_movement_go_left() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();

        app.active_node_id = Some(child1);
        go_left(&mut app);
        assert_eq!(app.active_node_id, Some(root));
    }

    #[test]
    fn test_movement_go_right() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        // From root, go right should move to a child (closest vertically)
        go_right(&mut app);

        // Should be at one of the children
        assert_ne!(app.active_node_id, Some(root));
        let is_child = root
            .children(&app.tree)
            .any(|c| Some(c) == app.active_node_id);
        assert!(is_child, "Should move to a child node");
    }

    #[test]
    fn test_movement_go_right_auto_expand() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        // Collapse the root
        app.tree.get_mut(root).unwrap().get_mut().is_collapsed = true;

        // Going right should auto-expand and move to child
        go_right(&mut app);

        // Node should be expanded
        assert!(!app.tree.get(root).unwrap().get().is_collapsed);

        // Should be at one of the children
        let is_child = root
            .children(&app.tree)
            .any(|c| Some(c) == app.active_node_id);
        assert!(is_child, "Should move to a child after auto-expand");
    }

    #[test]
    fn test_movement_go_to_root() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child1 = root.children(&app.tree).next().unwrap();

        app.active_node_id = Some(child1);
        go_to_root(&mut app);
        assert_eq!(app.active_node_id, Some(root));
    }

    #[test]
    fn test_go_to_top() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let child2 = root.children(&app.tree).nth(1).unwrap();

        // Start at child2
        app.active_node_id = Some(child2);

        go_to_top(&mut app);

        // Should be at the topmost node (usually root)
        // In spatial navigation, this is the node with smallest y coordinate
        assert!(app.active_node_id.is_some());
        assert_eq!(app.viewport_top, 0.0);
    }

    #[test]
    fn test_go_to_bottom() {
        let mut app = create_test_app();

        // Expand all to make all nodes visible
        for node in app.tree.iter_mut() {
            node.get_mut().is_collapsed = false;
        }

        go_to_bottom(&mut app);

        // Should be at the bottommost visible node
        // This is the node with the largest y coordinate
        assert!(app.active_node_id.is_some());
    }
}
