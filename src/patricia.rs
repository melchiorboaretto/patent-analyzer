struct Node<V: Copy> {

    radix: Vec<u8>,
    value: Option<V>,
    children: Vec<(u8, Box<Node<V>>)>,
}

struct Patricia<V: Copy> {

    root: Node<V>,
}

impl<V: Copy> Node<V> {

    fn new(radix: impl Into<Vec<u8>>, value: Option<V>, children: Option<Vec<(u8, Box<Node<V>>)>>) -> Self {

        let radix = radix.into();
        let children = children.unwrap_or_default();

        Node {
            radix,
            value,
            children,
        }

    }

    fn split(&mut self, match_pos: usize) {

        let new_child = Box::new(
            Node::new(
         self.radix.split_off(match_pos),
                self.value,
                Some(std::mem::take(&mut self.children)),
            )
        );

        // Here I'll use "unsafely" radix[0] because if the vector is empty, there shouldn't be a
        // split.
        self.children = vec![(new_child.radix[0] ,new_child)];
        self.value = None;

    }

}




#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn split() {

        let porca = String::from("porca");
        let por = String::from("por");

        let mut original_node = Node::new(porca, Some(7), None);

        let match_pos = {

            let mut iter = por.as_bytes().iter().enumerate();
            loop {
                if let Some((pos, letter)) = iter.next() {
                    if *letter != original_node.radix[pos] {
                        break pos;
                    }

                } else {
                    break por.len();
                }
            }
        };

        assert_eq!(original_node.children.len(), 0);

        original_node.split(match_pos);

        assert_eq!(original_node.children[0].0, b'c');
        assert_eq!(original_node.children[0].1.value.unwrap(), 7);
        assert_eq!(original_node.radix, por.as_bytes());
        assert_eq!(original_node.children[0].1.radix.len(), 2);


    }

}
