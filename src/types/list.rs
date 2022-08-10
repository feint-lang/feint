use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::{RuntimeErr, VM};

use super::create;
use super::meth::{make_meth, use_arg, use_arg_usize, use_this};
use super::result::{Args, GetAttrResult, This};

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// List Type -----------------------------------------------------------

pub static LIST_TYPE: Lazy<Arc<RwLock<ListType>>> =
    Lazy::new(|| Arc::new(RwLock::new(ListType::new())));

pub struct ListType {
    namespace: Namespace,
}

impl ListType {
    pub fn new() -> Self {
        Self {
            namespace: Namespace::with_entries(vec![
                // Class Attributes
                ("$name", create::new_str("List")),
                ("$full_name", create::new_str("builtins.List")),
                // Instance Methods
                make_meth!(List, length, vec![], |this: ObjectRef, _, _| {
                    let this = use_this!(this);
                    let this = this.down_to_list().unwrap();
                    Ok(create::new_int(this.len()))
                }),
                make_meth!(List, is_empty, vec![], |this: ObjectRef, _, _| {
                    let this = use_this!(this);
                    let this = this.down_to_list().unwrap();
                    Ok(create::new_bool(this.len() == 0))
                }),
                // Push item and return it.
                make_meth!(
                    List,
                    push,
                    vec!["item"],
                    |this: ObjectRef, args: Args, _| {
                        let this = use_this!(this);
                        let this = this.down_to_list().unwrap();
                        let arg = args[0].clone();
                        this.push(arg.clone());
                        Ok(arg)
                    }
                ),
                // Push items and return this.
                make_meth!(List, extend, vec![""], |this: ObjectRef, args: Args, _| {
                    let return_val = this.clone();
                    let this = use_this!(this);
                    let this = this.down_to_list().unwrap();
                    for arg in args {
                        this.push(arg);
                    }
                    Ok(return_val)
                }),
                make_meth!(List, pop, vec![], |this: ObjectRef, _, _| {
                    let this = use_this!(this);
                    let this = this.down_to_list().unwrap();
                    let result = match this.pop() {
                        Some(obj) => obj,
                        None => create::new_nil(),
                    };
                    Ok(result)
                }),
                make_meth!(
                    List,
                    get,
                    vec!["index"],
                    |this: ObjectRef, args: Args, _| {
                        let this = use_this!(this);
                        let this = this.down_to_list().unwrap();
                        let arg = use_arg!(args, 0);
                        let index = use_arg_usize!(arg);
                        let result = match this.get(index) {
                            Some(obj) => obj,
                            None => create::new_nil(),
                        };
                        Ok(result)
                    }
                ),
                make_meth!(
                    List,
                    map,
                    vec!["map_fn"],
                    |this: ObjectRef, args: Args, vm: &mut VM| {
                        let this = use_this!(this);
                        let this = this.down_to_list().unwrap();
                        let map_fn = use_arg!(args, 0);
                        let items = this.items.read().unwrap();
                        let mut results = vec![];
                        for (i, item) in items.iter().enumerate() {
                            map_fn.call(vec![item.clone(), create::new_int(i)], vm)?;
                            results.push(vm.pop_obj()?);
                        }
                        Ok(create::new_tuple(results))
                    }
                ),
            ]),
        }
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

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

impl ObjectTrait for ListType {
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

// List Object ---------------------------------------------------------

pub struct List {
    namespace: Namespace,
    items: RwLock<Vec<ObjectRef>>,
}

impl List {
    pub fn new(items: Vec<ObjectRef>) -> Self {
        Self { namespace: Namespace::new(), items: RwLock::new(items) }
    }

    pub fn len(&self) -> usize {
        let items = self.items.read().unwrap();
        items.len()
    }

    pub fn push(&self, item: ObjectRef) {
        let items = &mut self.items.write().unwrap();
        items.push(item);
    }

    pub fn extend(&self, new_items: Vec<ObjectRef>) {
        let items = &mut self.items.write().unwrap();
        for item in new_items {
            items.push(item);
        }
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

    fn class(&self) -> TypeRef {
        LIST_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        LIST_TYPE.clone()
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
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
