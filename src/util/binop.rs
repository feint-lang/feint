use std::fmt;
use std::str;

pub enum BinaryOperator {
    Multiply,
    Divide,
    FloorDiv,
    Modulo,
    Add,
    Subtract,
}

impl str::FromStr for BinaryOperator {
    type Err = String;

    fn from_str(op: &str) -> Result<Self, Self::Err> {
        let op = match op {
            "*" => Self::Multiply,
            "/" => Self::Divide,
            "//" => Self::FloorDiv,
            "%" => Self::Modulo,
            "+" => Self::Add,
            "-" => Self::Subtract,
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
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::FloorDiv => "//",
            Self::Modulo => "%",
            Self::Add => "+",
            Self::Subtract => "-",
        };
        write!(f, "{}", string)
    }
}

impl fmt::Debug for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
