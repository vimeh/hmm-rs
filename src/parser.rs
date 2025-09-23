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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_content() {
        let result = parse_hmm_content("").unwrap();
        let (tree, root_id) = result;

        assert_eq!(tree.count(), 1);
        assert_eq!(tree.get(root_id).unwrap().get().title, "New Mind Map");
    }

    #[test]
    fn test_parse_single_node() {
        let content = "Root Node";
        let (tree, root_id) = parse_hmm_content(content).unwrap();

        // Parser creates synthetic root but uses the single node as root
        assert_eq!(tree.count(), 2); // synthetic root + actual node
        assert_eq!(tree.get(root_id).unwrap().get().title, "Root Node");
    }

    #[test]
    fn test_parse_simple_tree() {
        let content = "Root\n\tChild 1\n\tChild 2\n\t\tGrandchild";
        let (tree, root_id) = parse_hmm_content(content).unwrap();

        // Parser creates synthetic root + 4 actual nodes
        assert_eq!(tree.count(), 5);
        assert_eq!(tree.get(root_id).unwrap().get().title, "Root");

        let children: Vec<_> = root_id.children(&tree).collect();
        assert_eq!(children.len(), 2);

        let child1 = children[0];
        assert_eq!(tree.get(child1).unwrap().get().title, "Child 1");

        let child2 = children[1];
        assert_eq!(tree.get(child2).unwrap().get().title, "Child 2");

        let grandchildren: Vec<_> = child2.children(&tree).collect();
        assert_eq!(grandchildren.len(), 1);
        assert_eq!(
            tree.get(grandchildren[0]).unwrap().get().title,
            "Grandchild"
        );
    }

    #[test]
    fn test_parse_with_bullets() {
        let content = "Root\n\t* Child with asterisk\n\t- Child with dash";
        let (tree, root_id) = parse_hmm_content(content).unwrap();

        // Parser creates synthetic root + 3 actual nodes
        assert_eq!(tree.count(), 4);

        let children: Vec<_> = root_id.children(&tree).collect();
        assert_eq!(children.len(), 2);
        assert_eq!(
            tree.get(children[0]).unwrap().get().title,
            "Child with asterisk"
        );
        assert_eq!(
            tree.get(children[1]).unwrap().get().title,
            "Child with dash"
        );
    }

    #[test]
    fn test_parse_with_spaces_indentation() {
        let content = "Root\n  Child 1\n    Grandchild\n  Child 2";
        let (tree, root_id) = parse_hmm_content(content).unwrap();

        // Parser creates synthetic root + 4 actual nodes
        assert_eq!(tree.count(), 5);
        assert_eq!(tree.get(root_id).unwrap().get().title, "Root");
    }

    #[test]
    fn test_parse_multiple_roots() {
        let content = "Root 1\nRoot 2\n\tChild of Root 2";
        let (tree, root_id) = parse_hmm_content(content).unwrap();

        // Should create a synthetic root
        assert_eq!(tree.get(root_id).unwrap().get().title, "root");

        let roots: Vec<_> = root_id.children(&tree).collect();
        assert_eq!(roots.len(), 2);
        assert_eq!(tree.get(roots[0]).unwrap().get().title, "Root 1");
        assert_eq!(tree.get(roots[1]).unwrap().get().title, "Root 2");
    }

    #[test]
    fn test_round_trip() {
        let original = "Root\n\tChild 1\n\t\tGrandchild 1\n\tChild 2\n\t\tGrandchild 2";
        let (tree, root_id) = parse_hmm_content(original).unwrap();

        let exported = map_to_list(&tree, root_id, false, 0);
        let (tree2, root_id2) = parse_hmm_content(&exported).unwrap();

        // Compare tree structures
        assert_eq!(tree.count(), tree2.count());
        assert_eq!(
            tree.get(root_id).unwrap().get().title,
            tree2.get(root_id2).unwrap().get().title
        );
    }

    #[test]
    fn test_parse_with_empty_lines() {
        let content = "Root\n\n\tChild 1\n\n\n\tChild 2";
        let (tree, root_id) = parse_hmm_content(content).unwrap();

        // Parser creates synthetic root + 3 actual nodes
        assert_eq!(tree.count(), 4);
        let children: Vec<_> = root_id.children(&tree).collect();
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_parse_with_unicode() {
        let content = "Root ✓\n\t子节点 🎯\n\t✗ Failed node";
        let (tree, root_id) = parse_hmm_content(content).unwrap();

        // Parser creates synthetic root + 3 actual nodes
        assert_eq!(tree.count(), 4);
        assert_eq!(tree.get(root_id).unwrap().get().title, "Root ✓");

        let children: Vec<_> = root_id.children(&tree).collect();
        assert_eq!(tree.get(children[0]).unwrap().get().title, "子节点 🎯");
        assert_eq!(tree.get(children[1]).unwrap().get().title, "✗ Failed node");
    }

    #[test]
    fn test_save_file_creates_correct_format() {
        use tempfile::NamedTempFile;

        let mut tree = Arena::new();
        let root = tree.new_node(Node::new("Root".to_string()));
        let child1 = tree.new_node(Node::new("Child 1".to_string()));
        let child2 = tree.new_node(Node::new("Child 2".to_string()));

        root.append(child1, &mut tree);
        root.append(child2, &mut tree);

        let temp_file = NamedTempFile::new().unwrap();
        save_file(&tree, root, temp_file.path()).unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(content, "Root\n\tChild 1\n\tChild 2\n");
    }
}
