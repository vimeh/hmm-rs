use crate::app::AppState;
use crate::model::NodeId;
use crate::ui::canvas::BufferCanvas;
use crate::ui::constants::{connections, junction};
use crate::ui::view::ViewMap;
use ratatui::layout::Rect;
use std::cmp::{max, min};

/// Renders connections between nodes based on pre-calculated screen positions.
pub struct ConnectionRenderer<'a> {
    canvas: &'a mut BufferCanvas,
    app: &'a AppState,
    view_map: &'a ViewMap,
    area: Rect,
}

impl<'a> ConnectionRenderer<'a> {
    pub fn new(
        canvas: &'a mut BufferCanvas,
        app: &'a AppState,
        view_map: &'a ViewMap,
        area: Rect,
    ) -> Self {
        Self {
            canvas,
            app,
            view_map,
            area,
        }
    }

    /// Recursively draw connections for a node and its visible children.
    pub fn draw_connections_for_node(&mut self, node_id: NodeId) {
        let Some(node_ref) = self.app.tree.get(node_id) else {
            return;
        };
        let node = node_ref.get();

        let Some(parent_view) = self.view_map.get(&node_id) else {
            return;
        };

        let all_children: Vec<_> = node_id.children(&self.app.tree).collect();
        let visible_children: Vec<_> = all_children
            .iter()
            .filter(|id| self.view_map.contains_key(id))
            .cloned()
            .collect();

        let has_hidden = all_children.len() != visible_children.len();

        let parent_exit_x = (parent_view.screen_rect.x + parent_view.screen_rect.width) as i32;
        let parent_exit_y = parent_view.connection_y as i32;

        // --- Simplified Drawing Logic ---
        if node.is_collapsed && !all_children.is_empty() {
            self.draw_collapsed_indicator(parent_exit_x, parent_exit_y, has_hidden);
        } else if !visible_children.is_empty() {
            self.draw_child_connections(
                node_id,
                parent_exit_x,
                parent_exit_y,
                &visible_children,
                has_hidden,
            );
        } else if has_hidden {
            // Only hidden children
            self.draw_hidden_only_indicator(parent_exit_x, parent_exit_y);
        }

        // Recurse if the node is not collapsed
        if !node.is_collapsed {
            for child_id in visible_children {
                self.draw_connections_for_node(child_id);
            }
        }
    }

    fn draw_child_connections(
        &mut self,
        parent_node_id: NodeId,
        parent_x: i32,
        parent_y: i32,
        children: &[NodeId],
        has_hidden: bool,
    ) {
        if children.len() == 1 {
            // Single child - draw simple horizontal line
            let child_view = self.view_map.get(&children[0]).unwrap();
            let child_x = child_view.screen_rect.x as i32;
            let child_y = child_view.connection_y as i32;

            let connector = if has_hidden {
                connections::SINGLE_HIDDEN
            } else {
                connections::SINGLE
            };
            self.draw_text(parent_x, parent_y.min(child_y), connector);

            // Draw vertical connection if needed
            if (parent_y - child_y).abs() > 0 {
                let spine_x = child_x - 2;
                self.draw_vertical_line(
                    spine_x,
                    parent_y.min(child_y),
                    parent_y.max(child_y),
                    junction::VERTICAL,
                );

                // Draw corners
                let corner = if child_y > parent_y {
                    junction::BOTTOM_CORNER
                } else {
                    junction::TOP_CORNER
                };
                self.canvas
                    .set_char(spine_x as usize, child_y as usize, corner);

                let top_corner = if child_y > parent_y {
                    junction::TOP_CORNER
                } else {
                    junction::BOTTOM_CORNER
                };
                self.canvas
                    .set_char(spine_x as usize, parent_y.min(child_y) as usize, top_corner);
            }
        } else {
            // Multiple children
            let first_child_view = self.view_map.get(&children[0]).unwrap();
            let last_child_view = self.view_map.get(children.last().unwrap()).unwrap();

            // 1. Draw horizontal line from parent to the spine
            let connector = if has_hidden {
                connections::MULTI_HIDDEN
            } else {
                connections::MULTI
            };
            self.draw_text(parent_x, parent_y, " "); // Add space after parent text
            self.draw_text(parent_x + 1, parent_y, connector); // Draw connector after the space

            let spine_x = parent_x + 1 + connector.chars().count() as i32; // Account for the space
            let spine_start_y = first_child_view.connection_y as i32;
            let spine_end_y = last_child_view.connection_y as i32;

            // 2. Draw vertical spine
            self.draw_vertical_line(spine_x, spine_start_y, spine_end_y, junction::VERTICAL);

            // 3. Draw junction from parent to spine
            self.fix_junction(spine_x, parent_y, junction::MIDDLE_RIGHT);

            // 4. Draw connectors from spine to each child
            for (i, child_id) in children.iter().enumerate() {
                let child_view = self.view_map.get(child_id).unwrap();
                let child_x = child_view.screen_rect.x as i32;
                let child_y = child_view.connection_y as i32;

                // Use junction constants
                let junction_char = match i {
                    0 => junction::TOP_CORNER,
                    n if n == children.len() - 1 => junction::BOTTOM_CORNER,
                    _ => junction::MIDDLE_LEFT,
                };

                // Check if this is a direct child of root
                let is_root_child = self.app.root_id == Some(parent_node_id);

                if is_root_child {
                    // For direct children of root, junction is at the spine
                    self.canvas.set_char(spine_x as usize, child_y as usize, junction_char);
                    // Always draw a space after the junction
                    self.canvas.set_char((spine_x + 1) as usize, child_y as usize, ' ');
                    // Draw horizontal line from after the space to before the text
                    for x in (spine_x + 2)..child_x {
                        self.canvas.set_char(x as usize, child_y as usize, '─');
                    }
                } else {
                    // For grandchildren and deeper, junction is 2 chars before text
                    self.draw_text(child_x - 2, child_y, &junction_char.to_string());
                    self.draw_text(child_x - 1, child_y, " ");
                    // Draw horizontal line from spine to junction if there's a gap
                    for x in spine_x..(child_x - 2) {
                        self.canvas.set_char(x as usize, child_y as usize, '─');
                    }
                }
            }
        }
    }

    fn draw_collapsed_indicator(&mut self, x: i32, y: i32, has_hidden: bool) {
        let text = if has_hidden {
            connections::COLLAPSED_HIDDEN
        } else {
            connections::COLLAPSED
        };
        self.draw_text(x, y, text);
    }

    fn draw_hidden_only_indicator(&mut self, x: i32, y: i32) {
        self.draw_text(x, y, connections::HIDDEN_ONLY);
    }

    // --- Low-level, Idiomatic Drawing Primitives ---

    fn draw_text(&mut self, x: i32, y: i32, text: &str) {
        if y >= 0 && y < self.area.height as i32 {
            for (i, ch) in text.chars().enumerate() {
                let cx = x + i as i32;
                if cx >= 0 && cx < self.area.width as i32 {
                    self.canvas.set_char(cx as usize, y as usize, ch);
                }
            }
        }
    }

    fn draw_vertical_line(&mut self, x: i32, y1: i32, y2: i32, ch: char) {
        if x >= 0 && x < self.area.width as i32 {
            for y in min(y1, y2)..=max(y1, y2) {
                if y >= 0 && y < self.area.height as i32 {
                    self.canvas.set_char(x as usize, y as usize, ch);
                }
            }
        }
    }

    /// Overwrites a character at a junction point, for example turning a '|' into a '├'.
    fn fix_junction(&mut self, x: i32, y: i32, ch: char) {
        if self.is_in_bounds(x, y) {
            self.canvas.set_char(x as usize, y as usize, ch);
        }
    }

    fn is_in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.area.width as i32 && y < self.area.height as i32
    }
}
