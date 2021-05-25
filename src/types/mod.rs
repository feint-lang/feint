pub(crate) use builtins::Builtins;
pub(crate) use class::{Type, TypeRef};
pub(crate) use object::{Object, ObjectExt, ObjectRef};

pub(crate) mod builtins;

mod class;
mod complex;
mod object;
mod result;

#[cfg(test)]
mod tests;
