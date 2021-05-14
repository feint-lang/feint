//! # FeInt

use super::result::ExitResult;
use super::scanner::{self, ScanError, ScanErrorKind};
use super::vm::{ExecutionResult, Instruction, Instructions, Namespace, VMState, VM};
use crate::scanner::TokenWithLocation;

/// Run text source.
pub fn run(source: &str, debug: bool) -> ExitResult {
    let namespace = Namespace::default();
    let vm = VM::new(namespace);
    let mut runner = Runner::new(vm, debug);
    runner.run(source)
}

/// Run source from file.
pub fn run_file(file_name: &str, debug: bool) -> ExitResult {
    let namespace = Namespace::default();
    let vm = VM::new(namespace);
    let mut runner = Runner::new(vm, debug);
    runner.run_file(file_name)
}

struct Runner<'a> {
    vm: VM<'a>,
    debug: bool,
}

impl<'a> Runner<'a> {
    fn new(vm: VM<'a>, debug: bool) -> Self {
        Runner { vm, debug }
    }

    fn run(&mut self, text: &str) -> ExitResult {
        self.process_scan_result(scanner::scan(text))
    }

    fn run_file(&mut self, file_name: &str) -> ExitResult {
        self.process_scan_result(scanner::scan_file(file_name))
    }

    fn process_scan_result(
        &mut self,
        tokens: Result<Vec<TokenWithLocation>, ScanError>,
    ) -> ExitResult {
        match tokens {
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
                    ScanError {
                        error: ScanErrorKind::UnexpectedCharacter(c),
                        location,
                    } => {
                        let message = format!(
                            "Syntax error: unexpected character at line {} column {}: {}",
                            location.line, location.col, c
                        );
                        Err((1, message))
                    }
                    ScanError {
                        error: ScanErrorKind::UnterminatedString(string),
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

        let mut instructions: Instructions = vec![];
        instructions.push(Instruction::Halt(0));

        match self.vm.execute(&instructions) {
            Ok(VMState::Idle) => Err((1, "Execution never halted".to_string())),
            Ok(VMState::Halted(0)) => Ok(Some("Halted".to_owned())),
            Ok(VMState::Halted(code)) => {
                Err((code, format!("Halted abnormally: {}", code)))
            }
            Err(err) => Err((1, err.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_text() {
        let source = "x = 1\ny = 2\n1 + 2";
        if let (Ok(_)) = run(source, true) {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
