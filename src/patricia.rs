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

    fn get_next(&mut self, next: u8) -> Option<&mut Self> {

        let mut node = None;

        for child in self.children.iter_mut() {
            if child.0 == next {
                node = Some(&mut child.1);
                break;
            }
        }

        if let Some(return_val) = node {
            Some(&mut *return_val)
        } else {
            None
        }
    }

}


impl<V: Copy> Patricia<V> {

    fn new() -> Self {
        Patricia {
            root: Node::new("", None, None),
        }
    }

    /// Try to insert a radix and a value corresponding to it.
    /// Returns the value added wrapped in Ok() if it could be added or
    /// the the old value wrapped in Err() if a value was already there
    fn insert(&mut self, radix: impl Into<Vec<u8>>, value: V) -> Result<V, V> {

        let input_radix = radix.into();
        let mut curr_input_idx = 0;

        // This is the match_pos from the split function...
        let mut found_unmatch = false;

        // Here I have to initialize the variables because the compiler
        // is unable to know that the only way to make found_unmatch == true is
        // to attribute a value to all of the three inside the for-if block
        let mut unmatch_pos = 0;
        let mut unmatch_input_idx = 0;

        let mut node = self.root();

        loop {

            let mut curr_node_idx = 0;
            let mut is_prefix = false;

            for (cmp_idx, cmp_byte) in node.radix.iter().enumerate() {

                if curr_input_idx == input_radix.len() || input_radix[curr_input_idx] != *cmp_byte {
                    if curr_input_idx == input_radix.len() {
                        is_prefix = true;
                    }
                    unmatch_input_idx = curr_input_idx;
                    unmatch_pos = cmp_idx;
                    found_unmatch = true;
                    break;
                }
                curr_input_idx += 1;
                curr_node_idx += 1;

            }

            // If I tried to insert over an existing node
            if !found_unmatch 
            && node.radix.len() == curr_node_idx
            && curr_input_idx == input_radix.len() {

                // Value implements Copy so this is fine (I guess)
                break if let Some(node_value) = node.value {

                    Err(node_value)

                } else {

                    node.value = Some(value);
                    Ok(value)

                };
            }

            // If there was not an unmatch and the input radix has ended
            // like in (porco) and trying to insert "por"
            if is_prefix {
                node.split(unmatch_pos);
                node.value = Some(value);

                break Ok(value);
            }

            // Kinda obvious...
            if found_unmatch {
                node.split(unmatch_pos);
                let new_boxed_node = Box::new(
                    Node::new(&input_radix[unmatch_input_idx..], Some(value), None)
                );

                node.children.push((new_boxed_node.radix[0], new_boxed_node));

                break Ok(value);
            } else {

                // If there exists a child starting with the next letter...
                if let Some(child_idx) = node.children.iter().position(|pair| pair.0 == input_radix[curr_input_idx]) {
                    node = &mut node.children[child_idx].1;

                    // If there's not...
                } else {
                    let new_boxed_node = Box::new(
                        Node::new(&input_radix[curr_input_idx..], Some(value), None)
                    );

                    node.children.push((new_boxed_node.radix[0], new_boxed_node));

                    break Ok(value);
                }

            }

        }
    }

    /// Retrieves a value corresponding to a given radix
    fn get(&self, radix: impl AsRef<[u8]>) -> Option<V> {

        let radix = radix.as_ref();
        let mut input_idx = 0;

        let mut node = &self.root;

        loop {

            if radix[input_idx..].len() == node.radix.len() {

                if radix[input_idx..] == node.radix {

                    break node.value;

                } else {
                    break None;
                }

            } else if radix[input_idx..].len() > node.radix.len() {

                for cmp_byte in &node.radix {
                    if radix[input_idx] != *cmp_byte {
                        // Return early of course hehe
                        return None;
                    }

                    input_idx += 1;
                }

                if let Some(child_idx) = node.children
                    .iter()
                    .position(|pair| pair.0 == radix[input_idx]) {

                    node = &node.children[child_idx].1;
                } else {

                    return None;

                }

            } else {

                break None;

            }

        }

    }

    /// Updates the value corresponding to the given radix.
    /// Returns None if the node did not exist or the old value wrapped in Some
    fn update(&mut self, radix: impl AsRef<[u8]>, value: V) -> Option<V> {

        let radix = radix.as_ref();
        let mut input_idx = 0;

        let mut node = self.root();

        loop {

            if radix[input_idx..].len() == node.radix.len() {

                if radix[input_idx..] == node.radix && node.value.is_some() {

                    let ret_value = node.value;
                    node.value = Some(value);
                    break ret_value;

                } else {
                    break None;
                }

            } else if radix[input_idx..].len() > node.radix.len() {

                for cmp_byte in &node.radix {
                    if radix[input_idx] != *cmp_byte {
                        // Return early of course hehe
                        return None;
                    }

                    input_idx += 1;
                }

                if let Some(child_idx) = node.children
                    .iter()
                    .position(|pair| pair.0 == radix[input_idx]) {

                    node = &mut node.children[child_idx].1;
                } else {

                    return None;

                }

            } else {

                break None;

            }

        }


    }

    #[inline]
    fn root(&mut self) -> &mut Node<V> {
        &mut self.root
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

    #[test]
    fn insertion_and_get() {

        let porca = String::from("porca");
        let por = String::from("por");
        let porcaria = String::from("porcaria");
        let potro = String::from("potro");
        let potrinho = String::from("potrinho");
        let porquinho = String::from("porquinha");

        let mut patricia = Patricia::new();
        patricia.insert(por.clone(), 0).unwrap();
        patricia.insert(porca.clone(), 1).unwrap();
        patricia.insert(porquinho.clone(), 2).unwrap();
        patricia.insert(potrinho.clone(), 3).unwrap();
        patricia.insert(potro.clone(), 4).unwrap();
        patricia.insert(porcaria.clone(), 5).unwrap();

        assert_eq!(patricia.get(&por).unwrap(), 0);
        assert_eq!(patricia.get(&porca).unwrap(), 1);
        assert_eq!(patricia.get(&porquinho).unwrap(), 2);
        assert_eq!(patricia.get(&potrinho).unwrap(), 3);
        assert_eq!(patricia.get(&potro).unwrap(), 4);
        assert_eq!(patricia.get(&porcaria).unwrap(), 5);


        assert_eq!(patricia.insert(potrinho.clone(), 9), Err(3));

    }
}
