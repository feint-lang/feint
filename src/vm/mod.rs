pub(crate) use frame::Frame;
pub(crate) use instruction::Instruction;
pub(crate) use namespace::Namespace;
pub(crate) use vm::{VMState, VM};

mod frame;
mod instruction;
mod namespace;
mod vm;
