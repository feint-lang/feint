pub(crate) use compiler::compile;
pub(crate) use result::{CompilationErr, CompilationErrKind};

mod compiler;
mod result;
mod scope;
