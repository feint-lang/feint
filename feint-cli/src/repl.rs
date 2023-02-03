//! # FeInt REPL
use std::path::PathBuf;

use rustyline::config::Configurer;
use rustyline::error::ReadlineError;

use feint_builtins::types::{new, ObjectRef, ObjectTrait};
use feint_compiler::{CompErrKind, ParseErrKind, ScanErrKind};
use feint_driver::result::{DriverErr, DriverErrKind, DriverOptResult, DriverResult};
use feint_driver::Driver;

pub struct Repl {
    module: ObjectRef,
    reader: rustyline::Editor<()>,
    history_path: Option<PathBuf>,
    driver: Driver,
}

impl Repl {
    pub fn new(history_path: Option<PathBuf>, mut driver: Driver) -> Self {
        let module = new::module("$repl", "<repl>", "FeInt REPL module", &[]);
        driver.add_module("$repl", module.clone());

        let mut reader =
            rustyline::Editor::<()>::new().expect("Could not initialize readline");
        reader.set_indent_size(4);
        reader.set_tab_stop(4);

        Repl { module, reader, history_path, driver }
    }

    pub fn run(&mut self) -> DriverResult {
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
                    let result = self.eval(input.as_str(), true);
                    match result {
                        Ok(Some(code)) => break Ok(code),
                        Ok(None) => (),
                        Err(err) => break Err(err),
                    }
                }
                // User hit Ctrl-C
                Err(ReadlineError::Interrupted) => {
                    println!("Use Ctrl-D or .exit to exit");
                }
                // User hit Ctrl-D
                Err(ReadlineError::Eof) => break Ok(0),
                // Unexpected error encountered while attempting to read
                // a line.
                Err(err) => {
                    let msg = format!("Could not read line: {err}");
                    break Err(DriverErr::new(DriverErrKind::ReplErr(msg)));
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
    /// continue reading and evaluating input. Returns `Some(u8)` to
    /// indicate to the main loop to exit.
    pub fn eval(&mut self, text: &str, continue_on_err: bool) -> DriverOptResult {
        self.add_history_entry(text);

        if matches!(text, ".exit" | ".quit") {
            return Ok(Some(0));
        } else if self.handle_command(text) {
            return Ok(None);
        }

        // If there's an error executing the current input, try to add
        // more lines *if* the error can potentially be recovered from
        // by adding more input.
        self.driver.execute_repl(text, self.module.clone()).or_else(|err| {
            if let Some(code) = err.exit_code() {
                return Ok(Some(code));
            }

            if continue_on_err && self.continue_on_err(&err) {
                // Add input until 2 successive blank lines are entered.
                let mut input = text.to_owned();
                let mut blank_line_count = 0;
                loop {
                    match self.read_line("+ ", false) {
                        Ok(None) => unreachable!(
                            "read_line should never return None in this context"
                        ),
                        Ok(Some(new_input)) => {
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
                        }
                        Err(err) => {
                            break Err(DriverErr::new(DriverErrKind::ReplErr(
                                format!("{err}"),
                            )));
                        }
                    }
                }
            } else {
                if matches!(
                    &err.kind,
                    DriverErrKind::Bootstrap(_)
                        | DriverErrKind::CouldNotReadSourceFile(_)
                        | DriverErrKind::ModuleDirNotFound(_)
                        | DriverErrKind::ModuleNotFound(_)
                        | DriverErrKind::ReplErr(_)
                ) {
                    eprintln!("{err}");
                }
                Ok(None)
            }
        })
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
                let module = self.module.read().unwrap();
                let module = module.down_to_mod().unwrap();
                eprintln!("{:=>72}", "");
                eprintln!("GLOBALS for module {:?} ", module.name());
                eprintln!("{:->72}", "");
                for (name, val) in module.ns().iter() {
                    println!("{name} = {:?}", &*val.read().unwrap());
                }
                eprintln!("{:=>72}", "");
            }
            ".constants" => {
                let module = self.module.read().unwrap();
                let module = module.down_to_mod().unwrap();
                for (i, val) in module.code().iter_constants().enumerate() {
                    println!("{i} = {:?}", &*val.read().unwrap());
                }
            }
            ".dis" => {
                self.driver.disassemble_module(self.module.clone());
            }
            ".stack" => {
                self.driver.display_stack();
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

    fn continue_on_err(&self, err: &DriverErr) -> bool {
        if let DriverErrKind::ScanErr(kind) = &err.kind {
            use ScanErrKind::*;
            return matches!(
                kind,
                ExpectedBlock
                    | ExpectedIndentedBlock(_)
                    | UnmatchedOpeningBracket(_)
                    | UnterminatedStr(_)
            );
        } else if let DriverErrKind::ParseErr(kind) = &err.kind {
            use ParseErrKind::*;
            return matches!(kind, ExpectedBlock(_));
        } else if let DriverErrKind::CompErr(kind) = &err.kind {
            use CompErrKind::*;
            return matches!(kind, LabelNotFoundInScope(..));
        }
        false
    }

    fn load_history(&mut self) {
        match &self.history_path {
            Some(path) => {
                println!("REPL history will be saved to {}", path.to_string_lossy());
                match self.reader.load_history(path.as_path()) {
                    Ok(_) => (),
                    Err(err) => eprintln!("Could not load REPL history: {err}"),
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
                        eprintln!("WARNING: Could not save REPL history: {err}")
                    }
                }
            }
            None => (),
        }
    }
}
