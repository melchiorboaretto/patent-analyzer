pub mod leaf;
pub mod internal;

pub struct InternalNode<K, const M: usize> {
    pub keys: Vec<K>,
    pub children: Vec<usize>, // This is a vector of indexes, these index a vector with all the nodes
}

pub struct LeafNode<K, V, const M: usize> {
    pub content: Vec<(K, V)>,
    pub next: Option<usize>,
}

pub enum Node<K, V, const M: usize> {
    Internal(InternalNode<K, M>),
    Leaf(LeafNode<K, V, M>),
}

// Impl Node 
impl<K: Ord + Copy, V, const M: usize> Node<K, V, M> {
    pub fn new_leaf() -> Self {
        Node::Leaf(LeafNode::new(None))
    }

    pub fn new_internal() -> Self {
        Node::Internal(InternalNode::new())
    }

    pub fn should_split(&self) -> bool {
        match self {
            Node::Internal(int_node) => int_node.keys.len() == M,
            Node::Leaf(leaf_node) => leaf_node.content.len() == M,
        }
    }

}
