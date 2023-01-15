//! # FeInt
//!
//! FeInt is a stack-based bytecode interpreter.
pub mod config;
pub mod dis;
pub mod exe;
pub mod repl;
pub mod result;
pub mod vm;

mod ast;
mod compiler;
mod format;
mod modules;
mod parser;
mod scanner;
mod types;
mod util;

#[cfg(test)]
mod tests;
