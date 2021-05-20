//! # FeInt
//!
//! FeInt is a stack-based bytecode interpreter.

pub mod repl;
pub mod run;

mod ast;
mod parser;
mod result;
mod scanner;
mod types;
mod util;
mod vm;
