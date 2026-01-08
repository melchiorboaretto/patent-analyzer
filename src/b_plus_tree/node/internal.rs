use super::InternalNode;

// Impl Internal
impl<K: Ord + Copy, const M: usize> InternalNode<K, M> {

    pub fn new() -> Self {

        InternalNode {
            keys: Vec::with_capacity(M),
            children: Vec::with_capacity(M), // This is a vector of indexes, these index a vector with all the nodes
        }

    }

    // Find the index of the next child for a key using binary search over the current node
    //
    // IMPORTANT: 
    // Assuming the tree follow the LOWER left and GREATER OR EQUAL right convention.
    pub fn find_child_index(&self, key: K) -> usize {

        let pre_index = self.keys.binary_search(&key);

        // This match uses the singular implementation of the binary_search
        // method to figure where the child node is
        let index = match pre_index {
            Ok(index_minus_one) => {
                index_minus_one + 1
            },
            Err(right_index) => {
                right_index
            }
        };

        self.children[index]
    }

}
