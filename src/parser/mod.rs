pub(crate) use parser::{parse_file, parse_stdin, parse_text};
pub(crate) use result::{ParseError, ParseErrorKind, ParseResult};

mod parser;
mod precedence;
mod result;

#[cfg(test)]
mod tests;
