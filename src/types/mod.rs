pub(crate) use object::*;
pub(crate) use result::*;
pub(crate) use types::*;

pub(crate) mod builtins;

mod object;
mod result;
mod types;

#[cfg(test)]
mod tests;
