use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use num_traits::ToPrimitive;
use once_cell::sync::Lazy;

use feint_code_gen::*;

use super::new;
use super::util::{eq_int_float, float_gt_int, float_lt_int};

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Float Type ----------------------------------------------------------

type_and_impls!(FloatType, Float);

pub static FLOAT_TYPE: Lazy<obj_ref_t!(FloatType)> = Lazy::new(|| {
    let type_ref = obj_ref!(FloatType::new());
    let mut type_obj = type_ref.write().unwrap();

    type_obj.add_attrs(&[
        // Class Methods -----------------------------------------------
        meth!("new", type_ref, &["value"], "", |this, args| {
            let arg = use_arg!(args, 0);
            let float = if let Some(val) = arg.get_float_val() {
                new::float(*val)
            } else if let Some(val) = arg.get_int_val() {
                new::float(val.to_f64().unwrap())
            } else if let Some(val) = arg.get_str_val() {
                new::float_from_string(val)
            } else {
                let msg = format!("Float.new() expected string or float; got {arg}");
                new::type_err(msg, this)
            };
            float
        }),
    ]);

    type_ref.clone()
});

// Float Object --------------------------------------------------------

macro_rules! make_op {
    ( $meth:ident, $op:tt, $message:literal, $trunc:literal ) => {
        fn $meth(&self, rhs: &dyn ObjectTrait) -> Option<ObjectRef> {
            let value = if let Some(rhs) = rhs.down_to_float() {
                *rhs.value()
            } else if let Some(rhs) = rhs.down_to_int() {
                rhs.value().to_f64().unwrap()
            } else {
                return None
            };
            let mut value = &self.value $op value;
            if $trunc {
                value = value.trunc();
            }
            let value = new::float(value);
            Some(value)
        }
    };
}

pub struct Float {
    ns: Namespace,
    value: f64,
}

standard_object_impls!(Float);

impl Float {
    pub fn new(value: f64) -> Self {
        Self { ns: Namespace::default(), value }
    }

    pub fn value(&self) -> &f64 {
        &self.value
    }
}

impl ObjectTrait for Float {
    object_trait_header!(FLOAT_TYPE);

    fn negate(&self) -> Option<ObjectRef> {
        Some(new::float(-*self.value()))
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if self.is(rhs) || rhs.is_always() {
            true
        } else if let Some(rhs) = rhs.down_to_float() {
            self.value() == rhs.value()
        } else if let Some(rhs) = rhs.down_to_int() {
            eq_int_float(rhs, self)
        } else {
            false
        }
    }

    fn less_than(&self, rhs: &dyn ObjectTrait) -> Option<bool> {
        if let Some(rhs) = rhs.down_to_float() {
            Some(self.value() < rhs.value())
        } else if let Some(rhs) = rhs.down_to_int() {
            Some(float_lt_int(self, rhs))
        } else {
            None
        }
    }

    fn greater_than(&self, rhs: &dyn ObjectTrait) -> Option<bool> {
        if let Some(rhs) = rhs.down_to_float() {
            Some(self.value() > rhs.value())
        } else if let Some(rhs) = rhs.down_to_int() {
            Some(float_gt_int(self, rhs))
        } else {
            return None;
        }
    }

    fn pow(&self, rhs: &dyn ObjectTrait) -> Option<ObjectRef> {
        let exp = if let Some(rhs) = rhs.down_to_float() {
            *rhs.value()
        } else if let Some(rhs) = rhs.down_to_int() {
            rhs.value().to_f64().unwrap()
        } else {
            return None;
        };
        let value = self.value().powf(exp);
        let value = new::float(value);
        Some(value)
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
