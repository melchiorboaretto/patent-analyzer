mod node;
mod utils;
mod iter;

use node::{
    InternalNode,
    LeafNode,
    Node,
};

struct BPlusTree< K: Ord + Copy, V, const M: usize> {
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

    // Returns a reference to the value corresponding to the key.
    // May be None if the key cannot be find.
    pub fn get(&self, key: K) -> Option<&V> {

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


