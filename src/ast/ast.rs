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
    Expression(Box<Expression>),
}

impl Statement {
    pub fn new(kind: StatementKind) -> Self {
        Self { kind }
    }

    pub fn new_expression(expression: Expression) -> Self {
        Self { kind: StatementKind::Expression(Box::new(expression)) }
    }
}

impl fmt::Debug for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

impl fmt::Debug for StatementKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expression(expression) => {
                write!(f, "({:?})", expression)
            }
        }
    }
}

/// Expression - a statement that returns a value.
#[derive(PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
}

#[derive(PartialEq)]
pub enum ExpressionKind {
    UnaryOperation(UnaryOperator, Box<Expression>),
    BinaryOperation(Box<Expression>, BinaryOperator, Box<Expression>),
    Block(Box<Block>),
    Function(String, Box<Block>),
    Literal(Box<Literal>),
    Identifier(Box<Identifier>),
}

impl Expression {
    pub fn new(kind: ExpressionKind) -> Self {
        Self { kind }
    }

    pub fn new_literal(literal: Literal) -> Self {
        Self { kind: ExpressionKind::Literal(Box::new(literal)) }
    }

    pub fn new_identifier(identifier: Identifier) -> Self {
        Self { kind: ExpressionKind::Identifier(Box::new(identifier)) }
    }

    pub fn new_unary_operation(operator: &str, a: Expression) -> Self {
        let operator = match UnaryOperator::from_str(operator) {
            Ok(op) => op,
            Err(err) => panic!("{}", err),
        };
        Self { kind: ExpressionKind::UnaryOperation(operator, Box::new(a)) }
    }

    pub fn new_binary_operation(a: Expression, operator: &str, b: Expression) -> Self {
        let operator = match BinaryOperator::from_str(operator) {
            Ok(op) => op,
            Err(err) => panic!("{}", err),
        };
        Self {
            kind: ExpressionKind::BinaryOperation(Box::new(a), operator, Box::new(b)),
        }
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

impl fmt::Debug for ExpressionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnaryOperation(op, b) => write!(f, "({:?}{:?})", op, b),
            Self::BinaryOperation(a, op, b) => write!(f, "({:?} {:?} {:?})", a, op, b),
            Self::Literal(literal) => write!(f, "{:?}", literal),
            Self::Identifier(identifier) => write!(f, "{:?}", identifier),
            _ => unimplemented!(),
        }
    }
}

/// Block - list of statements in a new scope.
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
        write!(f, "{:?}", self.statements)
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
    Float(f64),
    Int(BigInt),
}

impl Literal {
    pub fn new(kind: LiteralKind) -> Self {
        Self { kind }
    }

    pub fn new_float(value: f64) -> Self {
        Self { kind: LiteralKind::Float(value) }
    }

    pub fn new_int(value: BigInt) -> Self {
        Self { kind: LiteralKind::Int(value) }
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
            Self::Float(value) => value.to_string(),
            Self::Int(value) => value.to_string(),
        };
        write!(f, "{}", string)
    }
}

/// Identifiers - names for variables, functions, types, and methods
#[derive(PartialEq)]
pub struct Identifier {
    pub kind: IdentifierKind,
}

#[derive(PartialEq)]
pub enum IdentifierKind {
    Identifier(String),
    TypeIdentifier(String),
}

impl Identifier {
    pub fn new(kind: IdentifierKind) -> Self {
        Self { kind }
    }

    pub fn new_identifier(name: String) -> Self {
        Self { kind: IdentifierKind::Identifier(name) }
    }

    pub fn new_type_identifier(name: String) -> Self {
        Self { kind: IdentifierKind::TypeIdentifier(name) }
    }
}

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

impl fmt::Debug for IdentifierKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Identifier(name) => name,
            Self::TypeIdentifier(name) => name,
        };
        write!(f, "{}", name)
    }
}
