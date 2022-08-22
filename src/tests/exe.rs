use crate::exe::Executor;
use crate::result::{ExeErrKind, ExeResult};
use crate::vm::{RuntimeContext, RuntimeErrKind, VM};

fn execute(source: &str) -> ExeResult {
    let mut vm = VM::new(RuntimeContext::new(), 16);
    let mut exe = Executor::default(&mut vm);
    exe.execute_text(source)
}

#[test]
fn test_too_much_recursion() {
    let result = execute("f = () -> f()\nf()");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err.kind,
        ExeErrKind::RuntimeErr(RuntimeErrKind::RecursionDepthExceeded(_))
    ));
}
