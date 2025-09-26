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

        // Check if any children are hidden
        let has_hidden = all_children.len() != visible_children.len()
            || all_children.iter().any(|child_id| {
                self.app
                    .tree
                    .get(*child_id)
                    .map(|n| n.get().is_hidden())
                    .unwrap_or(false)
            });

        let parent_exit_x = parent_view.exit_point.0 as i32;
        let parent_exit_y = parent_view.exit_point.1 as i32;

        // Simplified drawing logic using pre-calculated points
        if node.is_collapsed && !all_children.is_empty() {
            self.draw_collapsed_indicator(parent_exit_x, parent_exit_y, has_hidden);
        } else if !visible_children.is_empty() {
            self.draw_child_connections(parent_view, &visible_children, has_hidden);
        } else if has_hidden {
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
        parent_view: &crate::ui::view::ViewNode,
        children: &[NodeId],
        has_hidden: bool,
    ) {
        let parent_exit_x = parent_view.exit_point.0 as i32;
        let parent_exit_y = parent_view.exit_point.1 as i32;

        if children.len() == 1 {
            // Single child - draw direct connection
            let child_view = self.view_map.get(&children[0]).unwrap();
            let child_entry_x = child_view.entry_point.0 as i32;
            let child_entry_y = child_view.entry_point.1 as i32;

            // Draw horizontal line from parent exit to child entry
            let connector = if has_hidden {
                connections::SINGLE_HIDDEN
            } else {
                connections::SINGLE
            };
            self.draw_text(parent_exit_x, parent_exit_y, " ");
            self.draw_text(parent_exit_x + 1, parent_exit_y, connector);

            // If vertical alignment differs, draw vertical connector
            if parent_exit_y != child_entry_y {
                let spine_x = child_entry_x - 2;
                self.draw_vertical_line(
                    spine_x,
                    parent_exit_y.min(child_entry_y),
                    parent_exit_y.max(child_entry_y),
                    junction::VERTICAL,
                );
                self.canvas.set_char(
                    spine_x as usize,
                    child_entry_y as usize,
                    if child_entry_y > parent_exit_y {
                        junction::BOTTOM_CORNER
                    } else {
                        junction::TOP_CORNER
                    },
                );
            }
        } else if let Some(spine_x) = parent_view.child_spine_x {
            // Multiple children - use the pre-calculated spine position
            let spine_x = spine_x as i32;

            // Draw horizontal from parent to spine
            let connector = if has_hidden {
                connections::MULTI_HIDDEN
            } else {
                connections::MULTI
            };
            self.draw_text(parent_exit_x, parent_exit_y, " ");
            self.draw_text(parent_exit_x + 1, parent_exit_y, connector);
            self.draw_horizontal_line(
                parent_exit_x + 1 + connector.len() as i32,
                spine_x,
                parent_exit_y,
                '─',
            );

            // Draw vertical spine
            let first_child_y = self.view_map.get(&children[0]).unwrap().entry_point.1 as i32;
            let last_child_y = self
                .view_map
                .get(children.last().unwrap())
                .unwrap()
                .entry_point
                .1 as i32;
            self.draw_vertical_line(spine_x, first_child_y, last_child_y, junction::VERTICAL);

            // Fix junction at parent connection
            self.fix_junction(spine_x, parent_exit_y, junction::MIDDLE_RIGHT);

            // Draw connections from spine to each child
            for (i, child_id) in children.iter().enumerate() {
                let child_view = self.view_map.get(child_id).unwrap();
                let child_entry_x = child_view.entry_point.0 as i32;
                let child_entry_y = child_view.entry_point.1 as i32;

                let junction_char = match i {
                    0 => junction::TOP_CORNER,
                    n if n == children.len() - 1 => junction::BOTTOM_CORNER,
                    _ => junction::MIDDLE_LEFT,
                };

                // Draw horizontal from spine to child
                self.draw_horizontal_line(spine_x, child_entry_x, child_entry_y, '─');
                self.fix_junction(spine_x, child_entry_y, junction_char);
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

    fn draw_horizontal_line(&mut self, x1: i32, x2: i32, y: i32, ch: char) {
        if y >= 0 && y < self.area.height as i32 {
            for x in min(x1, x2)..=max(x1, x2) {
                if x >= 0 && x < self.area.width as i32 {
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
