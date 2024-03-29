use crate::exe::Executor;
use crate::result::ExeResult;

fn run_text(text: &str) -> ExeResult {
    let mut exe = Executor::new(16, vec![], false, false, false);
    exe.bootstrap()?;
    exe.execute_text(text)
}

fn assert_result_is_ok(result: ExeResult) {
    assert!(result.is_ok(), "{:?}", result.err());
}

fn assert_result_is_err(result: ExeResult) {
    assert!(result.is_err(), "{:?}", result);
}

mod basics {
    use super::*;

    #[test]
    fn test_add() {
        assert_result_is_ok(run_text("1 + 2"));
    }

    #[test]
    fn test_to_str() {
        assert_result_is_ok(run_text("1.to_str == \"1\""));
        assert_result_is_ok(run_text("[].to_str == \"[]\""));
    }
}

mod err {
    use super::*;

    #[test]
    fn test_new() {
        assert_result_is_ok(run_text(
            "Err.new(ErrType.assertion, \"assertion failed :(\")",
        ));
    }

    #[test]
    fn test_every_obj_has_err_attr() {
        assert_result_is_ok(run_text("nil.err"));
        assert_result_is_ok(run_text("true.err"));
        assert_result_is_ok(run_text("false.err"));
        assert_result_is_ok(run_text("1.err"));
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
            "l = []\nl.push(1)\nl.push('a')\nprint(l.length == 2)",
        ));
    }

    #[test]
    fn test_extend() {
        assert_result_is_ok(run_text(
            "l = []\nl.extend([1, 'a', 2, ()])\nprint(l.length == 4)",
        ));
    }

    #[test]
    fn test_pop() {
        assert_result_is_ok(run_text("l = [1]\nl.pop()\nprint(l.length == 0)"));
    }
}

mod str {
    use super::*;

    #[test]
    fn test_new() {
        assert_result_is_ok(run_text("Str.new(\"string\")"));
        assert_result_is_ok(run_text("Str.new(nil) == \"nil\""));
        assert_result_is_ok(run_text("Str.new(true) == \"true\""));
        assert_result_is_ok(run_text("Str.new(false) == \"false\""));
        assert_result_is_ok(run_text("Str.new(1) == \"1\""));
        assert_result_is_ok(run_text("Str.new(1.0) == \"1.0\""));
        assert_result_is_ok(run_text("Str.new(()) == \"()\""));
        assert_result_is_ok(run_text("Str.new([]) == \"[]\""));
        assert_result_is_ok(run_text("Str.new({}) == \"{}\""));
        assert_result_is_ok(run_text(
            "Str.new(print).starts_with(\"function print/0+ @\")",
        ));
        assert_result_is_ok(run_text(
            "Str.new(() => nil).starts_with(\"function <anonymous>/0 @\")",
        ));
    }

    #[test]
    fn test_starts_with() {
        assert_result_is_ok(run_text("'abc'.starts_with('a')"));
    }

    #[test]
    fn test_starts_with_bad_arg() {
        assert_result_is_err(run_text("assert('abc'.starts_with(1), '', true)"));
    }
}

mod tuple {
    use super::*;

    #[test]
    fn test_map() {
        assert_result_is_ok(run_text("t = (1, 2)\nt.map((item, i) => (item, i))"));
    }
}
