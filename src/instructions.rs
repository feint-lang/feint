#[derive(Clone, Debug)]
pub enum Instruction {
    Print(u8),       // Print value(s) at top of stack
    Constant(usize), // ???
    Jump(usize),     // Jump uncoditionally
    JumpIf(usize),   // Jump if top of stack is true
    Return(usize),
    Push(usize),
    Add, // Add top items in stack; push result back
    Halt(i32),
}
