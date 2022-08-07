use std::any::Any;
use std::fmt;
use std::slice::Iter;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::{RuntimeErr, VM};

use super::create;
use super::meth::{make_meth, use_arg, use_this};
use super::result::{Args, GetAttrResult, This};

use super::base::{ObjectRef, ObjectTrait, ObjectTraitExt, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Tuple Type ----------------------------------------------------------

pub static TUPLE_TYPE: Lazy<Arc<RwLock<TupleType>>> =
    Lazy::new(|| Arc::new(RwLock::new(TupleType::new())));

pub struct TupleType {
    namespace: Namespace,
}

impl TupleType {
    pub fn new() -> Self {
        let mut ns = Namespace::new();

        ns.add_obj("$name", create::new_str("Tuple"));
        ns.add_obj("$full_name", create::new_str("builtins.Tuple"));

        ns.add_entry(make_meth!(
            Tuple,
            length,
            Some(vec![]) as Option<Vec<&str>>,
            |this: ObjectRef, _, _| {
                let this = use_this!(this);
                let this = this.down_to_tuple().unwrap();
                Ok(create::new_int(this.len()))
            }
        ));

        ns.add_entry(make_meth!(
            Tuple,
            map,
            Some(vec!["map_fn"]),
            |this: ObjectRef, args: Args, vm: &mut VM| {
                let this = use_this!(this);
                let this = this.down_to_tuple().unwrap();
                let map_fn = use_arg!(args, 0);
                for (i, item) in this.iter().enumerate() {
                    let i = create::new_int(i);
                    map_fn.call(vec![item.clone(), i], vm)?;
                }
                Ok(create::new_nil())
            }
        ));

        Self { namespace: ns }
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
}

impl ObjectTrait for TupleType {
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

// Tuple Object --------------------------------------------------------

pub struct Tuple {
    namespace: Namespace,
    items: Vec<ObjectRef>,
}

impl Tuple {
    pub fn new(items: Vec<ObjectRef>) -> Self {
        Self { namespace: Namespace::new(), items }
    }

    pub fn iter(&self) -> Iter<'_, ObjectRef> {
        self.items.iter()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl ObjectTrait for Tuple {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        TUPLE_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        TUPLE_TYPE.clone()
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
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
