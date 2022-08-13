//! # FeInt
//!
//! FeInt is a stack-based bytecode interpreter.
extern crate core;

pub mod dis;
pub mod exe;
pub mod repl;
pub mod run;
pub mod vm;

mod ast;
mod builtin_funcs;
mod compiler;
mod format;
mod modules;
mod parser;
mod result;
mod scanner;
mod types;
mod util;

#[cfg(test)]
mod tests;
