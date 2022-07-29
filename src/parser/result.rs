use crate::ast;
use crate::scanner::{ScanErr, Token, TokenWithLocation};
use crate::util::Location;

pub type BoolResult = Result<bool, ParseErr>;
pub type ParseResult = Result<ast::Program, ParseErr>;
pub type StatementResult = Result<ast::Statement, ParseErr>;
pub type StatementsResult = Result<Vec<ast::Statement>, ParseErr>;
pub type BlockResult = Result<ast::StatementBlock, ParseErr>;
pub type ExprResult = Result<ast::Expr, ParseErr>;
pub type MaybeExprResult = Result<(bool, ast::Expr), ParseErr>;
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
    ScanErr(ScanErr),

    MismatchedBracket(Location),

    /// Generic syntax error
    SyntaxErr(Location),

    ExpectedBlock(Location),
    ExpectedExpr(Location),
    ExpectedIdent(Location),
    ExpectedOperand(Location),
    ExpectedToken(Location, Token),

    UnexpectedBlock(Location),
    UnexpectedToken(TokenWithLocation),

    UnexpectedBreak(Location),
    UnexpectedContinue(Location),
}
