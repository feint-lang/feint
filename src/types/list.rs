use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::{RuntimeErr, RuntimeResult};

use super::gen;

use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// List Type -----------------------------------------------------------

gen::type_and_impls!(ListType, List);

pub static LIST_TYPE: Lazy<new::obj_ref_t!(ListType)> = Lazy::new(|| {
    let type_ref = new::obj_ref!(ListType::new());
    let mut class = type_ref.write().unwrap();

    class.ns_mut().add_entries(&[
        // Instance Attributes -----------------------------------------
        gen::prop!("length", type_ref, |this, _, _| {
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            Ok(new::int(this.len()))
        }),
        gen::prop!("is_empty", type_ref, |this, _, _| {
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            Ok(new::bool(this.len() == 0))
        }),
        // Instance Methods --------------------------------------------
        // Push item and return it.
        gen::meth!("push", type_ref, &["item"], |this, args, _| {
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            let arg = args[0].clone();
            this.push(arg.clone());
            Ok(arg)
        }),
        // Push items and return this.
        gen::meth!("extend", type_ref, &["items"], |this, args, _| {
            let return_val = this.clone();
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            this.extend(args[0].clone())?;
            Ok(return_val)
        }),
        gen::meth!("pop", type_ref, &[], |this, _, _| {
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            let result = match this.pop() {
                Some(obj) => obj,
                None => new::nil(),
            };
            Ok(result)
        }),
        gen::meth!("get", type_ref, &["index"], |this, args, _| {
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            let arg = gen::use_arg!(args, 0);
            let index = gen::use_arg_usize!(arg);
            let result = match this.get(index) {
                Some(obj) => obj,
                None => new::nil(),
            };
            Ok(result)
        }),
        gen::meth!("map", type_ref, &["map_fn"], |this, args, vm| {
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            let items = this.items.read().unwrap();
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

// List Object ---------------------------------------------------------

pub struct List {
    ns: Namespace,
    items: RwLock<Vec<ObjectRef>>,
}

gen::standard_object_impls!(List);

impl List {
    pub fn new(items: Vec<ObjectRef>) -> Self {
        Self { ns: Namespace::new(), items: RwLock::new(items) }
    }

    pub fn len(&self) -> usize {
        let items = self.items.read().unwrap();
        items.len()
    }

    pub fn push(&self, item: ObjectRef) {
        let items = &mut self.items.write().unwrap();
        items.push(item);
    }

    pub fn extend(&self, obj: ObjectRef) -> RuntimeResult {
        let obj = obj.read().unwrap();
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
            return Err(RuntimeErr::type_err(msg));
        }
        Ok(())
    }

    pub fn pop(&self) -> Option<ObjectRef> {
        let items = &mut self.items.write().unwrap();
        if let Some(item) = items.pop() {
            Some(item.clone())
        } else {
            None
        }
    }

    pub fn get(&self, index: usize) -> Option<ObjectRef> {
        let items = self.items.read().unwrap();
        if let Some(item) = items.get(index) {
            Some(item.clone())
        } else {
            None
        }
    }
}

impl ObjectTrait for List {
    gen::object_trait_header!(LIST_TYPE);

    fn get_item(&self, index: usize, this: ObjectRef) -> ObjectRef {
        if let Some(item) = self.get(index) {
            item.clone()
        } else {
            self.index_out_of_bounds(index, this)
        }
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if let Some(rhs) = rhs.down_to_list() {
            if self.is(rhs) {
                return true;
            }
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
        write!(f, "[{}]", items_str)
    }
}

impl fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
