use crate::exe::Executor;
use crate::repl::Repl;
use crate::vm::VM;

#[test]
fn eval_empty() {
    eval("");
}

#[test]
fn eval_arithmetic() {
    eval("2 * (3 + 4)");
}

#[test]
fn eval_string() {
    eval("\"abc\"");
}

#[test]
fn eval_multiline_string() {
    eval("\"a \nb c\"");
}

// TODO: Figure out how to automatically send closing quote and
//       newline to stdin.
#[test]
fn eval_unterminated_string() {
    eval("x = \"abc");
}

#[test]
fn eval_if_with_no_block() {
    eval("if true ->");
}

// Utilities -----------------------------------------------------------
fn eval(input: &str) {
    let mut vm = VM::default();
    let executor = Executor::new(&mut vm, false, false, false);
    let mut repl = Repl::new(None, executor);
    match repl.eval(input, true) {
        Some(Ok(_)) => assert!(false),
        Some(Err(_)) => assert!(false),
        None => assert!(true), // eval returns None on valid input
    }
}
