use crate::model::{Node, NodeId};
use anyhow::Result;
use indextree::Arena;
use std::fs;
use std::path::Path;

pub fn load_file(path: &Path) -> Result<(Arena<Node>, NodeId)> {
    let content = fs::read_to_string(path)?;
    parse_hmm_content(&content)
}

pub fn parse_hmm_content(content: &str) -> Result<(Arena<Node>, NodeId)> {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return create_empty_map();
    }

    // Calculate minimum indentation and clean up lines
    let mut min_indent = usize::MAX;
    let mut cleaned_lines = Vec::new();

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }

        let mut clean_line = line.to_string();

        // Replace bullet points with spaces
        clean_line = clean_line.replace("•", "*");
        clean_line = clean_line.replace('\t', "  ");

        // Calculate indentation
        let indent = clean_line.len() - clean_line.trim_start().len();
        let trimmed = clean_line.trim_start();

        // Handle list markers (* or -)
        if trimmed.starts_with("* ") || trimmed.starts_with("- ") {
            clean_line = format!("{}{}", " ".repeat(indent + 2), &trimmed[2..]);
        }

        if !clean_line.trim().is_empty() {
            let actual_indent = clean_line.len() - clean_line.trim_start().len();
            min_indent = min_indent.min(actual_indent);
            cleaned_lines.push(clean_line);
        }
    }

    if cleaned_lines.is_empty() {
        return create_empty_map();
    }

    // Build the tree
    let mut tree = Arena::new();

    // Create a synthetic root node
    let root_node = tree.new_node(Node::new("root".to_string()));

    let mut level_stack: Vec<(NodeId, usize)> = vec![(root_node, 0)];
    let mut first_level_nodes = Vec::new();

    for line in cleaned_lines {
        let indent = line.len() - line.trim_start().len() - min_indent;
        let title = line.trim().to_string();

        if title.is_empty() {
            continue;
        }

        // Find the appropriate parent based on indentation
        while level_stack.len() > 1 && level_stack.last().unwrap().1 >= indent {
            level_stack.pop();
        }

        let parent_id = level_stack.last().unwrap().0;
        let new_node = tree.new_node(Node::new(title));

        parent_id.append(new_node, &mut tree);

        // Track first-level nodes
        if parent_id == root_node {
            first_level_nodes.push(new_node);
        }

        // Add to stack for potential children
        level_stack.push((new_node, indent));
    }

    // If there's only one first-level node, use it as root
    // Otherwise, use the synthetic root
    let final_root = if first_level_nodes.len() == 1 {
        first_level_nodes[0]
    } else {
        root_node
    };

    Ok((tree, final_root))
}

fn create_empty_map() -> Result<(Arena<Node>, NodeId)> {
    let mut tree = Arena::new();
    let root = tree.new_node(Node::new("New Mind Map".to_string()));
    Ok((tree, root))
}

pub fn save_file(tree: &Arena<Node>, root_id: NodeId, path: &Path) -> Result<()> {
    let content = map_to_list(tree, root_id, false, 0);
    fs::write(path, content)?;
    Ok(())
}

pub fn map_to_list(
    tree: &Arena<Node>,
    node_id: NodeId,
    exclude_parent: bool,
    base_indent: usize,
) -> String {
    let mut result = String::new();

    if !exclude_parent {
        let node = tree.get(node_id).unwrap().get();
        result.push_str(&"\t".repeat(base_indent));
        result.push_str(&node.title);
        result.push('\n');
    }

    for child_id in node_id.children(tree) {
        let child_content = map_to_list(
            tree,
            child_id,
            false,
            base_indent + 1 - (exclude_parent as usize),
        );
        result.push_str(&child_content);
    }

    result
}
