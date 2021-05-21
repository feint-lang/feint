pub(crate) use object::{Fundamental, Object, ObjectTrait};
pub(crate) use result::ErrorKind;
pub(crate) use types::Type;

mod builtins;
mod object;
mod result;
mod types;

#[cfg(test)]
mod tests;
