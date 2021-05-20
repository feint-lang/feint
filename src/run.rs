//! # FeInt

use crate::parser::{self, ParseError, ParseErrorKind};
use crate::result::ExitResult;
use crate::scanner::{ScanError, ScanErrorKind, TokenWithLocation};
use crate::util::Location;
use crate::vm::{
    execute_file, execute_text, CompilationErrorKind, ExecutionErrorKind, VMState,
};

/// Run text source.
pub fn run(text: &str, debug: bool) -> ExitResult {
    let mut runner = Runner::new(debug);
    runner.run(text)
}

/// Run source from file.
pub fn run_file(file_path: &str, debug: bool) -> ExitResult {
    let mut runner = Runner::new(debug);
    runner.run_file(file_path)
}

struct Runner {
    debug: bool,
}

impl Runner {
    fn new(debug: bool) -> Self {
        Runner { debug }
    }

    fn run(&mut self, text: &str) -> ExitResult {
        match execute_text(text, self.debug) {
            Ok(vm_state) => self.vm_state_to_exit_result(vm_state),
            Err(err) => self.handle_execution_err(err.kind),
        }
    }

    fn run_file(&mut self, file_path: &str) -> ExitResult {
        match execute_file(file_path, self.debug) {
            Ok(vm_state) => self.vm_state_to_exit_result(vm_state),
            Err(err) => self.handle_execution_err(err.kind),
        }
    }

    fn vm_state_to_exit_result(&self, vm_state: VMState) -> ExitResult {
        if self.debug {
            eprintln!("{:?}", vm_state);
        }
        match vm_state {
            VMState::Halted(0) => Ok(None),
            VMState::Halted(code) => {
                Err((code, format!("Halted abnormally: {}", code)))
            }
            VMState::Idle => Err((-1, "Never halted".to_owned())),
        }
    }

    fn handle_execution_err(&mut self, kind: ExecutionErrorKind) -> ExitResult {
        let message = match kind {
            ExecutionErrorKind::CompilationError(err) => {
                return self.handle_compilation_err(err.kind);
            }
            ExecutionErrorKind::ParserError(err) => {
                return self.handle_parse_err(err.kind);
            }
            err => format!("Unhandled execution error: {:?}", err),
        };
        Err((4, message))
    }

    fn handle_compilation_err(&mut self, kind: CompilationErrorKind) -> ExitResult {
        let message = match kind {
            err => format!("Unhandled compilation error: {:?}", err),
        };
        Err((3, message))
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
