pub enum OpCode {
    Constant,
    Jump { target: usize },
    Return,
}
