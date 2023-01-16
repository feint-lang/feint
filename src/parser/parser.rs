//! Parse a stream of tokens into an AST.
use std::collections::VecDeque;
use std::iter::{Iterator, Peekable};

use crate::ast;
use crate::format::FormatStrToken;
use crate::parser::result::StatementResult;
use crate::scanner::{ScanErr, ScanTokenResult, Token, TokenWithLocation};
use crate::source::Location;

use super::precedence::{
    get_binary_precedence, get_unary_precedence, is_right_associative,
};
use super::result::{
    BlockResult, BoolResult, ExprResult, NextInfixResult, NextTokenResult, ParseErr,
    ParseErrKind, ParseResult, PeekTokenResult, StatementsResult,
};

/// Parse tokens and return the resulting AST or error.
pub fn parse_tokens(tokens: Vec<TokenWithLocation>) -> ParseResult {
    let scanner: Vec<ScanTokenResult> = vec![];
    let mut parser = Parser::new(scanner.into_iter());
    parser.lookahead_queue.extend(tokens);
    parser.parse()
}

pub struct Parser<I: Iterator<Item = ScanTokenResult>> {
    current_token: Option<TokenWithLocation>,
    token_stream: Peekable<I>,
    lookahead_queue: VecDeque<TokenWithLocation>,
    statement_level: u32,
    expr_level: u32,
    func_level: u32,
    loop_level: u32,
}

impl<I: Iterator<Item = ScanTokenResult>> Parser<I> {
    pub fn new(token_iter: I) -> Self {
        Self {
            current_token: None,
            token_stream: token_iter.peekable(),
            lookahead_queue: VecDeque::new(),
            statement_level: 0,
            expr_level: 0,
            func_level: 0,
            loop_level: 0,
        }
    }

    // Parse entry point -----------------------------------------------

    /// Parse token stream a produce a module, which is a sequence of
    /// statements.
    pub fn parse(&mut self) -> ParseResult {
        log::trace!("BEGIN MODULE");
        let statements = self.statements()?;
        let module = ast::Module::new(statements);
        log::trace!("END MODULE");
        Ok(module)
    }

    // Grammar ---------------------------------------------------------

    /// Get a list of statements. Collect statements until there's
    /// either no more input or the end of a scope is reached.
    fn statements(&mut self) -> StatementsResult {
        let mut statements = vec![];
        loop {
            if !self.has_tokens()? || self.peek_token_is(&Token::ScopeEnd)? {
                break;
            }
            let statement = self.statement()?;
            statements.push(statement);
        }
        Ok(statements)
    }

    /// Get the next statement (which might be an expression). In
    /// certain cases, multiple statements may be returned (e.g.,
    /// loops).
    fn statement(&mut self) -> StatementResult {
        let level = self.statement_level;
        log::trace!("BEGIN STATEMENT level {level}");
        self.statement_level += 1;
        use Token::{
            Break, Continue, EndOfStatement, Halt, Import, Jump, Label, Print, Return,
        };
        let token = self.expect_next_token()?;
        let start = token.start;
        let statement = match token.token {
            Break => self.break_(start)?,
            Continue => self.continue_(start, token.end)?,
            Import => self.import(start)?,
            Jump => self.jump(start)?,
            Label(name) => self.label(name, start)?,
            Return => self.return_(start)?,
            Halt => self.halt(start)?,
            Print => self.print(start)?,
            _ => {
                self.lookahead_queue.push_front(token);
                let expr = self.expr(0)?;
                log::trace!("STATEMENT EXPR = {expr:?}");
                log::trace!("NEXT TOKEN = {:?}", self.peek_token()?);
                let end = expr.end;
                ast::Statement::new_expr(expr, start, end)
            }
        };
        self.expect_token(&EndOfStatement)?;
        self.statement_level -= 1;
        log::trace!("END STATEMENT level {level}: = {statement:?}");
        Ok(statement)
    }

    /// Handle jump statement.
    fn jump(&mut self, start: Location) -> StatementResult {
        if let Some(ident_token) = self.next_token()? {
            if let Token::Ident(name) = ident_token.token {
                Ok(ast::Statement::new_jump(name, start, ident_token.end))
            } else {
                Err(self.err(ParseErrKind::UnexpectedToken(ident_token)))
            }
        } else {
            Err(self.err(ParseErrKind::ExpectedIdent(self.next_loc())))
        }
    }

    /// Handle label statement.
    fn label(&mut self, name: String, start: Location) -> StatementResult {
        let expr = self.next_expr_or_nil(start)?;
        let end = expr.end;
        Ok(ast::Statement::new_label(name, expr, start, end))
    }

    /// Handle `break`, ensuring it's contained in a `loop`.
    fn break_(&mut self, start: Location) -> StatementResult {
        if self.loop_level == 0 {
            return Err(self.err(ParseErrKind::UnexpectedBreak(start)));
        }
        let expr = self.next_expr_or_nil(start)?;
        let end = expr.end;
        Ok(ast::Statement::new_break(expr, start, end))
    }

    /// Handle `continue`, ensuring it's contained in a `loop`.
    fn continue_(&mut self, start: Location, end: Location) -> StatementResult {
        if self.loop_level == 0 {
            return Err(self.err(ParseErrKind::UnexpectedContinue(start)));
        }
        Ok(ast::Statement::new_continue(start, end))
    }

    /// Handle `return`.
    fn return_(&mut self, start: Location) -> StatementResult {
        if self.func_level == 0 {
            return Err(self.err(ParseErrKind::UnexpectedReturn(start)));
        }
        let expr = self.next_expr_or_nil(start)?;
        let end = expr.end;
        Ok(ast::Statement::new_return(expr, start, end))
    }

    /// Handle `$halt`. Arg should be an int in the u8 range.
    fn halt(&mut self, start: Location) -> StatementResult {
        let expr = self.expr(0)?;
        let end = expr.end;
        Ok(ast::Statement::new_halt(expr, start, end))
    }

    /// Handle `$print`.
    fn print(&mut self, start: Location) -> StatementResult {
        let expr = self.expr(0)?;
        let end = expr.end;
        Ok(ast::Statement::new_print(expr, start, end))
    }

    /// Handle `import`.
    fn import(&mut self, start: Location) -> StatementResult {
        let name_expr = self.expr(0)?;
        if let Some(name) = name_expr.is_ident() {
            let end = name_expr.end;
            Ok(ast::Statement::new_import(name, start, end))
        } else {
            Err(self.err(ParseErrKind::ExpectedIdent(self.loc())))
        }
    }

    /// Get the next expression, possibly recurring to handle nested
    /// expressions, unary & binary expressions, blocks, functions, etc.
    fn expr(&mut self, prec: u8) -> ExprResult {
        let level = self.expr_level;
        log::trace!("BEGIN EXPR level {level}");
        self.expr_level += 1;
        use Token::*;
        let token = self.expect_next_token()?;
        let start = token.start;
        let end = token.end; // Default end location for simple expressions
        let expr = match token.token {
            LParen => {
                let expr = self.parenthesized(start, false)?;
                if self.peek_token_is_func_scope_start()? {
                    self.func(expr, start)?
                } else {
                    expr
                }
            }
            LBracket => self.list(start)?,
            LBrace => self.map(start)?,
            Nil => ast::Expr::new_nil(start, end),
            True => ast::Expr::new_true(start, end),
            False => ast::Expr::new_false(start, end),
            At => ast::Expr::new_always(start, end),
            Ellipsis => ast::Expr::new_ellipsis(start, end),
            Int(value) => ast::Expr::new_int(value, start, end),
            Float(value) => ast::Expr::new_float(value, start, end),
            Str(string) => ast::Expr::new_string(string, start, end),
            FormatStr(tokens) => self.format_string(tokens, start, end)?,
            Block => {
                let block = self.block(ScopeKind::Block, start)?;
                let start = block.start;
                let end = block.end;
                ast::Expr::new_block(block, start, end)
            }
            If => self.conditional(start)?,
            Match => self.match_conditional(start)?,
            Loop => self.loop_(start)?,
            Ident(name) | ConstIdent(name) => {
                ast::Expr::new_ident(ast::Ident::new_ident(name), start, end)
            }
            SpecialIdent(name) => {
                ast::Expr::new_ident(ast::Ident::new_special_ident(name), start, end)
            }
            TypeIdent(name) => {
                ast::Expr::new_ident(ast::Ident::new_type_ident(name), start, end)
            }
            _ => self.expect_unary_expr(&token)?,
        };
        // If the expression is followed by a binary operator, a binary
        // expression will be parsed and returned. Otherwise, the
        // expression will be returned as is.
        let expr = self.maybe_binary_expr(prec, expr)?;
        self.expr_level -= 1;
        log::trace!("END EXPR level {level} = {expr:?}");
        Ok(expr)
    }

    /// Handle parenthesized expressions. Cases:
    ///
    /// 1. A grouped expression such as `(1)` or `(1 + 2)`.
    /// 2. A tuple such as `(1,)` or `(1, 2)`.
    /// 3. One of the above followed by `->`, indicating that the
    ///    parenthesized expression is a function parameter list.
    fn parenthesized(&mut self, start: Location, force_tuple: bool) -> ExprResult {
        let level = self.expr_level;
        log::trace!("BEGIN PARENTHESIZED EXPR level {level}");
        use Token::{Comma, RParen};
        if self.next_token_is(&RParen)? {
            log::trace!("PARENTHESIZED: is empty tuple");
            return Ok(ast::Expr::new_tuple(vec![], start, self.loc()));
        }
        log::trace!("PARENTHESIZED: get first item");
        let first_item = self.expr(0)?;
        let expr = if self.peek_token_is(&Comma)? {
            log::trace!("PARENTHESIZED EXPR is tuple");
            let mut items = vec![first_item];
            loop {
                if self.next_token_is(&RParen)? {
                    break;
                }
                self.expect_token(&Comma)?;
                if self.next_token_is(&RParen)? {
                    break;
                }
                let item = self.expr(0)?;
                items.push(item);
            }
            ast::Expr::new_tuple(items, start, self.loc())
        } else {
            log::trace!("PARENTHESIZED EXPR is single item (not tuple)");
            self.expect_token(&RParen)?;
            if force_tuple {
                ast::Expr::new_tuple(vec![first_item], start, self.loc())
            } else {
                first_item
            }
        };
        log::trace!("END PARENTHESIZED EXPR level {level}");
        Ok(expr)
    }

    fn list(&mut self, start: Location) -> ExprResult {
        use Token::{Comma, RBracket};
        if self.next_token_is(&RBracket)? {
            return Ok(ast::Expr::new_list(vec![], start, self.loc()));
        }
        let first_item = self.expr(0)?;
        let expr = if self.peek_token_is(&Comma)? {
            let mut items = vec![first_item];
            loop {
                if self.next_token_is(&RBracket)? {
                    break;
                }
                self.expect_token(&Comma)?;
                if self.next_token_is(&RBracket)? {
                    break;
                }
                let item = self.expr(0)?;
                items.push(item);
            }
            ast::Expr::new_list(items, start, self.loc())
        } else {
            self.expect_token(&RBracket)?;
            ast::Expr::new_list(vec![first_item], start, self.loc())
        };
        Ok(expr)
    }

    fn map(&mut self, start: Location) -> ExprResult {
        use Token::{Colon, Comma, RBrace};
        if self.next_token_is(&RBrace)? {
            return Ok(ast::Expr::new_map(vec![], start, self.loc()));
        }
        let name = self.expr(0)?;
        self.expect_token(&Colon)?;
        let value = self.expr(0)?;
        let first_entry = (name, value);
        let expr = if self.peek_token_is(&Comma)? {
            let mut entries = vec![first_entry];
            loop {
                if self.next_token_is(&RBrace)? {
                    break;
                }
                self.expect_token(&Comma)?;
                if self.next_token_is(&RBrace)? {
                    break;
                }
                let name = self.expr(0)?;
                self.expect_token(&Colon)?;
                let value = self.expr(0)?;
                entries.push((name, value));
            }
            ast::Expr::new_map(entries, start, self.loc())
        } else {
            self.expect_token(&RBrace)?;
            ast::Expr::new_map(vec![first_entry], start, self.loc())
        };
        Ok(expr)
    }

    /// Handle format strings (AKA $ strings).
    fn format_string(
        &mut self,
        format_string_tokens: Vec<FormatStrToken>,
        start: Location,
        end: Location,
    ) -> ExprResult {
        let mut items = vec![];
        for format_string_token in format_string_tokens {
            match format_string_token {
                FormatStrToken::Str(value) => {
                    // NOTE: Locations aren't correct, but it shouldn't
                    //       matter for string parts.
                    items.push(ast::Expr::new_string(value, start, end));
                }
                FormatStrToken::Expr(tokens) => {
                    let mut adjusted_tokens = vec![];
                    for t in tokens.iter() {
                        let (s, e) = (t.start, t.end);
                        adjusted_tokens.push(TokenWithLocation::new(
                            t.token.clone(),
                            Location::new(start.line + s.line, start.col + s.col),
                            Location::new(end.line + s.line, end.col + e.col),
                        ));
                    }
                    let program = parse_tokens(adjusted_tokens)?;
                    for statement in program.statements {
                        if let ast::StatementKind::Expr(expr) = statement.kind {
                            items.push(expr)
                        } else {
                            return Err(
                                self.err(ParseErrKind::ExpectedExpr(statement.start))
                            );
                        }
                    }
                }
            };
        }
        // TODO: Fix end
        Ok(ast::Expr::new_format_string(items, start, self.next_loc()))
    }

    /// Handle `block ->`, `if <expr> ->`, etc.
    fn block(&mut self, kind: ScopeKind, start: Location) -> BlockResult {
        use ParseErrKind::{ExpectedBlock, ExpectedToken};
        use ScopeKind::*;
        use Token::{
            FuncInlineScopeStart, FuncScopeStart, InlineScopeEnd, InlineScopeStart,
            ScopeEnd, ScopeStart,
        };

        let expected_token = match kind {
            Block => ScopeStart,
            Func => FuncScopeStart,
        };

        let statements = match (kind, self.next_token_token()?) {
            (Block, Some(ScopeStart)) | (Func, Some(FuncScopeStart)) => {
                log::trace!("SUITE BLOCK");
                let statements = self.statements()?;
                if statements.is_empty() {
                    return Err(self.err(ExpectedBlock(self.next_loc())));
                }
                self.expect_token(&ScopeEnd)?;
                statements
            }
            (Block, Some(InlineScopeStart)) | (Func, Some(FuncInlineScopeStart)) => {
                log::trace!("INLINE BLOCK");
                let statement = self.statement()?;
                self.expect_token(&InlineScopeEnd)?;
                vec![statement]
            }
            _ => {
                let err = ExpectedToken(self.next_loc(), expected_token);
                return Err(self.err(err));
            }
        };

        let end = statements[statements.len() - 1].end;
        Ok(ast::StatementBlock::new(statements, start, end))
    }

    /// Handle `if <expr> -> ...`.
    fn conditional(&mut self, start: Location) -> ExprResult {
        use Token::{Else, EndOfStatement, If};
        let mut branches = vec![];
        let mut end;
        let cond = self.expr(0)?;
        let block = self.block(ScopeKind::Block, cond.end)?;
        end = block.end;
        branches.push((cond, block));
        while let true = self.next_tokens_are(vec![&EndOfStatement, &Else, &If])? {
            let cond = self.expr(0)?;
            let block = self.block(ScopeKind::Block, cond.end)?;
            end = block.end;
            branches.push((cond, block))
        }
        let default = match self.next_tokens_are(vec![&EndOfStatement, &Else])? {
            true => {
                let block = self.block(ScopeKind::Block, self.loc())?;
                end = block.end;
                Some(block)
            }
            false => None,
        };
        Ok(ast::Expr::new_conditional(branches, default, start, end))
    }

    /// Handle `match <expr> -> ...`. Inline `match` expressions aren't
    /// supported because they would be too confusing.
    fn match_conditional(&mut self, start: Location) -> ExprResult {
        use ParseErrKind::{
            ExpectedToken, InlineMatchNotAllowed, MatchDefaultMustBeLast,
        };
        use Token::{
            EndOfStatement, EqualEqual, InlineScopeStart, ScopeEnd, ScopeStart, Star,
        };
        let lhs = self.expr(0)?;
        // let lhs = self.expr(0).map_err(|e| self.err({ ExpectedExpr(self.loc()) }))?;
        let mut branches = vec![];
        let mut default = None;
        let mut end = start;
        if self.next_token_is(&ScopeStart)? {
            loop {
                if !self.has_tokens()? || self.peek_token_is(&ScopeEnd)? {
                    break;
                }
                if self.next_token_is(&Star)? {
                    let block = self.block(ScopeKind::Block, start)?;
                    end = block.end;
                    default = Some(block);
                    self.expect_token(&EndOfStatement)?;
                    if !self.peek_token_is(&ScopeEnd)? {
                        return Err(self.err(MatchDefaultMustBeLast(self.next_loc())));
                    }
                    break;
                } else {
                    let rhs = self.expr(0)?;
                    let rhs_end = rhs.end;
                    let cond = ast::Expr::new_binary_op(
                        lhs.clone(),
                        &EqualEqual,
                        rhs,
                        start,
                        rhs_end,
                    );
                    let block = self.block(ScopeKind::Block, start)?;
                    end = block.end;
                    branches.push((cond, block));
                    self.expect_token(&EndOfStatement)?;
                }
            }
            self.expect_token(&ScopeEnd)?;
            Ok(ast::Expr::new_conditional(branches, default, start, end))
        } else if self.next_token_is(&InlineScopeStart)? {
            Err(self.err(InlineMatchNotAllowed(self.next_loc())))
        } else {
            Err(self.err(ExpectedToken(self.next_loc(), ScopeStart)))
        }
    }

    /// Handle `loop -> ...` and `loop <cond> -> ...` (`while` loops).
    /// TODO: Handle `for` loops.
    fn loop_(&mut self, start: Location) -> ExprResult {
        self.loop_level += 1;
        let cond = match self.peek_token_is_scope_start()? {
            true => ast::Expr::new_true(self.next_loc(), self.next_loc()),
            false => self.expr(0)?,
        };
        let block = self.block(ScopeKind::Block, start)?;
        let end = block.end;
        self.loop_level -= 1;
        Ok(ast::Expr::new_loop(cond, block, start, end))
    }

    /// Handle function definition.
    fn func(&mut self, params_expr: ast::Expr, start: Location) -> ExprResult {
        self.func_level += 1;
        log::trace!("FUNC level {}", self.func_level);
        let param_exprs = match params_expr.kind {
            // Function has multiple parameters.
            ast::ExprKind::Tuple(items) => items,
            // Function has a single parameter.
            _ => vec![params_expr],
        };
        let mut params = vec![];
        // Ensure all items are identifiers
        let param_count = param_exprs.len();
        if param_count > 0 {
            let last = param_exprs.len() - 1;
            for (i, item) in param_exprs.iter().enumerate() {
                if item.is_ellipsis() {
                    if i == last {
                        params.push("".to_owned());
                        continue;
                    } else {
                        return Err(self.err(ParseErrKind::VarArgsMustBeLast(start)));
                    }
                }
                if let Some(name) = item.is_ident() {
                    params.push(name);
                } else {
                    return Err(self.err(ParseErrKind::ExpectedIdent(item.start)));
                }
            }
        }
        log::trace!("GET FUNC BLOCK");
        let block = self.block(ScopeKind::Func, start)?;
        let def_end = block.end;
        self.func_level -= 1;
        // NOTE: The name for a func will be set later if the function
        //       is assigned to a var. Since this is done at compile
        //       time, the function will retain its initial name even
        //       if reassigned.
        Ok(ast::Expr::new_func(params, block, start, def_end))
    }

    /// Handle function call.
    fn call(&mut self, callable: ast::Expr, args_start: Location) -> ExprResult {
        let args = self.parenthesized(args_start, true)?;
        let start = callable.start;
        let end = args.end;
        let args = if let ast::ExprKind::Tuple(items) = args.kind {
            items
        } else {
            panic!("Expected args to be a tuple; got {args:?}");
        };
        Ok(ast::Expr::new_call(callable, args, start, end))
    }

    /// The current token should represent a unary operator and should
    /// be followed by an expression.
    fn expect_unary_expr(&mut self, prefix_token: &TokenWithLocation) -> ExprResult {
        use ParseErrKind::{ExpectedOperand, UnexpectedToken};
        let prec = get_unary_precedence(&prefix_token.token);
        if prec == 0 {
            return Err(self.err(UnexpectedToken(prefix_token.clone())));
        }
        if !self.has_tokens()? {
            return Err(self.err(ExpectedOperand(prefix_token.end)));
        }
        let op_token = &prefix_token.token;
        let rhs = self.expr(prec)?;
        let (start, end) = (prefix_token.start, rhs.end);
        Ok(ast::Expr::new_unary_op(op_token, rhs, start, end))
    }

    /// See if the expr is followed by an infix operator. If so, get the
    /// RHS expression and return a binary expression. If not, just
    /// return the original expr.
    fn maybe_binary_expr(&mut self, prec: u8, mut lhs: ast::Expr) -> ExprResult {
        log::trace!("BEGIN maybe binary expr {lhs:?}");
        use ParseErrKind::ExpectedOperand;
        let start = lhs.start;
        loop {
            let next = self.next_infix_token(prec)?;
            if let Some((infix_token, mut infix_prec)) = next {
                if !self.has_tokens()? {
                    return Err(self.err(ExpectedOperand(infix_token.end)));
                }
                // Lower precedence of right-associative operator when
                // fetching its RHS expr.
                if is_right_associative(&infix_token.token) {
                    infix_prec -= 1;
                }
                let op_token = &infix_token.token;
                lhs = match op_token {
                    // Assignment
                    Token::Equal => {
                        log::trace!("BINOP: assignment");
                        log::trace!("ASSIGNMENT: get value expr");
                        let value = self.expr(infix_prec)?;
                        let end = value.end;
                        if lhs.ident_name().is_some() {
                            log::trace!("ASSIGN TO IDENT: {lhs:?} = {value:?}");
                            ast::Expr::new_declaration_and_assignment(
                                lhs, value, start, end,
                            )
                        } else {
                            log::trace!(
                                "ASSIGN TO OTHER (non-ident): {lhs:?} = {value:?}"
                            );
                            ast::Expr::new_assignment(lhs, value, start, end)
                        }
                    }
                    // Call
                    Token::LParen => {
                        log::trace!("BINOP: call {lhs:?}");
                        self.call(lhs, infix_token.start)?
                    }
                    // Binary operation
                    _ => {
                        log::trace!("BINOP: get right-hand side");
                        let rhs = self.expr(infix_prec)?;
                        log::trace!("BINOP: {lhs:?} {op_token} {rhs:?}");
                        let end = rhs.end;
                        ast::Expr::new_binary_op(lhs, op_token, rhs, start, end)
                    }
                }
            } else {
                log::trace!("END maybe binary expr: NOT binary expr");
                break Ok(lhs);
            }
        }
    }

    // Errors ----------------------------------------------------------

    /// Make creating errors a little less tedious.
    fn err(&self, kind: ParseErrKind) -> ParseErr {
        ParseErr::new(kind)
    }

    fn scan_err(&self, err: ScanErr) -> ParseErr {
        self.err(ParseErrKind::ScanErr(err))
    }

    // Tokens ----------------------------------------------------------

    /// Are there any tokens left in the stream?
    fn has_tokens(&mut self) -> BoolResult {
        Ok(self.peek_token()?.is_some())
    }

    /// Consume and return the next token unconditionally. If no tokens
    /// are left, return `None`.
    fn next_token(&mut self) -> NextTokenResult {
        if let Some(t) = self.lookahead_queue.pop_front() {
            self.current_token = Some(t.clone());
            return Ok(Some(t));
        }
        if let Some(result) = self.token_stream.next() {
            return result
                .map(|t| {
                    self.current_token = Some(t.clone());
                    Some(t)
                })
                .map_err(|err| self.scan_err(err));
        }
        Ok(None)
    }

    fn next_token_token(&mut self) -> Result<Option<Token>, ParseErr> {
        if let Some(token_with_location) = self.next_token()? {
            Ok(Some(token_with_location.token))
        } else {
            Ok(None)
        }
    }

    /// Get the next token. If there isn't a next token, panic! This is
    /// used where there *should* be a next token and if there isn't
    /// that indicates an internal logic/processing error.
    fn expect_next_token(&mut self) -> Result<TokenWithLocation, ParseErr> {
        Ok(self.next_token()?.expect("Expected token"))
    }

    /// Expect the next token to be the specified token. If it is,
    /// consume the token and return nothing. If it's not, return an
    /// error.
    fn expect_token(&mut self, token: &Token) -> Result<(), ParseErr> {
        if !self.next_token_is(token)? {
            return Err(
                self.err(ParseErrKind::ExpectedToken(self.loc(), token.clone()))
            );
        }
        Ok(())
    }

    /// Consume the next token and return it if the specified condition
    /// is true. Otherwise, return `None`.
    fn next_token_if(&mut self, func: impl FnOnce(&Token) -> bool) -> NextTokenResult {
        if let Some(t) = self.peek_token()? {
            if func(&t.token) {
                return self.next_token();
            }
        }
        Ok(None)
    }

    /// Consume next token and return true *if* the next token is equal
    /// to specified token. Otherwise, leave the token in the stream and
    /// return false.
    fn next_token_is(&mut self, token: &Token) -> BoolResult {
        if (self.next_token_if(|t| t == token)?).is_some() {
            return Ok(true);
        }
        Ok(false)
    }

    /// Consume next N tokens and return true *if* the next N tokens are
    /// equal to specified tokens. Otherwise, leave the tokens in the
    /// stream and return false.
    fn next_tokens_are(&mut self, tokens: Vec<&Token>) -> BoolResult {
        assert!(!tokens.is_empty(), "At least one token is required");
        let mut temp_queue: VecDeque<TokenWithLocation> = VecDeque::new();
        for token in tokens {
            match self.next_token_if(|t| t == token)? {
                Some(token) => {
                    temp_queue.push_front(token);
                }
                None => {
                    for twl in temp_queue {
                        self.lookahead_queue.push_front(twl);
                    }
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// Return the next token along with its precedence *if* it's both
    /// an infix operator *and* its precedence is greater than the
    /// current precedence level.
    fn next_infix_token(&mut self, current_prec: u8) -> NextInfixResult {
        if let Some(token) = self.next_token_if(|t| {
            let p = get_binary_precedence(t);
            p > 0 && p > current_prec
        })? {
            let prec = get_binary_precedence(&token.token);
            return Ok(Some((token, prec)));
        }
        Ok(None)
    }

    /// Return the next token without consuming it. If no tokens are
    /// left, return `None`.
    fn peek_token(&mut self) -> PeekTokenResult {
        if let Some(t) = self.lookahead_queue.front() {
            return Ok(Some(t));
        }
        if let Some(result) = self.token_stream.peek() {
            return result
                .as_ref()
                .map(Some)
                .map_err(|err| ParseErr::new(ParseErrKind::ScanErr(err.clone())));
        }
        Ok(None)
    }

    /// Look at the next token and return it if it's equal to the
    /// specified token. Otherwise, return `None`.
    fn peek_token_if(&mut self, func: impl FnOnce(&Token) -> bool) -> PeekTokenResult {
        if let Some(t) = self.peek_token()? {
            if func(&t.token) {
                return Ok(Some(t));
            }
        }
        Ok(None)
    }

    /// Look at the next token and return true if it's equal to the
    /// specified token. Otherwise, return false.
    fn peek_token_is(&mut self, token: &Token) -> BoolResult {
        if (self.peek_token_if(|t| t == token)?).is_some() {
            return Ok(true);
        }
        Ok(false)
    }

    /// Check whether the next token is a block or inline scope start
    /// token.
    fn peek_token_is_scope_start(&mut self) -> BoolResult {
        use Token::{InlineScopeStart, ScopeStart};
        if let Some(TokenWithLocation { token, .. }) = self.peek_token()? {
            Ok(token == &ScopeStart || token == &InlineScopeStart)
        } else {
            Ok(false)
        }
    }

    fn peek_token_is_func_scope_start(&mut self) -> BoolResult {
        use Token::{FuncInlineScopeStart, FuncScopeStart};
        if let Some(TokenWithLocation { token, .. }) = self.peek_token()? {
            Ok(token == &FuncScopeStart || token == &FuncInlineScopeStart)
        } else {
            Ok(false)
        }
    }

    /// Get the next expression if available, otherwise return a nil
    /// expression.
    fn next_expr_or_nil(&mut self, start: Location) -> ExprResult {
        let expr = match self.peek_token()? {
            Some(TokenWithLocation { token: Token::EndOfStatement, .. }) | None => {
                ast::Expr::new_nil(start, start)
            }
            _ => self.expr(0)?,
        };
        Ok(expr)
    }

    /// Get location of current token.
    fn loc(&self) -> Location {
        match &self.current_token {
            Some(t) => t.start,
            None => Location::new(0, 0),
        }
    }

    /// Get location after current token.
    fn next_loc(&self) -> Location {
        if let Some(t) = &self.current_token {
            match t.token {
                Token::EndOfStatement => Location::new(t.end.line + 1, 1),
                _ => Location::new(t.end.line, t.end.col + 1),
            }
        } else {
            Location::new(0, 0)
        }
    }
}

// Scope ---------------------------------------------------------------

enum ScopeKind {
    Block,
    Func,
}
