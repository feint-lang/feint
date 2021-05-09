use std::fmt;

use crate::scanner::Token;

#[derive(Debug)]
pub struct AST {
    pub storage: Vec<ASTNode>,
}

#[derive(Debug)]
pub struct ASTNode {
    pub index: usize,
    pub value: ASTNodeValue,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

/// NOTE: String is standing in for actual Objects for the time being.
#[derive(Debug, PartialEq)]
pub enum ASTNodeValue {
    ExpressionList,

    Object(String),
    BinaryOperation(char),
    Assignment(String),
    Reference(String),

    // TEMP
    UnknownToken(Token),

    // Marker
    Indent(u8),
}

impl AST {
    pub fn new() -> Self {
        Self {
            storage: vec![ASTNode::new(0, ASTNodeValue::ExpressionList, None)],
        }
    }

    /// Return reference to root node.
    pub fn root(&self) -> &ASTNode {
        self.get(0).unwrap()
    }

    /// Return reference to root node value.
    pub fn root_value(&self) -> &ASTNodeValue {
        self.get_value(0).unwrap()
    }

    /// Get node at index.
    pub fn get(&self, index: usize) -> Option<&ASTNode> {
        self.storage.get(index)
    }

    pub fn get_value(&self, index: usize) -> Option<&ASTNodeValue> {
        match self.storage.get(index) {
            Some(ast_node) => Some(&ast_node.value),
            None => None,
        }
    }

    /// Pop the last-added node.
    // pub fn pop(&mut self) -> Option<ASTNode> {
    //     if self.size() == 1 {
    //         panic!("Can't pop root node");
    //     }
    //     let node_option = self.storage.pop();
    //     match self.storage.pop() {
    //         Some(node) => {
    //             // Remove node from parent's child list
    //             node.children.Some(node)
    //         }
    //         None => None,
    //     }
    // }

    /// Get total number of nodes in AST.
    pub fn size(&self) -> usize {
        self.storage.len()
    }

    /// Add node to tree and return its index.
    pub fn add(&mut self, value: ASTNodeValue, parent_index: usize) -> usize {
        let index = self.size();
        let node = ASTNode::new(index, value, Some(parent_index));
        self.storage.push(node);
        match self.get(parent_index) {
            Some(parent_node) => self.storage[parent_index].children.push(index),
            None => panic!("Parent node not found: {}", parent_index),
        }
        index
    }

    pub fn add_many(&mut self, values: Vec<ASTNodeValue>, parent_index: usize) {
        for value in values {
            self.add(value, parent_index);
        }
    }

    /// Remove and return the last child from the specified node.
    /// TODO: Remove the child index from the node's children.
    pub fn pop_child(&mut self, index: usize) -> ASTNode {
        match self.get(index) {
            Some(mut parent_node) => match parent_node.last() {
                Some(child_index) => self.storage.remove(*child_index),
                None => panic!(),
            },
            None => panic!(),
        }
    }
}

impl fmt::Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let root = self.root();
        write!(f, "{:?}: {:?}", root.index, root.children)
    }
}

impl ASTNode {
    pub fn new(index: usize, value: ASTNodeValue, parent: Option<usize>) -> Self {
        Self {
            index,
            value,
            parent,
            children: vec![],
        }
    }

    /// Return the index of the last child.
    pub fn last(&self) -> Option<&usize> {
        self.children.last()
    }

    /// Return the number of children this node has.
    pub fn size(&self) -> usize {
        self.children.len()
    }

    /// Pop and return the index of the last child.
    pub fn pop(&mut self) -> Option<usize> {
        self.children.pop()
    }
}
