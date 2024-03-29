// Objects
pub(crate) use base::{ObjectRef, ObjectTrait};
pub(crate) use map::Map;

// Namespacing
pub(crate) use module::Module;
pub(crate) use ns::Namespace;

// Functions
pub(crate) use func::Func;
pub(crate) use func_trait::FuncTrait;
pub(crate) use intrinsic_func::IntrinsicFunc;

pub(crate) mod new;
pub(crate) use result::{Args, Params, ThisOpt};

mod base;
mod func_trait;

// Namespace (not a type)
pub(crate) mod ns;

// Intrinsic Types
pub(crate) mod always;
pub(crate) mod bool;
pub(crate) mod bound_func;
pub(crate) mod cell;
pub(crate) mod class;
pub(crate) mod closure;
pub(crate) mod custom;
pub(crate) mod err;
pub(crate) mod err_type;
pub(crate) mod file;
pub(crate) mod float;
pub(crate) mod func;
pub(crate) mod gen;
pub(crate) mod int;
pub(crate) mod intrinsic_func;
pub(crate) mod iterator;
pub(crate) mod list;
pub(crate) mod map;
pub(crate) mod module;
pub(crate) mod nil;
pub(crate) mod prop;
pub(crate) mod result;
pub(crate) mod seq;
pub(crate) mod str;
pub(crate) mod tuple;
pub(crate) mod util;
