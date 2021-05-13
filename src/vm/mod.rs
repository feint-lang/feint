pub(crate) use frame::Frame;
pub(crate) use instruction::{
    format_instructions, BinaryOperator, Instruction, Instructions,
};
pub(crate) use namespace::Namespace;
pub(crate) use result::{ExecuteError, ExecuteErrorKind, ExecuteResult, VMState};
pub(crate) use vm::VM;

mod frame;
mod instruction;
mod namespace;
mod result;
mod vm;
