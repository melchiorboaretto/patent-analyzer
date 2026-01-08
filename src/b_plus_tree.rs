
struct InternalNode<K> {
    keys: Vec<K>,
    children: Vec<usize>, // This is a vector of indexes, these index a vector with all the nodes
}

struct LeafNode<K, V> {
    content: Vec<(K, V)>,
    next: Option<usize>,
}

enum Node<K, V> {
    Internal(InternalNode<K>),
    Leaf(LeafNode<K, V>),
}

struct BPlusTree< K: Ord + Copy, V> {
    root_idx: Option<usize>,
    nodes: Vec<Node<K, V>>,
    degree_t: usize,
}

// Impl Leaf
impl< K: Ord + Copy, V> LeafNode<K, V> {

    fn new(next: Option<usize>) -> Self {

        LeafNode {
            content: Vec::new(),
            next,
        }

    }

    // It will use binary search on a leaf to find the value binded to a key.
    fn retrieve_value(&self, key: K) -> Option<&V> {
        if let Ok(index) = self
            .content.binary_search_by_key(&key, |pair| pair.0) {

            Some(&self.content[index].1)

        } else {
            None
        }
    }



}



// Impl Node 
impl< K: Ord + Copy, V> Node<K, V> {


}

impl< K: Ord + Copy, V> BPlusTree<K, V> {

    fn new(t: usize) -> Self {
        BPlusTree {
            root_idx: None,
            nodes: Vec::new(),
            degree_t: t,
        }
    }

    // Done (Can, and will, be improved)
    fn create_first_root(&mut self, key: K, value: V) {
        self.nodes.push(Node::Leaf(LeafNode::new(None)));

        let root_idx = 0;
        self.root_idx = Some(root_idx);

        if let Node::Leaf(new_leaf) = &mut self.nodes[root_idx] {
            new_leaf.content.push((key, value));
        }
    }

    // How many nodes does the tree have
    // NOTE: tem que ver como faz pra ficar inline isso aqui
    // note2: descobri como faz, agora tem que gerar o assembly pra ver se funciona
    #[inline]
    fn size(&self) -> usize {
        self.nodes.len()
    }

    fn insert(&mut self, key: K, value: V) {

        // Considering the empty tree case. Putting the CPU's branch predictor to test
        // on every insertion hehe
        if self.root_idx.is_none() {
            self.create_first_root(key, value);
            return;
        }

        todo!()




    }

    // Returns the value removed (like a pop from stack instruction)
    fn remove(&mut self, key: K) -> Option<V> {
        todo!()


    }

    // Returns the old value or None (if the key was not found)
    fn update(&mut self, key: K, value: V) -> Option<V> {
        todo!()


    }




}



#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn insert_and_read() {

        todo!();

    }

    #[test]
    fn insert_and_scan() {

        todo!();

    }

    #[test]
    fn insert_update_scan() {

        todo!();

    }

}
