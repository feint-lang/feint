#[derive(Debug)]
pub enum OpCode<'a> {
    Halt(i32, &'a str),
    Constant,
    Jump(usize),
    Return,
}
