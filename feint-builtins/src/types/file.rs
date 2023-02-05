use std::any::Any;
use std::fmt;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use once_cell::sync::OnceCell;

use feint_code_gen::*;

use crate::BUILTINS;

use super::base::{ObjectRef, ObjectTrait, TypeRef};

use super::ns::Namespace;

// pub fn make_file_type() -> obj_ref_t!(FileType) {
//     let type_ref = obj_ref!(FileType::new());
//     let mut type_obj = type_ref.write().unwrap();
//
//     type_obj.add_attrs(&[
//         // Class Methods
//         meth!("new", type_ref, &["file_name"], "", |_, args| {
//             let arg = use_arg!(args, 0);
//             if let Some(file_name) = arg.get_str_val() {
//                 let path = Path::new(file_name);
//                 if path.is_file() {
//                     BUILTINS.file(file_name)
//                 } else {
//                     BUILTINS.file_not_found_err(file_name, BUILTINS.nil())
//                 }
//             } else {
//                 let message = format!("File.new(file_name) expected string; got {arg}");
//                 BUILTINS.arg_err(message, BUILTINS.nil())
//             }
//         }),
//         // Instance Attributes
//         prop!("text", type_ref, "", |this, _| {
//             let this = this.read().unwrap();
//             let this = this.down_to_file().unwrap();
//             this.text()
//         }),
//         prop!("lines", type_ref, "", |this, _| {
//             let this = this.read().unwrap();
//             let this = &mut this.down_to_file().unwrap();
//             this.lines()
//         }),
//     ]);
//
//     type_ref.clone()
// }

// File ----------------------------------------------------------

pub struct File {
    class: TypeRef,
    ns: Namespace,
    file_name: String,
    path: PathBuf,
    text: OnceCell<ObjectRef>,
    lines: OnceCell<ObjectRef>,
}

standard_object_impls!(File);

impl File {
    pub fn new(class: TypeRef, file_name: String) -> Self {
        let path = fs::canonicalize(&file_name);
        let path = path.map_or_else(|_| Path::new(&file_name).to_path_buf(), |p| p);
        let name_obj = BUILTINS.str(file_name.as_str());
        Self {
            class,
            ns: Namespace::with_entries(&[("name", name_obj)]),
            file_name,
            path,
            text: OnceCell::default(),
            lines: OnceCell::default(),
        }
    }

    fn text(&self) -> ObjectRef {
        let result = self.text.get_or_try_init(|| {
            fs::read_to_string(&self.file_name)
                .map(|contents| BUILTINS.str(contents))
                .map_err(|err| {
                    BUILTINS.file_unreadable_err(err.to_string(), BUILTINS.nil())
                })
        });
        match result {
            Ok(text) => text.clone(),
            Err(err) => err,
        }
    }

    fn lines(&self) -> ObjectRef {
        let result = self.lines.get_or_try_init(|| {
            let file = fs::File::open(&self.file_name);
            file.map(|file| {
                let reader = BufReader::new(file);
                let lines = reader
                    .lines()
                    // TODO: Handle lines that can't be read
                    .map(|line| BUILTINS.str(line.unwrap()))
                    .collect();
                BUILTINS.tuple(lines)
            })
            .map_err(|err| {
                BUILTINS.file_unreadable_err(err.to_string(), BUILTINS.nil())
            })
        });
        match result {
            Ok(lines) => lines.clone(),
            Err(err) => err,
        }
    }
}

impl ObjectTrait for File {
    object_trait_header!();

    fn bool_val(&self) -> Option<bool> {
        Some(false)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<file: {}>", &self.path.display())
    }
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
