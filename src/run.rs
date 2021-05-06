/// Run provided source, typically from a file, to completion.
use std::fs;

use crate::instructions::Instruction;
use crate::namespace::Namespace;
use crate::scanner::scan;
use crate::tokens::{Token};
use crate::vm::{VMState, VM};

type ExitData = (i32, String);
type ExitResult = Result<Option<String>, ExitData>;

pub fn run(source: &str, debug: bool) -> ExitResult {
    let namespace = Namespace::default();
    let vm = VM::new(namespace);
    let mut runner = Runner::new(vm, debug);
    runner.run(source)
}

pub fn run_file(file_name: &str, debug: bool) -> ExitResult {
    let namespace = Namespace::default();
    let vm = VM::new(namespace);
    let mut runner = Runner::new(vm, debug);
    runner.run_file(file_name)
}

pub struct Runner<'a> {
    vm: VM<'a>,
    debug: bool,
}

impl<'a> Runner<'a> {
    pub fn new(vm: VM<'a>, debug: bool) -> Self {
        Runner { vm, debug }
    }

    pub fn run_file(&mut self, file_name: &str) -> ExitResult {
        match fs::read_to_string(file_name) {
            Ok(source) => {
                if self.debug {
                    println!("# Source from file: {}", file_name);
                    println!("{}", source.trim_end());
                }
                self.run(source.as_str())
            }
            Err(err) => Err((1, format!("Could not read source file: {}", err))),
        }
    }

    pub fn run(&mut self, source: &str) -> ExitResult {
        let tokens = match scan(source, 1, 1) {
            Ok(tokens) => {
                if self.debug {
                    for t in tokens.iter() {
                        eprintln!("{}", t);
                    }
                }
                tokens
            }
            Err((error_token, _)) => {
                return match error_token.token {
                    Token::Unknown(c) => {
                        let message = format!(
                            "Syntax error: unknown token at line {} column {}: {}",
                            error_token.line_no, error_token.col_no, c
                        );
                        Err((1, message))
                    }
                    Token::UnterminatedString(string) => {
                        let message = format!(
                            "{}\nUnterminated string starting on line {} at column {}",
                            string, error_token.line_no, error_token.col_no
                        );
                        Err((1, message))
                    }
                    _ => {
                        // This shouldn't happen.
                        Err((1, format!("{:?}", error_token)))
                    }
                };
            }
        };

        let mut instructions: Vec<Instruction> = vec![];
        instructions.push(Instruction::Push(1));
        instructions.push(Instruction::Push(2));
        instructions.push(Instruction::Add);
        instructions.push(Instruction::Halt(0));

        match self.vm.execute(&instructions) {
            VMState::Halted(0, option_message) => Ok(option_message),
            VMState::Halted(code, Some(message)) => Err((code, message)),
            VMState::Halted(code, None) => Err((code, "Unknown error".to_string())),
            VMState::Idle => Err((i32::MAX, "Execution never halted".to_string())),
        }
    }
}
