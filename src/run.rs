//! # FeInt

use crate::compiler::CompilationErrorKind;
use crate::parser::ParseErrorKind;
use crate::result::ExitResult;
use crate::scanner::ScanErrorKind;
use crate::util::Location;
use crate::vm::{self, ExecutionResult, RuntimeErrorKind, VMState, VM};

/// Run text source.
pub fn run_text(text: &str, disassemble: bool, debug: bool) -> ExitResult {
    let mut vm = VM::default();
    let mut runner = Runner::new(debug);
    runner.exit(vm::execute_text(&mut vm, text, disassemble, debug))
}

/// Run source from file.
pub fn run_file(file_path: &str, disassemble: bool, debug: bool) -> ExitResult {
    let mut vm = VM::default();
    let mut runner = Runner::new(debug);
    runner.exit(vm::execute_file(&mut vm, file_path, disassemble, debug))
}

/// Read and run source from stdin.
pub fn run_stdin(disassemble: bool, debug: bool) -> ExitResult {
    let mut vm = VM::default();
    let mut runner = Runner::new(debug);
    runner.exit(vm::execute_stdin(&mut vm, disassemble, debug))
}

struct Runner {
    debug: bool,
}

impl Runner {
    fn new(debug: bool) -> Self {
        Runner { debug }
    }

    /// Take result from VM execution and return an appropriate exit
    /// result.
    fn exit(&mut self, result: ExecutionResult) -> ExitResult {
        match result {
            Ok(vm_state) => self.vm_state_to_exit_result(vm_state),
            Err(err) => self.handle_execution_err(err.kind),
        }
    }

    /// Convert VM state to exit result.
    fn vm_state_to_exit_result(&self, vm_state: VMState) -> ExitResult {
        if self.debug {
            eprintln!("VM STATE:\n{:?}", vm_state);
        }
        match vm_state {
            VMState::Halted(0) => Ok(None),
            VMState::Halted(code) => {
                Err((code, format!("Halted abnormally: {}", code)))
            }
            VMState::Idle => Err((-1, "Never halted".to_owned())),
        }
    }

    fn handle_execution_err(&mut self, kind: RuntimeErrorKind) -> ExitResult {
        let message = match kind {
            RuntimeErrorKind::CompilationError(err) => {
                return self.handle_compilation_err(err.kind);
            }
            RuntimeErrorKind::ParseError(err) => {
                return self.handle_parse_err(err.kind);
            }
            RuntimeErrorKind::TypeError(message) => {
                return Err((5, message));
            }
            err => format!("Unhandled execution error: {:?}", err),
        };
        Err((4, message))
    }

    /// Handle compilation errors.
    fn handle_compilation_err(&mut self, kind: CompilationErrorKind) -> ExitResult {
        let message = match kind {
            err => format!("Unhandled compilation error: {:?}", err),
        };
        Err((3, message))
    }

    /// Handle parsing errors.
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

    /// Handle scan errors.
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
                    "",
                    col,
                    c,
                    width = marker
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
    fn test_run_text() {
        let source = "1 + 2";
        let result = run_text(source, false, true);
        assert!(result.is_ok(), "{:?}", result.err());
    }
}
