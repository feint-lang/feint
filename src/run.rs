use std::fs;

use dirs;
use rustyline::error::ReadlineError;

use crate::instructions::Instruction;
use crate::scanner::{Scanner, TokenWithPosition};
use crate::tokens::Token;
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
                    println!("{}", source.trim_end());
                }
                // match self.get_instructions(source.as_str(), true) {
                //     Ok(instructions) => instructions,
                //     Err(message) => return self.halt_with_err(1, message),
                // }
                let i: Vec<Instruction> = vec![];
                i
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

        match self.vm.execute(&instructions) {
            VMState::Halted(0, option_message) => Ok(option_message),
            VMState::Halted(code, Some(message)) => Err((code, message)),
            VMState::Halted(code, None) => Err((code, "Unknown Error".to_string())),
            VMState::Idle => Err((i32::MAX, "Execution never halted".to_string())),
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
                    // Save history before eval in case of exit
                    rl.add_history_entry(input.clone());
                    match rl.save_history(history_path) {
                        Ok(_) => (),
                        Err(err) => eprintln!("Could not save REPL history: {}", err),
                    }

                    // Evaluate the input. If eval returns a result of
                    // any kind (ok or err), exit the loop and shut down
                    // the REPL. This will happen when the user types
                    // "exit" or Ctrl-D, etc. It will also happen if an
                    // error is encountered in tokenizing or running
                    // instructions.
                    match self.eval(input, line) {
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

    fn eval(&mut self, source: String, line: usize) -> Option<ExitResult> {
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
                let tokens;

                loop {
                    match self.get_tokens(source, false) {
                        Ok(t) => match t.last() {
                            Some(Token::NeedsMoreInput(_)) => return None,
                            _ => {
                                tokens = t;
                                break;
                            }
                        },
                        Err(message) => {
                            return Some(Err((1, message)));
                        }
                    }
                }

                let mut instructions = match self.get_instructions(tokens, false) {
                    Ok(instructions) => instructions,
                    Err(message) => return Some(Err((1, message))),
                };

                instructions
            }
        };

        match self.vm.execute(&instructions) {
            VMState::Halted(0, option_message) => Some(Ok(option_message)),
            VMState::Halted(code, Some(message)) => Some(Err((code, message))),
            VMState::Halted(code, None) => Some(Err((code, "Unknown Error".to_string()))),
            VMState::Idle => None,
        }
    }

    fn get_tokens(&self, source: String, finalize: bool) -> Result<Vec<Token>, String> {
        let mut scanner = Scanner::new();
        match scanner.scan(source, finalize) {
            Ok(tokens) => {
                if self.debug {
                    for t in tokens.iter() {
                        eprintln!("{:?}", t);
                    }
                }
                Ok(tokens.iter().map(|t| t.token.clone()).collect())
            }
            Err(message) => Err(message),
        }
    }

    fn get_instructions(
        &self,
        tokens: Vec<Token>,
        with_eof_halt: bool,
    ) -> Result<Vec<Instruction>, String> {
        let mut instructions: Vec<Instruction> = vec![];
        if with_eof_halt {
            for token in tokens {
                if token == Token::EndOfInput {
                    instructions.push(Instruction::Halt(0));
                }
            }
        }
        Ok(instructions)
    }
}
