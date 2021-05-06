use std::io::stdin;

use crate::repl::Runner;

fn new<'a>() -> Runner<'a> {
    Runner::new(None, false)
}

fn eval(input: &str) {
    let mut runner = new();
    match runner.eval(input) {
        Some(Ok(string)) => assert!(false),
        Some(Err((code, string))) => assert!(false),
        None => assert!(true), // eval returns None on valid input
    }
}

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
fn eval_unterminated_string() {
    eval("x = \"abc");
}
