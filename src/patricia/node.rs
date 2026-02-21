
pub struct Node<V: Copy> {

    radix: Vec<u8>,
    value: Option<V>,
    children: Vec<(u8, Box<Node<V>>)>,
}

impl<V: Copy> Node<V> {

    pub fn new(radix: impl Into<Vec<u8>>, value: Option<V>, children: Option<Vec<(u8, Box<Node<V>>)>>) -> Self {

        let radix = radix.into();
        let children = children.unwrap_or_default();

        Node {
            radix,
            value,
            children,
        }

    }

    pub fn split(&mut self, match_pos: usize) {

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

        if let Some(child_idx) = self.children().iter().position(|pair| pair.0 == next) {
            Some(&mut self.children[child_idx].1)
        } else {
            None
        }

    }

    pub fn first_byte(&self) -> u8 {
        self.radix[0]
    }


}

impl<V: Copy> Node<V> {

    #[inline]
    pub fn radix(&self) -> &[u8] {
        &self.radix
    }

    /// This is fine because value implements Copy
    #[inline]
    pub fn value(&self) -> Option<V> {
        self.value
    }

    #[inline]
    pub fn set_value(&mut self, value: V) {
        self.value = Some(value);
    }

    #[inline]
    pub fn set_value_none(&mut self) {
        self.value = None;
    }

    #[inline]
    pub fn children(&self) -> &Vec<(u8, Box<Node<V>>)> {
        &self.children
    }

    #[inline]
    pub fn children_mut(&mut self) -> &mut Vec<(u8, Box<Node<V>>)> {
        &mut self.children
    }

    /// This function will cause a panic if the node's Vec is empty
    #[inline]
    pub fn children_push(&mut self, node: Box<Node<V>>) {
        self.children.push((node.first_byte(), node));
    }

}
