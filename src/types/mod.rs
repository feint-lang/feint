pub(crate) use builtins::Builtins;
pub(crate) use object::ObjectRef;

mod bool;
mod builtins;
mod class;
mod complex;
mod float;
mod function;
mod int;
mod nil;
mod object;
mod string;
mod tuple;
mod util;

#[cfg(test)]
mod tests;
