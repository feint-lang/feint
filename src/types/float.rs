use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use num_traits::ToPrimitive;
use once_cell::sync::Lazy;

use crate::vm::{RuntimeBoolResult, RuntimeErr, RuntimeObjResult};

use super::gen;

use super::new;
use super::util::{eq_int_float, gt_int_float, lt_int_float};

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Float Type ----------------------------------------------------------

gen::type_and_impls!(FloatType, Float);

pub static FLOAT_TYPE: Lazy<new::obj_ref_t!(FloatType)> = Lazy::new(|| {
    let type_ref = new::obj_ref!(FloatType::new());
    let mut class = type_ref.write().unwrap();

    class.ns_mut().add_entries(&[
        // Class Methods
        gen::meth!("new", type_ref, &["value"], |_, args, _| {
            let arg = gen::use_arg!(args, 0);
            let float = if let Some(val) = arg.get_float_val() {
                new::float(*val)
            } else if let Some(val) = arg.get_int_val() {
                new::float(val.to_f64().unwrap())
            } else if let Some(val) = arg.get_str_val() {
                new::float_from_string(val)
            } else {
                let message = format!("Float new expected string or float; got {arg}");
                return Err(RuntimeErr::type_err(message));
            };
            Ok(float)
        }),
    ]);

    type_ref.clone()
});

// Float Object --------------------------------------------------------

macro_rules! make_op {
    ( $meth:ident, $op:tt, $message:literal, $trunc:literal ) => {
        fn $meth(&self, rhs: &dyn ObjectTrait) -> RuntimeObjResult {
            let value = if let Some(rhs) = rhs.down_to_float() {
                *rhs.value()
            } else if let Some(rhs) = rhs.down_to_int() {
                rhs.value().to_f64().unwrap()
            } else {
                return Err(RuntimeErr::type_err(format!($message, rhs.class().read().unwrap())));
            };
            let mut value = &self.value $op value;
            if $trunc {
                value = value.trunc();
            }
            let value = new::float(value);
            Ok(value)
        }
    };
}

pub struct Float {
    ns: Namespace,
    value: f64,
}

gen::standard_object_impls!(Float);

impl Float {
    pub fn new(value: f64) -> Self {
        Self { ns: Namespace::new(), value }
    }

    pub fn value(&self) -> &f64 {
        &self.value
    }
}

impl ObjectTrait for Float {
    gen::object_trait_header!(FLOAT_TYPE);

    fn negate(&self) -> RuntimeObjResult {
        Ok(new::float(-*self.value()))
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
            Err(RuntimeErr::type_err(format!(
                "Could not compare {} to {}: <",
                self.class().read().unwrap(),
                rhs.class().read().unwrap()
            )))
        }
    }

    fn greater_than(&self, rhs: &dyn ObjectTrait) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_float() {
            Ok(self.value() > rhs.value())
        } else if let Some(rhs) = rhs.down_to_int() {
            Ok(gt_int_float(rhs, self))
        } else {
            Err(RuntimeErr::type_err(format!(
                "Could not compare {} to {}: >",
                self.class().read().unwrap(),
                rhs.class().read().unwrap()
            )))
        }
    }

    fn pow(&self, rhs: &dyn ObjectTrait) -> RuntimeObjResult {
        let exp = if let Some(rhs) = rhs.down_to_float() {
            *rhs.value()
        } else if let Some(rhs) = rhs.down_to_int() {
            rhs.value().to_f64().unwrap()
        } else {
            return Err(RuntimeErr::type_err(format!(
                "Could not raise {} by {}",
                self.class().read().unwrap(),
                rhs.class().read().unwrap()
            )));
        };
        let value = self.value().powf(exp);
        let value = new::float(value);
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
