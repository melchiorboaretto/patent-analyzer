use crate::b_plus_tree::Node;

use std::ops::Bound;

pub struct BPlusTreeIter<'a, K: Copy + Ord, V, const M: usize> {

    nodes: &'a Vec<Node<K, V, M>>,
    cursor_index: Option<usize>,
    curr_index: usize, // This is the intra-leaf index

    // Range control
    upper_bound: std::ops::Bound<K>,
}

impl<'a, K: Copy + Ord, V, const M: usize> BPlusTreeIter<'a, K, V, M> {

    pub fn new(nodes: &'a Vec<Node<K, V, M>>, cursor_index: Option<usize>, curr_index: usize, upper_bound: Bound<K>) -> Self {

        BPlusTreeIter {
            nodes,
            cursor_index,
            curr_index,
            upper_bound,
        }
    }
}

impl<'a, K: Copy + Ord, V, const M: usize> Iterator for BPlusTreeIter<'a, K, V, M> {
    type Item = &'a (K, V);

    fn next(&mut self) -> Option<Self::Item> {

        let to_return = loop {
            if let Some(node_index) = self.cursor_index { // If None, it is the end

                if let Node::Leaf(leaf_ref) = &self.nodes[node_index] {

                    // Test if the leaf content is over or not
                    if let Some(key_value_ref) = leaf_ref.content.get(self.curr_index) {
                        break Some(key_value_ref);
                    } else {

                        self.curr_index = 0;
                        self.cursor_index = leaf_ref.next;
                    }

                } else {
                    unsafe {
                        std::hint::unreachable_unchecked();
                    }
                }


            } else {
                break None;
            }

        };

        self.curr_index += 1;

        if let Some(return_value) = to_return {

            let key = return_value.0;

            match self.upper_bound {
                Bound::Excluded(upp_bound) => {
                    if key >= upp_bound {
                        self.cursor_index = None;
                        None

                    } else {
                        to_return
                    }
                },

                Bound::Included(upp_bound) => {
                    if key > upp_bound {
                        self.cursor_index = None;
                        None

                    } else {
                        to_return
                    }
                },

                Bound::Unbounded => to_return,
            }

        } else {
            None
        }

    }

}
