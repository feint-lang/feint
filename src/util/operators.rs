//! Unary and binary operators used in the parser and VM. The operators
//! are split out based on operation and/or return type, which makes it
//! easier to handle different types of operations in the VM.

use std::fmt;
use std::str;

/// Unary operators.
#[derive(Clone, PartialEq)]
pub enum UnaryOperator {
    Plus,
    Negate,
}

impl str::FromStr for UnaryOperator {
    type Err = String;

    fn from_str(op: &str) -> Result<Self, Self::Err> {
        let op = match op {
            "+" => Self::Plus,
            "-" => Self::Negate,
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
        };
        write!(f, "{}", string)
    }
}

impl fmt::Debug for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Unary comparison operators.
#[derive(Clone, PartialEq)]
pub enum UnaryCompareOperator {
    Not,
    AsBool,
}

impl str::FromStr for UnaryCompareOperator {
    type Err = String;

    fn from_str(op: &str) -> Result<Self, Self::Err> {
        let op = match op {
            "!" => Self::Not,
            "!!" => Self::AsBool,
            _ => {
                return Err(format!("Unknown unary comparison operator: \"{}\"", op));
            }
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
        write!(f, "{}", string)
    }
}

impl fmt::Debug for UnaryCompareOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Binary operators.
#[derive(Clone, PartialEq)]
pub enum BinaryOperator {
    Pow,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Add,
    Sub,
    Assign,
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
            "-" => Self::Sub,
            "=" => Self::Assign,
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
            Self::Sub => "-",
            Self::Assign => "=",
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

/// Binary comparison operators (operators that return bool).
#[derive(Clone, PartialEq)]
pub enum CompareOperator {
    Is,
    IsEqual,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
}

impl str::FromStr for CompareOperator {
    type Err = String;

    fn from_str(op: &str) -> Result<Self, Self::Err> {
        let op = match op {
            "===" => Self::Is,
            "==" => Self::IsEqual,
            "!=" => Self::NotEqual,
            "<" => Self::LessThan,
            "<=" => Self::LessThanOrEqual,
            ">" => Self::GreaterThan,
            ">=" => Self::GreaterThanOrEqual,
            "&&" => Self::And,
            "||" => Self::Or,
            _ => {
                return Err(format!("Unknown comparison operator: {}", op));
            }
        };
        Ok(op)
    }
}

impl fmt::Display for CompareOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::Is => "===",
            Self::IsEqual => "==",
            Self::NotEqual => "!=",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
            Self::And => "&&",
            Self::Or => "||",
        };
        write!(f, "{}", string)
    }
}

impl fmt::Debug for CompareOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

/// Inplace binary operators (e.g. +=)
#[derive(Clone, PartialEq)]
pub enum InplaceOperator {
    AddEqual,
    SubEqual,
}

impl str::FromStr for InplaceOperator {
    type Err = String;

    fn from_str(op: &str) -> Result<Self, Self::Err> {
        let op = match op {
            "+=" => Self::AddEqual,
            "-=" => Self::SubEqual,
            _ => {
                return Err(format!("Unknown inplace operator: {}", op));
            }
        };
        Ok(op)
    }
}

impl fmt::Display for InplaceOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::AddEqual => "+=",
            Self::SubEqual => "-=",
        };
        write!(f, "{}", string)
    }
}

impl fmt::Debug for InplaceOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
