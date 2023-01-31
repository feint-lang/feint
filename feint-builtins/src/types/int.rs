use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};

use once_cell::sync::Lazy;

use feint_code_gen::*;

use super::new;
use super::util::{eq_int_float, int_gt_float, int_lt_float};

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Int Type ------------------------------------------------------------

static DOC: &str = "
Intrinsic Int type
";

type_and_impls!(IntType, Int);

pub static INT_TYPE: Lazy<obj_ref_t!(IntType)> = Lazy::new(|| {
    let type_ref = obj_ref!(IntType::new());
    let mut type_obj = type_ref.write().unwrap();

    type_obj.add_attrs(&[
        ("$doc", new::str(DOC)),
        // Class Methods -----------------------------------------------
        meth!("new", type_ref, &["value"], "", |this, args| {
            let arg = use_arg!(args, 0);
            let int = if let Some(val) = arg.get_int_val() {
                new::int(val.clone())
            } else if let Some(val) = arg.get_float_val() {
                new::int(BigInt::from_f64(*val).unwrap())
            } else if let Some(val) = arg.get_str_val() {
                new::int_from_string(val)
            } else {
                let msg = format!("Int.new() expected number or string; got {arg}");
                new::type_err(msg, this)
            };
            int
        }),
    ]);

    type_ref.clone()
});

// Int Object ----------------------------------------------------------

macro_rules! make_op {
    ( $meth:ident, $op:tt ) => {
        fn $meth(&self, rhs: &dyn ObjectTrait) -> Option<ObjectRef> {
            if let Some(rhs) = rhs.down_to_int() {
                // XXX: Return Int
                let value = self.value() $op rhs.value();
                Some(new::int(value))
            } else if let Some(rhs) = rhs.down_to_float() {
                // XXX: Return Float
                let value = self.value().to_f64().unwrap() $op rhs.value();
                Some(new::float(value))
            } else {
                None
            }
        }
    };
}

pub struct Int {
    ns: Namespace,
    value: BigInt,
}

standard_object_impls!(Int);

impl Int {
    pub fn new(value: BigInt) -> Self {
        Self { ns: Namespace::default(), value }
    }

    pub fn value(&self) -> &BigInt {
        &self.value
    }

    // Cast both LHS and RHS to f64 and divide them
    fn div_f64(&self, rhs: &dyn ObjectTrait) -> Option<f64> {
        let lhs_val = self.value().to_f64().unwrap();
        let rhs_val = if let Some(rhs) = rhs.down_to_int() {
            rhs.value().to_f64().unwrap()
        } else if let Some(rhs) = rhs.down_to_float() {
            *rhs.value()
        } else {
            return None;
        };
        Some(lhs_val / rhs_val)
    }
}

impl ObjectTrait for Int {
    object_trait_header!(INT_TYPE);

    fn negate(&self) -> Option<ObjectRef> {
        Some(new::int(-self.value.clone()))
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if self.is(rhs) || rhs.is_always() {
            true
        } else if let Some(rhs) = rhs.down_to_int() {
            self.value() == rhs.value()
        } else if let Some(rhs) = rhs.down_to_float() {
            eq_int_float(self, rhs)
        } else {
            false
        }
    }

    fn less_than(&self, rhs: &dyn ObjectTrait) -> Option<bool> {
        if let Some(rhs) = rhs.down_to_int() {
            Some(self.value() < rhs.value())
        } else if let Some(rhs) = rhs.down_to_float() {
            Some(int_lt_float(self, rhs))
        } else {
            None
        }
    }

    fn greater_than(&self, rhs: &dyn ObjectTrait) -> Option<bool> {
        if let Some(rhs) = rhs.down_to_int() {
            Some(self.value() > rhs.value())
        } else if let Some(rhs) = rhs.down_to_float() {
            Some(int_gt_float(self, rhs))
        } else {
            None
        }
    }

    fn pow(&self, rhs: &dyn ObjectTrait) -> Option<ObjectRef> {
        if let Some(rhs) = rhs.down_to_int() {
            // XXX: Return Int
            let base = self.value();
            let exp = rhs.value().to_u32().unwrap();
            let value = base.pow(exp);
            let value = new::int(value);
            Some(value)
        } else if let Some(rhs) = rhs.down_to_float() {
            // XXX: Return Float
            let base = self.value().to_f64().unwrap();
            let exp = *rhs.value();
            let value = base.powf(exp);
            let value = new::float(value);
            Some(value)
        } else {
            None
        }
    }

    make_op!(modulo, %);
    make_op!(mul, *);
    make_op!(add, +);
    make_op!(sub, -);

    // Int division *always* returns a Float
    fn div(&self, rhs: &dyn ObjectTrait) -> Option<ObjectRef> {
        if let Some(value) = self.div_f64(rhs) {
            Some(new::float(value))
        } else {
            None
        }
    }

    // Int *floor* division *always* returns an Int
    fn floor_div(&self, rhs: &dyn ObjectTrait) -> Option<ObjectRef> {
        if let Some(value) = self.div_f64(rhs) {
            let value = BigInt::from_f64(value).unwrap();
            Some(new::int(value))
        } else {
            None
        }
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
