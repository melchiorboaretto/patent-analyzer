use super::LeafNode;

// Impl Leaf
impl<K: Ord + Copy, V, const M: usize> LeafNode<K, V, M> {

    pub fn new(next: Option<usize>) -> Self {

        LeafNode {
            content: Vec::with_capacity(M),
            next,
        }

    }

    // It will use binary search on a leaf to find the value binded to a key.
    pub fn retrieve_value(&self, key: K) -> Option<&V> {
        if let Ok(index) = self
            .content.binary_search_by_key(&key, |pair| pair.0) {

            Some(&self.content[index].1)

        } else {
            None
        }
    }

    pub fn update_value(&mut self, key: K, value: V) -> Option<V> {

        if let Ok(index) = self
            .content.binary_search_by_key(&key, |pair| pair.0) {

            Some(std::mem::replace(&mut self.content[index].1, value))

        } else {
            None
        }
    }



}
