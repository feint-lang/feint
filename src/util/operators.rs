//! Unary and binary operators used in the parser and VM.

use std::fmt;
use std::str;

/// Unary operators
#[derive(PartialEq)]
pub enum UnaryOperator {
    Plus,
    Negate,
    Not,
}

impl str::FromStr for UnaryOperator {
    type Err = String;

    fn from_str(op: &str) -> Result<Self, Self::Err> {
        let op = match op {
            "+" => Self::Plus,
            "-" => Self::Negate,
            "!" => Self::Not,
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
    Raise,
    Multiply,
    Divide,
    FloorDiv,
    Modulo,
    Add,
    Subtract,
    Equality,
    Assign,
}

impl str::FromStr for BinaryOperator {
    type Err = String;

    fn from_str(op: &str) -> Result<Self, Self::Err> {
        let op = match op {
            "^" => Self::Raise,
            "*" => Self::Multiply,
            "/" => Self::Divide,
            "//" => Self::FloorDiv,
            "%" => Self::Modulo,
            "+" => Self::Add,
            "-" => Self::Subtract,
            "==" => Self::Equality,
            "=" => Self::Assign,
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
            Self::Raise => "^",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::FloorDiv => "//",
            Self::Modulo => "%",
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Assign => "=",
            Self::Equality => "==",
        };
        write!(f, "{}", string)
    }
}

impl fmt::Debug for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
