use crate::run::*;

#[test]
fn test_run_text() {
    let source = "1 + 2";
    let result = run_text(source, 0, false, true);
    assert!(result.is_ok(), "{:?}", result.err());
}
