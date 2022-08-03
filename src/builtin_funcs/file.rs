use std::fs::{read_to_string, File};
use std::io::{BufRead, BufReader};

use crate::types::{create, Args, CallResult, This};
use crate::vm::{RuntimeErr, RuntimeErrKind, VM};

/// Read file into a string.
/// Returns Str
pub fn read_file(_this: This, args: Args, _vm: &mut VM) -> CallResult {
    let arg = args.get(0).unwrap();
    let arg = arg.read().unwrap();
    if let Some(file_name) = arg.get_str_val() {
        match read_to_string(file_name) {
            Ok(contents) => Ok(create::new_str(contents)),
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
pub fn read_file_lines(_this: This, args: Args, _vm: &mut VM) -> CallResult {
    let arg = args.get(0).unwrap();
    let arg = arg.read().unwrap();
    if let Some(file_name) = arg.get_str_val() {
        let file = File::open(file_name).unwrap();
        let reader = BufReader::new(file);
        let mut items = vec![];
        for line in reader.lines() {
            let item = create::new_str(line.unwrap());
            items.push(item);
        }
        Ok(create::new_tuple(items))
    } else {
        Err(RuntimeErr::new_type_err("Expected string"))
    }
}
