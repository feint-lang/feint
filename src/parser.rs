use std::iter::Peekable;
use std::slice::Iter;

use crate::tokens::{Token, TokenWithPosition};

pub struct Parser<'a> {
    stream: Peekable<Iter<'a, TokenWithPosition>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<TokenWithPosition>) -> Parser<'a> {
        Parser {
            stream: tokens.iter().peekable(),
        }
    }

    pub fn parse(&mut self, tokens: &'a Vec<TokenWithPosition>) -> i32 {
        self.stream = tokens.iter().peekable();
        if self.peek().is_none() {
            return 0;
        }
        self.expr()
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

    fn expr(&mut self) -> i32 {
        let mut result = self.term();
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.next();
                    result += self.term();
                }
                Some(Token::Minus) => {
                    self.next();
                    result -= self.term();
                }
                _ => break,
            }
        }
        result
    }

    fn term(&mut self) -> i32 {
        let mut result = self.factor();
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.next();
                    result *= self.factor();
                }
                Some(Token::Slash) => {
                    self.next();
                    result /= self.factor();
                }
                _ => break,
            }
        }
        result
    }

    fn factor(&mut self) -> i32 {
        match self.next() {
            Some(Token::Int(digits)) => digits.parse::<i32>().unwrap(),
            Some(Token::LeftParen) => {
                let result = self.expr();
                match self.next() {
                    Some(Token::RightParen) => (),
                    Some(token) => panic!("Unexpected token: {:?}", token),
                    None => panic!("Unexpected end of expression"),
                }

                result
            }
            Some(Token::String(_)) => 0,
            Some(Token::Identifier(_)) => 0,
            Some(token) => panic!("Unexpected token: {:?}", token),
            None => panic!("Unexpected end of expression"),
        }
    }
}
