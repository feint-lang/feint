use std::iter::Peekable;
use std::slice::Iter;

use crate::ast::{ASTNode, Node, AST};
use crate::tokens::{Token, TokenWithPosition};

type NextOption<'a> = Option<(&'a Token, Option<&'a Token>, Option<&'a Token>)>;

/// Parse tokens and ...
pub fn parse(tokens: &Vec<TokenWithPosition>) -> AST {
    let mut parser = Parser::new(tokens);
    parser.parse();
    parser.ast
}

pub struct Parser<'a> {
    stream: Peekable<Iter<'a, TokenWithPosition>>,
    one_ahead_stream: Peekable<Iter<'a, TokenWithPosition>>,
    two_ahead_stream: Peekable<Iter<'a, TokenWithPosition>>,
    ast: AST,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<TokenWithPosition>) -> Self {
        let stream = tokens.iter().peekable();
        let mut one_ahead_stream = tokens.iter().peekable();
        let mut two_ahead_stream = tokens.iter().peekable();
        one_ahead_stream.next();
        two_ahead_stream.next();
        two_ahead_stream.next();
        let instance = Self {
            stream,
            one_ahead_stream,
            two_ahead_stream,
            ast: AST::new(),
        };

        instance
    }

    pub fn parse(&mut self) {
        self.program();
    }

    fn next(&mut self) -> NextOption {
        match self.stream.next() {
            Some(token) => {
                let one_ahead = match self.one_ahead_stream.next() {
                    Some(t) => Some(&t.token),
                    None => None,
                };
                let two_ahead = match self.two_ahead_stream.next() {
                    Some(t) => Some(&t.token),
                    None => None,
                };
                Some((&token.token, one_ahead, two_ahead))
            }
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
        let mut index = 0;
        let mut prev_node: Node;
        loop {
            if self.peek().is_none() {
                break;
            }
            match self.expression() {
                Ok(node) => {
                    eprintln!("{:?}", node);
                    self.ast.add(node, Some(index));
                }
                Err(message) => {
                    eprintln!("{}", message);
                    break;
                }
            }
        }
    }

    fn expression(&mut self) -> Result<Node, String> {
        let node = match self.next() {
            // Atoms
            Some((Token::True, _, _)) => Node::Object("true".to_string()),
            Some((Token::False, _, _)) => Node::Object("false".to_string()),
            Some((Token::Float(digits), _, _)) => Node::Object(digits.to_string()),
            Some((Token::Int(digits), _, _)) => Node::Object(digits.to_string()),
            Some((Token::String(string), _, _)) => Node::Object(string.to_string()),
            // Assignment
            Some((Token::Identifier(identifier), Some(Token::Equal), Some(_))) => {
                let name = identifier.to_string();
                self.next();
                match self.expression() {
                    Ok(Node::Object(obj)) => Node::Assignment(name, obj),
                    Ok(node) => return Err(format!("RHS is not an object: {:?}", node)),
                    Err(message) => return Err(message),
                }
            }
            // Binary operators
            Some((Token::Plus, Some(_), _)) => {
                eprintln!("++++");
                let a = match self.ast.storage.pop() {
                    Some(index) => match self.ast.get_node(index) {
                        Some(Node::Object(s)) => s.to_string(),
                        _ => return Err("XXX".to_string()),
                    },
                    _ => return Err("Not an object".to_string()),
                };
                match self.expression() {
                    Ok(Node::Object(b)) => Node::BinaryOperation('+', a, b),
                    Ok(node) => return Err(format!("RHS is not an object: {:?}", node)),
                    Err(message) => return Err(message),
                }
            }
            Some(token) => return Err(format!("Unhandled token: {:?}", token)),
            None => return Err("Unexpectedly ran out of tokens".to_string()),
        };

        Ok(node)
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
