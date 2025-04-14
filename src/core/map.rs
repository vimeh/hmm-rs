use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error; // Placeholder for potential errors later

// Define a specific type for node IDs for clarity and type safety.
pub type NodeId = usize;

// Represents a single node in the mind map.
#[derive(Debug, Clone)] // Derive Debug for easy printing and Clone for undo history.
pub struct Node {
    pub id: NodeId,
    pub title: String,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub collapsed: bool,
    pub hidden: bool,
    pub rank_pos: u32,
    pub rank_neg: u32,
    pub stars: u8,
    pub symbol: Option<String>, // For '✓', '✗', etc.
}

impl Node {
    // Helper constructor for new nodes.
    pub fn new(id: NodeId, title: String, parent: Option<NodeId>) -> Self {
        Node {
            id,
            title,
            parent,
            children: Vec::new(),
            collapsed: false,
            hidden: false,
            rank_pos: 0,
            rank_neg: 0,
            stars: 0,
            symbol: None,
        }
    }
}

// Represents the entire mind map state.
#[derive(Debug, Clone)]
pub struct MindMap {
    pub nodes: HashMap<NodeId, Node>,
    pub root_id: NodeId,
    pub active_node_id: NodeId,
    pub next_id: NodeId, // To generate unique IDs for new nodes.
    pub filename: Option<PathBuf>,
    pub modified: bool,
}

// Define potential errors related to map operations.
#[derive(Error, Debug)]
pub enum MapError {
    #[error("Node with ID {0} not found")]
    NodeNotFound(NodeId),
    #[error("Attempted to operate on root node inappropriately")]
    RootNodeOperation,
    #[error("Cannot determine parent for node {0}")]
    ParentNotFound(NodeId),
}

impl MindMap {
    /// Creates a new, empty MindMap with a default root node.
    pub fn new() -> Self {
        let root_id = 0; // Conventionally start root at 0 or 1
        let mut nodes = HashMap::new();
        let root_node = Node::new(root_id, "root".to_string(), None);
        nodes.insert(root_id, root_node);

        MindMap {
            nodes,
            root_id,
            active_node_id: root_id,
            next_id: root_id + 1, // Start assigning IDs from 1
            filename: None,
            modified: false,
        }
    }

    /// Generates the next available NodeId.
    fn get_next_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Adds a new node with the given title as a child of the parent_id.
    /// Returns the ID of the newly created node.
    pub fn add_node(&mut self, title: String, parent_id: NodeId) -> Result<NodeId, MapError> {
        // Ensure parent exists
        if !self.nodes.contains_key(&parent_id) {
            return Err(MapError::NodeNotFound(parent_id));
        }

        let new_id = self.get_next_id();
        let new_node = Node::new(new_id, title, Some(parent_id));

        // Insert the new node
        self.nodes.insert(new_id, new_node);

        // Update the parent's children list
        if let Some(parent_node) = self.nodes.get_mut(&parent_id) {
            parent_node.children.push(new_id);
        } else {
            // This should ideally not happen due to the check above,
            // but robust handling is good. Clean up the orphaned node.
            self.nodes.remove(&new_id);
            return Err(MapError::NodeNotFound(parent_id)); // Re-signal error
        }

        self.modified = true;
        Ok(new_id)
    }

    /// Gets an immutable reference to a node by its ID.
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Gets a mutable reference to a node by its ID.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    // /// Removes a node and all its descendants recursively.
    // /// Returns an error if the node_id doesn't exist.
    // pub fn remove_node_recursive(&mut self, node_id: NodeId) -> Result<(), MapError> {
    //     if node_id == self.root_id {
    //         return Err(MapError::RootNodeOperation); // Cannot remove the root this way
    //     }

    //     let node_to_remove = self.nodes.get(&node_id).cloned(); // Clone to avoid borrow issues

    //     if let Some(node) = node_to_remove {
    //         // Recursively remove children first
    //         for child_id in node.children.clone() { // Clone children list for iteration
    //             self.remove_node_recursive(child_id)?;
    //         }

    //         // Remove from parent's children list
    //         if let Some(parent_id) = node.parent {
    //             if let Some(parent_node) = self.nodes.get_mut(&parent_id) {
    //                 parent_node.children.retain(|&id| id != node_id);
    //             } else {
    //                 // Parent doesn't exist? Data inconsistency.
    //                 // Depending on strictness, could error or just warn.
    //                 // For now, let's proceed with removal.
    //                 eprintln!("Warning: Parent node {} not found for node {}", parent_id, node_id);
    //             }
    //         } else if node_id != self.root_id {
    //              // Node other than root has no parent? Inconsistency.
    //              eprintln!("Warning: Node {} has no parent but is not root.", node_id);
    //         }

    //         // Finally, remove the node itself
    //         self.nodes.remove(&node_id);

    //         // If the removed node was the active one, try to move active status
    //         if self.active_node_id == node_id {
    //            self.active_node_id = node.parent.unwrap_or(self.root_id); // Move to parent or root
    //         }

    //         self.modified = true;
    //         Ok(())
    //     } else {
    //         Err(MapError::NodeNotFound(node_id))
    //     }
    // }
}

// Basic tests for MindMap functionality
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_map() {
        let map = MindMap::new();
        assert_eq!(map.nodes.len(), 1);
        assert!(map.nodes.contains_key(&map.root_id));
        assert_eq!(map.active_node_id, map.root_id);
        assert_eq!(map.next_id, map.root_id + 1);
        assert_eq!(map.nodes[&map.root_id].title, "root");
    }

    #[test]
    fn test_add_node() {
        let mut map = MindMap::new();
        let root_id = map.root_id;
        let child1_id = map.add_node("Child 1".to_string(), root_id).unwrap();

        assert_eq!(map.nodes.len(), 2);
        assert!(map.nodes.contains_key(&child1_id));
        assert_eq!(map.nodes[&child1_id].title, "Child 1");
        assert_eq!(map.nodes[&child1_id].parent, Some(root_id));
        assert!(map.nodes[&root_id].children.contains(&child1_id));
        assert!(map.modified);

        let grandchild1_id = map.add_node("Grandchild 1".to_string(), child1_id).unwrap();
        assert_eq!(map.nodes.len(), 3);
        assert_eq!(map.nodes[&grandchild1_id].parent, Some(child1_id));
        assert!(map.nodes[&child1_id].children.contains(&grandchild1_id));
    }

    #[test]
    fn test_add_node_invalid_parent() {
        let mut map = MindMap::new();
        let invalid_parent_id = 999;
        let result = map.add_node("Orphan?".to_string(), invalid_parent_id);
        assert!(matches!(result, Err(MapError::NodeNotFound(id)) if id == invalid_parent_id));
        assert_eq!(map.nodes.len(), 1); // No node should have been added
        assert!(!map.modified);
    }

    #[test]
    fn test_get_node() {
        let mut map = MindMap::new();
        let root_id = map.root_id;
        let child1_id = map.add_node("Child 1".to_string(), root_id).unwrap();

        assert_eq!(map.get_node(root_id).unwrap().id, root_id);
        assert_eq!(map.get_node(child1_id).unwrap().title, "Child 1");
        assert!(map.get_node(999).is_none());
    }

    #[test]
    fn test_get_node_mut() {
        let mut map = MindMap::new();
        let root_id = map.root_id;
        let child1_id = map.add_node("Child 1".to_string(), root_id).unwrap();

        {
            let node = map.get_node_mut(child1_id).unwrap();
            node.title = "Updated Child 1".to_string();
        }

        assert_eq!(map.get_node(child1_id).unwrap().title, "Updated Child 1");
    }

    // TODO: Add tests for remove_node_recursive once implemented and stable
}

// Default implementation for MindMap
impl Default for MindMap {
    fn default() -> Self {
        Self::new()
    }
}
