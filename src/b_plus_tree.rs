
struct InternalNode<K> {
    keys: Vec<K>,
    children: Vec<usize>,
}

struct LeafNode<K, V> {
    content: Vec<(K, V)>,
    next: Option<usize>,
}

enum Node<K, V> {
    Internal(InternalNode<K>),
    Leaf(LeafNode<K, V>),
}

struct BPlusTree<K: PartialOrd, V> {
    root: Option<Node<K, V>>,
    leaves: Vec<LeafNode<K, V>>,
    size: usize,
    degree_t: usize,
}

// Impl Leaf
impl<K: PartialOrd, V> LeafNode<K, V> {




}



// Impl Node 
impl<K: PartialOrd, V> Node<K, V> {


}

impl<K: PartialOrd, V> BPlusTree<K, V> {

    fn new(t: usize) -> Self {
        BPlusTree {
            root: None,
            leaves: Vec::new(),
            size: 0,
            degree_t: t,
        }
    }

    fn insert(&mut self, key: K, value: V) {


    }


}
