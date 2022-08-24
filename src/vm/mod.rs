pub use result::CallDepth;
pub use vm::{DEFAULT_MAX_CALL_DEPTH, VM};

pub(crate) use code::Code;
pub(crate) use context::RuntimeContext;
pub(crate) use inst::Inst;
pub(crate) use result::VMState;
pub(crate) use result::{
    RuntimeBoolResult, RuntimeErr, RuntimeErrKind, RuntimeObjResult, RuntimeResult,
    VMExeResult,
};

mod code;
mod context;
mod inst;
mod result;
mod vm;
