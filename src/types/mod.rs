pub(crate) use builtins::Builtins;
pub(crate) use object::ObjectRef;

pub(crate) mod builtins;

mod class;
mod complex;
mod object;

#[cfg(test)]
mod tests;
