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

pub mod code;
pub mod new;

// Intrinsic Types
pub mod always;
pub mod bool;
pub mod bound_func;
pub mod cell;
pub mod class;
pub mod closure;
pub mod custom;
pub mod err;
pub mod err_type;
pub mod file;
pub mod float;
pub mod func;
pub mod int;
pub mod intrinsic_func;
pub mod iterator;
pub mod list;
pub mod map;
pub mod module;
pub mod nil;
pub mod prop;
pub mod seq;
pub mod str;
pub mod tuple;
pub mod util;
