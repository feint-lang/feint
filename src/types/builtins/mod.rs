pub(crate) use self::bool::{Bool, FALSE, TRUE};
pub(crate) use float::Float;
pub(crate) use int::Int;
pub(crate) use nil::{Nil, NIL};
pub(crate) use types::BUILTIN_TYPES;

mod bool;
mod cmp;
mod float;
mod int;
mod nil;
mod types;
