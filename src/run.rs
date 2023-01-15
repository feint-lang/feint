//! # FeInt code runner
use std::path::Path;

use crate::exe::Executor;
use crate::result::{ExeErrKind, ExeResult, ExitResult};
use crate::vm::{CallDepth, VMState};

/// Run source from file.
pub fn run_file(
    file_path: &Path,
    max_call_depth: CallDepth,
    argv: Vec<String>,
    dis: bool,
    debug: bool,
) -> ExitResult {
    let mut executor = Executor::new(max_call_depth, argv, false, dis, debug);
    let result = executor.execute_file(file_path);
    exit(result)
}

/// Read and run source from stdin.
pub fn run_stdin(
    max_call_depth: CallDepth,
    argv: Vec<String>,
    dis: bool,
    debug: bool,
) -> ExitResult {
    let mut executor = Executor::new(max_call_depth, argv, false, dis, debug);
    let result = executor.execute_stdin();
    exit(result)
}

/// Run text source.
pub fn run_text(
    text: &str,
    max_call_depth: CallDepth,
    argv: Vec<String>,
    dis: bool,
    debug: bool,
) -> ExitResult {
    let mut executor = Executor::new(max_call_depth, argv, false, dis, debug);
    let result = executor.execute_text(text);
    exit(result)
}

fn exit(result: ExeResult) -> ExitResult {
    match result {
        Ok(vm_state) => match vm_state {
            VMState::Running => {
                Err((255, Some("VM should be idle or halted, not running".to_owned())))
            }
            VMState::Idle(_) => Ok(None),
            VMState::Halted(0) => Ok(None),
            VMState::Halted(code) => Err((code, None)),
        },
        Err(err) => {
            let message = match err.kind {
                ExeErrKind::CouldNotReadSourceFile(message) => Some(message),
                _ => None,
            };
            Err((1, message))
        }
    }
}
