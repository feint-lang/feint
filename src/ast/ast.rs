use std::fmt;
use std::str::FromStr;

use num_bigint::BigInt;

use crate::util::{BinaryOperator, UnaryOperator};

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
}

#[derive(PartialEq)]
pub enum StatementKind {
    Print(Expr),
    Jump(String),
    Label(String),
    Expr(Expr),
}

impl Statement {
    pub fn new(kind: StatementKind) -> Self {
        Self { kind }
    }

    pub fn new_print(expr: Expr) -> Self {
        Self::new(StatementKind::Print(expr))
    }

    pub fn new_jump(name: String) -> Self {
        Self::new(StatementKind::Jump(name))
    }

    pub fn new_label(name: String) -> Self {
        Self::new(StatementKind::Label(name))
    }

    pub fn new_expr(expr: Expr) -> Self {
        Self::new(StatementKind::Expr(expr))
    }

    pub fn tuple_items(&self) -> Option<&Vec<Expr>> {
        if let StatementKind::Expr(expr) = &self.kind {
            return expr.tuple_items();
        }
        None
    }

    pub fn ident_name(&self) -> Option<&String> {
        if let StatementKind::Expr(expr) = &self.kind {
            return expr.ident_name();
        }
        None
    }
}

impl fmt::Debug for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Statement({:?})", self.kind)
    }
}

impl fmt::Debug for StatementKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expr(expr) => write!(f, "Expr({:?})", expr),
            Self::Print(_expr) => write!(f, "Print"),
            Self::Label(label_index) => write!(f, "Label: {}", label_index),
            Self::Jump(label_index) => write!(f, "Jump: {}", label_index),
        }
    }
}

/// Expression - a statement that returns a value.
#[derive(PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
}

#[derive(PartialEq)]
pub enum ExprKind {
    UnaryOp(UnaryOperator, Box<Expr>),
    BinaryOp(Box<Expr>, BinaryOperator, Box<Expr>),
    Block(Block),
    Conditional(Vec<(Expr, Block)>, Option<Block>),
    Func(Func),
    Call(Call),
    Literal(Literal),
    Loop(Box<Expr>, Block),
    Ident(Ident),
    Tuple(Vec<Expr>),
    FormatString(Vec<Expr>),
}

impl Expr {
    pub fn new(kind: ExprKind) -> Self {
        Self { kind }
    }

    pub fn new_block(block: Block) -> Self {
        Self::new(ExprKind::Block(block))
    }

    pub fn new_conditional(
        branches: Vec<(Expr, Block)>,
        default: Option<Block>,
    ) -> Self {
        Self::new(ExprKind::Conditional(branches, default))
    }

    pub fn new_loop(expr: Expr, block: Block) -> Self {
        Self::new(ExprKind::Loop(Box::new(expr), block))
    }

    pub fn new_func(name: String, params: Vec<String>, block: Block) -> Self {
        Self::new(ExprKind::Func(Func::new(name, params, block)))
    }

    pub fn new_call(name: String, args: Vec<Expr>) -> Self {
        Self::new(ExprKind::Call(Call::new(name, args)))
    }

    pub fn new_unary_op(operator: &str, a: Expr) -> Self {
        let operator = match UnaryOperator::from_str(operator) {
            Ok(op) => op,
            Err(err) => panic!("{}", err),
        };
        Self::new(ExprKind::UnaryOp(operator, Box::new(a)))
    }

    pub fn new_binary_op(a: Expr, operator: &str, b: Expr) -> Self {
        let operator = match BinaryOperator::from_str(operator) {
            Ok(op) => op,
            Err(err) => panic!("{}", err),
        };
        Self::new(ExprKind::BinaryOp(Box::new(a), operator, Box::new(b)))
    }

    pub fn new_ident(ident: Ident) -> Self {
        Self::new(ExprKind::Ident(ident))
    }

    pub fn new_literal(literal: Literal) -> Self {
        Self::new(ExprKind::Literal(literal))
    }

    pub fn new_string(value: &str) -> Self {
        Self::new_literal(Literal::new_string(value))
    }

    pub fn new_format_string(items: Vec<Expr>) -> Self {
        Self::new(ExprKind::FormatString(items))
    }

    pub fn new_tuple(items: Vec<Expr>) -> Self {
        Self::new(ExprKind::Tuple(items))
    }

    pub fn tuple_items(&self) -> Option<&Vec<Expr>> {
        if let ExprKind::Tuple(items) = &self.kind {
            return Some(items);
        }
        None
    }

    pub fn ident_name(&self) -> Option<&String> {
        if let ExprKind::Ident(Ident { kind: IdentKind::Ident(name) }) = &self.kind {
            return Some(name);
        }
        None
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

impl fmt::Debug for ExprKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(literal) => write!(f, "{:?}", literal),
            Self::UnaryOp(op, b) => write!(f, "({:?}{:?})", op, b),
            Self::BinaryOp(a, op, b) => write!(f, "({:?} {:?} {:?})", a, op, b),
            Self::Ident(ident) => write!(f, "{:?}", ident),
            Self::Block(block) => write!(f, "{:?}", block),
            Self::Conditional(branches, default) => {
                write!(f, "{branches:?} {default:?}")
            }
            Self::Loop(expr, block) => write!(f, "Loop {expr:?}\n{block:?}"),
            Self::Func(func) => write!(f, "{:?}", func),
            Self::Call(func) => write!(f, "{:?}", func),
            Self::FormatString(items) => write!(f, "{:?}", items),
            Self::Tuple(items) => write!(f, "{:?}", items),
        }
    }
}

/// Block - a list of statements in a new scope.
#[derive(PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

impl Block {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
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

/// Conditional
#[derive(PartialEq)]
pub struct Conditional {
    pub if_expr: Expr,
    pub if_statements: Vec<Statement>,
}

impl Conditional {
    pub fn new(if_expr: Expr, if_statements: Vec<Statement>) -> Self {
        Self { if_expr, if_statements }
    }
}

impl fmt::Debug for Conditional {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Conditional")
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
            "Function {} ({}) ->\n    {}",
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

    pub fn new_float(value: f64) -> Self {
        Self::new(LiteralKind::Float(value))
    }

    pub fn new_int(value: BigInt) -> Self {
        Self::new(LiteralKind::Int(value))
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
            Self::Float(value) => value.to_string(),
            Self::Int(value) => value.to_string(),
            Self::String(value) => value.clone(),
        };
        write!(f, "{}", string)
    }
}

/// Identifiers - names for variables, functions, types, and methods
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
