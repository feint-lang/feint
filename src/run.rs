use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

use dirs;
use rustyline::error::ReadlineError;

use crate::opcodes::OpCode;
use crate::scanner::Scanner;
use crate::types::Type;
use crate::vm::VM;

pub struct Runner {
    pub vm: VM<OpCode>,
}

// Facade for running code on VM
impl Runner {
    pub fn new() -> Runner {
        Runner { vm: VM::new() }
    }

    pub fn run(&mut self, file_name: &str) {
        let result = fs::read_to_string(file_name);

        if result.is_err() {
            eprintln!("Could not read source file");
            return;
        };

        let source = result.unwrap();
        println!("{}", source);
        println!();
        let mut scanner = Scanner::new(source.as_str());
        let tokens = scanner.scan();

        for token in tokens {
            println!("{}", token);
        }

        // TODO: Send parsed code to VM and run
        //self.vm.run(Vec::new());
    }

    pub fn repl(&self) {
        let mut rl = rustyline::Editor::<()>::new();

        let home = dirs::home_dir();
        let base_path = home.unwrap_or_default();
        let history_path_buf = base_path.join(".interpreter_history");
        let history_path = history_path_buf.as_path();

        println!("Welcome to the interpreter REPL (read/eval/print loop)");
        println!("Type a line of code, then hit Enter to evaluate it");
        println!("Type 'exit' or 'quit' to exit (without quotes)");
        println!("REPL history will be saved to {}", history_path.to_string_lossy());

        match rl.load_history(history_path) {
            Ok(_) => (),
            Err(err) => eprintln!("Could not load REPL history: {}", err),
        }

        loop {
            let result = rl.readline("→ ");
            match result {
                Ok(input) => {
                    rl.add_history_entry(input.as_str());
                    if input == "exit" || input == "quit" {
                        break;
                    }
                    let result = self.eval(input.as_str());
                    self.print(result);
                }
                Err(ReadlineError::Interrupted) |
                Err(ReadlineError::Eof) => {
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }

        match rl.save_history(history_path) {
            Ok(_) => (),
            Err(err) => eprintln!("Could not save REPL history: {}", err),
        }
    }

    fn eval(&self, input: &str) -> &str {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan();
        for token in tokens {
            println!("{}", token);
        }
        //self.vm.run(Vec::new());
        ""
    }

    fn print(&self, string: &str) {
        if string.len() == 0 {
            print!("{}", string);
            if io::stdout().flush().is_err() {
                eprintln!("Error while flushing")
            }
        } else {
            println!("{}", string);
        }
    }
}
