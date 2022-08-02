use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::sync::Arc;

use num_traits::ToPrimitive;
use once_cell::sync::Lazy;

use crate::builtin_funcs::float;
use crate::vm::{RuntimeBoolResult, RuntimeErr, RuntimeObjResult};

use super::create;
use super::util::{eq_int_float, gt_int_float, lt_int_float};

use super::base::{ObjectRef, ObjectTrait, ObjectTraitExt, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Float Type ----------------------------------------------------------

pub static FLOAT_TYPE: Lazy<Arc<FloatType>> = Lazy::new(|| Arc::new(FloatType::new()));

pub struct FloatType {
    namespace: RefCell<Namespace>,
}

unsafe impl Send for FloatType {}
unsafe impl Sync for FloatType {}

impl FloatType {
    pub fn new() -> Self {
        let mut ns = Namespace::new();
        ns.add_obj("$name", create::new_str("Float"));
        ns.add_obj("$full_name", create::new_str("builtins.Float"));
        ns.add_obj(
            "new",
            create::new_builtin_func("map", Some(vec!["value"]), float::new),
        );
        Self { namespace: RefCell::new(ns) }
    }
}

impl TypeTrait for FloatType {
    fn name(&self) -> &str {
        "Float"
    }

    fn full_name(&self) -> &str {
        "builtins.Float"
    }
}

impl ObjectTrait for FloatType {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        TYPE_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        TYPE_TYPE.clone()
    }

    fn namespace(&self) -> &RefCell<Namespace> {
        &self.namespace
    }
}

// Float Object --------------------------------------------------------

macro_rules! make_op {
    ( $meth:ident, $op:tt, $message:literal, $trunc:literal ) => {
        fn $meth(&self, rhs: &dyn ObjectTrait) -> RuntimeObjResult {
            let value = if let Some(rhs) = rhs.down_to_float() {
                *rhs.value()
            } else if let Some(rhs) = rhs.down_to_int() {
                rhs.value().to_f64().unwrap()
            } else {
                return Err(RuntimeErr::new_type_err(format!($message, rhs.class())));
            };
            let mut value = &self.value $op value;
            if $trunc {
                value = value.trunc();
            }
            let value = create::new_float(value);
            Ok(value)
        }
    };
}

pub struct Float {
    namespace: RefCell<Namespace>,
    value: f64,
}

unsafe impl Send for Float {}
unsafe impl Sync for Float {}

impl Float {
    pub fn new(value: f64) -> Self {
        Self { namespace: RefCell::new(Namespace::new()), value }
    }

    pub fn value(&self) -> &f64 {
        &self.value
    }
}

impl ObjectTrait for Float {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        FLOAT_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        FLOAT_TYPE.clone()
    }

    fn namespace(&self) -> &RefCell<Namespace> {
        &self.namespace
    }

    fn negate(&self) -> RuntimeObjResult {
        Ok(create::new_float(-*self.value()))
    }

    fn bool_val(&self) -> RuntimeBoolResult {
        Ok(*self.value() != 0.0)
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if let Some(rhs) = rhs.down_to_float() {
            self.is(rhs) || self.value() == rhs.value()
        } else if let Some(rhs) = rhs.down_to_int() {
            eq_int_float(rhs, self)
        } else {
            false
        }
    }

    fn less_than(&self, rhs: &dyn ObjectTrait) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_float() {
            Ok(self.value() < rhs.value())
        } else if let Some(rhs) = rhs.down_to_int() {
            Ok(lt_int_float(rhs, self))
        } else {
            Err(RuntimeErr::new_type_err(format!(
                "Could not compare {} to {}: <",
                self.class(),
                rhs.class()
            )))
        }
    }

    fn greater_than(&self, rhs: &dyn ObjectTrait) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_float() {
            Ok(self.value() > rhs.value())
        } else if let Some(rhs) = rhs.down_to_int() {
            Ok(gt_int_float(rhs, self))
        } else {
            Err(RuntimeErr::new_type_err(format!(
                "Could not compare {} to {}: >",
                self.class(),
                rhs.class()
            )))
        }
    }

    fn pow(&self, rhs: &dyn ObjectTrait) -> RuntimeObjResult {
        let exp = if let Some(rhs) = rhs.down_to_float() {
            *rhs.value()
        } else if let Some(rhs) = rhs.down_to_int() {
            rhs.value().to_f64().unwrap()
        } else {
            return Err(RuntimeErr::new_type_err(format!(
                "Could not raise {} by {}",
                self.class(),
                rhs.class()
            )));
        };
        let value = self.value().powf(exp);
        let value = create::new_float(value);
        Ok(value)
    }

    make_op!(modulo, %, "Could not divide {} with Float", false);
    make_op!(mul, *, "Could not multiply {} with Float", false);
    make_op!(div, /, "Could not divide {} into Float", false);
    make_op!(floor_div, /, "Could not divide {} into Float", true); // truncates
    make_op!(add, +, "Could not add {} to Float", false);
    make_op!(sub, -, "Could not subtract {} from Float", false);
}

// Display -------------------------------------------------------------

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.value().fract() == 0.0 {
            write!(f, "{}.0", self.value)
        } else {
            write!(f, "{}", self.value)
        }
    }
}

impl fmt::Debug for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
