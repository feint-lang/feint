//! # FeInt REPL

use std::path::Path;

use rustyline::error::ReadlineError;

use crate::compiler::CompilationErrKind;
use crate::parser::ParseErrKind;
use crate::result::ExitResult;
use crate::scanner::ScanErrKind;
use crate::util::Location;
use crate::vm::{execute, execute_text, Inst, RuntimeErrKind, VMState, VM};

/// Run FeInt REPL until user exits.
pub fn run(history_path: Option<&Path>, dis: bool, debug: bool) -> ExitResult {
    let mut repl = Repl::new(history_path, VM::default(), dis, debug);
    repl.run()
}

struct Repl<'a> {
    reader: rustyline::Editor<()>,
    history_path: Option<&'a Path>,
    vm: VM,
    dis: bool,
    debug: bool,
}

impl<'a> Repl<'a> {
    fn new(history_path: Option<&'a Path>, vm: VM, dis: bool, debug: bool) -> Self {
        Repl { reader: rustyline::Editor::<()>::new(), history_path, vm, dis, debug }
    }

    fn run(&mut self) -> ExitResult {
        println!("Welcome to the FeInt REPL (read/eval/print loop)");
        println!("Type a line of code, then hit Enter to evaluate it");
        self.load_history();
        println!("Type .exit or .quit to exit");

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
                    match self.eval(input.as_str(), false) {
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
    fn eval(&mut self, text: &str, bail: bool) -> Option<ExitResult> {
        self.add_history_entry(text);

        let result = match text.trim() {
            "?" | ".help" => {
                eprintln!("{:=>72}", "");
                eprintln!("FeInt Help");
                eprintln!("{:->72}", "");
                eprintln!(".help  -> show help");
                eprintln!(".exit  -> exit");
                eprintln!(".stack -> show VM stack (top first)");
                eprintln!("{:=>72}", "");
                return None;
            }
            ".exit" | ".quit" => return Some(Ok(None)),
            ".stack" => {
                self.vm.display_stack();
                return None;
            }
            _ => execute_text(&mut self.vm, text, self.dis, self.debug),
        };

        if let Ok(vm_state) = result {
            // Assign _ to value at top of stack
            let var = "_";
            let mut instructions = vec![Inst::AssignVar(var.to_owned())];
            if let Some(&index) = self.vm.peek() {
                // Don't print nil when the result of an expression is nil
                if index != 0 {
                    instructions.push(Inst::Print);
                }
            }
            if let Err(err) = execute(&mut self.vm, instructions, false, false) {
                // If stack is empty, assign _ to nil
                if let RuntimeErrKind::NotEnoughValuesOnStack(_) = err.kind {
                    let instructions =
                        vec![Inst::Push(0), Inst::AssignVar(var.to_owned())];
                    if let Err(err) = execute(&mut self.vm, instructions, false, false)
                    {
                        eprintln!(
                            "ERROR: Could not assign _ to top of stack or to nil:\n{}",
                            err
                        );
                    }
                }
            }
            return self.vm_state_to_exit_result(vm_state);
        }

        let err = result.unwrap_err();

        if self.handle_execution_err(err.kind, bail) {
            // Keep adding input until 2 successive blank lines are
            // entered.
            let mut input = text.to_owned();
            let mut blank_line_count = 0;
            loop {
                match self.read_line("+ ", false) {
                    Ok(None) => unreachable!(),
                    Ok(Some(new_input)) if new_input == "" => {
                        input.push('\n');
                        if blank_line_count > 0 {
                            break self.eval(input.as_str(), true);
                        }
                        blank_line_count += 1;
                    }
                    Ok(Some(new_input)) => {
                        input.push('\n');
                        input.push_str(new_input.as_str());
                        if blank_line_count > 0 {
                            break self.eval(input.as_str(), true);
                        }
                        blank_line_count = 0;
                    }
                    Err(err) => break Some(Err((2, format!("{}", err)))),
                }
            }
        } else {
            None
        }
    }

    fn vm_state_to_exit_result(&self, vm_state: VMState) -> Option<ExitResult> {
        match vm_state {
            VMState::Idle => None,
            VMState::Halted(0) => None,
            VMState::Halted(code) => {
                Some(Err((code, format!("Halted abnormally: {}", code))))
            }
        }
    }

    /// Handle VM execution errors.
    ///
    /// A return value of true means eval should continue trying to add
    /// text to the original input while false means eval should give up
    /// on the input that caused the error. This applies to execution
    /// errors and any nested error types.
    fn handle_execution_err(&mut self, kind: RuntimeErrKind, bail: bool) -> bool {
        let message = match kind {
            RuntimeErrKind::CompilationError(err) => {
                return self.handle_compilation_err(err.kind, bail);
            }
            RuntimeErrKind::ParseError(err) => {
                return self.handle_parse_err(err.kind, bail);
            }
            RuntimeErrKind::TypeError(message) => {
                format!("{}", message)
            }
            err => {
                format!("Unhandled execution error: {:?}", err)
            }
        };
        eprintln!("{}", message);
        false
    }

    /// Handle compilation errors.
    fn handle_compilation_err(&mut self, kind: CompilationErrKind, bail: bool) -> bool {
        let message = match kind {
            err => format!("Unhandled compilation error: {:?}", err),
        };
        eprintln!("{}", message);
        false
    }

    /// Handle parse errors.
    fn handle_parse_err(&mut self, kind: ParseErrKind, bail: bool) -> bool {
        match kind {
            ParseErrKind::ScanError(err) => {
                return self.handle_scan_err(err.kind, err.location, bail);
            }
            ParseErrKind::UnhandledToken(token) => {
                let loc = token.start;
                eprintln!("{: >width$}^", "", width = loc.col + 1);
                eprintln!("Parse error: unhandled token at {}: {:?}", loc, token.token);
            }
            ParseErrKind::ExpectedBlock(loc) => {
                if bail {
                    eprintln!("{: >width$}^", "", width = loc.col + 1);
                    eprintln!("Parse error: expected indented block at {}", loc);
                } else {
                    return true;
                }
            }
            err => {
                eprintln!("Unhandled parse error: {:?}", err);
            }
        }
        false
    }

    /// Handle scan errors.
    fn handle_scan_err(
        &mut self,
        kind: ScanErrKind,
        loc: Location,
        bail: bool,
    ) -> bool {
        match kind {
            ScanErrKind::UnexpectedCharacter(c) => {
                let col = loc.col;
                eprintln!("{: >width$}^", "", width = col + 1);
                eprintln!(
                    "Syntax error: unexpected character at column {}: '{}'",
                    col, c
                );
            }
            ScanErrKind::UnmatchedOpeningBracket(_)
            | ScanErrKind::UnterminatedString(_) => {
                return true;
            }
            ScanErrKind::InvalidIndent(num_spaces) => {
                let col = loc.col;
                eprintln!("{: >width$}^", "", width = col + 1);
                eprintln!("Syntax error: invalid indent with {} spaces (should be a multiple of 4)", num_spaces);
            }
            ScanErrKind::UnexpectedIndent(_) => {
                let col = loc.col;
                eprintln!("{: >width$}^", "", width = col + 1);
                eprintln!("Syntax error: unexpected indent");
            }
            ScanErrKind::WhitespaceAfterIndent | ScanErrKind::UnexpectedWhitespace => {
                let col = loc.col;
                eprintln!("{: >width$}^", "", width = col + 1);
                eprintln!("Syntax error: unexpected whitespace");
            }
            err => {
                eprintln!("Unhandled scan error at {}: {:?}", loc, err);
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
    //     eval("x = \"abc", true);
    // }

    // Utilities -----------------------------------------------------------

    fn new<'a>() -> Repl<'a> {
        let vm = VM::default();
        Repl::new(None, vm, false, false)
    }

    fn eval(input: &str) {
        let mut runner = new();
        match runner.eval(input, true) {
            Some(Ok(string)) => assert!(false),
            Some(Err((code, string))) => assert!(false),
            None => assert!(true), // eval returns None on valid input
        }
    }
}
