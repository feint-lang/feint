//! # FeInt REPL
use std::path::PathBuf;

use rustyline::config::Configurer;
use rustyline::error::ReadlineError;

use crate::dis;
use crate::exe::Executor;
use crate::parser::ParseErrKind;
use crate::result::{ExeErr, ExeErrKind, ExeResult};
use crate::scanner::ScanErrKind;
use crate::types::{Module, ObjectTrait};
use crate::vm::VMState;

pub struct Repl {
    module: Module,
    reader: rustyline::Editor<()>,
    history_path: Option<PathBuf>,
    executor: Executor,
}

impl Repl {
    pub fn new(history_path: Option<PathBuf>, executor: Executor) -> Self {
        let module = Module::with_name("$repl", "<repl>", Some("FeInt REPL module"));
        let mut reader =
            rustyline::Editor::<()>::new().expect("Could initialize readline");
        reader.set_indent_size(4);
        reader.set_tab_stop(4);
        Repl { module, reader, history_path, executor }
    }

    pub fn run(&mut self) -> ExeResult {
        println!("Welcome to the FeInt REPL (read/eval/print loop)");
        println!("Type a line of code, then hit Enter to evaluate it");
        self.load_history();
        println!("Type .exit or .quit to exit");
        let result = loop {
            match self.read_line("→ ", true) {
                Ok(None) => {
                    // Blank or all-whitespace line.
                }
                Ok(Some(input)) => {
                    // Evaluate the input. If eval returns a result of
                    // any kind (ok or err), shut down the REPL.
                    if let Some(result) = self.eval(input.as_str(), true) {
                        break result;
                    }
                }
                // User hit Ctrl-C
                Err(ReadlineError::Interrupted) => {
                    println!("Use Ctrl-D or .exit to exit");
                }
                // User hit Ctrl-D
                Err(ReadlineError::Eof) => {
                    break Ok(VMState::Halted(0));
                }
                // Unexpected error encountered while attempting to read
                // a line.
                Err(err) => {
                    let msg = format!("Could not read line: {}", err);
                    break Err(ExeErr::new(ExeErrKind::ReplErr(msg)));
                }
            }
        };
        result
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
            Ok(input) if trim_blank && input.trim().is_empty() => Ok(None),
            Ok(input) => Ok(Some(input)),
            Err(err) => Err(err),
        }
    }

    /// Evaluate text. Returns `None` to indicate to the main loop to
    /// continue reading and evaluating input. Returns an `ExeResult` to
    /// indicate to the main loop to exit.
    pub fn eval(&mut self, text: &str, continue_on_err: bool) -> Option<ExeResult> {
        self.add_history_entry(text);

        if matches!(text, ".exit" | ".quit") {
            return Some(Ok(VMState::Halted(0)));
        } else if self.handle_command(text) {
            return None;
        }

        let result = self.executor.execute_repl(text, &mut self.module);

        match result {
            Ok(vm_state) => {
                return match vm_state {
                    VMState::Running => None,
                    VMState::Idle(_) => None,
                    // Halted:
                    state => Some(Ok(state)),
                };
            }
            Err(err) => {
                // If the special Exit err is returned, exit.
                if let Some(code) = err.exit_code() {
                    return Some(Ok(VMState::Halted(code)));
                }

                // If there's an error executing the current input, try
                // to add more lines *if* the error can potentially be
                // recovered from by adding more input.
                if !(continue_on_err && self.continue_on_err(&err)) {
                    eprintln!("ERROR: {err}");
                    return None;
                }
            }
        }

        // Add input until 2 successive blank lines are entered.
        let mut input = text.to_owned();
        let mut blank_line_count = 0;
        loop {
            let read_line_result = self.read_line("+ ", false);
            if let Ok(None) = read_line_result {
                unreachable!();
            } else if let Ok(Some(new_input)) = read_line_result {
                input.push('\n');
                if new_input.is_empty() {
                    if blank_line_count > 0 {
                        break self.eval(input.as_str(), false);
                    }
                    blank_line_count += 1;
                } else {
                    input.push_str(new_input.as_str());
                    if blank_line_count > 0 {
                        break self.eval(input.as_str(), false);
                    }
                    blank_line_count = 0;
                }
            } else {
                let msg = format!("{}", read_line_result.unwrap_err());
                break Some(Err(ExeErr::new(ExeErrKind::ReplErr(msg))));
            }
        }
    }

    fn handle_command(&mut self, text: &str) -> bool {
        match text.trim() {
            "?" | ".help" => {
                eprintln!("{:=>72}", "");
                eprintln!("FeInt Help");
                eprintln!("{:->72}", "");
                eprintln!(".help      -> show this help");
                eprintln!(".exit      -> exit");
                eprintln!(".globals   -> show REPL module globals");
                eprintln!(".constants -> show REPL module constants");
                eprintln!(".dis       -> disassemble REPL module");
                eprintln!(".stack     -> show VM stack (top first)");
                eprintln!(".emacs     -> switch to emacs-style input (default)");
                eprintln!(".vi        -> switch to vi-style input");
                eprintln!("{:=>72}", "");
            }
            ".globals" => {
                eprintln!("{:=>72}", "");
                eprintln!("GLOBALS for module {:?} ", self.module.name());
                eprintln!("{:->72}", "");
                for (name, val) in self.module.ns().iter() {
                    println!("{name} = {:?}", &*val.read().unwrap());
                }
                eprintln!("{:=>72}", "");
            }
            ".constants" => {
                for (i, val) in self.module.code().iter_constants().enumerate() {
                    println!("{i} = {:?}", &*val.read().unwrap());
                }
            }
            ".dis" => {
                let mut disassembler = dis::Disassembler::new();
                disassembler.disassemble(self.module.code());
            }
            ".stack" => {
                self.executor.display_stack();
            }
            ".emacs" => {
                self.reader.set_edit_mode(rustyline::config::EditMode::Emacs);
            }
            ".vi" | ".vim" => {
                self.reader.set_edit_mode(rustyline::config::EditMode::Vi);
            }
            _ => return false,
        }
        true
    }

    fn continue_on_err(&self, err: &ExeErr) -> bool {
        if let ExeErrKind::ScanErr(kind) = &err.kind {
            use ScanErrKind::*;
            if let ExpectedBlock
            | ExpectedIndentedBlock(_)
            | UnmatchedOpeningBracket(_)
            | UnterminatedStr(_) = kind
            {
                return true;
            }
        } else if let ExeErrKind::ParseErr(kind) = &err.kind {
            use ParseErrKind::*;
            if let ExpectedBlock(_) = kind {
                return true;
            }
        }
        false
    }

    fn load_history(&mut self) {
        match &self.history_path {
            Some(path) => {
                println!("REPL history will be saved to {}", path.to_string_lossy());
                match self.reader.load_history(path.as_path()) {
                    Ok(_) => (),
                    Err(err) => eprintln!("Could not load REPL history: {}", err),
                }
            }
            None => (),
        }
    }

    fn add_history_entry(&mut self, input: &str) {
        match &self.history_path {
            Some(path) => {
                self.reader.add_history_entry(input);
                match self.reader.save_history(path.as_path()) {
                    Ok(_) => (),
                    Err(err) => {
                        eprintln!("WARNING: Could not save REPL history: {}", err)
                    }
                }
            }
            None => (),
        }
    }
}
