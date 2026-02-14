struct Node<V: Copy> {

    radix: Vec<u8>,
    value: Option<V>,
    children: Vec<(u8, Box<Node<V>>)>,
}

struct Patricia<V: Copy> {

    root: Node<V>,
}
