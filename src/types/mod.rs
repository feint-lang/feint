pub(crate) use builtins::Builtins;
pub(crate) use func::Func;
pub(crate) use native::NativeFn;
pub(crate) use object::ObjectRef;

mod bool;
mod builtins;
mod class;
mod complex;
mod float;
mod func;
mod int;
mod native;
mod nil;
mod object;
mod result;
mod str;
mod tuple;
mod util;

#[cfg(test)]
mod tests;
