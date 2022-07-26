//! Built in tuple type
use std::any::Any;
use std::fmt;

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::vm::{RuntimeBoolResult, RuntimeContext, RuntimeErr};

use super::class::Type;
use super::object::{Object, ObjectExt, ObjectRef};
use super::result::GetAttributeResult;
use super::types::TYPES;

pub struct Tuple {
    items: Vec<ObjectRef>,
}

impl Tuple {
    pub fn new(items: Vec<ObjectRef>) -> Self {
        Self { items }
    }

    pub fn items(&self) -> &Vec<ObjectRef> {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl Object for Tuple {
    fn class(&self) -> &Type {
        TYPES.get("Tuple").unwrap()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_equal(&self, rhs: &ObjectRef, ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            if self.is(rhs) {
                return Ok(true);
            }
            if self.len() != rhs.len() {
                return Ok(false);
            }
            for (a, b) in self.items().iter().zip(rhs.items()) {
                if !a.is_equal(b, ctx)? {
                    return Ok(false);
                }
            }
            return Ok(true);
        } else {
            Ok(false)
        }
    }

    fn get_attribute(&self, name: &str, ctx: &RuntimeContext) -> GetAttributeResult {
        match name {
            "length" => Ok(ctx.builtins.new_int(self.len())),
            _ => Err(RuntimeErr::new_attribute_does_not_exit(name)),
        }
    }

    fn get_item(&self, index: &BigInt, _ctx: &RuntimeContext) -> GetAttributeResult {
        let index = index.to_usize().unwrap();
        match self.items.get(index) {
            Some(obj) => Ok(obj.clone()),
            None => return Err(RuntimeErr::new_index_out_of_bounds(index)),
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let num_items = self.items().len();
        let items: Vec<String> =
            self.items().iter().map(|item| format!("{item}")).collect();
        let trailing_comma = if num_items == 1 { "," } else { "" };
        write!(f, "({}{})", items.join(", "), trailing_comma)
    }
}

impl fmt::Debug for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
