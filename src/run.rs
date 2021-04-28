use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

use dirs;
use rustyline::error::ReadlineError;

use crate::opcodes::OpCode;
use crate::scanner::{Scanner, TokenWithPosition};
use crate::types::Type;
use crate::vm::VM;
use crate::tokens::Token;

type ExitResult = Result<String, (i32, String)>;

pub struct Runner<'a> {
    pub vm: VM<'a>,
}

// Facade for running code on VM
impl<'a> Runner<'a> {
    pub fn new() -> Runner<'a> {
        Runner { vm: VM::new() }
    }

    pub fn run(&mut self, file_name: &str) -> ExitResult {
        let mut instructions = match fs::read_to_string(file_name) {
            Ok(source) => {
                #[cfg(debug_assertions)]
                println!("{}", source);
                self.get_instructions(source.as_str())
            },
            Err(_) => {
                return Err((1, "Could not read source file".to_string()))
            }
        };

        // XXX: TEMP
        instructions.clear();
        instructions.push(OpCode::Push(1));
        instructions.push(OpCode::Push(2));
        instructions.push(OpCode::Add);
        instructions.push(OpCode::Halt(0));

        let result = self.vm.run(&instructions, 0);

        match self.vm.peek() {
            Some(a) => println!("Top of stack: {}", a),
            None => (),
        }

        result
    }

    pub fn repl(&mut self) -> ExitResult {
        let mut rl = rustyline::Editor::<()>::new();

        let home = dirs::home_dir();
        let base_path = home.unwrap_or_default();
        let history_path_buf = base_path.join(".interpreter_history");
        let history_path = history_path_buf.as_path();

        // There's one set of instructions for the REPL, which means we
        // need to take care to start interpreting instructions at the
        // right place.
        let mut instructions: Vec<OpCode> = vec!();

        println!("Welcome to the FeInt REPL (read/eval/print loop)");
        println!("Type a line of code, then hit Enter to evaluate it");
        println!("Type 'exit' or 'quit' to exit (without quotes)");
        println!("REPL history will be saved to {}", history_path.to_string_lossy());

        match rl.load_history(history_path) {
            Ok(_) => (),
            Err(err) => eprintln!("Could not load REPL history: {}", err),
        }

        loop {
            match rl.readline("→ ") {
                Ok(input) if input.trim().len() == 0 => {
                    // Skip empty/blank lines
                },
                Ok(input) => {
                    let input = input.as_str();

                    // Save history before eval in case of exit
                    rl.add_history_entry(input);
                    match rl.save_history(history_path) {
                        Ok(_) => (),
                        Err(err) => eprintln!("Could not save REPL history: {}", err),
                    }

                    match self.eval(input, &mut instructions) {
                        Ok(message) => return Ok(message),
                        Err((i32::MAX, _)) => (),
                        Err((exit_code, message)) => return Err((exit_code, message)),
                    }
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    return Ok("".to_string());
                },
                Err(err) => {
                    let message = format!("Could not read line: {}", err);
                    return Err((1, message));
                },
            }
        }
    }

    fn eval(
        &mut self,
        source: &str,
        repl_instructions: &mut Vec<OpCode<'a>>,
    ) -> ExitResult {
        let start = repl_instructions.len();
        match source.trim() {
            "exit" | "halt" | "quit" => {
                // Explicitly instruct the VM to halt
                repl_instructions.push(OpCode::Halt(0, ""));
            },
            _ => {
                let instructions = self.get_instructions(source);
                repl_instructions.extend(instructions);

                // XXX: TEMP
                repl_instructions.push(OpCode::Jump(start));
            },
        }
        self.vm.run(repl_instructions, start)
    }

    fn get_tokens(&self, source: &str) -> Vec<TokenWithPosition> {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan();
        #[cfg(debug_assertions)]
        for token in tokens.iter() {
            println!("{}", token);
        }
        tokens
    }

    fn get_instructions(&self, source: &str) -> Vec<OpCode<'a>> {
        let tokens = self.get_tokens(source);
        let mut instructions: Vec<OpCode> = vec!();
        for token in tokens {
            if token.token == Token::Eof {
                instructions.push(OpCode::Halt(0, ""));
            }
        }
        instructions
    }
}
