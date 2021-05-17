use std::fmt;

use crate::util::{BinaryOperator, UnaryOperator};

pub type Instructions = Vec<Instruction>;

#[derive(Debug)]
pub enum Instruction {
    Push(usize),
    Pop,
    Jump(usize),        // Jump unconditionally
    JumpIfTrue(usize),  // Jump if top of stack is true
    JumpIfFalse(usize), // Jump if top of stack is false
    UnaryOperation(UnaryOperator),
    BinaryOperation(BinaryOperator),
    Return,
    StoreConst(usize),
    LoadConst(usize),
    Halt(i32),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::StoreConst(v) => format_aligned("STORE_CONST", v),
            Self::LoadConst(v) => format_aligned("LOAD_CONST", v),
            Self::UnaryOperation(operator) => {
                format_aligned("UNARY_OPERATION", operator.to_string())
            }
            Self::BinaryOperation(operator) => {
                format_aligned("BINARY_OPERATION", operator.to_string())
            }
            Self::Return => format!("RETURN"),
            i => format!("{:?}", i),
        };
        write!(f, "{}", string)
    }
}

fn format_aligned<T: fmt::Display>(name: &str, value: T) -> String {
    format!("{: <w$}{: >x$}", name, value, w = 16, x = 4)
}

/// Return a nicely formatted string of instructions.
///
/// NOTE: We can't directly implement Display for Vec<Instruction> and
///       creating a wrapper type just for that purpose seems like a
///       pain.
pub fn format_instructions(instructions: &Instructions) -> String {
    let mut offset = 0;
    instructions
        .iter()
        .map(|instruction| {
            let result =
                format!("{:0>offset_width$} {}", offset, instruction, offset_width = 4);
            offset += 1;
            result
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// This isn't a very useful test...
    fn test_format_instructions() {
        let instructions: Instructions = vec![
            Instruction::StoreConst(1), // value
            Instruction::StoreConst(2), // value
            Instruction::LoadConst(0),  // index
            Instruction::LoadConst(1),  // index
            Instruction::BinaryOperation(BinaryOperator::Add),
            Instruction::Return,
        ];
        let string = format_instructions(&instructions);
        assert_eq!(
            string,
            "\
0000 STORE_CONST        1
0001 STORE_CONST        2
0002 LOAD_CONST         0
0003 LOAD_CONST         1
0004 BINARY_OPERATION   +
0005 RETURN"
        )
    }
}
