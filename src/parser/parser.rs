use std::iter::Peekable;
use std::slice::Iter;

use crate::scanner::{self, Token, TokenWithLocation};

type NextOption<'a> = Option<(&'a Token, Option<&'a Token>, Option<&'a Token>)>;

/// Parse tokens and return an AST.
pub fn parse(tokens: &Vec<TokenWithLocation>) {
    let mut parser = Parser::new(tokens);
    parser.parse();
}

/// Scan source, parse tokens, and return an AST.
pub fn parse_from_source(source: &str) {}

struct Parser<'a> {
    stream: Peekable<Iter<'a, TokenWithLocation>>,
    one_ahead_stream: Peekable<Iter<'a, TokenWithLocation>>,
    two_ahead_stream: Peekable<Iter<'a, TokenWithLocation>>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a Vec<TokenWithLocation>) -> Self {
        let stream = tokens.iter().peekable();
        let mut one_ahead_stream = tokens.iter().peekable();
        let mut two_ahead_stream = tokens.iter().peekable();
        one_ahead_stream.next();
        two_ahead_stream.next();
        two_ahead_stream.next();
        let instance = Self { stream, one_ahead_stream, two_ahead_stream };

        instance
    }

    fn parse(&mut self) {
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

    fn expression(&mut self, parent_index: usize) {}
}

#[cfg(test)]
mod tests {
    // All these tests except empty input hang, so they're ignored for
    // the time being.

    use super::*;

    #[test]
    fn parse_empty() {
        parse_from_source("");
        assert!(false);
    }

    #[test]
    fn parse_int() {
        parse_from_source("1");
        assert!(false);
    }

    #[test]
    fn parse_simple_assignment() {
        //      R
        //      |
        //      n=
        //      |
        //      1
        parse_from_source("n = 1");
        assert!(false);
    }

    #[test]
    fn parse_add() {
        //      R
        //      |
        //      +
        //     / \
        //    1   2
        parse_from_source("1 + 2");
        assert!(false);
    }

    #[test]
    fn parse_assign_to_addition() {
        parse_from_source("n = 1 + 2");
        assert!(false);
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
        parse_from_source("a = 1\nb = a + 2\n");
        assert!(false);
    }
}
