pub(crate) use boolean::Bool;
pub(crate) use float::Float;
pub(crate) use int::Int;
pub(crate) use kind::Type;
pub(crate) use method::Method;
pub(crate) use object::Object;

mod boolean;
mod float;
mod int;
mod kind;
mod method;
mod object;

use std::collections::HashMap;

pub(crate) fn init_builtin_types() -> HashMap<&'static str, Type<'static>> {
    let mut types = HashMap::new();

    let slots = vec!["value"];
    let methods = HashMap::new();
    let kind = Type::new("builtins", "Int", slots, methods);
    types.insert(kind.name, kind);

    types
}
