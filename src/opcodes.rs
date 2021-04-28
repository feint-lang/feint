#[derive(Clone, Debug)]
pub enum OpCode<'a> {
    Constant(usize),     // ???
    Jump(usize),         // Jump uncoditionally
    JumpIf(usize),       // Jump if top of stack is true
    Return(usize),
    Push(usize),
    Add,                 // Add top items in stack; push result back
    Halt(i32),
}
