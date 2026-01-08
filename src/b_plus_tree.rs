mod node;
mod utils;
mod iter;

use node::{
    InternalNode,
    LeafNode,
    Node,
};

use crate::b_plus_tree::iter::BPlusTreeIter;

pub struct BPlusTree< K: Ord + Copy, V, const M: usize> {
    root_idx: usize,
    nodes: Vec<Node<K, V, M>>,
}

impl< K: Ord + Copy, V, const M: usize> BPlusTree<K, V, M> {

    pub fn new() -> Self {

        BPlusTree {
            root_idx: 0,
            nodes: vec![Node::new_leaf()],
        }
    }

    // How many nodes does the tree have
    #[inline]
    pub fn size(&self) -> usize {
        self.nodes.len()
    }

    fn degree_t(&self) -> usize {
        M.div_ceil(2)
    }

    fn degree_m(&self) -> usize {
        M
    }

    // Gets a reference to the root node (or None)
    fn root(&self) -> &Node<K, V, M> {
        &self.nodes[self.root_idx]
    }

    pub fn find_first_leaf_index(&self) -> usize {
        let mut curr_node = self.root();
        let mut curr_index = self.root_idx;

        while let Node::Internal(int_node_ref) = curr_node {
            curr_index = int_node_ref.children[0];
            curr_node = &self.nodes[curr_index];
        }

        curr_index
    }

    pub fn find_leaf_index(&self, key: K) -> usize {
        let mut curr_node = self.root();
        let mut curr_index = self.root_idx;

        while let Node::Internal(int_node_ref) = curr_node {
            curr_index = int_node_ref.find_child_index(key);
            curr_node = &self.nodes[curr_index];
        }

        curr_index
    }

    // Returns a reference to the value corresponding to the key.
    // May be None if the key cannot be find.
    pub fn get(&self, key: K) -> Option<&V> {

        let leaf_index = self.find_leaf_index(key);

        if let Node::Leaf(leaf_node) = &self.nodes[leaf_index] {
            leaf_node.retrieve_value(key)
        } else {
            None
        }


    }

    pub fn insert(&mut self, key: K, value: V) {

        // NOTE: ANTES DA FUNCAO de remove funcionar eu vou simplesmente usar
        // push no vetor de nodos em caso de split, DEPOIS eu vou fazer pra usar o primeiro num banco
        // de posicoes removidas ou nao utilizadas


        // REMEMBERING AGAIN: The return of the function is the only way to build an "upper" level
        // or, in other words, a new root.

        if let Some(root_key_index) = self.insert_aux(key, value, self.root_idx) {

            let mut new_root: InternalNode<K, M> = InternalNode::new();
            let left_index = self.root_idx;
            let right_index = root_key_index.1;

            let last_index = self.size();

            new_root.keys.push(root_key_index.0);
            new_root.children.push(left_index);
            new_root.children.push(right_index);

            self.root_idx = last_index;

            self.nodes.push(Node::Internal(new_root));
        }

    }

    // Returns the value removed (like a pop from stack instruction)
    pub fn remove(&mut self, key: K) -> Option<V> {
        todo!()


    }

    // Returns the old value or None (if the key was not found)
    pub fn update(&mut self, key: K, value: V) -> Option<V> {

        // Starts over the root index
        let mut curr_index = self.root_idx;

        // Reaches the leaf
        while let Node::Internal(int_node) = &self.nodes[curr_index] {
            curr_index = int_node.find_child_index(key);
        }

        // Changes the value
        if let Node::Leaf(leaf_node) = &mut self.nodes[curr_index] {
            leaf_node.update_value(key, value)
        } else {
            None
        }

    }


    pub fn range<'a, R: std::ops::RangeBounds<K>>(&'a self, range: R) -> BPlusTreeIter<'a, K, V, M> {

        let cursor_index;
        let curr_index;

        match range.start_bound() {
            std::ops::Bound::Excluded(ex_key) => {
                let leaf_idx = self.find_leaf_index(*ex_key);
                cursor_index = Some(leaf_idx);

                if let Node::Leaf(leaf_ref) = &self.nodes[leaf_idx] {
                    curr_index = match leaf_ref.content.binary_search_by_key(ex_key, |pair| pair.0) {
                        Ok(ok_idx) => ok_idx + 1, // The exact idx must be excluded
                        Err(err_idx) => err_idx,
                    }
                } else {
                    unsafe {
                        std::hint::unreachable_unchecked();
                    }
                }
            },
            std::ops::Bound::Included(in_key) => {
                let leaf_idx = self.find_leaf_index(*in_key);
                cursor_index = Some(leaf_idx);

                if let Node::Leaf(leaf_ref) = &self.nodes[leaf_idx] {
                    curr_index = match leaf_ref.content.binary_search_by_key(in_key, |pair| pair.0) {
                        //  key = 2 [1, 2, 3, 4, 5, 6] -> index = 1 (o exato)
                        //  key = 2 [1, 3, 5, 7] -> index = 1 (o exato pra comecar a busca tambem)
                        Ok(ok_idx) => ok_idx,
                        Err(err_idx) => err_idx,
                    }
                } else {
                    unsafe {
                        std::hint::unreachable_unchecked();
                    }
                }


            },

            std::ops::Bound::Unbounded => {
                let leaf_idx = self.find_first_leaf_index();
                cursor_index = Some(leaf_idx);

                curr_index = 0;

            },
        };

        let upper_bound =  range.end_bound().cloned();

        BPlusTreeIter::new (
            &self.nodes,
            cursor_index,
            curr_index,
            upper_bound,
        )


    }

}

impl<'a, K: Copy + Ord, V, const M: usize> IntoIterator for &'a BPlusTree<K, V, M> {
    type Item = &'a (K, V);
    type IntoIter = BPlusTreeIter<'a, K, V, M>;

    fn into_iter(self) -> Self::IntoIter {
        self.range(..)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn insert_and_read() {


    }

    #[test]
    fn insert_and_scan() {


    }

    #[test]
    fn insert_update_scan() {

        let mut tree = BPlusTree::<_, _, 2>::new();

        tree.insert(4, String::from("4 value"));
        tree.insert(6, String::from("6 value"));
        tree.insert(8, String::from("8 value"));
        tree.insert(10, String::from("10 value"));
        tree.insert(12, String::from("12 value"));
        tree.insert(14, String::from("14 value"));

        tree.insert(49, String::from("to update"));

        assert_eq!(tree.update(49, String::from("49 value")), Some(String::from("to update")));

        assert_eq!(tree.get(49), Some(String::from("49 value")).as_ref());

        for pair in &tree {

            let test = format!("{} value", pair.0);
            assert_eq!(pair.1, test);

        }

    }
}
