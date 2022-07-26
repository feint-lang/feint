use std::fs::{read_to_string, File};
use std::io::{BufRead, BufReader};

use crate::types::{Args, CallResult};
use crate::vm::{RuntimeContext, RuntimeErr, RuntimeErrKind};

/// Read file into a string.
pub fn read_file(args: Args, ctx: &RuntimeContext) -> CallResult {
    let arg = args.get(0).unwrap();
    if let Some(file_name) = arg.str_val() {
        match read_to_string(file_name) {
            Ok(contents) => Ok(Some(ctx.builtins.new_string(contents))),
            Err(err) => {
                Err(RuntimeErr::new(RuntimeErrKind::CouldNotReadFile(err.to_string())))
            }
        }
    } else {
        Err(RuntimeErr::new_type_err("Expected a string"))
    }
}

/// Read lines of file into tuple.
pub fn read_file_lines(args: Args, ctx: &RuntimeContext) -> CallResult {
    let arg = args.get(0).unwrap();
    if let Some(file_name) = arg.str_val() {
        let file = File::open(file_name).unwrap();
        let reader = BufReader::new(file);
        let mut items = vec![];
        for line in reader.lines() {
            let item = ctx.builtins.new_string(line.unwrap());
            items.push(item);
        }
        Ok(Some(ctx.builtins.new_tuple(items)))
    } else {
        Err(RuntimeErr::new_type_err("Expected a string"))
    }
}
