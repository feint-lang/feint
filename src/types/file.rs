use std::any::Any;
use std::fmt;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use once_cell::sync::{Lazy, OnceCell};

use crate::vm::{RuntimeBoolResult, RuntimeErr, RuntimeObjResult};

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// File Type ------------------------------------------------------------

gen::type_and_impls!(FileType, File);

pub static FILE_TYPE: Lazy<new::obj_ref_t!(FileType)> = Lazy::new(|| {
    let type_ref = new::obj_ref!(FileType::new());
    let mut class = type_ref.write().unwrap();

    class.ns_mut().add_entries(&[
        // Class Methods
        gen::meth!("new", type_ref, &["file_name"], |_, args, _| {
            let arg = gen::use_arg!(args, 0);
            if let Some(file_name) = arg.get_str_val() {
                Ok(new::file(file_name))
            } else {
                let message = format!("File.new(file_name) expected string; got {arg}");
                Err(RuntimeErr::arg_err(message))
            }
        }),
        // Instance Attributes
        gen::meth!("text", type_ref, &[], |this, _, _| {
            let this = this.read().unwrap();
            let this = this.down_to_file().unwrap();
            this.text()
        }),
        gen::meth!("lines", type_ref, &[], |this, _, _| {
            let mut this = this.write().unwrap();
            let this = &mut this.down_to_file_mut().unwrap();
            let mut this = this.read().unwrap();
            let this = &mut this.down_to_file().unwrap();
            this.lines()
        }),
    ]);

    type_ref.clone()
});

// File Object ----------------------------------------------------------

pub struct File {
    ns: Namespace,
    file_name: String,
    path: PathBuf,
    text: OnceCell<ObjectRef>,
    lines: OnceCell<ObjectRef>,
}

gen::standard_object_impls!(File);

impl File {
    pub fn new(file_name: String) -> Self {
        let path = fs::canonicalize(&file_name);
        let path = path.map_or_else(|_| Path::new(&file_name).to_path_buf(), |p| p);
        let name_obj = new::str(file_name.as_str());
        Self {
            ns: Namespace::with_entries(&[("name", name_obj)]),
            file_name,
            path,
            text: OnceCell::default(),
            lines: OnceCell::default(),
        }
    }

    fn text(&self) -> RuntimeObjResult {
        let text = self.text.get_or_try_init(|| {
            fs::read_to_string(&self.file_name)
                .map(new::str)
                .map_err(|err| RuntimeErr::could_not_read_file(err.to_string()))
        })?;
        Ok(text.clone())
    }

    fn lines(&self) -> RuntimeObjResult {
        let lines = self.lines.get_or_try_init(|| {
            let file = fs::File::open(&self.file_name);
            file.map(|file| {
                let reader = BufReader::new(file);
                let lines = reader
                    .lines()
                    // TODO: Handle lines that can't be read
                    .map(|line| new::str(line.unwrap()))
                    .collect();
                new::tuple(lines)
            })
            .map_err(|err| RuntimeErr::could_not_read_file(err.to_string()))
        })?;
        Ok(lines.clone())
    }
}

impl ObjectTrait for File {
    gen::object_trait_header!(FILE_TYPE);

    fn bool_val(&self) -> RuntimeBoolResult {
        Ok(false)
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
