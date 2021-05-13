use std::fmt;
use std::fmt::Formatter;

pub type Instructions = Vec<Instruction>;

#[derive(Debug)]
pub enum BinaryOperator {
    Multiply,
    Divide,
    Add,
    Subtract,
}

#[derive(Debug)]
pub enum Instruction {
    Print(u8), // Print up to 256 value(s) at top of stack
    Push(usize),
    PushConst(usize),   // ???
    Jump(usize),        // Jump unconditionally
    JumpIfTrue(usize),  // Jump if top of stack is true
    JumpIfFalse(usize), // Jump if top of stack is false
    BinaryOperation(BinaryOperator),
    Return,
    Add, // Add top two items in stack; push result back
    Halt(i32),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Instruction::*;
        write!(
            f,
            "{}",
            match self {
                Add => format!("ADD"),
                Print(v) => format_aligned("PRINT", v),
                Push(v) => format_aligned("PUSH", v),
                PushConst(v) => format_aligned("PUSH_CONST", v),
                Return => format!("RETURN"),
                i => format!("{:?}", i),
            }
        )
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
            offset += 1;
            format!("{:0>offset_width$} {}", offset, instruction, offset_width = 4)
        })
        .collect::<Vec<String>>()
        .join("\n")
}

mod tests {
    use super::*;

    #[test]
    fn test_format_instructions() {
        let instructions: Instructions = vec![
            Instruction::Push(1),
            Instruction::Push(2),
            Instruction::Add,
            Instruction::Print(1),
        ];
        let string = format_instructions(&instructions);
        assert_eq!(
            string,
            "\
0001 PUSH               1
0002 PUSH               2
0003 ADD
0004 PRINT              1"
        )
    }
}
