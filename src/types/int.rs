use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};

use once_cell::sync::Lazy;

use crate::vm::{RuntimeBoolResult, RuntimeErr, RuntimeObjResult};

use super::gen;

use super::new;
use super::util::{eq_int_float, gt_int_float, lt_int_float};

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Int Type ------------------------------------------------------------

gen::type_and_impls!(IntType, Int);

pub static INT_TYPE: Lazy<new::obj_ref_t!(IntType)> = Lazy::new(|| {
    let type_ref = new::obj_ref!(IntType::new());
    let mut class = type_ref.write().unwrap();

    class.ns_mut().add_entries(&[
        // Class Methods
        gen::meth!("new", type_ref, &["value"], |_, args, _| {
            let arg = gen::use_arg!(args, 0);
            let int = if let Some(val) = arg.get_int_val() {
                new::int(val.clone())
            } else if let Some(val) = arg.get_float_val() {
                new::int(BigInt::from_f64(*val).unwrap())
            } else if let Some(val) = arg.get_str_val() {
                new::int_from_string(val)
            } else {
                let message = format!("Int.new() expected number or string; got {arg}");
                return Err(RuntimeErr::type_err(message));
            };
            Ok(int)
        }),
    ]);

    type_ref.clone()
});

// Int Object ----------------------------------------------------------

macro_rules! make_op {
    ( $meth:ident, $op:tt, $message:literal ) => {
        fn $meth(&self, rhs: &dyn ObjectTrait) -> RuntimeObjResult {
            if let Some(rhs) = rhs.down_to_int() {
                // XXX: Return Int
                let value = self.value() $op rhs.value();
                let value = new::int(value);
                Ok(value)
            } else if let Some(rhs) = rhs.down_to_float() {
                // XXX: Return Float
                let value = self.value().to_f64().unwrap() $op rhs.value();
                let value = new::float(value);
                Ok(value)
            } else {
                Err(RuntimeErr::type_err(format!($message, rhs.class().read().unwrap())))
            }
        }
    };
}

pub struct Int {
    ns: Namespace,
    value: BigInt,
}

gen::standard_object_impls!(Int);

impl Int {
    pub fn new(value: BigInt) -> Self {
        Self { ns: Namespace::new(), value }
    }

    pub fn value(&self) -> &BigInt {
        &self.value
    }

    // Cast both LHS and RHS to f64 and divide them
    fn div_f64(&self, rhs: &dyn ObjectTrait) -> Result<f64, RuntimeErr> {
        let lhs_val = self.value().to_f64().unwrap();
        let rhs_val = if let Some(rhs) = rhs.down_to_int() {
            rhs.value().to_f64().unwrap()
        } else if let Some(rhs) = rhs.down_to_float() {
            *rhs.value()
        } else {
            return Err(RuntimeErr::type_err(format!(
                "Could not divide {} into Int",
                rhs.class().read().unwrap()
            )));
        };
        Ok(lhs_val / rhs_val)
    }
}

impl ObjectTrait for Int {
    gen::object_trait_header!(INT_TYPE);

    fn negate(&self) -> RuntimeObjResult {
        Ok(new::int(-self.value.clone()))
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if let Some(rhs) = rhs.down_to_int() {
            self.is(rhs) || self.value() == rhs.value()
        } else if let Some(rhs) = rhs.down_to_float() {
            eq_int_float(self, rhs)
        } else {
            false
        }
    }

    fn less_than(&self, rhs: &dyn ObjectTrait) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_int() {
            Ok(self.value() < rhs.value())
        } else if let Some(rhs) = rhs.down_to_float() {
            Ok(lt_int_float(self, rhs))
        } else {
            Err(RuntimeErr::type_err(format!(
                "Could not compare {} to {}: >",
                self.class().read().unwrap(),
                rhs.class().read().unwrap()
            )))
        }
    }

    fn greater_than(&self, rhs: &dyn ObjectTrait) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_int() {
            Ok(self.value() > rhs.value())
        } else if let Some(rhs) = rhs.down_to_float() {
            Ok(gt_int_float(self, rhs))
        } else {
            Err(RuntimeErr::type_err(format!(
                "Could not compare {} to {}: >",
                self.class().read().unwrap(),
                rhs.class().read().unwrap()
            )))
        }
    }

    fn pow(&self, rhs: &dyn ObjectTrait) -> RuntimeObjResult {
        if let Some(rhs) = rhs.down_to_int() {
            // XXX: Return Int
            let base = self.value();
            let exp = rhs.value().to_u32().unwrap();
            let value = base.pow(exp);
            let value = new::int(value);
            Ok(value)
        } else if let Some(rhs) = rhs.down_to_float() {
            // XXX: Return Float
            let base = self.value().to_f64().unwrap();
            let exp = *rhs.value();
            let value = base.powf(exp);
            let value = new::float(value);
            Ok(value)
        } else {
            Err(RuntimeErr::type_err(format!(
                "Could not raise {} by {}",
                self.class().read().unwrap(),
                rhs.class().read().unwrap()
            )))
        }
    }

    make_op!(modulo, %, "Could not divide {} with Int");
    make_op!(mul, *, "Could not multiply {} with Int");
    make_op!(add, +, "Could not add {} to Int");
    make_op!(sub, -, "Could not subtract {} from Int");

    // Int division *always* returns a Float
    fn div(&self, rhs: &dyn ObjectTrait) -> RuntimeObjResult {
        let value = self.div_f64(rhs)?;
        let value = new::float(value);
        Ok(value)
    }

    // Int *floor* division *always* returns an Int
    fn floor_div(&self, rhs: &dyn ObjectTrait) -> RuntimeObjResult {
        let value = self.div_f64(rhs)?;
        let value = BigInt::from_f64(value).unwrap();
        let value = new::int(value);
        Ok(value)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Debug for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
