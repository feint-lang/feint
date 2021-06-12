//! # FeInt

use crate::compiler::CompilationErrKind;
use crate::parser::ParseErrKind;
use crate::result::ExitResult;
use crate::scanner::ScanErrKind;
use crate::util::Location;
use crate::vm::{self, ExeResult, RuntimeErrKind, VMState, VM};

/// Run text source.
pub fn run_text(text: &str, dis: bool, debug: bool) -> ExitResult {
    let mut vm = VM::default();
    let mut runner = Runner::new(debug);
    runner.exit(vm::execute_text(&mut vm, text, dis, debug))
}

/// Run source from file.
pub fn run_file(file_path: &str, dis: bool, debug: bool) -> ExitResult {
    let mut vm = VM::default();
    let mut runner = Runner::new(debug);
    runner.exit(vm::execute_file(&mut vm, file_path, dis, debug))
}

/// Read and run source from stdin.
pub fn run_stdin(dis: bool, debug: bool) -> ExitResult {
    let mut vm = VM::default();
    let mut runner = Runner::new(debug);
    runner.exit(vm::execute_stdin(&mut vm, dis, debug))
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
    fn exit(&mut self, result: ExeResult) -> ExitResult {
        match result {
            Ok(vm_state) => self.vm_state_to_exit_result(vm_state),
            Err(err) => self.handle_execution_err(err.kind),
        }
    }

    /// Convert VM state to exit result.
    fn vm_state_to_exit_result(&self, vm_state: VMState) -> ExitResult {
        match vm_state {
            VMState::Halted(0) => Ok(None),
            VMState::Halted(code) => {
                Err((code, format!("Halted abnormally: {}", code)))
            }
            VMState::Idle => Err((-1, "Never halted".to_owned())),
        }
    }

    fn handle_execution_err(&mut self, kind: RuntimeErrKind) -> ExitResult {
        let message = match kind {
            RuntimeErrKind::CompilationError(err) => {
                return self.handle_compilation_err(err.kind);
            }
            RuntimeErrKind::ParseError(err) => {
                return self.handle_parse_err(err.kind);
            }
            RuntimeErrKind::TypeError(message) => {
                return Err((5, message));
            }
            err => format!("Unhandled execution error: {:?}", err),
        };
        Err((4, message))
    }

    /// Handle compilation errors.
    fn handle_compilation_err(&mut self, kind: CompilationErrKind) -> ExitResult {
        let message = match kind {
            err => format!("Unhandled compilation error: {:?}", err),
        };
        Err((3, message))
    }

    /// Handle parsing errors.
    fn handle_parse_err(&mut self, kind: ParseErrKind) -> ExitResult {
        let message = match kind {
            ParseErrKind::ScanErr(err) => {
                return self.handle_scan_err(err.kind, err.location);
            }
            ParseErrKind::CouldNotOpenSourceFile(path, message) => {
                format!("Could not open source file: {}\n{}", path, message)
            }
            ParseErrKind::UnhandledToken(token) => {
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
    fn handle_scan_err(&mut self, kind: ScanErrKind, location: Location) -> ExitResult {
        let line = location.line;
        let col = location.col;
        let marker = col - 1;
        let message = match kind {
            ScanErrKind::UnexpectedCharacter(c) => {
                format!(
                    "{:>width$}^\nSyntax error: unexpected character at column {}: '{}'",
                    "",
                    col,
                    c,
                    width = marker
                )
            }
            ScanErrKind::UnterminatedString(_) => {
                format!(
                    "{:>width$}^\nSyntax error: unterminated string literal at line {line}, col {col}",
                    "", line = line, col = col, width = marker
                )
            }
            ScanErrKind::InvalidIndent(num_spaces) => {
                format!(
                    "{:>width$}^\nSyntax error: invalid indent with {} spaces (should be a multiple of 4)",
                    "", num_spaces, width = marker
                )
            }
            ScanErrKind::UnexpectedIndent(_) => {
                format!(
                    "{:>width$}^\nSyntax error: unexpected indent",
                    "",
                    width = marker
                )
            }
            ScanErrKind::WhitespaceAfterIndent | ScanErrKind::UnexpectedWhitespace => {
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
