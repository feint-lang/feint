//! # FeInt
//!
//! FeInt is a stack-based bytecode interpreter.

pub mod repl;
pub mod run;

mod ast;
mod builtins;
mod parser;
mod result;
mod scanner;
mod util;
mod vm;
