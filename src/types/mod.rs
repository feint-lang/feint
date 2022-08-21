// Objects
pub(crate) use base::{ObjectRef, ObjectTrait};

// Namespacing
pub(crate) use module::Module;
pub(crate) use ns::Namespace;

// Functions
pub(crate) use builtin_func::{BuiltinFn, BuiltinFunc};
pub(crate) use func::Func;
pub(crate) use func_trait::FuncTrait;

pub(crate) mod new;
pub(crate) use result::{Args, CallResult, Params, ThisOpt};

mod base;
mod func_trait;

// Namespace (not a type)
pub(crate) mod ns;

// Builtin Types
pub(crate) mod bool;
pub(crate) mod bound_func;
pub(crate) mod builtin_func;
pub(crate) mod cell;
pub(crate) mod class;
pub(crate) mod closure;
pub(crate) mod custom;
pub(crate) mod error;
pub(crate) mod float;
pub(crate) mod func;
pub(crate) mod gen;
pub(crate) mod int;
pub(crate) mod list;
pub(crate) mod map;
pub(crate) mod module;
pub(crate) mod nil;
pub(crate) mod result;
pub(crate) mod str;
pub(crate) mod tuple;
pub(crate) mod util;
