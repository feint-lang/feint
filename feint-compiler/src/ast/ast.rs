use std::fmt;

use num_bigint::BigInt;

use feint_builtins::types::Params;
use feint_util::op::{
    BinaryOperator, CompareOperator, InplaceOperator, ShortCircuitCompareOperator,
    UnaryOperator,
};
use feint_util::source::Location;

use crate::scanner::Token;

/// Module - A self-contained list of statements.
#[derive(PartialEq)]
pub struct Module {
    pub statements: Vec<Statement>,
}

impl Module {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items: Vec<String> =
            self.statements.iter().map(|statement| format!("{statement:?}")).collect();
        write!(f, "{}", items.join(" ; "))
    }
}

/// Statement - a discrete unit of code.
#[derive(Clone, PartialEq)]
pub struct Statement {
    pub kind: StatementKind,
    pub start: Location,
    pub end: Location,
}

#[derive(Clone, PartialEq)]
pub enum StatementKind {
    Break(Expr),
    Continue,
    Import(String, Option<String>),
    Jump(String),
    Label(String, Expr),
    Return(Expr),
    Halt(Expr),
    Print(Expr),
    Debug(Expr),
    Expr(Expr),
}

impl Statement {
    pub fn new(kind: StatementKind, start: Location, end: Location) -> Self {
        Self { kind, start, end }
    }

    pub fn new_break(expr: Expr, start: Location, end: Location) -> Self {
        Self::new(StatementKind::Break(expr), start, end)
    }

    pub fn new_continue(start: Location, end: Location) -> Self {
        Self::new(StatementKind::Continue, start, end)
    }

    pub fn new_import(
        name: String,
        as_name: Option<String>,
        start: Location,
        end: Location,
    ) -> Self {
        Self::new(StatementKind::Import(name, as_name), start, end)
    }

    pub fn new_jump(name: String, start: Location, end: Location) -> Self {
        Self::new(StatementKind::Jump(name), start, end)
    }

    pub fn new_label(name: String, expr: Expr, start: Location, end: Location) -> Self {
        Self::new(StatementKind::Label(name, expr), start, end)
    }

    pub fn new_return(expr: Expr, start: Location, end: Location) -> Self {
        Self::new(StatementKind::Return(expr), start, end)
    }

    pub fn new_halt(expr: Expr, start: Location, end: Location) -> Self {
        Self::new(StatementKind::Halt(expr), start, end)
    }

    pub fn new_print(expr: Expr, start: Location, end: Location) -> Self {
        Self::new(StatementKind::Print(expr), start, end)
    }

    pub fn new_debug(expr: Expr, start: Location, end: Location) -> Self {
        Self::new(StatementKind::Debug(expr), start, end)
    }

    pub fn new_expr(expr: Expr, start: Location, end: Location) -> Self {
        Self::new(StatementKind::Expr(expr), start, end)
    }

    pub fn expr(&self) -> Option<&Expr> {
        if let StatementKind::Expr(expr) = &self.kind {
            Some(expr)
        } else {
            None
        }
    }
}

impl fmt::Debug for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[[ {:?} [{}]:[{}] ]]", self.kind, self.start, self.end)
    }
}

impl fmt::Debug for StatementKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Break(expr) => write!(f, "break {expr:?}"),
            Self::Continue => write!(f, "continue"),
            Self::Import(name, as_name) => {
                if let Some(as_name) = as_name {
                    write!(f, "import {name:?} as {as_name:?}")
                } else {
                    write!(f, "import {name:?}")
                }
            }
            Self::Jump(label_index) => write!(f, "jump: {label_index}",),
            Self::Label(label_index, expr) => {
                write!(f, "label: {label_index} {expr:?}")
            }
            Self::Return(expr) => write!(f, "return {expr:?}"),
            Self::Halt(expr) => write!(f, "$halt {expr:?}"),
            Self::Print(expr) => write!(f, "$print {expr:?}"),
            Self::Debug(expr) => write!(f, "$debug {expr:?}"),
            Self::Expr(expr) => write!(f, "{expr:?}"),
        }
    }
}

/// Expression - a statement that returns a value.
#[derive(Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub start: Location,
    pub end: Location,
}

#[derive(Clone, PartialEq)]
pub enum ExprKind {
    Tuple(Vec<Expr>),
    List(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
    Literal(Literal),
    FormatString(Vec<Expr>),
    Ident(Ident),
    Block(StatementBlock),
    Conditional(Vec<(Expr, StatementBlock)>, Option<StatementBlock>),
    Loop(Box<Expr>, StatementBlock),
    Func(Func),
    Call(Call),
    DeclarationAndAssignment(Box<Expr>, Box<Expr>),
    Assignment(Box<Expr>, Box<Expr>),
    UnaryOp(UnaryOperator, Box<Expr>),
    BinaryOp(Box<Expr>, BinaryOperator, Box<Expr>),
    CompareOp(Box<Expr>, CompareOperator, Box<Expr>),
    ShortCircuitCompareOp(Box<Expr>, ShortCircuitCompareOperator, Box<Expr>),
    InplaceOp(Box<Expr>, InplaceOperator, Box<Expr>),
}

impl Expr {
    pub fn new(kind: ExprKind, start: Location, end: Location) -> Self {
        Self { kind, start, end }
    }

    pub fn new_tuple(items: Vec<Expr>, start: Location, end: Location) -> Self {
        Self::new(ExprKind::Tuple(items), start, end)
    }

    pub fn new_list(items: Vec<Expr>, start: Location, end: Location) -> Self {
        Self::new(ExprKind::List(items), start, end)
    }

    pub fn new_map(entries: Vec<(Expr, Expr)>, start: Location, end: Location) -> Self {
        Self::new(ExprKind::Map(entries), start, end)
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

    pub fn new_always(start: Location, end: Location) -> Self {
        Self::new_literal(Literal::new_always(), start, end)
    }

    pub fn new_ellipsis(start: Location, end: Location) -> Self {
        Self::new_literal(Literal::new_ellipsis(), start, end)
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

    pub fn new_block(block: StatementBlock, start: Location, end: Location) -> Self {
        Self::new(ExprKind::Block(block), start, end)
    }

    pub fn new_conditional(
        branches: Vec<(Expr, StatementBlock)>,
        default: Option<StatementBlock>,
        start: Location,
        end: Location,
    ) -> Self {
        Self::new(ExprKind::Conditional(branches, default), start, end)
    }

    pub fn new_loop(
        expr: Expr,
        block: StatementBlock,
        start: Location,
        end: Location,
    ) -> Self {
        Self::new(ExprKind::Loop(Box::new(expr), block), start, end)
    }

    pub fn new_ident(ident: Ident, start: Location, end: Location) -> Self {
        Self::new(ExprKind::Ident(ident), start, end)
    }

    pub fn new_declaration_and_assignment(
        lhs_expr: Expr,
        value_expr: Expr,
        start: Location,
        end: Location,
    ) -> Self {
        Self::new(
            ExprKind::DeclarationAndAssignment(
                Box::new(lhs_expr),
                Box::new(value_expr),
            ),
            start,
            end,
        )
    }

    pub fn new_assignment(
        lhs_expr: Expr,
        value_expr: Expr,
        start: Location,
        end: Location,
    ) -> Self {
        Self::new(
            ExprKind::Assignment(Box::new(lhs_expr), Box::new(value_expr)),
            start,
            end,
        )
    }

    pub fn new_func(
        params: Params,
        block: StatementBlock,
        start: Location,
        end: Location,
    ) -> Self {
        Self::new(ExprKind::Func(Func::new(params, block)), start, end)
    }

    pub fn new_call(
        callable: Expr,
        args: Vec<Expr>,
        start: Location,
        end: Location,
    ) -> Self {
        Self::new(ExprKind::Call(Call::new(callable, args)), start, end)
    }

    pub fn new_unary_op(
        op_token: &Token,
        a: Expr,
        start: Location,
        end: Location,
    ) -> Self {
        let kind = if let Ok(op) = UnaryOperator::from_token(op_token.as_str()) {
            ExprKind::UnaryOp(op, Box::new(a))
        } else {
            panic!("Unknown unary operator: {op_token}");
        };
        Self::new(kind, start, end)
    }

    pub fn new_binary_op(
        a: Expr,
        op_token: &Token,
        b: Expr,
        start: Location,
        end: Location,
    ) -> Self {
        use ExprKind::{BinaryOp, CompareOp, InplaceOp, ShortCircuitCompareOp};
        let kind = if let Ok(op) = BinaryOperator::from_token(op_token.as_str()) {
            BinaryOp(Box::new(a), op, Box::new(b))
        } else if let Ok(op) = CompareOperator::from_token(op_token.as_str()) {
            CompareOp(Box::new(a), op, Box::new(b))
        } else if let Ok(op) =
            ShortCircuitCompareOperator::from_token(op_token.as_str())
        {
            ShortCircuitCompareOp(Box::new(a), op, Box::new(b))
        } else if let Ok(op) = InplaceOperator::from_token(op_token.as_str()) {
            InplaceOp(Box::new(a), op, Box::new(b))
        } else {
            panic!("Unknown binary operator: {op_token}");
        };
        Self::new(kind, start, end)
    }

    // Expression type checkers ----------------------------------------

    pub fn assignment(&self) -> Option<(&Expr, &Expr)> {
        if let ExprKind::DeclarationAndAssignment(left, right) = &self.kind {
            Some((left, right))
        } else if let ExprKind::Assignment(left, right) = &self.kind {
            Some((left, right))
        } else {
            None
        }
    }

    /// Check if expression is ellipsis.
    pub fn is_ellipsis(&self) -> bool {
        matches!(&self.kind, ExprKind::Literal(Literal { kind: LiteralKind::Ellipsis }))
    }

    /// Check if expression is literal bool (`true` or `false`).
    pub fn is_bool(&self) -> bool {
        matches!(&self.kind, ExprKind::Literal(Literal { kind: LiteralKind::Bool(_) }))
    }

    /// Check if expression is literal `true`.
    pub fn is_true(&self) -> bool {
        matches!(
            &self.kind,
            ExprKind::Literal(Literal { kind: LiteralKind::Bool(true) })
        )
    }

    /// Check if expression is literal `false`.
    pub fn is_false(&self) -> bool {
        matches!(
            &self.kind,
            ExprKind::Literal(Literal { kind: LiteralKind::Bool(false) })
        )
    }

    /// Check if expression is *any* type of identifier. If so, return
    /// its name.
    pub fn ident_name(&self) -> Option<String> {
        self.is_ident()
            .or_else(|| self.is_special_ident())
            .or_else(|| self.is_type_ident())
    }

    /// Check if expression is a regular identifier. If so, return its
    /// name.
    pub fn is_ident(&self) -> Option<String> {
        if let ExprKind::Ident(Ident { kind: IdentKind::Ident(name) }) = &self.kind {
            Some(name.clone())
        } else {
            None
        }
    }

    /// Check if expression is a special identifier. If so, return its
    /// name.
    pub fn is_special_ident(&self) -> Option<String> {
        use IdentKind::SpecialIdent;
        if let ExprKind::Ident(Ident { kind: SpecialIdent(name) }) = &self.kind {
            Some(name.clone())
        } else {
            None
        }
    }

    /// Check if expression is a type identifier. If so, return its
    /// name.
    pub fn is_type_ident(&self) -> Option<String> {
        use IdentKind::TypeIdent;
        if let ExprKind::Ident(Ident { kind: TypeIdent(name) }) = &self.kind {
            Some(name.clone())
        } else {
            None
        }
    }

    /// Check if expression is a function.
    pub fn is_func(&self) -> bool {
        matches!(self.kind, ExprKind::Func(_))
    }

    pub fn tuple_items(&self) -> Option<&Vec<Expr>> {
        if let ExprKind::Tuple(items) = &self.kind {
            Some(items)
        } else {
            None
        }
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(( {:?} [{}]:[{}] ))", self.kind, self.start, self.end)
    }
}

impl fmt::Debug for ExprKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tuple(items) => write!(f, "({items:?})"),
            Self::List(items) => write!(f, "[{items:?}]"),
            Self::Map(entries) => write!(f, "[{entries:?}]"),
            Self::Literal(literal) => write!(f, "{literal:?}"),
            Self::FormatString(items) => write!(f, "{items:?}"),
            Self::Ident(ident) => write!(f, "{ident:?}"),
            Self::DeclarationAndAssignment(ident, expr) => {
                write!(f, "{ident:?} = {expr:?}")
            }
            Self::Assignment(ident, expr) => write!(f, "{ident:?} = {expr:?}"),
            Self::Block(block) => write!(f, "block {block:?}"),
            Self::Conditional(branches, default) => {
                write!(f, "{branches:?} {default:?}")
            }
            Self::Loop(expr, block) => write!(f, "loop {expr:?} {block:?}"),
            Self::Func(func) => write!(f, "{func:?}"),
            Self::Call(func) => write!(f, "{func:?}"),
            Self::UnaryOp(op, a) => write!(f, "({op:?}{a:?})"),
            Self::BinaryOp(a, op, b) => write!(f, "({a:?} {op:?} {b:?})"),
            Self::CompareOp(a, op, b) => write!(f, "({a:?} {op:?} {b:?})"),
            Self::ShortCircuitCompareOp(a, op, b) => write!(f, "({a:?} {op:?} {b:?})"),
            Self::InplaceOp(a, op, b) => write!(f, "({a:?} {op:?} {b:?})"),
        }
    }
}

/// Block - a list of statements in a new scope.
#[derive(Clone, PartialEq)]
pub struct StatementBlock {
    pub statements: Vec<Statement>,
    pub start: Location,
    pub end: Location,
}

impl StatementBlock {
    pub fn new(statements: Vec<Statement>, start: Location, end: Location) -> Self {
        Self { statements, start, end }
    }
}

impl fmt::Debug for StatementBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items: Vec<String> =
            self.statements.iter().map(|statement| format!("{statement:?}")).collect();
        write!(f, "-> {}", items.join(" ; "))
    }
}

/// Function
#[derive(Clone, PartialEq)]
pub struct Func {
    pub params: Params,
    pub block: StatementBlock,
}

impl Func {
    pub fn new(params: Params, block: StatementBlock) -> Self {
        Self { params, block }
    }
}

impl fmt::Debug for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut names = self.params.clone();
        let n_names = names.len();
        if let Some(name) = names.last() {
            if name.is_empty() {
                names[n_names - 1] = "...".to_string();
            }
        }
        let names = names.join(", ");
        write!(f, "func ({names}) {:?}", self.block)
    }
}

/// Call
#[derive(Clone, PartialEq)]
pub struct Call {
    pub callable: Box<Expr>,
    pub args: Vec<Expr>,
}

impl Call {
    pub fn new(callable: Expr, args: Vec<Expr>) -> Self {
        Self { callable: Box::new(callable), args }
    }
}

impl fmt::Debug for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "call ({})", self.args.len())
    }
}

/// Literal - a literal value written in the source code, such as 123,
/// 1.23, or "123".
#[derive(Clone, PartialEq)]
pub struct Literal {
    pub kind: LiteralKind,
}

#[derive(Clone, PartialEq)]
pub enum LiteralKind {
    Nil,
    Bool(bool),
    Always,
    Ellipsis,
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

    pub fn new_always() -> Self {
        Self::new(LiteralKind::Always)
    }

    pub fn new_ellipsis() -> Self {
        Self::new(LiteralKind::Ellipsis)
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
            Self::Always => "@".to_string(),
            Self::Ellipsis => "...".to_string(),
            Self::Int(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
            Self::String(value) => value.clone(),
        };
        write!(f, "{string}")
    }
}

/// Identifiers - names for variables, functions, and types
#[derive(Clone, Eq, PartialEq)]
pub struct Ident {
    pub kind: IdentKind,
}

#[derive(Clone, Eq, PartialEq)]
pub enum IdentKind {
    Ident(String),
    SpecialIdent(String),
    TypeIdent(String),
}

impl Ident {
    pub fn new(kind: IdentKind) -> Self {
        Self { kind }
    }

    /// Extract and return ident name for all ident types.
    pub fn name(&self) -> String {
        let name = match &self.kind {
            IdentKind::Ident(name) => name,
            IdentKind::SpecialIdent(name) => name,
            IdentKind::TypeIdent(name) => name,
        };
        name.to_owned()
    }

    pub fn new_ident(name: String) -> Self {
        Self::new(IdentKind::Ident(name))
    }

    pub fn new_special_ident(name: String) -> Self {
        Self::new(IdentKind::SpecialIdent(name))
    }

    pub fn new_type_ident(name: String) -> Self {
        Self::new(IdentKind::TypeIdent(name))
    }
}

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Debug for IdentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Ident(name) => name,
            Self::SpecialIdent(name) => name,
            Self::TypeIdent(name) => name,
        };
        write!(f, "{name}")
    }
}
