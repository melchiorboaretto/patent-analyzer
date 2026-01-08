use super::*;

impl< K: Ord + Copy, V, const M: usize> BPlusTree<K, V, M> {

    pub fn insert_aux(&mut self, key: K, value: V, index: usize) -> Option<(K, usize)> {


        let maybe_internal = match &self.nodes[index] {

            Node::Internal(int_ref ) => Some(int_ref.find_child_index(key)),
            Node::Leaf(_) => None,

        };

        if let Some(int_index) = maybe_internal {
            let prom_key_index = self.insert_aux(key, value, int_index);

            if let Some(key_index) = prom_key_index {

                // Here I insert on the internal node
                if let Node::Internal(int_ref) = &mut self.nodes[index] {

                    let ins_index = match int_ref.keys.binary_search(&key_index.0) {
                        Ok(idx) => idx, // Here I will just return on collisions, may be
                        // changed in the future. But I think there's no way I can insert repeated
                        // values into the internal nodes because of the leaf restriction.
                        Err(idx) => idx,
                    };

                    // Insert the key.
                    int_ref.keys.insert(ins_index, key_index.0);

                    // Insert the "pointer".
                    int_ref.children.insert(ins_index + 1, key_index.1);

                }

            } else {
                return None;
            }
        } else {

            // Here I insert into the leaf. After, I fix the split
            // Ainda nao sei se eu permito repeticao de chave, entao vou deixar como unit ()
            if let Node::Leaf(leaf_ref) = &mut self.nodes[index] {
                match leaf_ref.content.binary_search_by_key(&key, |pair | pair.0) {
                    Ok(_) => (),
                    Err(index) => leaf_ref.content.insert(index, (key, value)),
                }
            }
        }

        // Logic to split: the key from the exactly mid goes up or the mid-right for M even

        // REMINDER: HERE I'VE ALREADY INSERTED THE (KEY, VALUE), MOREOVER: ORDERED!!!
        if self.nodes[index].should_split() {

            let split_index = match &self.nodes[index] {
                Node::Internal(int) => {
                    int.keys.len() / 2 // [20 30 (40) | 50]     [20 30 (40) | 50 60] split_index + 1 INTERNO
                                       // [20 30 | (40) 50]    [20 30 | (40) 50 60] split_index FOLHA
                },
                Node::Leaf(leaf) => {
                    leaf.content.len() / 2 // Aqui ta certo
                },
            };
            let last_index = self.size(); // This will be the index of the right split of
            // the node


            match &mut self.nodes[index] {
                Node::Internal(int_node) => {

                    let mut new_int: InternalNode<K, M> = InternalNode::new();

                    // Here I'll have to pop the first part
                    new_int.keys = int_node.keys.split_off(split_index + 1);

                    let to_promote = int_node.keys
                        .pop()
                        .expect("THERE SHOULD BE A VALUE HERE TO SPLIT");

                    // Now the pointers
                    new_int.children = int_node.children.split_off(split_index + 1);

                    self.nodes.push(Node::Internal(new_int));

                    Some((to_promote, last_index))

                },

                Node::Leaf(leaf_node) => {

                    // Creates a new leaf with the "right-side" of the vector and it pointer
                    let mut new_leaf: LeafNode<K, V, M> = LeafNode::new(leaf_node.next);
                    new_leaf.content = leaf_node.content.split_off(split_index);

                    // The key implements copy so this is cheap
                    let to_promote = new_leaf.content[0].0;

                    // Attribute the next node logic to the existing leaf
                    leaf_node.next = Some(last_index);

                    // Insert the node into the MemPool
                    self.nodes.push(Node::Leaf(new_leaf)); // Eh importante so fazer o push aqui
                    // pro last index estar realmente certo

                    Some((to_promote, last_index))

                }
            }

        } else {
            None
        }


    }
}
