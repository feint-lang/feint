pub(crate) use builtin_func::BuiltinFn;
pub(crate) use builtin_types::BUILTIN_TYPES;
pub(crate) use builtins::Builtins;
pub(crate) use class::Type;
pub(crate) use func::Func;
pub(crate) use object::ObjectRef;
pub(crate) use result::{Args, CallResult};

mod bool;
mod builtin_func;
mod builtin_types;
mod builtins;
mod class;
mod custom;
mod float;
mod func;
mod int;
mod namespace;
mod nil;
mod object;
mod result;
mod str;
mod tuple;
mod util;

#[cfg(test)]
pub(crate) use object::{Object, ObjectExt};

mod tests;
