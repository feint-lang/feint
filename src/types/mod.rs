pub(crate) use builtins::BUILTIN_TYPES;
pub(crate) use object::{AttributeValue, Object, ObjectTrait, Primitive};
pub(crate) use result::ErrorKind;
pub(crate) use types::Type;

mod builtins;
mod object;
mod result;
mod types;

#[cfg(test)]
mod tests;
