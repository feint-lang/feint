//! # FeInt REPL

use std::path::{Path, PathBuf};

use dirs;
use rustyline::error::ReadlineError;

use super::result::ExitResult;
use super::scanner::{self, ScanError, ScanErrorKind, TokenWithLocation};
use super::vm::{Instruction, Instructions, Namespace, VMState, VM};
use crate::parser::{self, ParseError, ParseErrorKind};
use crate::util::Location;

/// Run FeInt REPL until user exits.
pub fn run(debug: bool) -> ExitResult {
    let history_path = Runner::default_history_path();
    let namespace = Namespace::default();
    let vm = VM::new(namespace);
    let mut runner = Runner::new(Some(history_path.as_path()), vm, debug);
    runner.run()
}

struct Runner<'a> {
    reader: rustyline::Editor<()>,
    history_path: Option<&'a Path>,
    vm: VM<'a>,
    debug: bool,
}

impl<'a> Runner<'a> {
    fn new(history_path: Option<&'a Path>, vm: VM<'a>, debug: bool) -> Self {
        Runner { reader: rustyline::Editor::<()>::new(), history_path, vm, debug }
    }

    /// Get the default history path, which is either ~/.feint_history
    /// or, if the user's home directory can't be located,
    /// ./.feint_history.
    fn default_history_path() -> PathBuf {
        let home = dirs::home_dir();
        let base_path = home.unwrap_or_default();
        let history_path_buf = base_path.join(".feint_history");
        history_path_buf
    }

    fn run(&mut self) -> ExitResult {
        println!("Welcome to the FeInt REPL (read/eval/print loop)");
        println!("Type a line of code, then hit Enter to evaluate it");
        println!("Type .exit or .quit to exit");

        self.load_history();

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

    fn eval(&mut self, text: &str) -> Option<ExitResult> {
        self.add_history_entry(text);

        match text.trim() {
            ".exit" | ".halt" | ".quit" => {
                return Some(Ok(Some("Exit".to_owned())));
            }
            _ => (),
        }

        match parser::parse_text(text) {
            Ok(program) => {
                eprintln!("{:?}", program);

                // TODO:
                let instructions = vec![Instruction::Return];

                match self.vm.execute(&instructions) {
                    Ok(VMState::Idle) => None,
                    Ok(VMState::Halted(0)) => Some(Ok(Some("Halted".to_owned()))),
                    Ok(VMState::Halted(code)) => {
                        Some(Err((code, format!("Halted abnormally: {}", code))))
                    }
                    Err(err) => Some(Err((1, err.to_string()))),
                }
            }
            Err(err) => {
                if self.handle_err(err) {
                    // Continue until valid input.
                    match self.read_line("+ ", false) {
                        Ok(None) => {
                            let input = text.to_string() + "\n";
                            self.eval(input.as_str())
                        }
                        Ok(Some(new_input)) => {
                            let input = text.to_string() + "\n" + new_input.as_str();
                            self.eval(input.as_str())
                        }
                        Err(err) => Some(Err((1, format!("{}", err)))),
                    }
                } else {
                    // Bail.
                    None
                }
            }
        }
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

    /// Handle error. For now, true means the eval should continue
    /// trying to add text to the original input while false means the
    /// eval loop should give up on the input that caused the error.
    fn handle_err(&mut self, err: ParseError) -> bool {
        self.handle_parse_err(err.kind)
    }

    /// Handle parse error.
    fn handle_parse_err(&mut self, kind: ParseErrorKind) -> bool {
        match kind {
            ParseErrorKind::ScanError(err) => {
                return self.handle_scan_err(err.kind, err.location);
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
        false
    }

    /// Handle scan error.
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

    fn new<'a>() -> Runner<'a> {
        let namespace = Namespace::new(None);
        let vm = VM::new(namespace);
        Runner::new(None, vm, false)
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
