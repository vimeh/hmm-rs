use indextree::NodeId as TreeNodeId;

pub type NodeId = TreeNodeId;

#[derive(Debug, Clone)]
pub struct Node {
    pub title: String,
    pub is_collapsed: bool,
    pub is_hidden: bool,
}

impl Node {
    pub fn new(title: String) -> Self {
        Self {
            title,
            is_collapsed: false,
            is_hidden: false,
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.is_hidden || self.title.starts_with("[HIDDEN] ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new("Test Node".to_string());
        assert_eq!(node.title, "Test Node");
        assert!(!node.is_collapsed);
        assert!(!node.is_hidden);
    }

    #[test]
    fn test_is_hidden_with_flag() {
        let mut node = Node::new("Normal Node".to_string());
        assert!(!node.is_hidden());

        node.is_hidden = true;
        assert!(node.is_hidden());
    }

    #[test]
    fn test_is_hidden_with_title_prefix() {
        let node = Node::new("[HIDDEN] Secret Node".to_string());
        assert!(node.is_hidden());
    }

    #[test]
    fn test_is_hidden_with_both() {
        let mut node = Node::new("[HIDDEN] Secret".to_string());
        node.is_hidden = true;
        assert!(node.is_hidden());
    }

    #[test]
    fn test_node_with_unicode_title() {
        let node = Node::new("✓ Task Complete 🎯".to_string());
        assert_eq!(node.title, "✓ Task Complete 🎯");
    }
}
