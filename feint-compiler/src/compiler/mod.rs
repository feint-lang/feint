pub use compiler::Compiler;
pub use result::{CompErr, CompErrKind};

mod compiler;
mod result;
mod scope;
mod visitor;
