//! Unary and binary operators used in the parser and VM. The operators
//! are split out based on operation and/or return type, which makes it
//! easier to handle different types of operations in the VM.
use std::fmt;

/// Unary operators.
#[derive(Clone, Eq, PartialEq)]
pub enum UnaryOperator {
    Plus,
    Negate,
    Not,
    AsBool,
}

impl UnaryOperator {
    pub fn from_token(token: &str) -> Result<Self, String> {
        let op = match token {
            "+" => Self::Plus,
            "-" => Self::Negate,
            "!" => Self::Not,
            "!!" => Self::AsBool,
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
            Self::Not => "!",
            Self::AsBool => "!!",
        };
        write!(f, "{string}")
    }
}

impl fmt::Debug for UnaryOperator {
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
    pub fn from_token(token: &str) -> Result<Self, String> {
        let op = match token {
            "^" => Self::Pow,
            "*" => Self::Mul,
            "/" => Self::Div,
            "//" => Self::FloorDiv,
            "%" => Self::Mod,
            "+" => Self::Add,
            "-" => Self::Sub,
            "." => Self::Dot,
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
}

impl CompareOperator {
    pub fn from_token(token: &str) -> Result<Self, String> {
        let op = match token {
            "$$" => Self::Is,
            "$!" => Self::IsNot,
            "===" => Self::IsTypeEqual,
            "!==" => Self::IsNotTypeEqual,
            "==" => Self::IsEqual,
            "!=" => Self::NotEqual,
            "<" => Self::LessThan,
            "<=" => Self::LessThanOrEqual,
            ">" => Self::GreaterThan,
            ">=" => Self::GreaterThanOrEqual,
            _ => return Err(format!("Unknown comparison operator: {token}")),
        };
        Ok(op)
    }
}

impl fmt::Display for CompareOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::Is => "$$",
            Self::IsNot => "$!",
            Self::IsTypeEqual => "===",
            Self::IsNotTypeEqual => "!==",
            Self::IsEqual => "==",
            Self::NotEqual => "!=",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
        };
        write!(f, "{string}")
    }
}

impl fmt::Debug for CompareOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

/// Short-circuiting binary comparison operators (operators that return bool).
#[derive(Clone, Eq, PartialEq)]
pub enum ShortCircuitCompareOperator {
    And,
    Or,
    NilOr,
}

impl ShortCircuitCompareOperator {
    pub fn from_token(token: &str) -> Result<Self, String> {
        let op = match token {
            "&&" => Self::And,
            "||" => Self::Or,
            "??" => Self::NilOr,
            _ => {
                return Err(format!(
                    "Unknown short-circuiting comparison operator: {token}"
                ))
            }
        };
        Ok(op)
    }
}

impl fmt::Display for ShortCircuitCompareOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::And => "&&",
            Self::Or => "||",
            Self::NilOr => "??",
        };
        write!(f, "{string}")
    }
}

impl fmt::Debug for ShortCircuitCompareOperator {
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
    pub fn from_token(token: &str) -> Result<Self, String> {
        let op = match token {
            "*=" => Self::Mul,
            "/=" => Self::Div,
            "+=" => Self::Add,
            "-=" => Self::Sub,
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
