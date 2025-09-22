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
