pub use result::VMState;
pub use result::{CallDepth, RuntimeErr};
pub use vm::{DEFAULT_MAX_CALL_DEPTH, VM};

pub(crate) use code::Code;
pub(crate) use context::ModuleExecutionContext;
pub(crate) use inst::Inst;
pub(crate) use inst::PrintFlags;
pub(crate) use result::{
    RuntimeBoolResult, RuntimeErrKind, RuntimeObjResult, RuntimeResult, VMExeResult,
};

pub(crate) mod globals;

mod code;
mod context;
mod inst;
mod result;
mod vm;
