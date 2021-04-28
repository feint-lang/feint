use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::Path;

use dirs;
use rustyline::error::ReadlineError;

use crate::instructions::Instruction;
use crate::scanner::{Scanner, TokenWithPosition};
use crate::tokens::Token;
use crate::types::Type;
use crate::vm::{VMState, VM};

type ExitResult = Result<Option<String>, (i32, String)>;

pub struct Runner<'a> {
    pub vm: VM<'a>,
    pub debug: bool,
}

// Facade for running code on VM
impl<'a> Runner<'a> {
    pub fn new(debug: bool) -> Runner<'a> {
        Runner {
            vm: VM::new(),
            debug,
        }
    }

    pub fn run(&mut self, file_name: &str) -> ExitResult {
        let mut instructions = match fs::read_to_string(file_name) {
            Ok(source) => {
                if self.debug {
                    println!("# Source from file: {}", file_name);
                    println!("{}", source);
                }
                self.get_instructions(source.as_str())
            }
            Err(err) => {
                self.vm.halt();
                return Err((1, format!("Could not read source file: {}", err)));
            }
        };

        // XXX: TEMP
        instructions.clear();
        instructions.push(Instruction::Push(1));
        instructions.push(Instruction::Push(2));
        instructions.push(Instruction::Add);
        instructions.push(Instruction::Halt(0));

        let state = self.vm.execute(&instructions);

        match self.get_exit_result(state, false) {
            Some(result) => result,
            None => panic!("This should never happen"),
        }
    }

    /// Get exit result based on VM state. If None is returned, that's
    /// indication to not exit (used in REPL mode).
    fn get_exit_result(&self, state: VMState, repl_mode: bool) -> Option<ExitResult> {
        match state {
            VMState::Halted(0, None) => Some(Ok(None)),
            VMState::Halted(0, message) => Some(Ok(message)),
            VMState::Halted(code, Some(message)) => Some(Err((code, message))),
            VMState::Halted(code, None) => Some(Err((code, "Error".to_string()))),
            VMState::Idle if repl_mode => None,
            VMState::Idle => Some(Err((i32::MAX, "Execution never halted".to_string()))),
        }
    }

    pub fn repl(&mut self) -> ExitResult {
        let mut rl = rustyline::Editor::<()>::new();

        let home = dirs::home_dir();
        let base_path = home.unwrap_or_default();
        let history_path_buf = base_path.join(".interpreter_history");
        let history_path = history_path_buf.as_path();

        println!("Welcome to the FeInt REPL (read/eval/print loop)");
        println!("Type a line of code, then hit Enter to evaluate it");
        println!("Type 'exit' or 'quit' to exit (without quotes)");
        println!(
            "REPL history will be saved to {}",
            history_path.to_string_lossy()
        );

        match rl.load_history(history_path) {
            Ok(_) => (),
            Err(err) => eprintln!("Could not load REPL history: {}", err),
        }

        let mut line = 1;

        loop {
            match rl.readline("→ ") {
                Ok(input) if input.trim().len() == 0 => {
                    // Skip empty/blank lines
                }
                Ok(input) => {
                    let input = input.as_str();

                    // Save history before eval in case of exit
                    rl.add_history_entry(input);
                    match rl.save_history(history_path) {
                        Ok(_) => (),
                        Err(err) => eprintln!("Could not save REPL history: {}", err),
                    }

                    let state = self.eval(input, line);
                    match self.get_exit_result(state, true) {
                        Some(result) => return result,
                        None => (),
                    }
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    self.vm.halt();
                    return Ok(None);
                }
                Err(err) => {
                    self.vm.halt();
                    return Err((1, format!("Could not read line: {}", err)));
                }
            }
            line += 1;
        }
    }

    fn eval(&mut self, source: &str, line: usize) -> VMState {
        let instructions = match source.trim() {
            "exit" | "halt" | "quit" => {
                vec![Instruction::Halt(0)]
            }
            "print" => {
                vec![Instruction::Print(8)]
            }
            "push" => {
                vec![Instruction::Push(line)]
            }
            _ => {
                let mut instructions = self.get_instructions(source);
                instructions.pop(); // Drop EOF halt instruction
                instructions
            }
        };
        self.vm.execute(&instructions)
    }

    fn get_tokens(&self, source: &str) -> Vec<TokenWithPosition> {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan();
        if self.debug {
            for token in tokens.iter() {
                println!("{}", token);
            }
        }
        tokens
    }

    fn get_instructions(&self, source: &str) -> Vec<Instruction> {
        let tokens = self.get_tokens(source);
        let mut instructions: Vec<Instruction> = vec![];
        for token in tokens {
            if token.token == Token::Eof {
                instructions.push(Instruction::Halt(0));
            }
        }
        instructions
    }
}
