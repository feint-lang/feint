use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::{RuntimeErr, RuntimeResult};

use super::meth::{make_meth, use_arg, use_arg_usize};
use super::new;
use super::result::GetAttrResult;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// List Type -----------------------------------------------------------

pub static LIST_TYPE: Lazy<Arc<RwLock<ListType>>> = Lazy::new(|| {
    let type_ref = Arc::new(RwLock::new(ListType::new()));
    let mut class = type_ref.write().unwrap();

    class.ns_mut().add_entries(&[
        // Class Attributes
        ("$name", new::str("List")),
        ("$full_name", new::str("builtins.List")),
        // Instance Methods
        make_meth!("length", type_ref, &[], |this, _, _| {
            let this = this.unwrap();
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            Ok(new::int(this.len()))
        }),
        make_meth!("is_empty", type_ref, &[], |this, _, _| {
            let this = this.unwrap();
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            Ok(new::bool(this.len() == 0))
        }),
        // Push item and return it.
        make_meth!("push", type_ref, &["item"], |this, args, _| {
            let this = this.unwrap();
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            let arg = args[0].clone();
            this.push(arg.clone());
            Ok(arg)
        }),
        // Push items and return this.
        make_meth!("extend", type_ref, &["items"], |this, args, _| {
            let this_ref = this.unwrap();
            let this = this_ref.read().unwrap();
            let this = this.down_to_list().unwrap();
            this.extend(args[0].clone())?;
            Ok(this_ref.clone())
        }),
        make_meth!("pop", type_ref, &[], |this, _, _| {
            let this = this.unwrap();
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            let result = match this.pop() {
                Some(obj) => obj,
                None => new::nil(),
            };
            Ok(result)
        }),
        make_meth!("get", type_ref, &["index"], |this, args, _| {
            let this = this.unwrap();
            let this = this.read().unwrap();
            let this = this.down_to_list().unwrap();
            let arg = use_arg!(args, 0);
            let index = use_arg_usize!(arg);
            let result = match this.get(index) {
                Some(obj) => obj,
                None => new::nil(),
            };
            Ok(result)
        }),
        make_meth!("map", type_ref, &["map_fn"], |this, args, vm| {
            let this = this.unwrap();
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

pub struct ListType {
    ns: Namespace,
}

impl ListType {
    pub fn new() -> Self {
        Self { ns: Namespace::new() }
    }
}

unsafe impl Send for ListType {}
unsafe impl Sync for ListType {}

impl TypeTrait for ListType {
    fn name(&self) -> &str {
        "List"
    }

    fn full_name(&self) -> &str {
        "builtins.List"
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }
}

impl ObjectTrait for ListType {
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

// List Object ---------------------------------------------------------

pub struct List {
    ns: Namespace,
    items: RwLock<Vec<ObjectRef>>,
}

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
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn class(&self) -> TypeRef {
        LIST_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        LIST_TYPE.clone()
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
    }

    fn get_item(&self, index: usize) -> GetAttrResult {
        if index >= self.len() {
            return Err(self.index_out_of_bounds(index));
        }
        if let Some(item) = self.get(index) {
            Ok(item.clone())
        } else {
            Err(self.item_does_not_exist(index))
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
