pub mod code;
pub mod new;

pub use base::{ObjectRef, ObjectTrait};
pub use func::Func;
pub use func_trait::FuncTrait;
pub use intrinsic_func::IntrinsicFunc;
pub use map::Map;
pub use module::Module;

pub type ThisOpt = Option<ObjectRef>;
pub type Params = Vec<String>;
pub type Args = Vec<ObjectRef>;
pub type CallResult = ObjectRef;

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
pub(crate) mod int;
pub(crate) mod intrinsic_func;
pub(crate) mod iterator;
pub(crate) mod list;
pub(crate) mod map;
pub(crate) mod module;
pub(crate) mod nil;
pub(crate) mod prop;
pub(crate) mod seq;
pub(crate) mod str;
pub(crate) mod tuple;
pub(crate) mod util;
