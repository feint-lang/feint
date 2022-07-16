//! # FeInt REPL
use rustyline::config::Configurer;
use std::path::Path;

use rustyline::error::ReadlineError;

use crate::exe::Executor;
use crate::parser::ParseErrKind;
use crate::result::{ExeErr, ExeErrKind, ExitResult};
use crate::scanner::ScanErrKind;
use crate::vm::{Inst, RuntimeErrKind, VMState, VM};

/// Run FeInt REPL until user exits.
pub fn run(history_path: Option<&Path>, dis: bool, debug: bool) -> ExitResult {
    let mut vm = VM::default();
    let executor = Executor::new(&mut vm, true, dis, debug);
    let mut repl = Repl::new(history_path, executor);
    repl.run()
}

struct Repl<'a> {
    reader: rustyline::Editor<()>,
    history_path: Option<&'a Path>,
    executor: Executor<'a>,
}

impl<'a> Repl<'a> {
    fn new(history_path: Option<&'a Path>, executor: Executor<'a>) -> Self {
        let mut reader = rustyline::Editor::<()>::new();
        reader.set_indent_size(4);
        reader.set_tab_stop(4);
        Repl { reader, history_path, executor }
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
                            self.executor.vm.halt();
                            break result;
                        }
                        None => (),
                    }
                }
                // User hit Ctrl-C or Ctrl-D.
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    self.executor.vm.halt();
                    break Ok(None);
                }
                // Unexpected error encountered while attempting to read
                // a line.
                Err(err) => {
                    self.executor.vm.halt();
                    break Err((1, Some(format!("Could not read line: {}", err))));
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
    fn eval(&mut self, text: &str, no_continue: bool) -> Option<ExitResult> {
        self.add_history_entry(text);

        let result = match text.trim() {
            "?" | ".help" => {
                eprintln!("{:=>72}", "");
                eprintln!("FeInt Help");
                eprintln!("{:->72}", "");
                eprintln!(".help  -> show help");
                eprintln!(".exit  -> exit");
                eprintln!(".stack -> show VM stack (top first)");
                eprintln!(".emacs -> switch to emacs-style input (default)");
                eprintln!(".vi    -> switch to vi-style input");
                eprintln!("{:=>72}", "");
                return None;
            }
            "\t" => {
                eprintln!("got tab");
                return None;
            }
            ".exit" | ".quit" => return Some(Ok(None)),
            ".stack" => {
                self.executor.vm.display_stack();
                return None;
            }
            ".emacs" => {
                self.reader.set_edit_mode(rustyline::config::EditMode::Emacs);
                return None;
            }
            ".vi" | ".vim" => {
                self.reader.set_edit_mode(rustyline::config::EditMode::Vi);
                return None;
            }
            _ => self.executor.execute_text(text, Some("<repl>")),
        };

        if let Ok(vm_state) = result {
            // Assign _ to value at top of stack
            let var = "_";
            let mut instructions = vec![Inst::AssignVar(var.to_owned())];
            if let Some(&index) = self.executor.vm.peek() {
                // Don't print nil when the result of an expression is nil
                if index != 0 {
                    instructions.push(Inst::Print);
                }
            }
            if let Err(err) = self.executor.vm.execute(instructions, false) {
                // If stack is empty, assign _ to nil
                if let RuntimeErrKind::NotEnoughValuesOnStack(_) = err.kind {
                    let instructions =
                        vec![Inst::Push(0), Inst::AssignVar(var.to_owned())];
                    if let Err(err) = self.executor.vm.execute(instructions, false) {
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

        if no_continue {
            None
        } else if self.continue_on_err(err) {
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
                    Err(err) => break Some(Err((2, Some(format!("{}", err))))),
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
                Some(Err((code, Some(format!("Halted abnormally: {}", code)))))
            }
        }
    }

    fn continue_on_err(&self, err: ExeErr) -> bool {
        use ParseErrKind::*;
        use ScanErrKind::*;
        match err.kind {
            ExeErrKind::ScanErr(scan_err_kind) => match scan_err_kind {
                UnmatchedOpeningBracket(_) | UnterminatedString(_) => true,
                _ => false,
            },
            ExeErrKind::ParseErr(parse_err_kind) => match parse_err_kind {
                ExpectedBlock(_) => true,
                _ => false,
            },
            _ => false,
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
    #[test]
    fn eval_unterminated_string() {
        eval("x = \"abc");
    }

    #[test]
    fn eval_if_with_no_block() {
        eval("if true ->");
    }

    // Utilities -----------------------------------------------------------
    fn eval(input: &str) {
        let mut vm = VM::default();
        let executor = Executor::new(&mut vm, false, false, false);
        let mut repl = Repl::new(None, executor);
        match repl.eval(input, true) {
            Some(Ok(string)) => assert!(false),
            Some(Err((code, string))) => assert!(false),
            None => assert!(true), // eval returns None on valid input
        }
    }
}
