use crate::tokens::Token;

#[derive(Debug)]
pub struct AST {
    storage: Vec<ASTNode>,
}

#[derive(Debug)]
pub struct ASTNode {
    index: usize,
    value: Node,
    parent: Option<usize>,
    children: Vec<usize>,
}

#[derive(Debug)]
pub enum Node {
    Program,
    Object(String),
    BinaryOperation(char, String, String),
    Assignment(String, String),
}

impl AST {
    pub fn new() -> Self {
        Self {
            storage: vec![ASTNode::new(0, Node::Program, None)],
        }
    }

    /// Return reference to root node.
    pub fn root(&self) -> &ASTNode {
        self.storage.get(0).unwrap()
    }

    /// Get node at index.
    pub fn get(&self, index: usize) -> Option<&ASTNode> {
        self.storage.get(index)
    }

    /// Get total number of nodes in AST.
    pub fn size(&self) -> usize {
        self.storage.len()
    }

    /// Add node to tree and return its index.
    pub fn add(&mut self, node: Node, parent: Option<usize>) -> usize {
        let index = self.size();
        let ast_node = ASTNode::new(index, node, parent);
        self.storage.push(ast_node);
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

impl ASTNode {
    pub fn new(index: usize, value: Node, parent: Option<usize>) -> Self {
        Self {
            index,
            value,
            parent,
            children: vec![],
        }
    }
}
