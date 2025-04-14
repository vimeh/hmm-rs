use uuid::Uuid;

pub type NodeId = Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub text: String,
    pub children: Vec<NodeId>,
}

impl Node {
    pub fn new(text: String) -> Self {
        Self {
            id: NodeId::new_v4(),
            text,
            children: Vec::new(),
        }
    }
}
