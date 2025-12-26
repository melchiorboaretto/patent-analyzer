
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
impl<K: Ord + Copy, V> LeafNode<K, V> {

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

// Impl Internal
impl<K: Ord + Copy> InternalNode<K> {


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
impl<K: Ord + Copy, V> Node<K, V> {


}

impl< K: Ord + Copy, V> BPlusTree<K, V> {

    fn new(t: usize) -> Self {
        BPlusTree {
            root_idx: None,
            nodes: Vec::new(),
            degree_t: t,
        }
    }

    // How many nodes does the tree have
    #[inline]
    fn size(&self) -> usize {
        self.nodes.len()
    }

    // Gets a reference to the root node (or None)
    fn root(&self) -> Option<&Node<K, V>> {
        if let Some(idx) = self.root_idx {
            Some(&self.nodes[idx])
        } else {
            None
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


    // Returns a reference to the value corresponding to the key.
    // May be None if the key cannot be find.
    fn get(&self, key: K) -> Option<&V> {

        let mut curr_node;

        // Get the root or return None, alright
        if let Some(root) = self.root() {
            curr_node = root;
        } else {
            return None;
        }

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

    fn insert(&mut self, key: K, value: V) {

        // Considering the empty tree case. Putting the CPU's branch predictor to test
        // on every insertion hehe
        if self.root_idx.is_none() {
            self.create_first_root(key, value);
            return;
        }


        // NOTE: ANTES DA FUNCAO de remove funcionar eu vou simplesmente usar
        // push no vetor de nodos em caso de split, DEPOIS eu vou fazer pra usar o primeiro num banco
        // de posicoes removidas ou nao utilizadas

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





// Ate agora tudo funcionando
pub fn teste() {


    let mut minha_arvore = BPlusTree::<usize, u64>::new(4);

    {
        if minha_arvore.root_idx.is_some() {

            let meu_nodo = &minha_arvore.nodes[0];

            if let Node::Leaf(folha) = meu_nodo {
                println!("O valor na folha eh {:?}", folha.content[0]);
            }
        } else {
            println!(" Nada ainda!!");
        }

    }

    minha_arvore.insert(3, 4);


    let meu_nodo = &minha_arvore.nodes[0];

    if let Node::Leaf(folha) = meu_nodo {
        println!("O valor na folha eh {:?}", folha.content[0]);
    } else {
        println!(" Nada ainda!!");
    }

}


