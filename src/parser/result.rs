use crate::ast;
use crate::scanner::{ScanError, TokenWithLocation};
use crate::util::Location;

pub type ParseResult = Result<ast::Program, ParseError>;
pub type StatementResult = Result<ast::Statement, ParseError>;
pub type StatementsResult = Result<Vec<ast::Statement>, ParseError>;
pub type ExprResult = Result<ast::Expr, ParseError>;
pub type ExprOptionResult = Result<Option<ast::Expr>, ParseError>;
pub type NextTokenResult = Result<Option<TokenWithLocation>, ParseError>;
pub type NextInfixResult = Result<Option<(TokenWithLocation, u8)>, ParseError>;
pub type PeekTokenResult<'a> = Result<Option<&'a TokenWithLocation>, ParseError>;

#[derive(Clone, Debug)]
pub struct ParseError {
    pub kind: ParseErrorKind,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind) -> Self {
        Self { kind }
    }
}

#[derive(Clone, Debug)]
pub enum ParseErrorKind {
    ScanError(ScanError),
    CouldNotOpenSourceFile(String, String),
    UnhandledToken(TokenWithLocation),
    ExpectedExpr(Location),
    UnclosedExpr(Location),
    ExpectedIdent(TokenWithLocation),
    SyntaxError(String, Location),
    ExpectedBlock(Location),
    UnexpectedBlock(Location),
}
