use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use once_cell::sync::Lazy;

use crate::vm::RuntimeErr;

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Map Type ------------------------------------------------------------

gen::type_and_impls!(MapType, Map);

pub static MAP_TYPE: Lazy<gen::obj_ref_t!(MapType)> = Lazy::new(|| {
    let type_ref = gen::obj_ref!(MapType::new());
    let mut type_obj = type_ref.write().unwrap();

    type_obj.add_attrs(&[
        // Instance Attributes -----------------------------------------
        gen::prop!("length", type_ref, "", |this, _, _| {
            let this = this.read().unwrap();
            let this = this.down_to_map().unwrap();
            Ok(new::int(this.len()))
        }),
        gen::prop!("is_empty", type_ref, "", |this, _, _| {
            let this = this.read().unwrap();
            let this = this.down_to_map().unwrap();
            Ok(new::bool(this.is_empty()))
        }),
        // Instance Methods --------------------------------------------
        gen::meth!(
            "add",
            type_ref,
            &["key", "val"],
            "Add entry to Map.

            # Args

            - key: Str
            - value: Any

            ",
            |this, args, _| {
                let this = this.read().unwrap();
                let this = this.down_to_map().unwrap();
                let arg = gen::use_arg!(args, 0);
                let key = gen::use_arg_str!(get, key, arg);
                let val = args[1].clone();
                this.add(key, val);
                Ok(new::nil())
            }
        ),
        gen::meth!(
            "each",
            type_ref,
            &["each_fn"],
            "Apply function to each Map entry.

            # Args
            
            - func: Func

              A function that will be passed the key and value of each entry in
              turn.

            ```
            → map = {'a': 'a', 'b': 'b'}
            {'a' => 'a', 'b' => 'b'}
            → fn = (k, v) => print($'{k} = {v}')
            function fn/2 @ <id>
            → map.each(fn)
            a = a
            b = b
            ```

            ",
            |this_obj, args, vm| {
                let this = this_obj.read().unwrap();
                let this = this.down_to_map().unwrap();
                let entries = &this.entries.read().unwrap();

                if entries.is_empty() {
                    return Ok(new::nil());
                }

                let each_fn = &args[0];
                let n_args = if let Some(f) = each_fn.read().unwrap().as_func() {
                    if f.has_var_args() {
                        3
                    } else {
                        f.arity()
                    }
                } else {
                    return Ok(new::arg_err(
                        "each/1 expects a function",
                        this_obj.clone(),
                    ));
                };

                for (i, (key, val)) in entries.iter().enumerate() {
                    let each = each_fn.clone();
                    let key = new::str(key);
                    if n_args == 1 {
                        vm.call(each, vec![key])?;
                    } else if n_args == 2 {
                        vm.call(each, vec![key, val.clone()])?;
                    } else {
                        vm.call(each, vec![key, val.clone(), new::int(i)])?;
                    }
                }

                Ok(new::nil())
            }
        ),
        gen::meth!(
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
            |this, args, _| {
                let this = this.read().unwrap();
                let this = this.down_to_map().unwrap();
                let arg = gen::use_arg!(args, 0);
                let key = gen::use_arg_str!(get, key, arg);
                let result = match this.get(key) {
                    Some(obj) => obj,
                    None => new::nil(),
                };
                Ok(result)
            }
        ),
    ]);

    type_ref.clone()
});

// Map Object ----------------------------------------------------------

pub struct Map {
    ns: Namespace,
    entries: RwLock<IndexMap<String, ObjectRef>>,
}

gen::standard_object_impls!(Map);

impl Map {
    pub fn new(entries: IndexMap<String, ObjectRef>) -> Self {
        Self { ns: Namespace::new(), entries: RwLock::new(entries) }
    }

    pub fn len(&self) -> usize {
        let entries = self.entries.read().unwrap();
        entries.len()
    }

    pub fn is_empty(&self) -> bool {
        let entries = self.entries.read().unwrap();
        entries.is_empty()
    }

    pub fn add<S: Into<String>>(&self, key: S, val: ObjectRef) {
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

    pub fn to_hash_map(&self) -> IndexMap<String, ObjectRef> {
        self.entries.read().unwrap().clone()
    }
}

impl ObjectTrait for Map {
    gen::object_trait_header!(MAP_TYPE);

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
