use std::fmt;
use std::str::FromStr;

use num_bigint::BigInt;

use crate::util::BinaryOperator;

/// Program - a list of statements.
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
        for s in self.statements.iter() {
            write!(f, "{:?}\n", s)?;
        }
        write!(f, "")
    }
}

/// Statement - a logical chunk of code.
pub struct Statement {
    pub kind: StatementKind,
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

pub enum StatementKind {
    Expression(Box<Expression>),
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
pub struct Expression {
    pub kind: ExpressionKind,
}

impl Expression {
    pub fn new(kind: ExpressionKind) -> Self {
        Self { kind }
    }

    pub fn new_binary_operation(operator: &str, a: Expression, b: Expression) -> Self {
        let operator = match BinaryOperator::from_str(operator) {
            Ok(op) => op,
            Err(err) => panic!("{}", err),
        };
        Self {
            kind: ExpressionKind::BinaryOperation(operator, Box::new(a), Box::new(b)),
        }
    }

    pub fn new_literal(literal: Literal) -> Self {
        Self { kind: ExpressionKind::Literal(Box::new(literal)) }
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

pub enum ExpressionKind {
    BinaryOperation(BinaryOperator, Box<Expression>, Box<Expression>),
    Block(Box<Block>),
    Function(String, Box<Block>),
    Literal(Box<Literal>),
}

impl fmt::Debug for ExpressionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BinaryOperation(op, a, b) => {
                write!(f, "{:?} {:?} {:?}", a, op, b)
            }
            Self::Literal(literal) => {
                write!(f, "{:?}", literal)
            }
            kind => write!(f, "{:?}", kind),
        }
    }
}

/// Block - list of statements in a new scope.
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
pub struct Literal {
    pub kind: LiteralKind,
}

impl Literal {
    pub fn new(kind: LiteralKind) -> Self {
        Self { kind }
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

pub enum LiteralKind {
    Float(f64),
    Int(BigInt),
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
