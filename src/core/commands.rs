use super::map::{MapError, MindMap, Node, NodeId};

// --- Node Creation ---

pub enum InsertMode {
    Sibling,
    Child,
}

/// Inserts a new node relative to the current active node.
pub fn insert_new_node(map: &mut MindMap, mode: InsertMode) -> Result<NodeId, MapError> {
    let active_id = map.active_node_id;

    // Determine the parent for the new node
    let parent_id = match mode {
        InsertMode::Sibling => {
            // Cannot insert sibling for the root node this way
            if active_id == map.root_id {
                return Err(MapError::RootNodeOperation);
            }
            map.get_node(active_id)
                .and_then(|n| n.parent)
                .ok_or(MapError::ParentNotFound(active_id))?
        }
        InsertMode::Child => active_id,
    };

    // Ensure parent node exists and mark it as non-leaf / expand it
    let parent_node = map
        .get_node_mut(parent_id)
        .ok_or(MapError::NodeNotFound(parent_id))?;
    // parent_node.is_leaf = false; // Handled implicitly by having children
    parent_node.collapsed = false;

    // Create the new node
    let new_id = map.add_node("NEW".to_string(), parent_id)?;

    // If inserting as sibling, place it after the active node in the parent's children list
    if let InsertMode::Sibling = mode {
        if let Some(parent_node_again) = map.get_node_mut(parent_id) {
            if let Some(pos) = parent_node_again
                .children
                .iter()
                .position(|&id| id == active_id)
            {
                // Find the actual new_id that was added (should be last)
                if let Some(new_node_pos) = parent_node_again
                    .children
                    .iter()
                    .position(|&id| id == new_id)
                {
                    let node_to_move = parent_node_again.children.remove(new_node_pos);
                    parent_node_again.children.insert(pos + 1, node_to_move);
                }
            }
            // else: active node wasn't found in parent's children? inconsistency
        }
    }

    // Set the new node as active
    map.active_node_id = new_id;
    map.modified = true;
    Ok(new_id)
}

// --- Node Deletion ---

/// Deletes the active node (and its descendants).
/// Returns the ID of the node that becomes active after deletion (parent or sibling).
pub fn delete_active_node(map: &mut MindMap) -> Result<NodeId, MapError> {
    let node_id_to_delete = map.active_node_id;
    if node_id_to_delete == map.root_id {
        return Err(MapError::RootNodeOperation);
    }

    let (parent_id, next_active_id) = determine_next_active_after_delete(map, node_id_to_delete)?;

    // Perform the recursive deletion
    remove_node_recursive(map, node_id_to_delete)?;

    map.active_node_id = next_active_id;
    map.modified = true;
    Ok(next_active_id)
}

/// Deletes only the children of the active node.
pub fn delete_active_node_children(map: &mut MindMap) -> Result<(), MapError> {
    let active_id = map.active_node_id;

    // Clone children IDs to avoid borrowing issues while removing
    let children_to_delete = map
        .get_node(active_id)
        .map(|n| n.children.clone())
        .ok_or(MapError::NodeNotFound(active_id))?;

    for child_id in children_to_delete {
        remove_node_recursive(map, child_id)?;
    }

    // Mark parent as potentially leaf if all children removed
    // if let Some(node) = map.get_node_mut(active_id) {
    //      node.is_leaf = node.children.is_empty(); // Handled by remove_node_recursive updating parent
    // }
    map.modified = true;
    Ok(())
}

/// Determines which node should become active after deleting `deleted_node_id`.
/// Returns `(parent_id, next_active_id)`
fn determine_next_active_after_delete(
    map: &MindMap,
    deleted_node_id: NodeId,
) -> Result<(NodeId, NodeId), MapError> {
    let deleted_node = map
        .get_node(deleted_node_id)
        .ok_or(MapError::NodeNotFound(deleted_node_id))?;
    let parent_id = deleted_node
        .parent
        .ok_or(MapError::ParentNotFound(deleted_node_id))?; // Should have parent if not root
    let parent_node = map
        .get_node(parent_id)
        .ok_or(MapError::NodeNotFound(parent_id))?;

    // Find position of deleted node among siblings
    if let Some(index) = parent_node
        .children
        .iter()
        .position(|&id| id == deleted_node_id)
    {
        // Try to select the previous sibling
        if index > 0 {
            if let Some(prev_sibling_id) = parent_node.children.get(index - 1) {
                if map.nodes.contains_key(prev_sibling_id) {
                    // Check if it still exists
                    return Ok((parent_id, *prev_sibling_id));
                }
            }
        }
        // Try to select the next sibling
        if let Some(next_sibling_id) = parent_node.children.get(index + 1) {
            if map.nodes.contains_key(next_sibling_id) {
                // Check if it still exists
                return Ok((parent_id, *next_sibling_id));
            }
        }
    }
    // If no siblings left or something went wrong, default to parent
    Ok((parent_id, parent_id))
}

/// Helper function to recursively remove a node and its descendants from the map.
/// Also removes the node from its parent's children list.
fn remove_node_recursive(map: &mut MindMap, node_id: NodeId) -> Result<(), MapError> {
    // Clone children to avoid borrowing map mutably while iterating
    let children_ids = match map.get_node(node_id) {
        Some(n) => n.children.clone(),
        None => return Ok(()), // Node already removed or never existed
    };

    // Recursively remove children first
    for child_id in children_ids {
        remove_node_recursive(map, child_id)?;
    }

    // Now remove the node itself after children are gone
    if let Some(removed_node) = map.nodes.remove(&node_id) {
        // Remove from parent's children list
        if let Some(parent_id) = removed_node.parent {
            if let Some(parent_node) = map.nodes.get_mut(&parent_id) {
                parent_node.children.retain(|&id| id != node_id);
                // parent_node.is_leaf = parent_node.children.is_empty(); // Implicitly handled
            } // else: Parent already removed? Ok.
        }
    } // else: Node was already removed

    Ok(())
}
