pub(crate) use compiler::compile;
pub(crate) use result::{CompilationError, CompilationErrorKind, CompilationResult};

mod compiler;
mod result;
