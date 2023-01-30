use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use feint_code_gen::*;

use crate::util::check_args;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::code::Code;
use super::map::Map;
use super::new;
use super::ns::Namespace;

// Module Type ---------------------------------------------------------

type_and_impls!(ModuleType, Module);

pub static MODULE_TYPE: Lazy<obj_ref_t!(ModuleType)> = Lazy::new(|| {
    let type_ref = obj_ref!(ModuleType::new());
    let mut type_obj = type_ref.write().unwrap();

    type_obj.add_attrs(&[meth!(
        "new",
        type_ref,
        &["name", "path", "doc", "attrs"],
        "Create a new Module
        
        # Args

        - name: Str
        - path: Str
        - doc: Str
        - attrs: Map

        # Returns

        Module",
        |_, args| {
            if let Err(err) = check_args("new", &args, false, 4, Some(4)) {
                return err;
            };

            let name_arg = use_arg!(args, 0);
            let path_arg = use_arg!(args, 1);
            let doc_arg = use_arg!(args, 2);
            let attrs_arg = use_arg!(args, 3);

            let name = use_arg_str!(new, name, name_arg);
            let path = use_arg_str!(new, path, path_arg);
            let doc = use_arg_str!(new, doc, doc_arg);
            let attrs = use_arg_map!(new, attrs, attrs_arg);

            let module = Module::with_map_entries(
                attrs,
                name.to_owned(),
                path.to_owned(),
                Code::default(),
                Some(doc.to_owned()),
            );

            obj_ref!(module)
        }
    )]);

    type_ref.clone()
});

// Module Object -------------------------------------------------------

pub struct Module {
    ns: Namespace,
    name: String,
    path: String,
    code: Code,
}

standard_object_impls!(Module);

impl Module {
    /// NOTE: The `$doc` attribute should only be passed for intrinsic
    ///       modules and for special cases such as the REPL module.
    ///       Modules implemented in FeInt will have their `$doc`
    ///       attribute initialized from their module level docstring.
    pub fn new(name: String, path: String, code: Code, doc: Option<String>) -> Self {
        let ns = Namespace::with_entries(&[
            ("$full_name", new::str(name.as_str())),
            ("$name", new::str(name.as_str())),
            ("$path", new::str(path.as_str())),
            ("$doc", if let Some(doc) = doc { new::str(doc) } else { code.get_doc() }),
        ]);
        Self { ns, path, name, code }
    }

    pub fn with_entries(
        entries: &[(&str, ObjectRef)],
        name: String,
        path: String,
        code: Code,
        doc: Option<String>,
    ) -> Self {
        let mut module = Self::new(name, path, code, doc);
        module.ns.extend(entries);
        module
    }

    pub fn with_map_entries(
        map: &Map,
        name: String,
        path: String,
        code: Code,
        doc: Option<String>,
    ) -> Self {
        let mut module = Self::new(name, path, code, doc);
        module.ns.extend_from_map(map);
        module
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    pub fn add_global(&mut self, name: &str, val: ObjectRef) {
        self.ns.insert(name, val.clone());
    }

    pub fn get_global(&self, name: &str) -> Option<ObjectRef> {
        self.ns.get(name)
    }

    pub fn has_global(&self, name: &str) -> bool {
        self.ns.contains_key(name)
    }

    pub fn iter_globals(&self) -> indexmap::map::Iter<'_, String, ObjectRef> {
        self.ns.iter()
    }

    pub fn get_main(&self) -> Option<ObjectRef> {
        // XXX: If an intrinsic module defines $main(), it will be a
        //      global. If a .fi module defines $main(), it may or may
        //      not have been copied to the module's globals.
        self.get_global("$main").or_else(|| self.code.get_main())
    }

    pub fn code(&self) -> &Code {
        &self.code
    }

    pub fn code_mut(&mut self) -> &mut Code {
        &mut self.code
    }

    pub fn set_code(&mut self, code: Code) {
        self.code = code;
    }
}

impl ObjectTrait for Module {
    object_trait_header!(MODULE_TYPE);
}

// Display -------------------------------------------------------------

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<module {} from {}>", self.name(), self.path())
    }
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<module {} from {} @ {}>", self.name(), self.path(), self.id())
    }
}
