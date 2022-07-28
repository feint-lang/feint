pub(crate) use parser::Parser;
pub(crate) use result::{ParseErr, ParseErrKind, ParseResult};

mod parser;
mod precedence;
mod result;
