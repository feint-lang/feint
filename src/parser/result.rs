use crate::ast;
use crate::scanner::{ScanErr, Token, TokenWithLocation};
use crate::util::Location;

pub type ParseResult = Result<ast::Program, ParseErr>;
pub type StatementsResult = Result<Vec<ast::Statement>, ParseErr>;
pub type ExprResult = Result<ast::Expr, ParseErr>;
pub type ExprOptionResult = Result<Option<ast::Expr>, ParseErr>;
pub type NextTokenResult = Result<Option<TokenWithLocation>, ParseErr>;
pub type NextInfixResult = Result<Option<(TokenWithLocation, u8)>, ParseErr>;
pub type PeekTokenResult<'a> = Result<Option<&'a TokenWithLocation>, ParseErr>;

#[derive(Clone, Debug)]
pub struct ParseErr {
    pub kind: ParseErrKind,
}

impl ParseErr {
    pub fn new(kind: ParseErrKind) -> Self {
        Self { kind }
    }
}

#[derive(Clone, Debug)]
pub enum ParseErrKind {
    ScanError(ScanErr),
    CouldNotOpenSourceFile(String, String),
    UnhandledToken(TokenWithLocation),
    ExpectedExpr(Location),
    ExpectedOperand(Location),
    UnclosedExpr(Location),
    ExpectedIdent(TokenWithLocation),
    SyntaxError(String, Location),
    ExpectedBlock(Location),
    UnexpectedBlock(Location),
    ExpectedToken(Location, Token),
    UnexpectedToken(TokenWithLocation),
}
