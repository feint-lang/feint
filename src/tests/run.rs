use crate::run::*;

#[test]
fn test_run_text() {
    let source = "1 + 2";
    let result = run_text(source, 0, false, true);
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_too_much_recursion() {
    let source = "f = () -> f()\nf()";
    let result = run_text(source, 8, false, true);
    // NOTE: This isn't a great test of too much recursion because the
    //       error could be due to parsing or some other issue other
    //       than recursion.
    assert!(result.is_err());
}
