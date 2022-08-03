pub(crate) use specs::get_builtin_func_specs;

// Functions for builtin types (AKA "methods")
pub mod float;
pub mod int;

mod file;
mod print;
mod specs;
mod types;
