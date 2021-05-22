pub(crate) use builtins::BUILTINS;
pub(crate) use class::Type;
pub(crate) use object::{Object, ObjectExt};

pub(crate) mod builtins;

mod class;
mod complex;
mod object;
mod result;

#[cfg(test)]
mod tests;
