use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use feint_code_gen::*;

use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;
use super::seq;

// List Type -----------------------------------------------------------

type_and_impls!(ListType, List);

pub static LIST_TYPE: Lazy<obj_ref_t!(ListType)> = Lazy::new(|| {
    let type_ref = obj_ref!(ListType::new());
    let mut type_obj = type_ref.write().unwrap();

    type_obj.add_attrs(&[
        // Instance Attributes -----------------------------------------
        prop!("length", type_ref, "", |this, _| {
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            new::int(this.len())
        }),
        prop!("is_empty", type_ref, "", |this, _| {
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            new::bool(this.len() == 0)
        }),
        prop!("sum", type_ref, "", |this, _| {
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            let items = &this.items.read().unwrap();
            seq::sum(items)
        }),
        // Instance Methods --------------------------------------------
        meth!(
            "extend",
            type_ref,
            &["items"],
            "Push items and return this.",
            |this, args| {
                let return_val = this.clone();
                let this = this.read().unwrap();
                let this = this.down_to_list().unwrap();
                if let Some(err) = this.extend(args[0].clone()) {
                    return err;
                }
                return_val
            }
        ),
        meth!("get", type_ref, &["index"], "", |this, args| {
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            let index = use_arg_usize!(get, index, args, 0);
            let result = match this.get(index) {
                Some(obj) => obj,
                None => new::nil(),
            };
            result
        }),
        meth!("has", type_ref, &["member"], "", |this, args| {
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            let items = &this.items.read().unwrap();
            seq::has(items, &args)
        }),
        meth!("iter", type_ref, &[], "", |this_ref, _| {
            let this = this_ref.read().unwrap();
            let this = this.down_to_list().unwrap();
            let items = this.items.read().unwrap();
            new::iterator(items.clone())
        }),
        meth!("join", type_ref, &["sep"], "", |this, args| {
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            let items = &this.items.read().unwrap();
            seq::join(items, &args)
        }),
        meth!("pop", type_ref, &[], "", |this, _| {
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            match this.pop() {
                Some(obj) => obj,
                None => new::nil(),
            }
        }),
        meth!("push", type_ref, &["item"], "Push item and return it.", |this, args| {
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            let arg = args[0].clone();
            this.push(arg.clone());
            arg
        }),
    ]);

    type_ref.clone()
});

// List Object ---------------------------------------------------------

pub struct List {
    ns: Namespace,
    items: RwLock<Vec<ObjectRef>>,
}

standard_object_impls!(List);

impl List {
    pub fn new(items: Vec<ObjectRef>) -> Self {
        Self { ns: Namespace::default(), items: RwLock::new(items) }
    }

    fn len(&self) -> usize {
        let items = self.items.read().unwrap();
        items.len()
    }

    fn push(&self, item: ObjectRef) {
        let items = &mut self.items.write().unwrap();
        items.push(item);
    }

    fn extend(&self, obj_ref: ObjectRef) -> Option<ObjectRef> {
        let obj = obj_ref.read().unwrap();
        let items = &mut self.items.write().unwrap();
        if let Some(list) = obj.down_to_list() {
            let new_items = list.items.read().unwrap();
            for item in new_items.iter() {
                items.push(item.clone());
            }
        } else if let Some(tuple) = obj.down_to_tuple() {
            for item in tuple.iter() {
                items.push(item.clone());
            }
        } else {
            // TODO: Do type checking at a higher level
            let msg = format!(
                "List.extend() expected List or Tuple; got {}",
                obj.class().read().unwrap()
            );
            return Some(new::type_err(msg, obj_ref.clone()));
        }
        None
    }

    fn pop(&self) -> Option<ObjectRef> {
        let items = &mut self.items.write().unwrap();
        if let Some(item) = items.pop() {
            Some(item.clone())
        } else {
            None
        }
    }

    fn get(&self, index: usize) -> Option<ObjectRef> {
        let items = self.items.read().unwrap();
        if let Some(item) = items.get(index) {
            Some(item.clone())
        } else {
            None
        }
    }
}

impl ObjectTrait for List {
    object_trait_header!(LIST_TYPE);

    fn get_item(&self, index: usize, this: ObjectRef) -> ObjectRef {
        if let Some(item) = self.get(index) {
            item.clone()
        } else {
            self.index_out_of_bounds(index, this)
        }
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if self.is(rhs) || rhs.is_always() {
            return true;
        }
        if let Some(rhs) = rhs.down_to_list() {
            if self.len() != rhs.len() {
                return false;
            }
            let items = self.items.read().unwrap();
            let rhs_items = rhs.items.read().unwrap();
            for (a, b) in items.iter().zip(rhs_items.iter()) {
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

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let this_id = self.id();
        let items = self.items.read().unwrap();
        let items: Vec<String> = items
            .iter()
            .map(|item| {
                let item = item.read().unwrap();
                if item.id() == this_id {
                    "[...]".to_owned()
                } else {
                    format!("{:?}", &*item)
                }
            })
            .collect();
        let items_str = items.join(", ");
        write!(f, "[{items_str}]")
    }
}

impl fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
