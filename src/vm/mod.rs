pub use result::CallDepth;
pub use vm::{DEFAULT_MAX_CALL_DEPTH, VM};

pub(crate) use context::RuntimeContext;
pub(crate) use inst::{Chunk, Inst};
pub(crate) use result::VMState;
pub(crate) use result::{
    RuntimeBoolResult, RuntimeErr, RuntimeErrKind, RuntimeObjResult,
};

mod constants;
mod context;
mod inst;
mod result;
mod vm;
