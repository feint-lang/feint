use crate::ast;
use crate::scanner::{ScanErr, Token, TokenWithLocation};
use crate::util::Location;

pub type BoolResult = Result<bool, ParseErr>;
pub type ParseResult = Result<ast::Program, ParseErr>;
pub type StatementResult = Result<ast::Statement, ParseErr>;
pub type StatementsResult = Result<Vec<ast::Statement>, ParseErr>;
pub type BlockResult = Result<ast::StatementBlock, ParseErr>;
pub type ExprResult = Result<ast::Expr, ParseErr>;
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

    pub fn loc(&self) -> Location {
        use ParseErrKind::*;
        let loc = match &self.kind {
            MismatchedBracket(loc) => loc,
            SyntaxErr(loc) => loc,
            ExpectedBlock(loc) => loc,
            ExpectedExpr(loc) => loc,
            ExpectedIdent(loc) => loc,
            ExpectedOperand(loc) => loc,
            ExpectedToken(loc, _) => loc,
            UnexpectedBlock(loc) => loc,
            UnexpectedToken(twl) => &twl.start,
            UnexpectedBreak(loc) => loc,
            UnexpectedContinue(loc) => loc,
            UnexpectedReturn(loc) => loc,
            InlineMatchNotAllowed(loc) => loc,
            MatchDefaultMustBeLast(loc) => loc,
            VarArgsMustBeLast(loc) => loc,
            // TODO: Extract from ScanErr?
            ScanErr(_) => return Location::default(),
        };
        *loc
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
    UnexpectedReturn(Location),

    InlineMatchNotAllowed(Location),
    MatchDefaultMustBeLast(Location),

    VarArgsMustBeLast(Location),
}
