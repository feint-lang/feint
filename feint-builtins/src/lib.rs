#[macro_use]
extern crate bitflags;

pub use builtins::BUILTINS;

pub mod builtins;
pub mod modules;
pub mod types;

mod util;

#[cfg(test)]
mod tests;
