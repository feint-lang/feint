pub(crate) use arena::ObjectStore;
pub(crate) use frame::Frame;
pub(crate) use instruction::{format_instructions, Instruction, Instructions};
pub(crate) use namespace::Namespace;
pub(crate) use result::{ExecutionError, ExecutionErrorKind, ExecutionResult, VMState};
pub(crate) use vm::VM;

mod arena;
mod frame;
mod instruction;
mod namespace;
mod result;
mod vm;
