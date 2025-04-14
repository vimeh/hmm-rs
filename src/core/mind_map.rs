use std::collections::{HashMap, VecDeque};

use super::node::{Node, NodeId};
use crate::errors::{AppError, AppResult};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MindMap {
    pub nodes: HashMap<NodeId, Node>,
    pub root: Option<NodeId>,
}

impl MindMap {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root: None,
        }
    }

    /// Adds a new node to the map.
    /// If parent_id is Some, adds the new node as a child of the specified parent.
    /// If parent_id is None, sets the new node as the root (if no root exists).
    /// Returns the ID of the newly added node.
    pub fn add_node(&mut self, text: String, parent_id: Option<NodeId>) -> AppResult<NodeId> {
        let new_node = Node::new(text);
        let new_id = new_node.id;

        match parent_id {
            Some(pid) => {
                // Find the parent node and add the new node as a child
                let parent_node = self
                    .nodes
                    .get_mut(&pid)
                    .ok_or(AppError::ParentNodeNotFound(pid))?;
                parent_node.children.push(new_id);
            }
            None => {
                // If no parent specified, this becomes the root node
                // Only allow setting the root if it doesn't exist yet
                if self.root.is_some() {
                    // Or handle this case differently, e.g., return an error or add as a detached node
                    // For now, let's implicitly add it without making it root if one exists.
                    // Consider adding an explicit error or alternative behavior later.
                } else {
                    self.root = Some(new_id);
                }
            }
        }

        self.nodes.insert(new_id, new_node);
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

    /// Edits the text of an existing node.
    pub fn edit_node(&mut self, id: NodeId, new_text: String) -> AppResult<()> {
        let node = self.nodes.get_mut(&id).ok_or(AppError::NodeNotFound(id))?;
        node.text = new_text;
        Ok(())
    }

    /// Deletes a node and all its descendants recursively.
    /// Returns an error if the node is not found or if trying to delete the root node.
    pub fn delete_node(&mut self, id: NodeId) -> AppResult<()> {
        // Prevent deleting the root node directly
        if self.root == Some(id) {
            return Err(AppError::CannotDeleteRoot);
        }

        // Check if the node exists before proceeding
        if !self.nodes.contains_key(&id) {
            return Err(AppError::NodeNotFound(id));
        }

        // Find the parent of the node to remove it from its children list
        let mut parent_id: Option<NodeId> = None;
        for (p_id, node) in self.nodes.iter() {
            if node.children.contains(&id) {
                parent_id = Some(*p_id);
                break;
            }
        }

        if let Some(pid) = parent_id {
            if let Some(parent_node) = self.nodes.get_mut(&pid) {
                parent_node.children.retain(|&child_id| child_id != id);
            }
        }

        // Collect all descendant IDs (including the starting node) using BFS
        let mut nodes_to_delete = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(id);

        while let Some(current_id) = queue.pop_front() {
            if let Some(node) = self.nodes.get(&current_id) {
                nodes_to_delete.push(current_id);
                for &child_id in &node.children {
                    queue.push_back(child_id);
                }
            }
        }

        // Remove all collected nodes from the map
        for node_id in nodes_to_delete {
            self.nodes.remove(&node_id);
        }

        Ok(())
    }
}
