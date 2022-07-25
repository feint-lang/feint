pub(crate) use compiler::compile;
pub(crate) use result::{CompErr, CompErrKind};

mod compiler;
mod result;
mod scope;
