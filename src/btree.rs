use crate::x::Window;

pub struct BinaryTree {
    pub value: Window,
    pub left: Option<Box<BinaryTree>>,
    pub right: Option<Box<BinaryTree>>,
}

impl BinaryTree {
    pub fn new(value: Window) -> Self {
        BinaryTree {
            value,
            left: None,
            right: None,
        }
    }

    pub fn left(mut self, node: BinaryTree) -> Self {
        self.left = Some(Box::new(node));
        self
    }

    pub fn right(mut self, node: BinaryTree) -> Self {
        self.right = Some(Box::new(node));
        self
    }
}
