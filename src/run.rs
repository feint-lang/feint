//! # FeInt

use super::parser::{self, ParseError, ParseErrorKind};
use super::result::ExitResult;
use super::scanner::{ScanError, ScanErrorKind, TokenWithLocation};
use super::util::Location;
use super::vm::{ExecutionResult, Instruction, Instructions, Namespace, VMState, VM};

/// Run text source.
pub fn run(source: &str, debug: bool) -> ExitResult {
    let namespace = Namespace::default();
    let vm = VM::new(namespace);
    let mut runner = Runner::new(vm, debug);
    runner.run(source)
}

/// Run source from file.
pub fn run_file(file_path: &str, debug: bool) -> ExitResult {
    let namespace = Namespace::default();
    let vm = VM::new(namespace);
    let mut runner = Runner::new(vm, debug);
    runner.run_file(file_path)
}

struct Runner {
    vm: VM,
    debug: bool,
}

impl Runner {
    fn new(vm: VM, debug: bool) -> Self {
        Runner { vm, debug }
    }

    fn run(&mut self, text: &str) -> ExitResult {
        match parser::parse_text(text, self.debug) {
            Ok(program) => {
                eprintln!("{:?}", program);
                Ok(None)
            }
            Err(err) => {
                eprintln!("{}", text);
                self.handle_err(err)
            }
        }
    }

    fn run_file(&mut self, file_path: &str) -> ExitResult {
        match parser::parse_file(file_path, self.debug) {
            Ok(program) => {
                eprintln!("{:?}", program);
                Ok(None)
            }
            Err(err) => self.handle_err(err),
        }
    }

    fn handle_err(&mut self, err: ParseError) -> ExitResult {
        self.handle_parse_err(err.kind)
    }

    /// Handle parse error.
    fn handle_parse_err(&mut self, kind: ParseErrorKind) -> ExitResult {
        let message = match kind {
            ParseErrorKind::ScanError(err) => {
                return self.handle_scan_err(err.kind, err.location);
            }
            ParseErrorKind::CouldNotOpenSourceFile(path, message) => {
                format!("Could not open source file: {}\n{}", path, message)
            }
            ParseErrorKind::UnhandledToken(token) => {
                let location = token.start;
                let col = location.col;
                let marker = if col == 0 { col } else { col - 1 };
                let token = token.token;
                format!(
                    "{:>width$}^\nParse error: unhandled token at {}: {:?}",
                    "",
                    location,
                    token,
                    width = marker
                )
            }
            err => {
                format!("Unhandled parse error: {:?}", err)
            }
        };
        Err((2, message))
    }

    /// Handle scan error.
    fn handle_scan_err(
        &mut self,
        kind: ScanErrorKind,
        location: Location,
    ) -> ExitResult {
        let line = location.line;
        let col = location.col;
        let marker = col - 1;
        let message = match kind {
            ScanErrorKind::UnexpectedCharacter(c) => {
                format!(
                    "{:>width$}^\nSyntax error: unexpected character at column {}: '{}'",
                    "", col, c, width = marker
                )
            }
            ScanErrorKind::UnterminatedString(_) => {
                format!(
                    "{:>width$}^\nSyntax error: unterminated string literal at line {line}, col {col}",
                    "", line = line, col = col, width = marker
                )
            }
            ScanErrorKind::InvalidIndent(num_spaces) => {
                format!(
                    "{:>width$}^\nSyntax error: invalid indent with {} spaces (should be a multiple of 4)",
                    "", num_spaces, width = marker
                )
            }
            ScanErrorKind::UnexpectedIndent(_) => {
                format!(
                    "{:>width$}^\nSyntax error: unexpected indent",
                    "",
                    width = marker
                )
            }
            ScanErrorKind::WhitespaceAfterIndent
            | ScanErrorKind::UnexpectedWhitespace => {
                format!(
                    "{:>width$}^\nSyntax error: unexpected whitespace",
                    "",
                    width = marker
                )
            }
            err => {
                format!("Unhandled scan error at {}: {:?}", location, err)
            }
        };
        Err((1, message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_text() {
        let source = "x = 1\ny = 2\n1 + 2";
        if let (Ok(_)) = run(source, true) {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
