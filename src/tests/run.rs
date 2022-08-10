use crate::result::ExitResult;
use crate::run;

fn run_text(source: &str) -> ExitResult {
    run::run_text(source, 16, false, false)
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

mod float {
    use super::*;

    #[test]
    fn test_new() {
        assert_result_is_ok(run_text("Float.new(1)"));
    }
}

mod int {
    use super::*;

    #[test]
    fn test_new() {
        assert_result_is_ok(run_text("Int.new(1)"));
    }
}

mod list {
    use super::*;

    #[test]
    fn test_equal() {
        assert_result_is_ok(run_text("print([] == [])"));
        assert_result_is_ok(run_text("print([1] == [1.0])"));
        assert_result_is_ok(run_text("print([1, 'a'] == [1, 'a'])"));
    }

    #[test]
    fn test_push() {
        assert_result_is_ok(run_text(
            "l = []\nl.push(1)\nl.push('a')\nprint(l.length() == 2)",
        ));
    }

    #[test]
    fn test_extend() {
        assert_result_is_ok(run_text(
            "l = []\nl.extend([1, 'a', 2, ()])\nprint(l.length() == 4)",
        ));
    }

    #[test]
    fn test_pop() {
        assert_result_is_ok(run_text("l = [1]\nl.pop()\nprint(l.length() == 0)"));
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
