use crate::app::AppState;
use crate::model::NodeId;
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

const CONN_LEFT_LEN: usize = 6;
const CONN_RIGHT_LEN: usize = 4;
const WIDTH_TOLERANCE: f32 = 1.3;
const LEFT_PADDING: usize = 1;

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub lh: f64, // Line height
    pub yo: f64, // Y offset for centering
    pub xo: f64, // X offset for unicode width compensation
}

pub struct LayoutEngine {
    pub nodes: HashMap<NodeId, LayoutNode>,
    pub map_width: f64,
    pub map_height: f64,
    pub map_top: f64,
    pub map_bottom: f64,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            map_width: 0.0,
            map_height: 0.0,
            map_top: 0.0,
            map_bottom: 0.0,
        }
    }

    pub fn calculate_layout(app: &AppState) -> Self {
        let mut engine = Self::new();

        if let Some(root_id) = app.root_id {
            // First pass: calculate widths and line heights
            engine.calculate_x_and_lh(app, root_id, 0.0);

            // Second pass: calculate heights
            engine.calculate_h(app, root_id);

            // Third pass: calculate y positions
            engine.calculate_y(app, root_id, 0.0);

            // Fourth pass: calculate x offsets for unicode width
            engine.calculate_xo(app);
        }

        engine
    }

    fn calculate_x_and_lh(&mut self, app: &AppState, node_id: NodeId, parent_x: f64) {
        let node = app.tree.get(node_id).unwrap().get();

        // Calculate x position
        let x = if node_id == app.root_id.unwrap() {
            LEFT_PADDING as f64
        } else {
            parent_x
                + self
                    .nodes
                    .get(&node_id.ancestors(&app.tree).nth(1).unwrap())
                    .map(|p| p.w)
                    .unwrap_or(0.0)
                + (CONN_LEFT_LEN + CONN_RIGHT_LEN + 1) as f64
        };

        // Determine if this is a leaf or collapsed node
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

        let at_the_end = children.is_empty() || node.is_collapsed;

        // Get max width for this node type
        let max_width = if at_the_end {
            app.config.max_leaf_node_width
        } else {
            app.config.max_parent_node_width
        };

        // Calculate width and line height
        let title_width = node.title.width();
        let (w, lh) = if title_width as f32 > WIDTH_TOLERANCE * max_width as f32 {
            // Need to wrap text
            let lines = wrap_text(&node.title, max_width);
            let max_line_width = lines.iter().map(|l| l.width()).max().unwrap_or(0);
            (max_line_width as f64, lines.len() as f64)
        } else {
            (title_width as f64, 1.0)
        };

        // Store the layout node
        self.nodes.insert(
            node_id,
            LayoutNode {
                x,
                y: 0.0, // Will be calculated later
                w,
                h: 0.0, // Will be calculated later
                lh,
                yo: 0.0, // Will be calculated later
                xo: 0.0, // Will be calculated later
            },
        );

        // Update map width
        self.map_width = self.map_width.max(x + w);

        // Recurse for children
        for child_id in children {
            self.calculate_x_and_lh(app, child_id, x);
        }
    }

    fn calculate_h(&mut self, app: &AppState, node_id: NodeId) -> f64 {
        let node = app.tree.get(node_id).unwrap().get();

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

        let at_the_end = children.is_empty() || node.is_collapsed;

        let h = if at_the_end {
            // Leaf node: height is line height plus spacing
            let layout = self.nodes.get(&node_id).unwrap();
            app.config.line_spacing as f64 + layout.lh
        } else {
            // Parent node: height is sum of children or own line height
            let mut children_height = 0.0;
            for child_id in &children {
                children_height += self.calculate_h(app, *child_id);
            }

            let layout = self.nodes.get(&node_id).unwrap();
            children_height.max(layout.lh + app.config.line_spacing as f64)
        };

        // Update the layout node with calculated height
        if let Some(layout) = self.nodes.get_mut(&node_id) {
            layout.h = h;
        }

        h
    }

    fn calculate_y(&mut self, app: &AppState, node_id: NodeId, current_y: f64) {
        let node = app.tree.get(node_id).unwrap().get();

        // Set this node's y position
        if let Some(layout) = self.nodes.get_mut(&node_id) {
            layout.y = current_y;

            // Calculate y offset for vertical centering
            layout.yo = ((layout.h - layout.lh) / 2.0).round();
        }

        // Update map boundaries
        let layout = self.nodes.get(&node_id).unwrap();
        self.map_bottom = self
            .map_bottom
            .max(current_y + layout.lh + app.config.line_spacing as f64);
        self.map_top = self.map_top.min(current_y);

        // Process children
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

            let mut child_y = current_y;
            for child_id in children {
                self.calculate_y(app, child_id, child_y);
                let child_layout = self.nodes.get(&child_id).unwrap();
                child_y += child_layout.h;
            }
        }

        self.map_height = self.map_bottom - self.map_top;
    }

    fn calculate_xo(&mut self, app: &AppState) {
        // Calculate x offset to compensate for unicode width differences
        for (node_id, layout) in self.nodes.iter_mut() {
            let node = app.tree.get(*node_id).unwrap().get();
            let title_len = node.title.len();
            let title_width = node.title.width();
            layout.xo = (title_len - title_width) as f64;
        }
    }

    pub fn get_visible_nodes(&self, viewport: (f64, f64, f64, f64)) -> Vec<NodeId> {
        let (vp_left, vp_top, vp_right, vp_bottom) = viewport;

        self.nodes
            .iter()
            .filter_map(|(id, layout)| {
                if layout.x + layout.w >= vp_left
                    && layout.x <= vp_right
                    && layout.y + layout.lh >= vp_top
                    && layout.y <= vp_bottom
                {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_width = word.width();

        if current_width > 0 && current_width + 1 + word_width > max_width {
            // Need to start a new line
            lines.push(current_line);
            current_line = word.to_string();
            current_width = word_width;
        } else {
            // Add to current line
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
