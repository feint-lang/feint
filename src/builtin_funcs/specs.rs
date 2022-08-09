use crate::types::BuiltinFn;

use super::file::*;
use super::print::*;
use super::types::*;

/// Get the specs for all builtin functions. A spec comprises a name,
/// formal parameters, function pointer. If the parameters are `None`,
/// that means the function accepts a variable number of args.
pub fn get_builtin_func_specs<'a>() -> Vec<(&'a str, Option<Vec<&'a str>>, BuiltinFn)> {
    vec![
        // File
        ("read_file", Some(vec!["file_name"]), read_file),
        ("read_file_lines", Some(vec!["file_name"]), read_file_lines),
        // Print
        ("print", None, print),
        // Type
        ("type_of", Some(vec!["object"]), type_of),
        ("obj_id", Some(vec!["object"]), obj_id),
    ]
}
