use std::any::Any;
use std::fmt;
use std::slice::Iter;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::{RuntimeErr, VM};

use super::meth::{make_meth, use_this};
use super::new;
use super::result::{Args, GetAttrResult, This};

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Tuple Type ----------------------------------------------------------

pub static TUPLE_TYPE: Lazy<Arc<RwLock<TupleType>>> =
    Lazy::new(|| Arc::new(RwLock::new(TupleType::new())));

pub struct TupleType {
    ns: Namespace,
}

impl TupleType {
    pub fn new() -> Self {
        Self {
            ns: Namespace::with_entries(&[
                // Class Attributes
                ("$name", new::str("Tuple")),
                ("$full_name", new::str("builtins.Tuple")),
                // Instance Methods
                make_meth!(Tuple, "length", &[], |this: ObjectRef, _, _| {
                    let this = use_this!(this);
                    let this = this.down_to_tuple().unwrap();
                    Ok(new::int(this.len()))
                }),
                make_meth!(Tuple, "is_empty", &[], |this: ObjectRef, _, _| {
                    let this = use_this!(this);
                    let this = this.down_to_tuple().unwrap();
                    Ok(new::bool(this.is_empty()))
                }),
                make_meth!(
                    Tuple,
                    "map",
                    &["map_fn"],
                    |this: ObjectRef, args: Args, vm: &mut VM| {
                        let this = use_this!(this);
                        let this = this.down_to_tuple().unwrap();
                        let items = &this.items;
                        let map_fn = &args[0];
                        let mut results = vec![];
                        for (i, item) in items.iter().enumerate() {
                            vm.call(map_fn.clone(), vec![item.clone(), new::int(i)])?;
                            results.push(vm.pop_obj()?);
                        }
                        Ok(new::tuple(results))
                    }
                ),
            ]),
        }
    }
}

unsafe impl Send for TupleType {}
unsafe impl Sync for TupleType {}

impl TypeTrait for TupleType {
    fn name(&self) -> &str {
        "Tuple"
    }

    fn full_name(&self) -> &str {
        "builtins.Tuple"
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }
}

impl ObjectTrait for TupleType {
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

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
    }
}

// Tuple Object --------------------------------------------------------

pub struct Tuple {
    ns: Namespace,
    items: Vec<ObjectRef>,
}

impl Tuple {
    pub fn new(items: Vec<ObjectRef>) -> Self {
        Self { ns: Namespace::new(), items }
    }

    pub fn iter(&self) -> Iter<'_, ObjectRef> {
        self.items.iter()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl ObjectTrait for Tuple {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn class(&self) -> TypeRef {
        TUPLE_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        TUPLE_TYPE.clone()
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
    }

    fn get_item(&self, index: usize) -> GetAttrResult {
        if index >= self.items.len() {
            return Err(self.index_out_of_bounds(index));
        }
        if let Some(item) = self.items.get(index) {
            Ok(item.clone())
        } else {
            Err(self.item_does_not_exist(index))
        }
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if let Some(rhs) = rhs.down_to_tuple() {
            if self.is(rhs) {
                return true;
            }
            if self.len() != rhs.len() {
                return false;
            }
            for (a, b) in self.iter().zip(rhs.iter()) {
                let a = a.read().unwrap();
                let b = b.read().unwrap();
                if !a.is_equal(&*b) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let num_items = self.len();
        let items: Vec<String> =
            self.iter().map(|item| format!("{:?}", &*item.read().unwrap())).collect();
        let items_str = items.join(", ");
        let trailing_comma = if num_items == 1 { "," } else { "" };
        write!(f, "({}{})", items_str, trailing_comma)
    }
}

impl fmt::Debug for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
