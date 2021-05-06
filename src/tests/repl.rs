use std::io::stdin;

use crate::namespace::Namespace;
use crate::repl::Runner;
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

// TODO: Figure out how to automatically send closing quote and newline
// #[test]
// fn eval_unterminated_string() {
//     eval("x = \"abc");
// }

// Utilities -----------------------------------------------------------

fn new<'a>() -> Runner<'a> {
    let namespace = Namespace::new(None);
    let vm = VM::new(namespace);
    Runner::new(None, vm, false)
}

fn eval(input: &str) {
    let mut runner = new();
    match runner.eval(input) {
        Some(Ok(string)) => assert!(false),
        Some(Err((code, string))) => assert!(false),
        None => assert!(true), // eval returns None on valid input
    }
}
