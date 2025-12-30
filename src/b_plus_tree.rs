
struct InternalNode<K, const M: usize> {
    keys: Vec<K>,
    children: Vec<usize>, // This is a vector of indexes, these index a vector with all the nodes
}

struct LeafNode<K, V, const M: usize> {
    content: Vec<(K, V)>,
    next: Option<usize>,
}

enum Node<K, V, const M: usize> {
    Internal(InternalNode<K, M>),
    Leaf(LeafNode<K, V, M>),
}

struct BPlusTree< K: Ord + Copy, V, const M: usize> {
    root_idx: usize,
    nodes: Vec<Node<K, V, M>>,
}

// Impl Leaf
impl<K: Ord + Copy, V, const M: usize> LeafNode<K, V, M> {

    fn new(next: Option<usize>) -> Self {

        LeafNode {
            content: Vec::with_capacity(M),
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

    fn update_value(&mut self, key: K, value: V) -> Option<V> {

        if let Ok(index) = self
            .content.binary_search_by_key(&key, |pair| pair.0) {

            Some(std::mem::replace(&mut self.content[index].1, value))

        } else {
            None
        }
    }



}

// Impl Internal
impl<K: Ord + Copy, const M: usize> InternalNode<K, M> {

    fn new() -> Self {

        InternalNode {
            keys: Vec::with_capacity(M),
            children: Vec::with_capacity(M), // This is a vector of indexes, these index a vector with all the nodes
        }

    }

    // Find the index of the next child for a key using binary search over the current node
    //
    // IMPORTANT: 
    // Assuming the tree follow the LOWER left and GREATER OR EQUAL right convention.
    fn find_child_index(&self, key: K) -> usize {

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



// Impl Node 
impl<K: Ord + Copy, V, const M: usize> Node<K, V, M> {
    fn new_leaf() -> Self {
        Node::Leaf(LeafNode::new(None))
    }

    fn new_internal() -> Self {
        Node::Internal(InternalNode::new())
    }

    fn should_split(&self) -> bool {
        match self {
            Node::Internal(int_node) => int_node.keys.len() == M,
            Node::Leaf(leaf_node) => leaf_node.content.len() == M,
        }
    }

}

impl< K: Ord + Copy, V, const M: usize> BPlusTree<K, V, M> {

    fn new() -> Self {

        BPlusTree {
            root_idx: 0,
            nodes: vec![Node::new_leaf()],
        }
    }

    // How many nodes does the tree have
    #[inline]
    fn size(&self) -> usize {
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

    // Returns a reference to the value corresponding to the key.
    // May be None if the key cannot be find.
    fn get(&self, key: K) -> Option<&V> {

        let mut curr_node;

        // Get the root or return None, alright
        curr_node = self.root();

        // Go to the corresponding leaf
        while let Node::Internal(int_node) = curr_node {
            curr_node = &self.nodes[int_node.find_child_index(key)];
        }

        if let Node::Leaf(leaf_node) = curr_node {
            leaf_node.retrieve_value(key)
        } else {
            None
        }


    }

    fn insert_aux(&mut self, key: K, value: V, index: usize) -> Option<(K, usize)> {


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

            let split_index = self.degree_t();
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

    fn insert(&mut self, key: K, value: V) {

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
    fn remove(&mut self, key: K) -> Option<V> {
        todo!()


    }

    // Returns the old value or None (if the key was not found)
    fn update(&mut self, key: K, value: V) -> Option<V> {

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




}





// Ate agora tudo funcionando
pub fn teste() {


    let mut minha_arvore = BPlusTree::<usize, u64, 3>::new();

    {

        let meu_nodo = minha_arvore.get(4);

        if let Some(folha) = meu_nodo {
            println!("O valor na folha eh {:?}", folha);
        }

    }

    minha_arvore.insert(3, 4);
    minha_arvore.insert(7, 2);
    minha_arvore.insert(11, 8);
    minha_arvore.insert(8, 3);
    minha_arvore.insert(1, 6);

    let outro = minha_arvore.update(11, 1);

    minha_arvore.insert(4, outro.unwrap());

    for i in 0..13 {

        let meu_nodo = minha_arvore.get(i);

        if let Some(folha) = meu_nodo {
            println!("O valor na folha de chave {} eh {:?}", i, folha);
        } else {
            println!(" Nada ainda!!");
        }
    }

}


