use std::any::Any;
use std::fmt;
use std::slice::Iter;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use feint_code_gen::*;

use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;
use super::seq;

// Tuple Type ----------------------------------------------------------

type_and_impls!(TupleType, Tuple);

pub static TUPLE_TYPE: Lazy<obj_ref_t!(TupleType)> = Lazy::new(|| {
    let type_ref = obj_ref!(TupleType::new());
    let mut type_obj = type_ref.write().unwrap();

    type_obj.add_attrs(&[
        // Instance Attributes -----------------------------------------
        prop!("length", type_ref, "", |this, _| {
            let this = this.read().unwrap();
            let this = this.down_to_tuple().unwrap();
            new::int(this.len())
        }),
        prop!("is_empty", type_ref, "", |this, _| {
            let this = this.read().unwrap();
            let this = this.down_to_tuple().unwrap();
            new::bool(this.len() == 0)
        }),
        prop!("sum", type_ref, "", |this, _| {
            let this = this.read().unwrap();
            let this = this.down_to_tuple().unwrap();
            seq::sum(&this.items)
        }),
        // Instance Methods --------------------------------------------
        meth!(
            "each",
            type_ref,
            &["each_fn"],
            "Apply function to each Tuple item.

            # Args

            - func: Func

              A function that will be passed each item in turn and, optionally, the
              index of the item.

            ",
            |this_obj, args| {
                let this = this_obj.read().unwrap();
                let this = this.down_to_tuple().unwrap();
                seq::each(&this_obj, &this.items, &args)
            }
        ),
        meth!("get", type_ref, &["index"], "", |this, args| {
            let this = this.read().unwrap();
            let this = this.down_to_tuple().unwrap();
            let index = use_arg_usize!(get, index, args, 0);
            match this.get(index) {
                Some(obj) => obj,
                None => new::nil(),
            }
        }),
        meth!("has", type_ref, &["member"], "", |this, args| {
            let this = this.read().unwrap();
            let this = this.down_to_tuple().unwrap();
            seq::has(&this.items, &args)
        }),
        meth!("iter", type_ref, &[], "", |this_ref, _| {
            let this = this_ref.read().unwrap();
            let this = this.down_to_tuple().unwrap();
            new::iterator(this.items.clone())
        }),
        meth!("join", type_ref, &["sep"], "", |this, args| {
            let this = this.read().unwrap();
            let this = this.down_to_tuple().unwrap();
            seq::join(&this.items, &args)
        }),
        meth!("map", type_ref, &["map_fn"], "", |this_obj, args| {
            let this = this_obj.read().unwrap();
            let this = this.down_to_tuple().unwrap();
            seq::map(&this_obj, &this.items, &args)
        }),
    ]);

    type_ref.clone()
});

// Tuple Object --------------------------------------------------------

pub struct Tuple {
    ns: Namespace,
    items: Vec<ObjectRef>,
}

standard_object_impls!(Tuple);

impl Tuple {
    pub fn new(items: Vec<ObjectRef>) -> Self {
        Self { ns: Namespace::default(), items }
    }

    pub fn iter(&self) -> Iter<'_, ObjectRef> {
        self.items.iter()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get(&self, index: usize) -> Option<ObjectRef> {
        if let Some(item) = self.items.get(index) {
            Some(item.clone())
        } else {
            None
        }
    }
}

impl ObjectTrait for Tuple {
    object_trait_header!(TUPLE_TYPE);

    fn get_item(&self, index: usize, this: ObjectRef) -> ObjectRef {
        if let Some(item) = self.items.get(index) {
            item.clone()
        } else {
            self.index_out_of_bounds(index, this)
        }
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if self.is(rhs) || rhs.is_always() {
            return true;
        }
        if let Some(rhs) = rhs.down_to_tuple() {
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
        write!(f, "({items_str}{trailing_comma})")
    }
}

impl fmt::Debug for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
