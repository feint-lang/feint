//! Unary and binary operators used in the parser and VM.

use std::fmt;
use std::str;

/// Unary operators
#[derive(PartialEq)]
pub enum UnaryOperator {
    Plus,
    Negate,
    Not,
    AsBool,
}

impl str::FromStr for UnaryOperator {
    type Err = String;

    fn from_str(op: &str) -> Result<Self, Self::Err> {
        let op = match op {
            "+" => Self::Plus,
            "-" => Self::Negate,
            "!" => Self::Not,
            "!!" => Self::AsBool,
            _ => {
                return Err(format!("Unknown unary operator: \"{}\"", op));
            }
        };
        Ok(op)
    }
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::Plus => "+",
            Self::Negate => "-",
            Self::Not => "!",
            Self::AsBool => "!!",
        };
        write!(f, "{}", string)
    }
}

impl fmt::Debug for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Binary operators
#[derive(PartialEq)]
pub enum BinaryOperator {
    Pow,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Add,
    AddEqual,
    Sub,
    SubEqual,
    Is,
    IsEqual,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
    Assign,
    Comma,
    Dot,
}

impl str::FromStr for BinaryOperator {
    type Err = String;

    fn from_str(op: &str) -> Result<Self, Self::Err> {
        let op = match op {
            "^" => Self::Pow,
            "*" => Self::Mul,
            "/" => Self::Div,
            "//" => Self::FloorDiv,
            "%" => Self::Mod,
            "+" => Self::Add,
            "+=" => Self::AddEqual,
            "-" => Self::Sub,
            "-=" => Self::SubEqual,
            "===" => Self::Is,
            "==" => Self::IsEqual,
            "!=" => Self::NotEqual,
            "<" => Self::LessThan,
            "<=" => Self::LessThanOrEqual,
            ">" => Self::GreaterThan,
            ">=" => Self::GreaterThanOrEqual,
            "&&" => Self::And,
            "||" => Self::Or,
            "=" => Self::Assign,
            "," => Self::Comma,
            "." => Self::Dot,
            _ => {
                return Err(format!("Unknown binary operator: {}", op));
            }
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
            Self::AddEqual => "+=",
            Self::Sub => "-",
            Self::SubEqual => "-=",
            Self::Is => "===",
            Self::IsEqual => "==",
            Self::NotEqual => "!=",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
            Self::And => "&&",
            Self::Or => "||",
            Self::Assign => "=",
            Self::Comma => ",",
            Self::Dot => ".",
        };
        write!(f, "{}", string)
    }
}

impl fmt::Debug for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
