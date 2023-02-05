use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use num_traits::ToPrimitive;

use feint_code_gen::*;

use super::util::{eq_int_float, float_gt_int, float_lt_int};
use crate::BUILTINS;

use super::base::{ObjectRef, ObjectTrait, TypeRef};

use super::ns::Namespace;

// pub fn make_float_type() -> obj_ref_t!(FloatType) {
//     let type_ref = obj_ref!(FloatType::new());
//     let mut type_obj = type_ref.write().unwrap();
//
//     type_obj.add_attrs(&[
//         // Class Methods -----------------------------------------------
//         meth!("new", type_ref, &["value"], "", |this, args| {
//             let arg = use_arg!(args, 0);
//             let float = if let Some(val) = arg.get_float_val() {
//                 BUILTINS.float(*val)
//             } else if let Some(val) = arg.get_int_val() {
//                 BUILTINS.float(val.to_f64().unwrap())
//             } else if let Some(val) = arg.get_str_val() {
//                 BUILTINS.float_from_string(val)
//             } else {
//                 let msg = format!("Float.new() expected string or float; got {arg}");
//                 BUILTINS.type_err(msg, this)
//             };
//             float
//         }),
//     ]);
//
//     type_ref.clone()
// }

// Float --------------------------------------------------------

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
            let value = BUILTINS.float(value);
            Some(value)
        }
    };
}

pub struct Float {
    class: TypeRef,
    ns: Namespace,
    value: f64,
}

standard_object_impls!(Float);

impl Float {
    pub fn new(class: TypeRef, value: f64) -> Self {
        Self { class, ns: Namespace::default(), value }
    }

    pub fn value(&self) -> &f64 {
        &self.value
    }
}

impl ObjectTrait for Float {
    object_trait_header!();

    fn negate(&self) -> Option<ObjectRef> {
        Some(BUILTINS.float(-*self.value()))
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
        let value = BUILTINS.float(value);
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
