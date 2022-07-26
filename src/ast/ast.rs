use std::fmt;
use std::str::FromStr;

use num_bigint::BigInt;

use crate::util::{BinaryOperator, Location, UnaryOperator};

/// Program - a list of statements.
#[derive(PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}

impl fmt::Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items: Vec<String> = self
            .statements
            .iter()
            .map(|statement| format!("{:?}", statement))
            .collect();
        write!(f, "{}", items.join("\n"))
    }
}

/// Statement - a logical chunk of code.
#[derive(PartialEq)]
pub struct Statement {
    pub kind: StatementKind,
    pub start: Location,
    pub end: Location,
}

#[derive(PartialEq)]
pub enum StatementKind {
    Jump(String),
    Label(String),
    Continue,
    Expr(Expr),
}

// TODO: Pass correct location from parser to all constructors
impl Statement {
    pub fn new(kind: StatementKind, start: Location, end: Location) -> Self {
        Self { kind, start, end }
    }

    pub fn new_jump(name: String, start: Location, end: Location) -> Self {
        Self::new(StatementKind::Jump(name), start, end)
    }

    pub fn new_label(name: String, start: Location, end: Location) -> Self {
        Self::new(StatementKind::Label(name), start, end)
    }

    pub fn new_continue(start: Location, end: Location) -> Self {
        Self::new(StatementKind::Continue, start, end)
    }

    pub fn new_expr(expr: Expr, start: Location, end: Location) -> Self {
        Self::new(StatementKind::Expr(expr), start, end)
    }
}

impl fmt::Debug for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Statement[{}, {}]({:?})", self.start, self.end, self.kind)
    }
}

impl fmt::Debug for StatementKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expr(expr) => write!(f, "Expr({:?})", expr),
            Self::Label(label_index) => write!(f, "Label: {}", label_index),
            Self::Jump(label_index) => write!(f, "Jump: {}", label_index),
            Self::Continue => write!(f, "Continue"),
        }
    }
}

/// Expression - a statement that returns a value.
#[derive(PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub start: Location,
    pub end: Location,
}

#[derive(PartialEq)]
pub enum ExprKind {
    Tuple(Vec<Expr>),
    Literal(Literal),
    FormatString(Vec<Expr>),
    Ident(Ident),
    Block(Block),
    Conditional(Vec<(Expr, Block)>, Option<Block>),
    Loop(Box<Expr>, Block),
    Break(Box<Expr>),
    Func(Func),
    Call(Call),
    UnaryOp(UnaryOperator, Box<Expr>),
    BinaryOp(Box<Expr>, BinaryOperator, Box<Expr>),
}

impl Expr {
    pub fn new(kind: ExprKind, start: Location, end: Location) -> Self {
        Self { kind, start, end }
    }

    pub fn new_tuple(items: Vec<Expr>, start: Location, end: Location) -> Self {
        Self::new(ExprKind::Tuple(items), start, end)
    }

    fn new_literal(literal: Literal, start: Location, end: Location) -> Self {
        Self::new(ExprKind::Literal(literal), start, end)
    }

    pub fn new_nil(start: Location, end: Location) -> Self {
        Self::new_literal(Literal::new_nil(), start, end)
    }

    pub fn new_true(start: Location, end: Location) -> Self {
        Self::new_literal(Literal::new_bool(true), start, end)
    }

    pub fn new_false(start: Location, end: Location) -> Self {
        Self::new_literal(Literal::new_bool(false), start, end)
    }

    pub fn new_int(value: BigInt, start: Location, end: Location) -> Self {
        Self::new_literal(Literal::new_int(value), start, end)
    }

    pub fn new_float(value: f64, start: Location, end: Location) -> Self {
        Self::new_literal(Literal::new_float(value), start, end)
    }

    pub fn new_string(string: String, start: Location, end: Location) -> Self {
        Self::new_literal(Literal::new_string(string), start, end)
    }

    pub fn new_format_string(items: Vec<Expr>, start: Location, end: Location) -> Self {
        Self::new(ExprKind::FormatString(items), start, end)
    }

    pub fn new_block(block: Block, start: Location, end: Location) -> Self {
        Self::new(ExprKind::Block(block), start, end)
    }

    pub fn new_conditional(
        branches: Vec<(Expr, Block)>,
        default: Option<Block>,
        start: Location,
        end: Location,
    ) -> Self {
        Self::new(ExprKind::Conditional(branches, default), start, end)
    }

    pub fn new_loop(expr: Expr, block: Block, start: Location, end: Location) -> Self {
        Self::new(ExprKind::Loop(Box::new(expr), block), start, end)
    }

    pub fn new_break(expr: Expr, start: Location, end: Location) -> Self {
        Self::new(ExprKind::Break(Box::new(expr)), start, end)
    }

    pub fn new_ident(ident: Ident, start: Location, end: Location) -> Self {
        Self::new(ExprKind::Ident(ident), start, end)
    }

    pub fn new_func(
        name: String,
        params: Vec<String>,
        block: Block,
        start: Location,
        end: Location,
    ) -> Self {
        Self::new(ExprKind::Func(Func::new(name, params, block)), start, end)
    }

    pub fn new_call(
        name: String,
        args: Vec<Expr>,
        start: Location,
        end: Location,
    ) -> Self {
        Self::new(ExprKind::Call(Call::new(name, args)), start, end)
    }

    pub fn new_unary_op(
        operator: &str,
        a: Expr,
        start: Location,
        end: Location,
    ) -> Self {
        let operator = match UnaryOperator::from_str(operator) {
            Ok(op) => op,
            Err(err) => panic!("{}", err),
        };
        Self::new(ExprKind::UnaryOp(operator, Box::new(a)), start, end)
    }

    pub fn new_binary_op(
        a: Expr,
        operator: &str,
        b: Expr,
        start: Location,
        end: Location,
    ) -> Self {
        let operator = match BinaryOperator::from_str(operator) {
            Ok(op) => op,
            Err(err) => panic!("{}", err),
        };
        Self::new(ExprKind::BinaryOp(Box::new(a), operator, Box::new(b)), start, end)
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}] {:?}", self.start, self.end, self.kind)
    }
}

impl fmt::Debug for ExprKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tuple(items) => write!(f, "{:?}", items),
            Self::Literal(literal) => write!(f, "{:?}", literal),
            Self::FormatString(items) => write!(f, "{:?}", items),
            Self::Ident(ident) => write!(f, "{:?}", ident),
            Self::Block(block) => write!(f, "{:?}", block),
            Self::Conditional(branches, default) => {
                write!(f, "{branches:?} {default:?}")
            }
            Self::Loop(expr, block) => write!(f, "loop {expr:?}\n{block:?}"),
            Self::Break(expr) => write!(f, "break {expr:?}"),
            Self::Func(func) => write!(f, "{:?}", func),
            Self::Call(func) => write!(f, "{:?}", func),
            Self::UnaryOp(op, b) => write!(f, "({:?}{:?})", op, b),
            Self::BinaryOp(a, op, b) => write!(f, "({:?} {:?} {:?})", a, op, b),
        }
    }
}

/// Block - a list of statements in a new scope.
#[derive(PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub start: Location,
    pub end: Location,
}

impl Block {
    pub fn new(statements: Vec<Statement>, start: Location, end: Location) -> Self {
        Self { statements, start, end }
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items: Vec<String> = self
            .statements
            .iter()
            .map(|statement| format!("{:?}", statement))
            .collect();
        write!(f, "Block ->\n    {}", items.join("\n    "))
    }
}

/// Function
#[derive(PartialEq)]
pub struct Func {
    pub name: String,
    pub params: Vec<String>,
    pub block: Block,
}

impl Func {
    pub fn new(name: String, params: Vec<String>, block: Block) -> Self {
        Self { name, params, block }
    }
}

impl fmt::Debug for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items: Vec<String> = self
            .block
            .statements
            .iter()
            .map(|statement| format!("{:?}", statement))
            .collect();
        write!(
            f,
            "Func {} ({}) ->\n    {}",
            self.name,
            self.params.join(", "),
            items.join("    \n    ")
        )
    }
}

/// Call
#[derive(PartialEq)]
pub struct Call {
    pub name: String,
    pub args: Vec<Expr>,
}

impl Call {
    pub fn new(name: String, args: Vec<Expr>) -> Self {
        Self { name, args }
    }
}

impl fmt::Debug for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Call {} ({})", self.name, self.args.len())
    }
}

/// Literal - a literal value written in the source code, such as 123,
/// 1.23, or "123".
#[derive(PartialEq)]
pub struct Literal {
    pub kind: LiteralKind,
}

#[derive(PartialEq)]
pub enum LiteralKind {
    Nil,
    Bool(bool),
    Float(f64),
    Int(BigInt),
    String(String),
}

impl Literal {
    pub fn new(kind: LiteralKind) -> Self {
        Self { kind }
    }

    pub fn new_nil() -> Self {
        Self::new(LiteralKind::Nil)
    }

    pub fn new_bool(value: bool) -> Self {
        Self::new(LiteralKind::Bool(value))
    }

    pub fn new_int(value: BigInt) -> Self {
        Self::new(LiteralKind::Int(value))
    }

    pub fn new_float(value: f64) -> Self {
        Self::new(LiteralKind::Float(value))
    }

    pub fn new_string<S: Into<String>>(value: S) -> Self {
        Self::new(LiteralKind::String(value.into()))
    }
}

impl fmt::Debug for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

impl fmt::Debug for LiteralKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::Nil => "nil".to_string(),
            Self::Bool(value) => value.to_string(),
            Self::Int(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
            Self::String(value) => value.clone(),
        };
        write!(f, "{}", string)
    }
}

/// Identifiers - names for variables, functions, and types
#[derive(PartialEq)]
pub struct Ident {
    pub kind: IdentKind,
}

#[derive(PartialEq)]
pub enum IdentKind {
    Ident(String),
    TypeIdent(String),
}

impl Ident {
    pub fn new(kind: IdentKind) -> Self {
        Self { kind }
    }

    pub fn new_ident(name: String) -> Self {
        Self::new(IdentKind::Ident(name))
    }

    pub fn new_type_ident(name: String) -> Self {
        Self::new(IdentKind::TypeIdent(name))
    }
}

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

impl fmt::Debug for IdentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Ident(name) => name,
            Self::TypeIdent(name) => name,
        };
        write!(f, "{}", name)
    }
}
