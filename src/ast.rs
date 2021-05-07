use crate::tokens::Token;

#[derive(Debug)]
pub struct AST {
    storage: Vec<Node>,
}

#[derive(Debug)]
pub struct Node {
    index: usize,
    value: NodeValue,
    parent: Option<usize>,
    children: Vec<usize>,
}

#[derive(Debug)]
pub enum NodeValue {
    Program,
    Object(String),
    BinaryOperation(char),
}

impl AST {
    pub fn new() -> Self {
        Self {
            storage: vec![Node::new(0, NodeValue::Program, None)],
        }
    }

    /// Return reference to root node.
    pub fn root(&self) -> &Node {
        self.storage.get(0).unwrap()
    }

    /// Get node at index.
    pub fn get(&self, index: usize) -> Option<&Node> {
        self.storage.get(index)
    }

    /// Get total number of nodes in AST.
    pub fn size(&self) -> usize {
        self.storage.len()
    }

    /// Add node to tree and return its index.
    pub fn add(&mut self, value: NodeValue, parent: Option<usize>) -> usize {
        let index = self.size();
        let node = Node::new(index, value, parent);
        self.storage.push(node);
        if parent.is_some() {
            let parent_index = parent.unwrap();
            match self.get(parent_index) {
                Some(parent_node) => self.storage[parent_index].children.push(index),
                None => panic!("Parent node not found: {}", parent_index),
            }
        }
        index
    }
}

impl Node {
    pub fn new(index: usize, value: NodeValue, parent: Option<usize>) -> Self {
        Self {
            index,
            value,
            parent,
            children: vec![],
        }
    }
}
