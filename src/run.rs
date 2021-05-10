/// Run provided source, typically from a file, to completion.
use std::fs;

use crate::scanner::{scan, ScanError, ScanErrorType};
use crate::vm::{Instruction, Namespace, VMState, VM};

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
        match scan(source) {
            Ok(tokens) => {
                if self.debug {
                    for t in tokens.iter() {
                        eprintln!("{}", t);
                    }
                }
                tokens
            }
            Err(err) => {
                return match err {
                    ScanError { error: ScanErrorType::UnknownToken(c), location } => {
                        let message = format!(
                            "Syntax error: unknown token at line {} column {}: {}",
                            location.line, location.col, c
                        );
                        Err((1, message))
                    }
                    ScanError {
                        error: ScanErrorType::UnterminatedString(string),
                        location,
                    } => {
                        let message = format!(
                            "{}\nUnterminated string starting on line {} at column {}",
                            string, location.line, location.col
                        );
                        Err((1, message))
                    }
                    err => Err((1, format!("Unhandled scan error: {:?}", err))),
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
