use std::fs::{read_to_string, File};
use std::io::{BufRead, BufReader};

use crate::types::{Args, CallResult};
use crate::vm::{RuntimeErr, RuntimeErrKind, VM};

/// Read file into a string.
/// Returns Str
pub fn read_file(args: Args, vm: &mut VM) -> CallResult {
    let arg = args.get(0).unwrap();
    if let Some(file_name) = arg.str_val() {
        match read_to_string(file_name) {
            Ok(contents) => Ok(Some(vm.ctx.builtins.new_str(contents))),
            Err(err) => {
                Err(RuntimeErr::new(RuntimeErrKind::CouldNotReadFile(err.to_string())))
            }
        }
    } else {
        Err(RuntimeErr::new_type_err("Expected string"))
    }
}

/// Read lines of file into tuple.
/// Returns Tuple<Str>
pub fn read_file_lines(args: Args, vm: &mut VM) -> CallResult {
    let arg = args.get(0).unwrap();
    if let Some(file_name) = arg.str_val() {
        let file = File::open(file_name).unwrap();
        let reader = BufReader::new(file);
        let mut items = vec![];
        for line in reader.lines() {
            let item = vm.ctx.builtins.new_str(line.unwrap());
            items.push(item);
        }
        Ok(Some(vm.ctx.builtins.new_tuple(items)))
    } else {
        Err(RuntimeErr::new_type_err("Expected string"))
    }
}
