use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use num_traits::ToPrimitive;
use once_cell::sync::Lazy;

use crate::vm::{RuntimeBoolResult, RuntimeErr, RuntimeObjResult, VM};

use super::create;
use super::meth::{make_meth, use_arg};
use super::result::{Args, This};
use super::util::{eq_int_float, gt_int_float, lt_int_float};

use super::base::{ObjectRef, ObjectTrait, ObjectTraitExt, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Float Type ----------------------------------------------------------

pub static FLOAT_TYPE: Lazy<Arc<RwLock<FloatType>>> =
    Lazy::new(|| Arc::new(RwLock::new(FloatType::new())));

pub struct FloatType {
    namespace: Namespace,
}

unsafe impl Send for FloatType {}
unsafe impl Sync for FloatType {}

impl FloatType {
    pub fn new() -> Self {
        Self {
            namespace: Namespace::with_entries(vec![
                // Class Attributes
                ("$name", create::new_str("Float")),
                ("$full_name", create::new_str("builtins.Float")),
                // Class Methods
                make_meth!(FloatType, new, Some(vec!["value"]), |_, args: Args, _| {
                    let arg = use_arg!(args, 0);
                    let float = if let Some(val) = arg.get_float_val() {
                        create::new_float(*val)
                    } else if let Some(val) = arg.get_int_val() {
                        create::new_float(val.to_f64().unwrap())
                    } else if let Some(val) = arg.get_str_val() {
                        create::new_float_from_string(val)
                    } else {
                        let message =
                            format!("Float new expected string or float; got {arg}");
                        return Err(RuntimeErr::new_type_err(message));
                    };
                    Ok(float)
                }),
            ]),
        }
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

    fn namespace(&self) -> &Namespace {
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
                return Err(RuntimeErr::new_type_err(format!($message, rhs.class().read().unwrap())));
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
    namespace: Namespace,
    value: f64,
}

unsafe impl Send for Float {}
unsafe impl Sync for Float {}

impl Float {
    pub fn new(value: f64) -> Self {
        Self { namespace: Namespace::new(), value }
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

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    fn negate(&self) -> RuntimeObjResult {
        Ok(create::new_float(-*self.value()))
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
            Err(RuntimeErr::new_type_err(format!(
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
            return Err(RuntimeErr::new_type_err(format!(
                "Could not raise {} by {}",
                self.class().read().unwrap(),
                rhs.class().read().unwrap()
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
