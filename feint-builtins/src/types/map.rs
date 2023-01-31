use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use once_cell::sync::Lazy;

use super::new;
use feint_code_gen::*;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Map Type ------------------------------------------------------------

type_and_impls!(MapType, Map);

pub static MAP_TYPE: Lazy<obj_ref_t!(MapType)> = Lazy::new(|| {
    let type_ref = obj_ref!(MapType::new());
    let mut type_obj = type_ref.write().unwrap();

    type_obj.add_attrs(&[
        // Instance Attributes -----------------------------------------
        prop!("length", type_ref, "", |this, _| {
            let this = this.read().unwrap();
            let this = this.down_to_map().unwrap();
            new::int(this.len())
        }),
        prop!("is_empty", type_ref, "", |this, _| {
            let this = this.read().unwrap();
            let this = this.down_to_map().unwrap();
            new::bool(this.is_empty())
        }),
        // Instance Methods --------------------------------------------
        meth!(
            "add",
            type_ref,
            &["key", "val"],
            "Add entry to Map.

            # Args

            - key: Str
            - value: Any

            ",
            |this, args| {
                let this = this.read().unwrap();
                let this = this.down_to_map().unwrap();
                let arg = use_arg!(args, 0);
                let key = use_arg_str!(get, key, arg);
                let val = args[1].clone();
                this.insert(key, val);
                new::nil()
            }
        ),
        meth!(
            "get",
            type_ref,
            &["key"],
            "Get value for key from Map.

            # Args

            - key: Key

            # Returns

            - Any: If key is present
            - nil: If key is not present

            > NOTE: There's no way to distinguish between a key that isn't present
            > versus a key that has `nil` as its value. To avoid ambiguity, don't
            > store `nil` values.

            ",
            |this, args| {
                let this = this.read().unwrap();
                let this = this.down_to_map().unwrap();
                let arg = use_arg!(args, 0);
                let key = use_arg_str!(get, key, arg);
                match this.get(key) {
                    Some(obj) => obj,
                    None => new::nil(),
                }
            }
        ),
        meth!("has", type_ref, &["member"], "", |this, args| {
            let this = this.read().unwrap();
            let this = this.down_to_map().unwrap();
            let arg = use_arg!(args, 0);
            let key = use_arg_str!(get, key, arg);
            let result = this.contains_key(key);
            new::bool(result)
        }),
        meth!("iter", type_ref, &[], "", |this_ref, _| {
            let this = this_ref.read().unwrap();
            let this = this.down_to_map().unwrap();
            let mut items = vec![];
            for (name, val) in this.entries.read().unwrap().iter() {
                items.push(new::tuple(vec![new::str(name), val.clone()]))
            }
            new::iterator(items)
        }),
    ]);

    type_ref.clone()
});

// Map Object ----------------------------------------------------------

pub struct Map {
    ns: Namespace,
    entries: RwLock<IndexMap<String, ObjectRef>>,
}

standard_object_impls!(Map);

impl Default for Map {
    fn default() -> Self {
        Self { ns: Namespace::default(), entries: RwLock::new(IndexMap::default()) }
    }
}

impl Map {
    pub fn new(entries: IndexMap<String, ObjectRef>) -> Self {
        Self { ns: Namespace::default(), entries: RwLock::new(entries) }
    }

    pub fn len(&self) -> usize {
        let entries = self.entries.read().unwrap();
        entries.len()
    }

    pub fn is_empty(&self) -> bool {
        let entries = self.entries.read().unwrap();
        entries.is_empty()
    }

    pub fn insert<S: Into<String>>(&self, key: S, val: ObjectRef) {
        let entries = &mut self.entries.write().unwrap();
        entries.insert(key.into(), val);
    }

    pub fn get(&self, name: &str) -> Option<ObjectRef> {
        let entries = self.entries.read().unwrap();
        if let Some(val) = entries.get(name) {
            Some(val.clone())
        } else {
            None
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        let entries = self.entries.read().unwrap();
        entries.contains_key(key)
    }

    pub fn entries(&self) -> &RwLock<IndexMap<String, ObjectRef>> {
        &self.entries
    }
}

impl ObjectTrait for Map {
    object_trait_header!(MAP_TYPE);

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if self.is(rhs) || rhs.is_always() {
            return true;
        }
        if let Some(rhs) = rhs.down_to_map() {
            if self.len() != rhs.len() {
                return false;
            }
            let entries = self.entries.read().unwrap();
            let rhs_entries = rhs.entries.read().unwrap();
            entries.iter().all(|(name, a_ref)| {
                if let Some(b_ref) = rhs_entries.get(name) {
                    let a = a_ref.read().unwrap();
                    let b = b_ref.read().unwrap();
                    a.is_equal(&*b)
                } else {
                    false
                }
            })
        } else {
            false
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let this_id = self.id();
        let entries = self.entries.read().unwrap();
        let entries: Vec<String> = entries
            .iter()
            .map(|(name, val)| {
                let val = val.read().unwrap();
                if val.id() == this_id {
                    "{...}".to_owned()
                } else {
                    format!("{name:?} => {:?}", &*val)
                }
            })
            .collect();
        let string = entries.join(", ");
        write!(f, "{{{string}}}")
    }
}

impl fmt::Debug for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
