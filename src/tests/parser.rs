use crate::ast::{ASTNode, ASTNodeValue};
use crate::parser::{self, Parser};
use crate::scanner::{Token, TokenWithLocation};

#[test]
fn parse_empty() {
    let ast = parser::parse_from_source("");
    assert_eq!(ast.size(), 1); // has only a root node
    let root = ast.root();
    assert_eq!(root.size(), 0);
}

#[test]
fn parse_int() {
    let ast = parser::parse_from_source("1");
    assert_eq!(ast.size(), 2);
    let root = ast.root();
    assert_eq!(root.size(), 1);
    match ast.get_value(1).unwrap() {
        ASTNodeValue::Object(string) => assert_eq!(string, "1"),
        _ => assert!(false, "Unexpected parse result for `1`"),
    }
}

#[test]
fn parse_simple_assignment() {
    //      R
    //      |
    //      n=
    //      |
    //      1
    let ast = parser::parse_from_source("n = 1");
    assert_eq!(ast.size(), 3, "{:?}", ast);
    let root = ast.root();
    assert_eq!(root.size(), 1);
    match ast.get(root.children[0]) {
        Some(ASTNode {
            index: _,
            value: ASTNodeValue::Assignment(name),
            parent: _,
            children: c,
        }) => {
            assert_eq!(name, "n");
            assert_eq!(c.len(), 1);
            assert_eq!(
                *ast.get_value(c[0]).unwrap(),
                ASTNodeValue::Object("1".to_owned()),
            );
        }
        _ => assert!(false, "Unexpected parse result for `n = 1`"),
    }
}

#[test]
fn parse_add() {
    //      R
    //      |
    //      +
    //     / \
    //    1   2
    let ast = parser::parse_from_source("1 + 2");
    eprintln!("{:?}", ast);
    assert_eq!(ast.size(), 4, "{:?}", ast);
    let root = ast.root();
    assert_eq!(root.size(), 1);
    match ast.get(root.children[0]) {
        Some(ASTNode {
            index: _,
            value: ASTNodeValue::BinaryOperation(operator),
            parent: _,
            children: c,
        }) => {
            assert_eq!(*operator, '+');
            assert_eq!(c.len(), 2);
            assert_eq!(
                *ast.get_value(c[0]).unwrap(),
                ASTNodeValue::Object("1".to_owned()),
            );
            assert_eq!(
                *ast.get_value(c[1]).unwrap(),
                ASTNodeValue::Object("2".to_owned())
            );
        }
        _ => assert!(false, "Unexpected parse result for `1 + 2`"),
    }
}

#[test]
fn parse_assign_to_addition() {
    let ast = parser::parse_from_source("n = 1 + 2");
    assert_eq!(ast.size(), 2);
    let root = ast.root();
    assert_eq!(root.size(), 1);
    match ast.get_value(1) {
        Some(ASTNodeValue::Assignment(name)) => {
            assert_eq!(name, "n");
            // assert_eq!(string, "1");
        }
        _ => assert!(false, "Unexpected parse result for `n = 1`"),
    }
}

#[test]
fn parse_simple_program() {
    //      ROOT
    //     /    \
    //    a=    b=
    //    |     |
    //    1     +
    //         / \
    //        a   1
    let ast = parser::parse_from_source("a = 1\nb = a + 2\n");
    assert_eq!(ast.size(), 7, "{:?}", ast);
    let root = ast.root();
    assert_eq!(root.size(), 2, "{:?}", ast);
    match ast.get_value(1) {
        Some(ASTNodeValue::Assignment(name)) => {
            assert_eq!(name, "n");
            // assert_eq!(string, "1");
        }
        _ => assert!(false, "Unexpected parse result for `n = 1`"),
    }
}

// Utilities -----------------------------------------------------------
