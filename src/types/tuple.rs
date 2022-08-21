use std::any::Any;
use std::fmt;
use std::slice::Iter;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::gen;

use super::new;
use super::result::GetAttrResult;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Tuple Type ----------------------------------------------------------

gen::type_and_impls!(TupleType, Tuple);

pub static TUPLE_TYPE: Lazy<new::obj_ref_t!(TupleType)> = Lazy::new(|| {
    let type_ref = new::obj_ref!(TupleType::new());
    let mut class = type_ref.write().unwrap();

    class.ns_mut().add_entries(&[
        // Class Attributes
        ("$name", new::str("Tuple")),
        ("$full_name", new::str("builtins.Tuple")),
        // Instance Methods
        gen::meth!("length", type_ref, &[], |this, _, _| {
            let this = this.read().unwrap();
            let this = this.down_to_tuple().unwrap();
            Ok(new::int(this.len()))
        }),
        gen::meth!("is_empty", type_ref, &[], |this, _, _| {
            let this = this.read().unwrap();
            let this = this.down_to_tuple().unwrap();
            Ok(new::bool(this.is_empty()))
        }),
        gen::meth!("map", type_ref, &["map_fn"], |this, args, vm| {
            let this = this.read().unwrap();
            let this = this.down_to_tuple().unwrap();
            let items = &this.items;
            let map_fn = &args[0];
            let mut results = vec![];
            for (i, item) in items.iter().enumerate() {
                vm.call(map_fn.clone(), vec![item.clone(), new::int(i)])?;
                results.push(vm.pop_obj()?);
            }
            Ok(new::tuple(results))
        }),
    ]);

    type_ref.clone()
});

// Tuple Object --------------------------------------------------------

pub struct Tuple {
    ns: Namespace,
    items: Vec<ObjectRef>,
}

gen::standard_object_impls!(Tuple);

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
    gen::object_trait_header!(TUPLE_TYPE);

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
