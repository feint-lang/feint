//! # FeInt code runner
use crate::exe::Executor;
use crate::result::{ExeResult, ExitResult};
use crate::vm::{VMState, VM};

/// Run source from file.
pub fn run_file(file_path: &str, dis: bool, debug: bool) -> ExitResult {
    let mut vm = VM::default();
    let mut executor = Executor::new(&mut vm, false, dis, debug);
    let result = executor.execute_file(file_path);
    exit(result)
}

/// Read and run source from stdin.
pub fn run_stdin(dis: bool, debug: bool) -> ExitResult {
    let mut vm = VM::default();
    let mut executor = Executor::new(&mut vm, false, dis, debug);
    let result = executor.execute_stdin();
    exit(result)
}

/// Run text source.
pub fn run_text(text: &str, dis: bool, debug: bool) -> ExitResult {
    let mut vm = VM::default();
    let mut executor = Executor::new(&mut vm, false, dis, debug);
    let result = executor.execute_text(text);
    exit(result)
}

fn exit(result: ExeResult) -> ExitResult {
    match result {
        Ok(vm_state) => match vm_state {
            VMState::Halted(0) => Ok(None),
            VMState::Halted(code) => {
                Err((code, Some(format!("Halted abnormally: {}", code))))
            }
            VMState::Idle => Err((255, Some("Never halted".to_owned()))),
        },
        // TODO: Return error code depending on error type?
        Err(_) => Err((1, None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_text() {
        let source = "1 + 2";
        let result = run_text(source, false, true);
        assert!(result.is_ok(), "{:?}", result.err());
    }
}
