use crate::types::BuiltinFn;

use super::assert::*;
use super::print::*;
use super::types::*;

/// Get the specs for all builtin functions. A spec comprises a name,
/// formal parameters, function pointer. If the parameters are `None`,
/// that means the function accepts a variable number of args.
pub fn get_builtin_func_specs<'a>() -> Vec<(&'a str, &'a [&'a str], BuiltinFn)> {
    vec![
        ("assert", &["assertion", ""], assert),
        // Print
        ("print", &[""], print),
        // Type
        ("type_of", &["object"], type_of),
        ("obj_id", &["object"], obj_id),
    ]
}
