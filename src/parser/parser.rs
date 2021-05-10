use std::iter::Peekable;
use std::slice::Iter;

use crate::ast::{ASTNode, ASTNodeValue, AST};
use crate::scanner::scan;
use crate::scanner::{Token, TokenWithLocation};

type ParseResult = Result<AST, String>;
type NextOption<'a> = Option<(&'a Token, Option<&'a Token>, Option<&'a Token>)>;

/// Parse tokens and return an AST.
pub fn parse(tokens: &Vec<TokenWithLocation>) -> AST {
    let mut parser = Parser::new(tokens);
    parser.parse();
    parser.ast
}

/// Scan source, parse tokens, and return an AST.
pub fn parse_from_source(source: &str) -> AST {
    match scan(source) {
        Ok(tokens) => parse(&tokens),
        _ => AST::new(),
    }
}

pub struct Parser<'a> {
    stream: Peekable<Iter<'a, TokenWithLocation>>,
    one_ahead_stream: Peekable<Iter<'a, TokenWithLocation>>,
    two_ahead_stream: Peekable<Iter<'a, TokenWithLocation>>,
    ast: AST,
    current_node: Option<&'a ASTNode>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<TokenWithLocation>) -> Self {
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
            current_node: None,
        };

        instance
    }

    pub fn parse(&mut self) {
        // A program is a list of expressions.
        self.expression_list(0);
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

    fn expression_list(&mut self, index: usize) {
        loop {
            if self.peek().is_none() {
                break;
            }
            self.expression(index);
        }
    }

    fn expression(&mut self, parent_index: usize) {
        // let node_value = self.get_expr();
        //
        // match self.peek() {
        //     Some(next_node_value @ ASTNodeValue::BinaryOperation(_)) => {
        //         let index = self.ast.add(next_node_value, parent_index);
        //         self.next();
        //         let rhs = self.get_expr();
        //         self.ast.add(node_value, index);
        //         self.ast.add(rhs, index);
        //     }
        //     _ => match node_value {
        //         (ASTNodeValue::Assignment(_), _) => {
        //             let index = self.ast.add(node_value, parent_index);
        //             self.next();
        //             let rhs = self.get_expr();
        //             self.ast.add(rhs, index);
        //         }
        //         Some(ASTNodeValue::BinaryOperation(_)) => {
        //             let index = self.ast.add(ASTNodeValue::Assignment(name), parent_index);
        //             self.next();
        //             let rhs = self.get_expr();
        //             self.ast.add(rhs, index);
        //         }
        //         (ASTNodeValue::Indent(0), _) => {
        //             return;
        //         }
        //         _ => {
        //             self.ast.add(node_value, parent_index);
        //         }
        //     },
        // }
    }

    fn get_expr(&mut self) -> ASTNodeValue {
        match self.next() {
            // Atoms
            Some((Token::True, _, _)) => ASTNodeValue::Object("true".to_owned()),
            Some((Token::False, _, _)) => ASTNodeValue::Object("false".to_owned()),
            Some((Token::Float(digits), _, _)) => {
                ASTNodeValue::Object(digits.to_owned())
            }
            Some((Token::Int(digits, radix), _, _)) => {
                ASTNodeValue::Object(digits.to_owned())
            }
            Some((Token::String(string), _, _)) => {
                ASTNodeValue::Object(string.to_owned())
            }
            // Assignment
            Some((Token::Identifier(name), Some(Token::Equal), _)) => {
                ASTNodeValue::Assignment(name.clone())
            }
            // Reference
            Some((Token::Identifier(name), _, _)) => {
                ASTNodeValue::Reference(name.clone())
            }
            // Binary operation
            Some((Token::Plus, _, _)) => ASTNodeValue::BinaryOperation('+'),
            // Other
            // Some((Token::Indent(n), _, _)) => ASTNodeValue::Indent(*n),
            Some((token, _, _)) => ASTNodeValue::UnknownToken(token.clone()),
            None => panic!("Unexpectedly ran out of tokens"),
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
