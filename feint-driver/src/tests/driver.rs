use feint_vm::RuntimeErrKind;

use crate::driver::Driver;
use crate::result::{DriverErrKind, DriverResult};

fn execute(source: &str) -> DriverResult {
    let mut driver = Driver::new(16, vec![], false, false, false);
    driver.bootstrap()?;
    driver.execute_text(source)
}

#[test]
fn test_too_much_recursion() {
    let result = execute("f = () => f()\nf()");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err.kind,
        DriverErrKind::RuntimeErr(RuntimeErrKind::RecursionDepthExceeded(_))
    ));
}
