use crate::types::BuiltinFn;

use super::file::*;
use super::print::*;
use super::types::*;

/// Get the specs for all builtin functions. A spec comprises a name,
/// function pointer, and arity. An arity of `None` means the function
/// accepts a variable number of args.
pub fn get_builtin_func_specs<'a>() -> Vec<(&'a str, BuiltinFn, Option<u8>)> {
    vec![
        // File
        ("read_file", read_file, Some(1)),
        ("read_file_lines", read_file_lines, Some(1)),
        // Print
        ("print", print, None),
        // Type
        ("type_of", type_of, None),
        ("obj_id", obj_id, None),
    ]
}
