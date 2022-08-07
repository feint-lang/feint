use crate::result::ExitResult;
use crate::run;

fn run_text(source: &str) -> ExitResult {
    run::run_text(source, 16, false, true)
}

fn assert_result_is_ok(result: ExitResult) {
    assert!(result.is_ok(), "{:?}", result.err());
}

fn assert_result_is_err(result: ExitResult) {
    assert!(result.is_err(), "{:?}", result);
}

mod basics {
    use super::*;

    #[test]
    fn test_add() {
        assert_result_is_ok(run_text("1 + 2"));
    }
}

mod int {
    use super::*;

    #[test]
    fn test_new() {
        assert_result_is_ok(run_text("Int.new(1)"));
    }
}

mod float {
    use super::*;

    #[test]
    fn test_new() {
        assert_result_is_ok(run_text("Float.new(1)"));
    }
}

mod str {
    use super::*;

    #[test]
    fn test_starts_with() {
        assert_result_is_ok(run_text("'abc'.starts_with('a')"));
    }

    #[test]
    fn test_starts_with_bad_arg() {
        assert_result_is_err(run_text("'abc'.starts_with(1)"));
    }
}

mod tuple {
    use super::*;

    #[test]
    fn test_map() {
        assert_result_is_ok(run_text("t = (1, 2)\nt.map((item, i) -> (item, i))"));
    }
}
