use crate::types::BuiltinFn;

use super::file::*;
use super::print::*;
use super::types::*;

/// Get the specs for all builtin functions. A spec comprises a name,
/// formal parameters, function pointer. If the parameters are `None`,
/// that means the function accepts a variable number of args.
pub fn get_builtin_func_specs<'a>() -> Vec<(&'a str, Vec<&'a str>, BuiltinFn)> {
    vec![
        // File
        ("read_file", vec!["file_name"], read_file),
        ("read_file_lines", vec!["file_name"], read_file_lines),
        // Print
        ("print", vec![""], print),
        // Type
        ("type_of", vec!["object"], type_of),
        ("obj_id", vec!["object"], obj_id),
    ]
}
