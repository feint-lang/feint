use crate::exe::Executor;
use crate::result::{ExeErrKind, ExeResult};
use crate::vm::RuntimeErrKind;

fn execute(source: &str) -> ExeResult {
    let mut exe = Executor::new(16, vec![], false, false, false);
    exe.execute_text(source)
}

#[test]
fn test_too_much_recursion() {
    let result = execute("f = () => f()\nf()");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err.kind,
        ExeErrKind::RuntimeErr(RuntimeErrKind::RecursionDepthExceeded(_))
    ));
}
