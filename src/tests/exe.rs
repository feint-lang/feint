use crate::exe::Executor;
use crate::result::ExeErrKind;
use crate::vm::{RuntimeContext, RuntimeErrKind, VM};

#[test]
fn test_recursive_func() {
    let mut vm = VM::new(RuntimeContext::new(), 8);
    let mut exe = Executor::default(&mut vm);
    let source = "f = () -> f()\nf()";
    let result = exe.execute_text(source, None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err.kind,
        ExeErrKind::RuntimeErr(RuntimeErrKind::RecursionDepthExceeded(_))
    ));
}
