use hmm_rs::{model::Node, NodeId};
use indextree::Arena;
use std::collections::HashSet;

/// Compare two trees for structural equality
pub fn trees_are_equal(
    tree1: &Arena<Node>,
    root1: NodeId,
    tree2: &Arena<Node>,
    root2: NodeId,
) -> bool {
    fn compare_nodes(
        tree1: &Arena<Node>,
        node1: NodeId,
        tree2: &Arena<Node>,
        node2: NodeId,
    ) -> bool {
        // Compare node titles
        let title1 = &tree1.get(node1).unwrap().get().title;
        let title2 = &tree2.get(node2).unwrap().get().title;

        if title1 != title2 {
            return false;
        }

        // Compare children
        let children1: Vec<_> = node1.children(tree1).collect();
        let children2: Vec<_> = node2.children(tree2).collect();

        if children1.len() != children2.len() {
            return false;
        }

        // Recursively compare each child
        for (child1, child2) in children1.iter().zip(children2.iter()) {
            if !compare_nodes(tree1, *child1, tree2, *child2) {
                return false;
            }
        }

        true
    }

    compare_nodes(tree1, root1, tree2, root2)
}

/// Generate a tree structure string for debugging
#[allow(dead_code)]
pub fn tree_to_string(tree: &Arena<Node>, root: NodeId) -> String {
    fn build_string(tree: &Arena<Node>, node: NodeId, depth: usize, result: &mut String) {
        let indent = "\t".repeat(depth);
        let title = &tree.get(node).unwrap().get().title;
        result.push_str(&format!("{}{}\n", indent, title));

        for child in node.children(tree) {
            build_string(tree, child, depth + 1, result);
        }
    }

    let mut result = String::new();
    build_string(tree, root, 0, &mut result);
    result
}

/// Count total nodes in tree (excluding removed)
#[allow(dead_code)]
pub fn count_nodes(tree: &Arena<Node>) -> usize {
    tree.iter().filter(|n| !n.is_removed()).count()
}

/// Get all node titles as a set for comparison
#[allow(dead_code)]
pub fn get_all_titles(tree: &Arena<Node>) -> HashSet<String> {
    tree.iter()
        .filter(|n| !n.is_removed())
        .map(|n| n.get().title.clone())
        .collect()
}

/// Find a node by title
pub fn find_node_by_title(tree: &Arena<Node>, title: &str) -> Option<NodeId> {
    tree.iter()
        .filter(|n| !n.is_removed() && n.get().title == title)
        .map(|n| tree.get_node_id(n).unwrap())
        .next()
}

/// Calculate the maximum depth of the tree
pub fn calculate_tree_depth(tree: &Arena<Node>, root: NodeId) -> usize {
    fn calc_depth(tree: &Arena<Node>, node: NodeId) -> usize {
        let children: Vec<_> = node.children(tree).collect();
        if children.is_empty() {
            0
        } else {
            1 + children
                .iter()
                .map(|&child| calc_depth(tree, child))
                .max()
                .unwrap_or(0)
        }
    }

    calc_depth(tree, root)
}

/// Count nodes at a specific depth level
pub fn count_at_depth(tree: &Arena<Node>, root: NodeId, target_depth: usize) -> usize {
    fn count_recursive(
        tree: &Arena<Node>,
        node: NodeId,
        current_depth: usize,
        target_depth: usize,
    ) -> usize {
        if current_depth == target_depth {
            return 1;
        }

        if current_depth > target_depth {
            return 0;
        }

        node.children(tree)
            .map(|child| count_recursive(tree, child, current_depth + 1, target_depth))
            .sum()
    }

    count_recursive(tree, root, 0, target_depth)
}

/// Verify parent-child relationships are consistent
pub fn verify_tree_integrity(tree: &Arena<Node>) -> Result<(), String> {
    for node_ref in tree.iter() {
        if node_ref.is_removed() {
            continue;
        }

        let node_id = tree.get_node_id(node_ref).unwrap();

        // Check that all children have this node as parent
        for child_id in node_id.children(tree) {
            if let Some(child) = tree.get(child_id) {
                if let Some(parent) = child.parent() {
                    if parent != node_id {
                        return Err(format!(
                            "Inconsistent parent-child relationship: {} -> {}",
                            tree.get(node_id).unwrap().get().title,
                            child.get().title
                        ));
                    }
                } else {
                    return Err(format!("Child {} has no parent", child.get().title));
                }
            }
        }
    }
    Ok(())
}

/// Create a simple test tree
pub fn create_test_tree() -> (Arena<Node>, NodeId) {
    let mut tree = Arena::new();

    let root = tree.new_node(Node::new("Root".to_string()));
    let child1 = tree.new_node(Node::new("Child 1".to_string()));
    let child2 = tree.new_node(Node::new("Child 2".to_string()));
    let grandchild = tree.new_node(Node::new("Grandchild".to_string()));

    root.append(child1, &mut tree);
    root.append(child2, &mut tree);
    child2.append(grandchild, &mut tree);

    (tree, root)
}

/// Create a tree with specific depth
pub fn create_tree_with_depth(depth: usize) -> (Arena<Node>, NodeId) {
    let mut tree = Arena::new();
    let root = tree.new_node(Node::new("Level 0".to_string()));

    if depth > 0 {
        let mut current = root;
        for i in 1..=depth {
            let child = tree.new_node(Node::new(format!("Level {}", i)));
            current.append(child, &mut tree);
            current = child;
        }
    }

    (tree, root)
}

/// Create a tree with specific breadth at root level
#[allow(dead_code)]
pub fn create_tree_with_breadth(breadth: usize) -> (Arena<Node>, NodeId) {
    let mut tree = Arena::new();
    let root = tree.new_node(Node::new("Root".to_string()));

    for i in 1..=breadth {
        let child = tree.new_node(Node::new(format!("Child {}", i)));
        root.append(child, &mut tree);
    }

    (tree, root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trees_are_equal() {
        let (tree1, root1) = create_test_tree();
        let (tree2, root2) = create_test_tree();

        assert!(trees_are_equal(&tree1, root1, &tree2, root2));

        // Modify tree2
        let (mut tree2_modified, root2_modified) = create_test_tree();
        let extra = tree2_modified.new_node(Node::new("Extra".to_string()));
        root2_modified.append(extra, &mut tree2_modified);

        assert!(!trees_are_equal(
            &tree1,
            root1,
            &tree2_modified,
            root2_modified
        ));
    }

    #[test]
    fn test_tree_depth() {
        let (tree, root) = create_tree_with_depth(5);
        assert_eq!(calculate_tree_depth(&tree, root), 5);

        let (tree, root) = create_test_tree();
        assert_eq!(calculate_tree_depth(&tree, root), 2); // Root -> Child2 -> Grandchild
    }

    #[test]
    fn test_count_at_depth() {
        let (tree, root) = create_test_tree();

        assert_eq!(count_at_depth(&tree, root, 0), 1); // Just root
        assert_eq!(count_at_depth(&tree, root, 1), 2); // Child1, Child2
        assert_eq!(count_at_depth(&tree, root, 2), 1); // Grandchild
        assert_eq!(count_at_depth(&tree, root, 3), 0); // Nothing at depth 3
    }

    #[test]
    fn test_find_node_by_title() {
        let (tree, _root) = create_test_tree();

        assert!(find_node_by_title(&tree, "Root").is_some());
        assert!(find_node_by_title(&tree, "Child 1").is_some());
        assert!(find_node_by_title(&tree, "Grandchild").is_some());
        assert!(find_node_by_title(&tree, "NonExistent").is_none());
    }

    #[test]
    fn test_tree_integrity() {
        let (tree, _root) = create_test_tree();
        assert!(verify_tree_integrity(&tree).is_ok());
    }
}
