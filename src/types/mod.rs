pub(crate) use base::{ObjectRef, ObjectTrait, ObjectTraitExt};
pub(crate) use builtin_func::{BuiltinFn, BuiltinFunc};
pub(crate) use builtins::BUILTINS;
pub(crate) use func::Func;
pub(crate) use ns::Namespace;
pub(crate) use result::{Args, CallResult, Params, This};

pub(crate) mod create;

mod base;
mod bool;
mod bound_func;
mod builtin_func;
mod builtins;
mod class;
mod custom;
mod float;
mod func;
mod int;
mod meth;
mod module;
mod nil;
mod ns;
mod result;
mod str;
mod tuple;
mod util;

#[cfg(test)]
pub(crate) use base::TypeTraitExt;
