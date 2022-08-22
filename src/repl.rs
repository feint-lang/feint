//! # FeInt REPL
use std::path::Path;

use rustyline::config::Configurer;
use rustyline::error::ReadlineError;

use crate::exe::Executor;
use crate::modules;
use crate::parser::ParseErrKind;
use crate::result::{ExeErr, ExeErrKind, ExitResult};
use crate::scanner::ScanErrKind;
use crate::types::{ObjectRef, ObjectTrait};
use crate::vm::{Code, RuntimeContext, VMState, VM};

/// Run FeInt REPL until user exits.
pub fn run(history_path: Option<&Path>, dis: bool, debug: bool) -> ExitResult {
    let mut vm = VM::new(RuntimeContext::new(), 256);
    vm.install_sigint_handler();
    let executor = Executor::new(&mut vm, true, dis, debug);
    let module = if let Ok(module) = modules::add_module("$repl", Code::new()) {
        module
    } else {
        panic!("Could not add $repl module");
    };
    let mut repl = Repl::new(history_path, executor, module);
    repl.run()
}

pub struct Repl<'a> {
    reader: rustyline::Editor<()>,
    history_path: Option<&'a Path>,
    executor: Executor<'a>,
    module: ObjectRef,
}

impl<'a> Repl<'a> {
    pub fn new(
        history_path: Option<&'a Path>,
        executor: Executor<'a>,
        module: ObjectRef,
    ) -> Self {
        let mut reader =
            rustyline::Editor::<()>::new().expect("Could initialize readline");
        reader.set_indent_size(4);
        reader.set_tab_stop(4);
        Repl { reader, history_path, executor, module }
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
                // User hit Ctrl-C
                Err(ReadlineError::Interrupted) => {
                    println!("Use Ctrl-D or .exit to exit");
                    self.executor.vm.halt();
                }
                // User hit Ctrl-D
                Err(ReadlineError::Eof) => {
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
            Ok(input) if trim_blank && input.trim().is_empty() => Ok(None),
            Ok(input) => Ok(Some(input)),
            Err(err) => Err(err),
        }
    }

    /// Evaluate text. Returns None to indicate to the main loop to
    /// continue reading and evaluating input. Returns some result to
    /// indicate to the main loop to exit.
    pub fn eval(&mut self, text: &str, no_continue: bool) -> Option<ExitResult> {
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
            _ => {
                let module_guard = self.module.read().unwrap();
                let module = module_guard.down_to_mod().unwrap();
                if let Some(obj_ref) = module.get_global(text) {
                    self.print(obj_ref);
                    return None;
                } else {
                    drop(module_guard); // prevent deadlock
                    self.executor.execute_repl(text, self.module.clone())
                }
            }
        };

        if let Ok(vm_state) = result {
            let module_guard = self.module.read().unwrap();
            let module = module_guard.down_to_mod().unwrap();
            let val = module.get_last_added_global();

            self.print(val.clone());
            drop(module_guard); // prevent deadlock

            let mut module = self.module.write().unwrap();
            let module = module.down_to_mod_mut().unwrap();
            module.add_global("_", val);

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
                    Ok(Some(new_input)) if new_input.is_empty() => {
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

    /// Print eval result if it's not nil.
    fn print(&self, obj_ref: ObjectRef) {
        let obj = obj_ref.read().unwrap();
        if !obj.is_nil() {
            println!("{:?}", &*obj);
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
        if let ExeErrKind::ScanErr(kind) = err.kind {
            use ScanErrKind::*;
            if let ExpectedBlock
            | ExpectedIndentedBlock(_)
            | UnmatchedOpeningBracket(_)
            | UnterminatedStr(_) = kind
            {
                return true;
            }
        } else if let ExeErrKind::ParseErr(kind) = err.kind {
            use ParseErrKind::*;
            if let ExpectedBlock(_) = kind {
                return true;
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
