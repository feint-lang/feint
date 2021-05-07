use std::iter::Peekable;
use std::slice::Iter;

use crate::ast::{Node, NodeValue, AST};
use crate::tokens::{Token, TokenWithPosition};

/// Parse tokens and ...
pub fn parse(tokens: &Vec<TokenWithPosition>) -> AST {
    let mut parser = Parser::new(tokens);
    parser.parse();
    parser.ast
}

pub struct Parser<'a> {
    stream: Peekable<Iter<'a, TokenWithPosition>>,
    ast: AST,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<TokenWithPosition>) -> Self {
        Self {
            stream: tokens.iter().peekable(),
            ast: AST::new(),
        }
    }

    pub fn parse(&mut self) {
        self.program();
    }

    fn next(&mut self) -> Option<&Token> {
        match self.stream.next() {
            Some(token) => Some(&token.token),
            _ => None,
        }
    }

    fn peek(&mut self) -> Option<&Token> {
        match self.stream.peek() {
            Some(token) => Some(&token.token),
            _ => None,
        }
    }

    // Grammar

    /// A program is a list of expressions.
    fn program(&mut self) {
        loop {
            match self.expression() {
                Some(node_value) => {
                    self.ast.add(node_value, Some(0));
                }
                None => break,
            }
        }
    }

    fn expression(&mut self) -> Option<NodeValue> {
        match self.next() {
            // Atoms
            Some(Token::True) => Some(NodeValue::Object("true".to_string())),
            Some(Token::False) => Some(NodeValue::Object("false".to_string())),
            Some(Token::Float(digits)) => Some(NodeValue::Object(digits.to_string())),
            Some(Token::Int(digits)) => Some(NodeValue::Object(digits.to_string())),
            Some(Token::String(string)) => Some(NodeValue::Object(string.to_string())),
            // Binary operations
            Some(Token::Plus) => Some(NodeValue::BinaryOperation('+')),
            Some(Token::Minus) => Some(NodeValue::BinaryOperation('-')),
            _ => None,
        }
    }

    // fn term(&mut self) -> i32 {
    //     let mut result = self.factor();
    //     loop {
    //         match self.peek() {
    //             Some(Token::Star) => {
    //                 self.next();
    //                 result *= self.factor();
    //             }
    //             Some(Token::Slash) => {
    //                 self.next();
    //                 result /= self.factor();
    //             }
    //             _ => break result,
    //         }
    //     }
    // }
    //
    // fn factor(&mut self) -> i32 {
    //     match self.next() {
    //         Some(Token::Int(digits)) => digits.parse::<i32>().unwrap(),
    //         Some(Token::LeftParen) => {
    //             let result = self.expr();
    //             match self.next() {
    //                 Some(Token::RightParen) => (),
    //                 Some(token) => panic!("Unexpected token: {:?}", token),
    //                 None => panic!("Unexpected end of expression"),
    //             }
    //
    //             result
    //         }
    //         Some(Token::String(_)) => 0,
    //         Some(Token::Identifier(_)) => 0,
    //         Some(token) => panic!("Unexpected token: {:?}", token),
    //         None => panic!("Unexpected end of expression"),
    //     }
    // }
}
