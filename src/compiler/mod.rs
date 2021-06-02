pub(crate) use compiler::compile;
pub(crate) use result::{CompilationError, CompilationErrorKind};

mod compiler;
mod result;
mod scope;
