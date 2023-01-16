//! Unary and binary operators used in the parser and VM. The operators
//! are split out based on operation and/or return type, which makes it
//! easier to handle different types of operations in the VM.
use std::fmt;

use crate::scanner::Token;

/// Unary operators.
#[derive(Clone, Eq, PartialEq)]
pub enum UnaryOperator {
    Plus,
    Negate,
}

impl UnaryOperator {
    pub fn from_token(token: &Token) -> Result<Self, String> {
        let op = match token {
            Token::Plus => Self::Plus,
            Token::Minus => Self::Negate,
            _ => return Err(format!("Unknown unary operator: {token}")),
        };
        Ok(op)
    }
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::Plus => "+",
            Self::Negate => "-",
        };
        write!(f, "{string}")
    }
}

impl fmt::Debug for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

/// Unary comparison operators.
#[derive(Clone, Eq, PartialEq)]
pub enum UnaryCompareOperator {
    Not,
    AsBool,
}

impl UnaryCompareOperator {
    pub fn from_token(token: &Token) -> Result<Self, String> {
        let op = match token {
            Token::Bang => Self::Not,
            Token::BangBang => Self::AsBool,
            _ => return Err(format!("Unknown unary comparison operator: {token}")),
        };
        Ok(op)
    }
}

impl fmt::Display for UnaryCompareOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::Not => "!",
            Self::AsBool => "!!",
        };
        write!(f, "{string}")
    }
}

impl fmt::Debug for UnaryCompareOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

/// Binary operators.
#[derive(Clone, Eq, PartialEq)]
pub enum BinaryOperator {
    Pow,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Add,
    Sub,
    Dot,
}

impl BinaryOperator {
    pub fn from_token(token: &Token) -> Result<Self, String> {
        let op = match token {
            Token::Caret => Self::Pow,
            Token::Star => Self::Mul,
            Token::Slash => Self::Div,
            Token::DoubleSlash => Self::FloorDiv,
            Token::Percent => Self::Mod,
            Token::Plus => Self::Add,
            Token::Minus => Self::Sub,
            Token::Dot => Self::Dot,
            _ => return Err(format!("Unknown binary operator: {token}")),
        };
        Ok(op)
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::Pow => "^",
            Self::Mul => "*",
            Self::Div => "/",
            Self::FloorDiv => "//",
            Self::Mod => "%",
            Self::Add => "+",
            Self::Sub => "-",
            Self::Dot => ".",
        };
        write!(f, "{string}")
    }
}

impl fmt::Debug for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

/// Binary comparison operators (operators that return bool).
#[derive(Clone, Eq, PartialEq)]
pub enum CompareOperator {
    Is,
    IsNot,
    IsTypeEqual,
    IsNotTypeEqual,
    IsEqual,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
}

impl CompareOperator {
    pub fn from_token(token: &Token) -> Result<Self, String> {
        let op = match token {
            Token::DollarDollar => Self::Is,
            Token::DollarNot => Self::IsNot,
            Token::EqualEqualEqual => Self::IsTypeEqual,
            Token::NotEqualEqual => Self::IsNotTypeEqual,
            Token::EqualEqual => Self::IsEqual,
            Token::NotEqual => Self::NotEqual,
            Token::LessThan => Self::LessThan,
            Token::LessThanOrEqual => Self::LessThanOrEqual,
            Token::GreaterThan => Self::GreaterThan,
            Token::GreaterThanOrEqual => Self::GreaterThanOrEqual,
            Token::And => Self::And,
            Token::Or => Self::Or,
            _ => return Err(format!("Unknown comparison operator: {token}")),
        };
        Ok(op)
    }
}

impl fmt::Display for CompareOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::Is => "$",
            Self::IsNot => "$!",
            Self::IsTypeEqual => "===",
            Self::IsNotTypeEqual => "!==",
            Self::IsEqual => "==",
            Self::NotEqual => "!=",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
            Self::And => "&&",
            Self::Or => "||",
        };
        write!(f, "{string}")
    }
}

impl fmt::Debug for CompareOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

/// Inplace binary operators (e.g. +=)
#[derive(Clone, Eq, PartialEq)]
pub enum InplaceOperator {
    Mul,
    Div,
    Add,
    Sub,
}

impl InplaceOperator {
    pub fn from_token(token: &Token) -> Result<Self, String> {
        let op = match token {
            Token::MulEqual => Self::Mul,
            Token::DivEqual => Self::Div,
            Token::PlusEqual => Self::Add,
            Token::MinusEqual => Self::Sub,
            _ => return Err(format!("Unknown inplace operator: {token}")),
        };
        Ok(op)
    }
}

impl fmt::Display for InplaceOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::Mul => "*=",
            Self::Div => "/=",
            Self::Add => "+=",
            Self::Sub => "-=",
        };
        write!(f, "{string}")
    }
}

impl fmt::Debug for InplaceOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
