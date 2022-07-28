//! Tuple type
use std::any::Any;
use std::fmt;

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::vm::{RuntimeContext, RuntimeErr};

use super::builtin_types::BUILTIN_TYPES;
use super::class::TypeRef;
use super::object::{Object, ObjectExt, ObjectRef};
use super::result::GetAttrResult;

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
    fn class(&self) -> &TypeRef {
        BUILTIN_TYPES.get("Tuple").unwrap()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_equal(&self, rhs: &dyn Object, _ctx: &RuntimeContext) -> bool {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            if self.is(rhs) {
                return true;
            }
            if self.len() != rhs.len() {
                return false;
            }
            for (a, b) in self.items().iter().zip(rhs.items()) {
                if !a.is_equal(&**b, _ctx) {
                    return false;
                }
            }
            return true;
        } else {
            false
        }
    }

    fn get_attr(&self, name: &str, ctx: &RuntimeContext) -> GetAttrResult {
        if let Some(attr) = self.get_base_attr(name, ctx) {
            return Ok(attr);
        }
        let attr = match name {
            "length" => ctx.builtins.new_int(self.len()),
            _ => {
                return Err(RuntimeErr::new_attr_does_not_exist(
                    self.qualified_type_name().as_str(),
                    name,
                ))
            }
        };
        Ok(attr)
    }

    fn get_item(&self, index: &BigInt, _ctx: &RuntimeContext) -> GetAttrResult {
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
