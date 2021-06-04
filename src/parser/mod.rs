pub(crate) use parser::{parse_file, parse_stdin, parse_text};
pub(crate) use result::{ParseErr, ParseErrKind, ParseResult};

mod parser;
mod precedence;
mod result;

#[cfg(test)]
mod tests;
