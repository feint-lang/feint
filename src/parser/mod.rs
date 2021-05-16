pub(crate) use parser::{parse_file, parse_text};
pub(crate) use result::{ParseError, ParseErrorKind, ParseResult};

mod parser;
mod result;
