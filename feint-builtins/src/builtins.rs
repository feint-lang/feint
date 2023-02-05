use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::{FromPrimitive, Num};
use once_cell::sync::{Lazy, OnceCell};

use feint_code_gen::{meth, obj_ref, obj_ref_t, use_arg};
use feint_util::string::format_doc;

use crate::types::{self, ObjectRef, ObjectTrait, Params, TypeRef};

use crate::types::always::Always;
use crate::types::bool::Bool;
use crate::types::bound_func::BoundFunc;
use crate::types::cell::Cell;
use crate::types::class::Type;
use crate::types::closure::Closure;
use crate::types::code::Code;
use crate::types::custom::CustomObj;
use crate::types::err::ErrObj;
use crate::types::err_type::ErrKind;
use crate::types::file::File;
use crate::types::float::Float;
use crate::types::func::Func;
use crate::types::int::Int;
use crate::types::intrinsic_func::{IntrinsicFn, IntrinsicFunc};
use crate::types::iterator::FIIterator;
use crate::types::list::List;
use crate::types::map::Map;
use crate::types::module::Module;
use crate::types::nil::Nil;
use crate::types::ns::Namespace;
use crate::types::prop::Prop;
use crate::types::str::Str;
use crate::types::tuple::Tuple;

pub static BUILTINS: Lazy<Builtins> = Lazy::new(Builtins::default);

pub struct Builtins {
    // Modules ---------------------------------------------------------
    pub modules: obj_ref_t!(Map),

    // Types -----------------------------------------------------------
    pub type_type: TypeRef,
    pub always_type: TypeRef,
    pub bool_type: TypeRef,
    pub bound_func_type: TypeRef,
    pub cell_type: TypeRef,
    pub closure_type: TypeRef,
    pub err_type: TypeRef,
    pub err_type_type: TypeRef,
    pub file_type: TypeRef,
    pub float_type: TypeRef,
    pub func_type: TypeRef,
    pub int_type: TypeRef,
    pub intrinsic_func_type: TypeRef,
    pub iterator_type: TypeRef,
    pub list_type: TypeRef,
    pub map_type: TypeRef,
    pub module_type: TypeRef,
    pub nil_type: TypeRef,
    pub prop_type: TypeRef,
    pub str_type: TypeRef,
    pub tuple_type: TypeRef,

    // Singletons ------------------------------------------------------
    pub nil: ObjectRef,
    pub true_: ObjectRef,
    pub false_: ObjectRef,
    pub always: ObjectRef,
    pub empty_str: ObjectRef,
    pub newline: ObjectRef,
    pub empty_tuple: ObjectRef,
    pub ok_err: ObjectRef,
}

unsafe impl Send for Builtins {}
unsafe impl Sync for Builtins {}

impl Default for Builtins {
    fn default() -> Self {
        fn new_type(name: &str) -> TypeRef {
            obj_ref!(Type::new("std", name))
        }

        let type_type = new_type("Type");
        let always_type = new_type("Always");
        let bool_type = new_type("Bool");
        let bound_func_type = new_type("BoundFunc");
        let cell_type = new_type("Cell");
        let closure_type = new_type("Closure");
        let err_type = new_type("Err");
        let err_type_type = new_type("ErrType");
        let file_type = new_type("File");
        let float_type = new_type("Float");
        let func_type = new_type("Func");
        let int_type = new_type("Int");
        let intrinsic_func_type = new_type("IntrinsicFunc");
        let iterator_type = new_type("Iterator");
        let list_type = new_type("List");
        let map_type = new_type("Map");
        let module_type = new_type("Module");
        let nil_type = new_type("Nil");
        let prop_type = new_type("Prop");
        let str_type = new_type("Str");
        let tuple_type = new_type("Tuple");

        let modules = obj_ref!(Map::new(map_type.clone(), IndexMap::default()));

        let nil = obj_ref!(Nil::new(nil_type.clone()));
        let true_ = obj_ref!(Bool::new(bool_type.clone(), true));
        let false_ = obj_ref!(Bool::new(bool_type.clone(), false));
        let always = obj_ref!(Always::new(always_type.clone()));
        let empty_str = obj_ref!(Str::new(str_type.clone(), "".to_owned()));
        let newline = obj_ref!(Str::new(str_type.clone(), "\n".to_owned()));
        let empty_tuple = obj_ref!(Tuple::new(tuple_type.clone(), vec![]));
        let ok_err = obj_ref!(ErrObj::new(
            err_type.clone(),
            ErrKind::Ok,
            "".to_string(),
            nil.clone()
        ));

        let instance = Self {
            // Modules ---------------------------------------------------------
            modules,
            // Types -----------------------------------------------------------
            type_type,
            always_type,
            bool_type,
            bound_func_type,
            cell_type,
            closure_type,
            err_type,
            err_type_type,
            file_type,
            float_type,
            func_type,
            int_type,
            intrinsic_func_type,
            iterator_type,
            list_type,
            map_type,
            module_type,
            nil_type,
            prop_type,
            str_type,
            tuple_type,
            // Singletons ------------------------------------------------------
            nil,
            true_,
            false_,
            always,
            empty_str,
            newline,
            empty_tuple,
            ok_err,
        };

        instance
    }
}

impl Builtins {
    // Modules ---------------------------------------------------------

    pub fn modules(&self) -> obj_ref_t!(Map) {
        self.modules.clone()
    }

    pub fn module(
        &self,
        name: &str,
        path: &str,
        doc: &str,
        entries: &[(&str, ObjectRef)],
    ) -> obj_ref_t!(types::module::Module) {
        obj_ref!(Module::with_entries(
            self.module_type(),
            entries,
            name.to_owned(),
            path.to_owned(),
            Code::default(),
            Some(doc.to_owned())
        ))
    }

    /// Add module to `std.system.modules`.
    pub fn add_module(&self, name: &str, module: ObjectRef) {
        let modules = self.modules();
        let modules = modules.write().unwrap();
        let modules = modules.down_to_map().unwrap();
        modules.insert(name, module);
    }

    /// Get module from `system.modules`.
    ///
    /// XXX: Panics if the module doesn't exist (since that shouldn't be
    ///      possible).
    pub fn get_module(&self, name: &str) -> ObjectRef {
        let modules = self.modules();
        let modules = modules.read().unwrap();
        let modules = modules.down_to_map().unwrap();
        if let Some(module) = modules.get(name) {
            module.clone()
        } else {
            panic!("Module not registered: {name}");
        }
    }

    /// Get module from `system.modules`.
    ///
    /// XXX: This will return `None` if the module doesn't exist. Generally,
    ///      this should only be used during bootstrap. In most cases,
    ///      `get_module` should be used instead.
    pub fn maybe_get_module(&self, name: &str) -> Option<ObjectRef> {
        let modules = self.modules();
        let modules = modules.read().unwrap();
        let modules = modules.down_to_map().unwrap();
        modules.get(name)
    }

    // Types -----------------------------------------------------------

    pub fn type_type(&self) -> TypeRef {
        self.type_type.clone()
    }

    pub fn always_type(&self) -> TypeRef {
        self.always_type.clone()
    }

    pub fn bool_type(&self) -> TypeRef {
        self.bool_type.clone()
    }

    pub fn bound_func_type(&self) -> TypeRef {
        self.bound_func_type.clone()
    }

    pub fn cell_type(&self) -> TypeRef {
        self.cell_type.clone()
    }

    pub fn closure_type(&self) -> TypeRef {
        self.closure_type.clone()
    }

    pub fn err_type(&self) -> TypeRef {
        self.err_type.clone()
        // self.err_type.get_or_init(make_err_type).clone()
    }

    pub fn err_type_type(&self) -> TypeRef {
        self.err_type_type.clone()
    }

    pub fn file_type(&self) -> TypeRef {
        self.file_type.clone()
        // self.file_type.get_or_init(make_file_type).clone()
    }

    pub fn float_type(&self) -> TypeRef {
        self.float_type.clone()
        // self.float_type.get_or_init(make_float_type).clone()
    }

    pub fn func_type(&self) -> TypeRef {
        self.func_type.clone()
    }

    pub fn intrinsic_func_type(&self) -> TypeRef {
        self.intrinsic_func_type.clone()
    }

    pub fn int_type(&self) -> TypeRef {
        self.int_type.clone()
        // self.int_type.get_or_init(make_int_type).clone()
    }

    pub fn iterator_type(&self) -> TypeRef {
        self.iterator_type.clone()
        // self.iterator_type.get_or_init(make_iterator_type).clone()
    }

    pub fn list_type(&self) -> TypeRef {
        self.list_type.clone()
        // self.list_type.get_or_init(make_list_type).clone()
    }

    pub fn map_type(&self) -> TypeRef {
        self.map_type.clone()
        // self.map_type.get_or_init(make_map_type).clone()
    }

    pub fn module_type(&self) -> TypeRef {
        self.module_type.clone()
        // self.module_type.get_or_init(make_module_type).clone()
    }

    pub fn nil_type(&self) -> TypeRef {
        self.nil_type.clone()
    }

    pub fn prop_type(&self) -> TypeRef {
        self.prop_type.clone()
    }

    pub fn str_type(&self) -> TypeRef {
        self.str_type.clone()
        // self.str_type.get_or_init(make_str_type).clone()
    }

    pub fn tuple_type(&self) -> TypeRef {
        self.tuple_type.clone()
        // self.tuple_type.get_or_init(make_tuple_type).clone()
    }

    // Singletons ------------------------------------------------------

    pub fn nil(&self) -> ObjectRef {
        self.nil.clone()
    }

    pub fn bool(&self, val: bool) -> ObjectRef {
        if val {
            self.true_()
        } else {
            self.false_()
        }
    }

    pub fn true_(&self) -> ObjectRef {
        self.true_.clone()
    }

    pub fn false_(&self) -> ObjectRef {
        self.false_.clone()
    }

    pub fn always(&self) -> ObjectRef {
        self.always.clone()
    }

    pub fn empty_str(&self) -> ObjectRef {
        self.empty_str.clone()
    }

    pub fn newline(&self) -> ObjectRef {
        self.newline.clone()
    }

    pub fn empty_tuple(&self) -> ObjectRef {
        self.empty_tuple.clone()
    }

    pub fn ok_err(&self) -> ObjectRef {
        self.ok_err.clone()
    }

    // Functions -----------------------------------------------------------

    pub fn intrinsic_func(
        &self,
        module_name: &str,
        name: &str,
        this_type: Option<ObjectRef>,
        params: &[&str],
        doc: &str,
        func: IntrinsicFn,
    ) -> ObjectRef {
        let params = params.iter().map(|n| n.to_string()).collect();
        obj_ref!(IntrinsicFunc::new(
            self.intrinsic_func_type(),
            module_name.to_owned(),
            name.to_owned(),
            this_type,
            params,
            format_doc(doc),
            func
        ))
    }

    pub fn func<S: Into<String>>(
        &self,
        module_name: S,
        func_name: S,
        params: Params,
        code: Code,
    ) -> ObjectRef {
        obj_ref!(Func::new(
            self.func_type(),
            module_name.into(),
            func_name.into(),
            params,
            code
        ))
    }

    pub fn bound_func(&self, func: ObjectRef, this: ObjectRef) -> ObjectRef {
        obj_ref!(BoundFunc::new(self.bound_func_type(), func, this))
    }

    pub fn closure(&self, func: ObjectRef, captured: ObjectRef) -> ObjectRef {
        obj_ref!(Closure::new(self.closure_type(), func, captured))
    }

    pub fn cell(&self) -> ObjectRef {
        obj_ref!(Cell::new(self.cell_type()))
    }

    pub fn cell_with_value(&self, value: ObjectRef) -> ObjectRef {
        obj_ref!(Cell::with_value(self.cell_type(), value))
    }

    pub fn argv_tuple(&self, argv: &[String]) -> ObjectRef {
        obj_ref!(Tuple::new(
            self.tuple_type(),
            argv.iter().map(|a| self.str(a)).collect()
        ))
    }

    // Numbers ---------------------------------------------------------

    pub fn float(&self, value: f64) -> ObjectRef {
        obj_ref!(Float::new(self.float_type(), value))
    }

    pub fn float_from_string<S: Into<String>>(&self, val: S) -> ObjectRef {
        let val = val.into();
        if let Ok(val) = val.parse::<f64>() {
            self.float(val)
        } else {
            self.type_err("Could not convert string to Float", self.str(val))
        }
    }

    pub fn int<I: Into<BigInt>>(&self, val: I) -> ObjectRef {
        let val = val.into();
        obj_ref!(Int::new(self.int_type(), val))
    }

    pub fn int_from_string<S: Into<String>>(&self, val: S) -> ObjectRef {
        let val = val.into();
        if let Ok(val) = BigInt::from_str_radix(val.as_ref(), 10) {
            self.int(val)
        } else if let Ok(val) = val.parse::<f64>() {
            self.int(BigInt::from_f64(val).unwrap())
        } else {
            self.type_err("Could not convert string to Int", self.str(val))
        }
    }

    // Strings ---------------------------------------------------------

    pub fn str<S: Into<String>>(&self, val: S) -> ObjectRef {
        let val = val.into();
        if val.is_empty() {
            self.empty_str()
        } else if val == "\n" {
            self.newline()
        } else {
            obj_ref!(Str::new(self.str_type(), val))
        }
    }

    // Collections -----------------------------------------------------

    pub fn iterator(&self, wrapped: Vec<ObjectRef>) -> ObjectRef {
        obj_ref!(FIIterator::new(self.iterator_type(), wrapped))
    }

    pub fn list(&self, items: Vec<ObjectRef>) -> ObjectRef {
        obj_ref!(List::new(self.list_type(), items.to_vec()))
    }

    pub fn map(&self, map: IndexMap<String, ObjectRef>) -> ObjectRef {
        obj_ref!(Map::new(self.map_type(), map))
    }

    pub fn empty_map(&self) -> ObjectRef {
        obj_ref!(Map::new(self.map_type(), IndexMap::default()))
    }

    pub fn map_from_keys_and_vals(
        &self,
        keys: Vec<String>,
        vals: Vec<ObjectRef>,
    ) -> ObjectRef {
        assert_eq!(keys.len(), vals.len());
        self.map(IndexMap::from_iter(keys.into_iter().zip(vals)))
    }

    pub fn tuple(&self, items: Vec<ObjectRef>) -> ObjectRef {
        if items.is_empty() {
            self.empty_tuple()
        } else {
            obj_ref!(Tuple::new(self.tuple_type(), items))
        }
    }

    // Miscellaneous ---------------------------------------------------

    pub fn file<S: Into<String>>(&self, file_name: S) -> ObjectRef {
        obj_ref!(File::new(self.file_type(), file_name.into()))
    }

    pub fn prop(&self, getter: ObjectRef) -> ObjectRef {
        obj_ref!(Prop::new(self.prop_type(), getter))
    }

    // Errors ----------------------------------------------------------

    pub fn err<S: Into<String>>(
        &self,
        kind: ErrKind,
        msg: S,
        obj: ObjectRef,
    ) -> ObjectRef {
        obj_ref!(ErrObj::new(self.err_type(), kind, msg.into(), obj))
    }

    pub fn err_with_responds_to_bool<S: Into<String>>(
        &self,
        kind: ErrKind,
        msg: S,
        obj: ObjectRef,
    ) -> ObjectRef {
        obj_ref!(ErrObj::with_responds_to_bool(self.err_type(), kind, msg.into(), obj))
    }

    pub fn arg_err<S: Into<String>>(&self, msg: S, obj: ObjectRef) -> ObjectRef {
        self.err(ErrKind::Arg, msg, obj)
    }

    pub fn attr_err<S: Into<String>>(&self, msg: S, obj: ObjectRef) -> ObjectRef {
        self.err(ErrKind::Attr, msg, obj)
    }

    pub fn attr_not_found_err<S: Into<String>>(
        &self,
        msg: S,
        obj: ObjectRef,
    ) -> ObjectRef {
        self.err(ErrKind::AttrNotFound, msg, obj)
    }

    pub fn file_not_found_err<S: Into<String>>(
        &self,
        msg: S,
        obj: ObjectRef,
    ) -> ObjectRef {
        self.err(ErrKind::FileNotFound, msg, obj)
    }

    pub fn file_unreadable_err<S: Into<String>>(
        &self,
        msg: S,
        obj: ObjectRef,
    ) -> ObjectRef {
        self.err(ErrKind::FileUnreadable, msg, obj)
    }

    pub fn index_out_of_bounds_err(&self, index: usize, obj: ObjectRef) -> ObjectRef {
        self.err(ErrKind::IndexOutOfBounds, index.to_string(), obj)
    }

    pub fn not_callable_err(&self, obj: ObjectRef) -> ObjectRef {
        self.err(ErrKind::NotCallable, format!("{}", obj.read().unwrap()), obj)
    }

    pub fn string_err<S: Into<String>>(&self, msg: S, obj: ObjectRef) -> ObjectRef {
        self.err(ErrKind::String, msg, obj)
    }

    pub fn type_err<S: Into<String>>(&self, msg: S, obj: ObjectRef) -> ObjectRef {
        self.err(ErrKind::Type, msg, obj)
    }

    // Custom type constructor ---------------------------------------------

    // pub fn custom_type(&self, module: ObjectRef, name: &str) -> ObjectRef {
    //     let class_ref = obj_ref!(CustomType::new(module.clone(), name.to_owned()));
    //
    //     {
    //         let mut class = class_ref.write().unwrap();
    //         let ns = class.ns_mut();
    //         ns.insert(
    //             "new",
    //             self.intrinsic_func(
    //                 module.read().unwrap().down_to_mod().unwrap().name(),
    //                 name,
    //                 Some(class_ref.clone()),
    //                 &["attrs"],
    //                 "Create a new custom type.
    //
    //             # Args
    //
    //             - class: TypeRef
    //             - type_obj: ObjectRef
    //             - attributes: Map
    //
    //             ",
    //                 |this, args| {
    //                     let attrs_arg = args.get(0).unwrap();
    //                     let attrs_arg = attrs_arg.read().unwrap();
    //                     let attrs = attrs_arg.down_to_map().unwrap();
    //
    //                     let mut ns = Namespace::default();
    //                     ns.extend_from_map(attrs);
    //
    //                     // XXX: Cloning the inner object is wonky and breaks
    //                     //      identity testing.
    //                     let type_obj = this.read().unwrap();
    //                     let type_obj = if type_obj.is_type_object() {
    //                         // Called via custom type.
    //                         let type_obj = type_obj.down_to_custom_type().unwrap();
    //                         obj_ref!(type_obj.clone())
    //                     } else {
    //                         // Called via custom instance.
    //                         // XXX: This branch isn't reachable because the
    //                         //      VM will panic due to the identity test
    //                         //      issue noted above.
    //                         let type_obj = type_obj.type_obj();
    //                         let type_obj = type_obj.read().unwrap();
    //                         let type_obj = type_obj.down_to_custom_type().unwrap();
    //                         obj_ref!(type_obj.clone())
    //                     };
    //
    //                     let instance = CustomObj::new(type_obj, ns);
    //                     obj_ref!(instance)
    //                 },
    //             ),
    //         );
    //     }
    //
    //     class_ref
    // }
}
