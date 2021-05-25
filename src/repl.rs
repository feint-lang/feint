//! # FeInt REPL

use std::path::Path;

use rustyline::error::ReadlineError;

use crate::compiler::CompilationErrorKind;
use crate::parser::{ParseError, ParseErrorKind};
use crate::result::ExitResult;
use crate::scanner::ScanErrorKind;
use crate::util::Location;
use crate::vm::{execute_text, RuntimeErrorKind, VMState, VM};

/// Run FeInt REPL until user exits.
pub fn run(history_path: Option<&Path>, debug: bool) -> ExitResult {
    let mut repl = Repl::new(history_path, VM::default(), debug);
    repl.run()
}

struct Repl<'a> {
    reader: rustyline::Editor<()>,
    history_path: Option<&'a Path>,
    vm: VM,
    debug: bool,
}

impl<'a> Repl<'a> {
    fn new(history_path: Option<&'a Path>, vm: VM, debug: bool) -> Self {
        Repl { reader: rustyline::Editor::<()>::new(), history_path, vm, debug }
    }

    fn run(&mut self) -> ExitResult {
        println!("Welcome to the FeInt REPL (read/eval/print loop)");
        println!("Type a line of code, then hit Enter to evaluate it");
        self.load_history();
        println!("Type exit or quit to exit");

        loop {
            match self.read_line("→ ", true) {
                Ok(None) => {
                    // Blank or all-whitespace line.
                    ()
                }
                Ok(Some(input)) => {
                    // Evaluate the input. If eval returns a result of
                    // any kind (ok or err), exit the loop and shut down
                    // the REPL.
                    match self.eval(input.as_str()) {
                        Some(result) => {
                            self.vm.halt();
                            break result;
                        }
                        None => (),
                    }
                }
                // User hit Ctrl-C or Ctrl-D.
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    self.vm.halt();
                    break Ok(None);
                }
                // Unexpected error encountered while attempting to read
                // a line.
                Err(err) => {
                    self.vm.halt();
                    break Err((1, format!("Could not read line: {}", err)));
                }
            }
        }
    }

    /// Get a line of input from the user. If the line comprises only
    /// whitespace *and* ``trim_blank`` is set, the line will be trimmed
    /// and ``None`` will be returned.
    fn read_line(
        &mut self,
        prompt: &str,
        trim_blank: bool,
    ) -> Result<Option<String>, ReadlineError> {
        match self.reader.readline(prompt) {
            Ok(input) if trim_blank && input.trim().len() == 0 => Ok(None),
            Ok(input) => Ok(Some(input)),
            Err(err) => Err(err),
        }
    }

    /// Evaluate text. Returns None to indicate to the main loop to
    /// continue reading and evaluating input. Returns some result to
    /// indicate to the main loop to exit.
    fn eval(&mut self, text: &str) -> Option<ExitResult> {
        self.add_history_entry(text);

        let result = match text.trim() {
            "exit" | "quit" => return Some(Ok(None)),
            _ => {
                // FIXME: This feels like a really hacky way to handle
                //        printing the result. Ideally, it would be
                //        stored in a temporary local var and that's
                //        what would be printed.
                if text.starts_with("print ") {
                    execute_text(&mut self.vm, text, self.debug)
                } else {
                    execute_text(
                        &mut self.vm,
                        format!("print {}", text).as_str(),
                        self.debug,
                    )
                }
            }
        };

        if let Ok(vm_state) = result {
            return self.vm_state_to_exit_result(vm_state);
        }

        let err = result.unwrap_err();

        if let RuntimeErrorKind::ParseError(ParseError {
            kind: ParseErrorKind::ScanError(scan_err),
        }) = err.kind
        {
            if self.handle_scan_err(scan_err.kind, scan_err.location) {
                match self.read_line("+ ", false) {
                    Ok(None) => {
                        let input = format!("{}\n", text);
                        self.eval(input.as_str())
                    }
                    Ok(Some(new_input)) => {
                        let input = format!("{}\n{}", text, new_input);
                        self.eval(input.as_str())
                    }
                    Err(err) => Some(Err((2, format!("{}", err)))),
                }
            } else {
                None
            }
        } else {
            self.handle_execution_err(err.kind);
            None
        }
    }

    fn vm_state_to_exit_result(&self, vm_state: VMState) -> Option<ExitResult> {
        if self.debug {
            eprintln!("VM STATE:\n{:?}", vm_state);
        }
        match vm_state {
            VMState::Idle => None,
            VMState::Halted(0) => None,
            VMState::Halted(code) => {
                Some(Err((code, format!("Halted abnormally: {}", code))))
            }
            VMState::Running => unreachable!(),
        }
    }

    /// Handle VM execution errors.
    fn handle_execution_err(&mut self, kind: RuntimeErrorKind) {
        let message = match kind {
            RuntimeErrorKind::ParseError(err) => {
                return self.handle_parse_err(err.kind);
            }
            RuntimeErrorKind::CompilationError(err) => {
                return self.handle_compilation_err(err.kind);
            }
            RuntimeErrorKind::TypeError(message) => {
                format!("{}", message)
            }
            err => {
                format!("Unhandled execution error: {:?}", err)
            }
        };
        eprintln!("{}", message);
    }

    /// Handle compilation errors.
    fn handle_compilation_err(&mut self, kind: CompilationErrorKind) {
        let message = match kind {
            err => format!("Unhandled compilation error: {:?}", err),
        };
        eprintln!("{}", message);
    }

    /// Handle parse errors.
    fn handle_parse_err(&mut self, kind: ParseErrorKind) {
        match kind {
            ParseErrorKind::ScanError(err) => {
                self.handle_scan_err(err.kind, err.location);
            }
            ParseErrorKind::UnhandledToken(token) => {
                let location = token.start;
                eprintln!("{: >width$}^", "", width = location.col + 1);
                eprintln!(
                    "Parse error: unhandled token at {}: {:?}",
                    location, token.token
                );
            }
            err => {
                eprintln!("Unhandled parse error: {:?}", err);
            }
        }
    }

    /// Handle scan errors. True means eval should continue trying to
    /// add text to the original input while false means eval should
    /// give up on the input that caused the error.
    fn handle_scan_err(&mut self, kind: ScanErrorKind, location: Location) -> bool {
        match kind {
            ScanErrorKind::UnexpectedCharacter(c) => {
                let col = location.col;
                eprintln!("{: >width$}^", "", width = col + 1);
                eprintln!(
                    "Syntax error: unexpected character at column {}: '{}'",
                    col, c
                );
            }
            ScanErrorKind::UnterminatedString(_) => {
                return true;
            }
            ScanErrorKind::InvalidIndent(num_spaces) => {
                let col = location.col;
                eprintln!("{: >width$}^", "", width = col + 1);
                eprintln!("Syntax error: invalid indent with {} spaces (should be a multiple of 4)", num_spaces);
            }
            ScanErrorKind::UnexpectedIndent(_) => {
                let col = location.col;
                eprintln!("{: >width$}^", "", width = col + 1);
                eprintln!("Syntax error: unexpected indent");
            }
            ScanErrorKind::WhitespaceAfterIndent
            | ScanErrorKind::UnexpectedWhitespace => {
                let col = location.col;
                eprintln!("{: >width$}^", "", width = col + 1);
                eprintln!("Syntax error: unexpected whitespace");
            }
            err => {
                eprintln!("Unhandled scan error at {}: {:?}", location, err);
            }
        }
        false
    }

    fn load_history(&mut self) {
        match self.history_path {
            Some(path) => {
                println!("REPL history will be saved to {}", path.to_string_lossy());
                match self.reader.load_history(path) {
                    Ok(_) => (),
                    Err(err) => eprintln!("Could not load REPL history: {}", err),
                }
            }
            None => (),
        }
    }

    fn add_history_entry(&mut self, input: &str) {
        match self.history_path {
            Some(path) => {
                self.reader.add_history_entry(input);
                match self.reader.save_history(path) {
                    Ok(_) => (),
                    Err(err) => eprintln!("Could not save REPL history: {}", err),
                }
            }
            None => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_empty() {
        eval("");
    }

    #[test]
    fn eval_arithmetic() {
        eval("2 * (3 + 4)");
    }

    #[test]
    fn eval_string() {
        eval("\"abc\"");
    }

    #[test]
    fn eval_multiline_string() {
        eval("\"a \nb c\"");
    }

    // TODO: Figure out how to automatically send closing quote and
    //       newline to stdin.
    // #[test]
    // fn eval_unterminated_string() {
    //     eval("x = \"abc");
    // }

    // Utilities -----------------------------------------------------------

    fn new<'a>() -> Repl<'a> {
        let vm = VM::default();
        Repl::new(None, vm, false)
    }

    fn eval(input: &str) {
        let mut runner = new();
        match runner.eval(input) {
            Some(Ok(string)) => assert!(false),
            Some(Err((code, string))) => assert!(false),
            None => assert!(true), // eval returns None on valid input
        }
    }
}
