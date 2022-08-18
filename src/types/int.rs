use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};

use once_cell::sync::Lazy;

use crate::vm::{RuntimeBoolResult, RuntimeErr, RuntimeObjResult, VM};

use super::create;
use super::meth::{make_meth, use_arg};
use super::result::{Args, This};
use super::util::{eq_int_float, gt_int_float, lt_int_float};

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Int Type ------------------------------------------------------------

pub static INT_TYPE: Lazy<Arc<RwLock<IntType>>> =
    Lazy::new(|| Arc::new(RwLock::new(IntType::new())));

pub struct IntType {
    namespace: Namespace,
}

unsafe impl Send for IntType {}
unsafe impl Sync for IntType {}

impl IntType {
    pub fn new() -> Self {
        Self {
            namespace: Namespace::with_entries(&[
                // Class Attributes
                ("$name", create::new_str("Int")),
                ("$full_name", create::new_str("builtins.Int")),
                // Class Methods
                make_meth!(IntType, "new", &["value"], |_, args: Args, _| {
                    let arg = use_arg!(args, 0);
                    let int = if let Some(val) = arg.get_int_val() {
                        create::new_int(val.clone())
                    } else if let Some(val) = arg.get_float_val() {
                        create::new_int(BigInt::from_f64(*val).unwrap())
                    } else if let Some(val) = arg.get_str_val() {
                        create::new_int_from_string(val)
                    } else {
                        let message =
                            format!("Int.new() expected number or string; got {arg}");
                        return Err(RuntimeErr::type_err(message));
                    };
                    Ok(int)
                }),
            ]),
        }
    }
}

impl TypeTrait for IntType {
    fn name(&self) -> &str {
        "Int"
    }

    fn full_name(&self) -> &str {
        "builtins.Int"
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

impl ObjectTrait for IntType {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
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

// Int Object ----------------------------------------------------------

macro_rules! make_op {
    ( $meth:ident, $op:tt, $message:literal ) => {
        fn $meth(&self, rhs: &dyn ObjectTrait) -> RuntimeObjResult {
            if let Some(rhs) = rhs.down_to_int() {
                // XXX: Return Int
                let value = self.value() $op rhs.value();
                let value = create::new_int(value);
                Ok(value)
            } else if let Some(rhs) = rhs.down_to_float() {
                // XXX: Return Float
                let value = self.value().to_f64().unwrap() $op rhs.value();
                let value = create::new_float(value);
                Ok(value)
            } else {
                Err(RuntimeErr::type_err(format!($message, rhs.class().read().unwrap())))
            }
        }
    };
}

pub struct Int {
    namespace: Namespace,
    value: BigInt,
}

unsafe impl Send for Int {}
unsafe impl Sync for Int {}

impl Int {
    pub fn new(value: BigInt) -> Self {
        Self { namespace: Namespace::new(), value }
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
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn class(&self) -> TypeRef {
        INT_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        INT_TYPE.clone()
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    fn negate(&self) -> RuntimeObjResult {
        Ok(create::new_int(-self.value.clone()))
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
            let value = create::new_int(value);
            Ok(value)
        } else if let Some(rhs) = rhs.down_to_float() {
            // XXX: Return Float
            let base = self.value().to_f64().unwrap();
            let exp = *rhs.value();
            let value = base.powf(exp);
            let value = create::new_float(value);
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
        let value = create::new_float(value);
        Ok(value)
    }

    // Int *floor* division *always* returns an Int
    fn floor_div(&self, rhs: &dyn ObjectTrait) -> RuntimeObjResult {
        let value = self.div_f64(rhs)?;
        let value = BigInt::from_f64(value).unwrap();
        let value = create::new_int(value);
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
