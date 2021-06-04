pub(crate) use context::RuntimeContext;
pub(crate) use instruction::{Instruction, Instructions};
pub(crate) use result::VMState;
pub(crate) use result::{
    ExecutionResult, RuntimeBoolResult, RuntimeError, RuntimeErrorKind, RuntimeResult,
};
pub(crate) use vm::{execute, execute_file, execute_stdin, execute_text, VM};

mod constants;
mod context;
mod frame;
mod instruction;
mod namespace;
mod result;
mod vm;
