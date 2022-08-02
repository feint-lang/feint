//! # FeInt
//!
//! FeInt is a stack-based bytecode interpreter.

pub mod exe;
pub mod repl;
pub mod run;

mod ast;
mod builtin_funcs;
mod compiler;
mod format;
mod parser;
mod result;
mod scanner;
mod types;
mod util;
mod vm;

#[cfg(test)]
mod tests;
