use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;
use tera::{Context, Tera};

use crate::vm::{RuntimeBoolResult, RuntimeErr, RuntimeObjResult};

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Str Type ------------------------------------------------------------

gen::type_and_impls!(StrType, Str);

pub static STR_TYPE: Lazy<new::obj_ref_t!(StrType)> = Lazy::new(|| {
    let type_ref = new::obj_ref!(StrType::new());
    let mut class = type_ref.write().unwrap();

    class.ns_mut().add_entries(&[
        // Class Methods -----------------------------------------------
        gen::meth!("new", type_ref, &["value"], |_, args, _| {
            let arg = gen::use_arg!(args, 0);
            Ok(if arg.is_str() { args[0].clone() } else { new::str(arg.to_string()) })
        }),
        // Instance Attributes -----------------------------------------
        gen::prop!("length", type_ref, |this, _, _| {
            let this = this.read().unwrap();
            let value = this.get_str_val().unwrap();
            Ok(new::int(value.len()))
        }),
        // Instance Methods --------------------------------------------
        gen::meth!("starts_with", type_ref, &["prefix"], |this, args, _| {
            let this = this.read().unwrap();
            let value = this.get_str_val().unwrap();
            let arg = gen::use_arg!(args, 0);
            let prefix = gen::use_arg_str!(arg);
            Ok(new::bool(value.starts_with(prefix)))
        }),
        gen::meth!("ends_with", type_ref, &["suffix"], |this, args, _| {
            let this = this.read().unwrap();
            let value = this.get_str_val().unwrap();
            let arg = gen::use_arg!(args, 0);
            let suffix = gen::use_arg_str!(arg);
            Ok(new::bool(value.ends_with(suffix)))
        }),
        gen::meth!("upper", type_ref, &[], |this, _, _| {
            let this = this.read().unwrap();
            let value = this.get_str_val().unwrap();
            Ok(new::str(value.to_uppercase()))
        }),
        gen::meth!("lower", type_ref, &[], |this, _, _| {
            let this = this.read().unwrap();
            let value = this.get_str_val().unwrap();
            Ok(new::str(value.to_lowercase()))
        }),
        gen::meth!("render", type_ref, &["context"], |this_obj, args, _| {
            let this = this_obj.read().unwrap();
            let value = this.get_str_val().unwrap();
            let arg = gen::use_arg!(args, 0);
            let context = gen::use_arg_map!(arg);
            let context = context.to_hash_map();
            let mut tera = Tera::default();
            let mut tera_context = Context::new();
            context.iter().for_each(|(k, v)| {
                let v = v.read().unwrap();
                let v = v.to_string();
                tera_context.insert(k, v.as_str());
            });
            tera.add_raw_template("str.render.template", value).unwrap();
            let result = tera.render("str.render.template", &tera_context);
            Ok(match result {
                Ok(output) => new::str(output),
                Err(err) => new::string_err(err.to_string(), this_obj.clone()),
            })
        }),
        gen::meth!("repeat", type_ref, &["count"], |this, args, _| {
            let this = this.read().unwrap();
            let value = this.get_str_val().unwrap();
            let arg1 = gen::use_arg!(args, 0);
            let count = gen::use_arg_usize!(arg1);
            Ok(new::str(value.repeat(count)))
        }),
        gen::meth!("replace", type_ref, &["old", "new"], |this, args, _| {
            let this = this.read().unwrap();
            let value = this.get_str_val().unwrap();
            let arg1 = gen::use_arg!(args, 0);
            let arg2 = gen::use_arg!(args, 1);
            let old = gen::use_arg_str!(arg1);
            let new = gen::use_arg_str!(arg2);
            let result = value.replace(old, new);
            Ok(new::str(result))
        }),
    ]);

    type_ref.clone()
});

// Str Object ----------------------------------------------------------

pub struct Str {
    ns: Namespace,
    value: String,
}

gen::standard_object_impls!(Str);

impl Str {
    pub fn new(value: String) -> Self {
        Self {
            ns: Namespace::with_entries(&[
                // Instance Attributes
                ("len", new::int(value.len())),
            ]),
            value,
        }
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

impl ObjectTrait for Str {
    gen::object_trait_header!(STR_TYPE);

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if self.is(rhs) || rhs.is_always() {
            true
        } else if let Some(rhs) = rhs.down_to_str() {
            self.is(rhs) || self.value() == rhs.value()
        } else {
            false
        }
    }

    fn add(&self, rhs: &dyn ObjectTrait) -> RuntimeObjResult {
        if let Some(rhs) = rhs.down_to_str() {
            let a = self.value();
            let b = rhs.value();
            let mut value = String::with_capacity(a.len() + b.len());
            value.push_str(a);
            value.push_str(b);
            let value = new::str(value);
            Ok(value)
        } else {
            Err(RuntimeErr::type_err(format!(
                "Cannot concatenate {} to {}",
                self.class().read().unwrap(),
                rhs.class().read().unwrap(),
            )))
        }
    }

    fn less_than(&self, rhs: &dyn ObjectTrait) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_str() {
            Ok(self.value() < rhs.value())
        } else {
            Err(RuntimeErr::type_err(format!(
                "Cannot compare {} to {}: <",
                self.class().read().unwrap(),
                rhs.class().read().unwrap(),
            )))
        }
    }

    fn greater_than(&self, rhs: &dyn ObjectTrait) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_str() {
            Ok(self.value() > rhs.value())
        } else {
            Err(RuntimeErr::type_err(format!(
                "Cannot compare {} to {}: >",
                self.class().read().unwrap(),
                rhs.class().read().unwrap(),
            )))
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Debug for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}
