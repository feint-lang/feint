use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::{RuntimeBoolResult, RuntimeErr, RuntimeObjResult};

use super::create;

use super::base::{ObjectRef, ObjectTrait, ObjectTraitExt, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Str Type ------------------------------------------------------------

pub static STR_TYPE: Lazy<Arc<RwLock<StrType>>> =
    Lazy::new(|| Arc::new(RwLock::new(StrType::new())));

pub struct StrType {
    namespace: Namespace,
}

impl StrType {
    pub fn new() -> Self {
        let mut ns = Namespace::new();

        ns.add_obj("$name", create::new_str("Str"));
        ns.add_obj("$full_name", create::new_str("builtins.Str"));

        ns.add_obj(
            "starts_with",
            create::new_builtin_func(
                "starts_with",
                Some(vec!["prefix"]),
                |this, args, _| {
                    let this = this.expect("Expected this");
                    let this = this.read().unwrap();
                    let this = this.down_to_str().unwrap();
                    let arg = args.get(0).unwrap();
                    let arg = arg.read().unwrap();
                    let arg = arg.down_to_str().unwrap();
                    Ok(create::new_bool(this.value.starts_with(&arg.value)))
                },
            ),
        );

        Self { namespace: ns }
    }
}

unsafe impl Send for StrType {}
unsafe impl Sync for StrType {}

impl TypeTrait for StrType {
    fn name(&self) -> &str {
        "Str"
    }

    fn full_name(&self) -> &str {
        "builtins.Str"
    }
}

impl ObjectTrait for StrType {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        TYPE_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        TYPE_TYPE.clone()
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

// Str Object ----------------------------------------------------------

pub struct Str {
    namespace: Namespace,
    value: String,
}

impl Str {
    pub fn new<S: Into<String>>(value: S) -> Self {
        let mut ns = Namespace::new();
        let value = value.into();
        ns.add_obj("length", create::new_int(value.len()));
        Self { namespace: ns, value }
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

impl ObjectTrait for Str {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        STR_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        STR_TYPE.clone()
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if let Some(rhs) = rhs.down_to_str() {
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
            let value = create::new_str(value);
            Ok(value)
        } else {
            Err(RuntimeErr::new_type_err(format!(
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
            Err(RuntimeErr::new_type_err(format!(
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
            Err(RuntimeErr::new_type_err(format!(
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
