use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use feint_code_gen::*;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::new;
use super::ns::Namespace;

// Str Type ------------------------------------------------------------

type_and_impls!(StrType, Str);

pub static STR_TYPE: Lazy<obj_ref_t!(StrType)> = Lazy::new(|| {
    let type_ref = obj_ref!(StrType::new());
    let mut type_obj = type_ref.write().unwrap();

    type_obj.add_attrs(&[
        // Class Methods -----------------------------------------------
        meth!("new", type_ref, &["value"], "", |_, args| {
            let arg = use_arg!(args, 0);
            if arg.is_str() {
                args[0].clone()
            } else {
                new::str(arg.to_string())
            }
        }),
        // Instance Attributes -----------------------------------------
        prop!("length", type_ref, "", |this, _| {
            let this = this.read().unwrap();
            let value = this.get_str_val().unwrap();
            new::int(value.len())
        }),
        // Instance Methods --------------------------------------------
        meth!("starts_with", type_ref, &["prefix"], "", |this, args| {
            let this = this.read().unwrap();
            let value = this.get_str_val().unwrap();
            let arg = use_arg!(args, 0);
            let prefix = use_arg_str!(starts_with, prefix, arg);
            new::bool(value.starts_with(prefix))
        }),
        meth!("ends_with", type_ref, &["suffix"], "", |this, args| {
            let this = this.read().unwrap();
            let value = this.get_str_val().unwrap();
            let arg = use_arg!(args, 0);
            let suffix = use_arg_str!(ends_with, suffix, arg);
            new::bool(value.ends_with(suffix))
        }),
        meth!("upper", type_ref, &[], "", |this, _| {
            let this = this.read().unwrap();
            let value = this.get_str_val().unwrap();
            new::str(value.to_uppercase())
        }),
        meth!("lower", type_ref, &[], "", |this, _| {
            let this = this.read().unwrap();
            let value = this.get_str_val().unwrap();
            new::str(value.to_lowercase())
        }),
        // meth!(
        //     "render",
        //     type_ref,
        //     &["context"],
        //     "Render string as template
        //
        //     Templates may contain `{{ name }}` vars which will be replaced with the
        //     values provided in the context map.
        //
        //     # Args
        //
        //     - context: Map<Str, Str> A map containing values to be rendered into the
        //       template.
        //
        //     ",
        //     |this, args| {
        //         let context = args[0].clone();
        //         let result = render_template(this.clone(), context)?;
        //         Ok(result)
        //     }
        // ),
        meth!("repeat", type_ref, &["count"], "", |this, args| {
            let this = this.read().unwrap();
            let value = this.get_str_val().unwrap();
            let count = use_arg_usize!(get, index, args, 0);
            new::str(value.repeat(count))
        }),
        meth!("replace", type_ref, &["old", "new"], "", |this, args| {
            let this = this.read().unwrap();
            let value = this.get_str_val().unwrap();
            let arg1 = use_arg!(args, 0);
            let arg2 = use_arg!(args, 1);
            let old = use_arg_str!(replace, old, arg1);
            let new = use_arg_str!(replace, new, arg2);
            let result = value.replace(old, new);
            new::str(result)
        }),
        meth!("remove_prefix", type_ref, &["prefix"], "", |this_ref, args| {
            let this = this_ref.read().unwrap();
            let val = this.get_str_val().unwrap();
            let arg = use_arg!(args, 0);
            let prefix = use_arg_str!(starts_with, prefix, arg);
            if let Some(new_val) = val.strip_prefix(prefix) {
                new::str(new_val)
            } else {
                drop(this);
                this_ref
            }
        }),
    ]);

    type_ref.clone()
});

// Str Object ----------------------------------------------------------

pub struct Str {
    ns: Namespace,
    value: String,
}

standard_object_impls!(Str);

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
    object_trait_header!(STR_TYPE);

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if self.is(rhs) || rhs.is_always() {
            true
        } else if let Some(rhs) = rhs.down_to_str() {
            self.is(rhs) || self.value() == rhs.value()
        } else {
            false
        }
    }

    fn add(&self, rhs: &dyn ObjectTrait) -> Option<ObjectRef> {
        if let Some(rhs) = rhs.down_to_str() {
            let a = self.value();
            let b = rhs.value();
            let mut value = String::with_capacity(a.len() + b.len());
            value.push_str(a);
            value.push_str(b);
            let value = new::str(value);
            Some(value)
        } else {
            None
        }
    }

    fn less_than(&self, rhs: &dyn ObjectTrait) -> Option<bool> {
        if let Some(rhs) = rhs.down_to_str() {
            Some(self.value() < rhs.value())
        } else {
            None
        }
    }

    fn greater_than(&self, rhs: &dyn ObjectTrait) -> Option<bool> {
        if let Some(rhs) = rhs.down_to_str() {
            Some(self.value() > rhs.value())
        } else {
            None
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
