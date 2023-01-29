macro_rules! obj_ref_t {
    ( $ty:ty ) => {
        Arc<RwLock<$ty>>
    };
}

macro_rules! obj_ref {
    ( $obj:expr ) => {
        Arc::new(RwLock::new($obj))
    };
}

pub(crate) use obj_ref;
pub(crate) use obj_ref_t;

// Types ---------------------------------------------------------------

/// Generate an intrinsic type definition. This includes the type's
/// struct and impl as well as the TypeTrait, Send, and Sync impls.
///
/// Args:
///
/// $type_name: ident
///     The type name. E.g., `NilType`
///
/// $name: ident
///     The type's object name. E.g., `Nil`
macro_rules! type_and_impls {
    ( $type_name:ident, $name:ident ) => {
        pub struct $type_name {
            ns: Namespace,
        }

        unsafe impl Send for $type_name {}
        unsafe impl Sync for $type_name {}

        impl $type_name {
            #[allow(clippy::new_without_default)]
            pub fn new() -> Self {
                let name = new::str(stringify!($name));
                let full_name = new::str(concat!("std.", stringify!($name)));
                Self {
                    ns: Namespace::with_entries(&[
                        ("$name", name),
                        ("$full_name", full_name),
                    ]),
                }
            }

            pub fn with_attrs(attrs: &[(&str, ObjectRef)]) -> Self {
                let mut type_obj = Self::new();
                type_obj.ns.extend(attrs);
                type_obj
            }

            pub fn add_attr(&mut self, name: &str, val: ObjectRef) {
                self.ns.insert(name, val);
            }

            pub fn add_attrs(&mut self, attrs: &[(&str, ObjectRef)]) {
                self.ns.extend(attrs);
            }
        }

        impl TypeTrait for $type_name {
            fn name(&self) -> &str {
                stringify!($name)
            }

            fn full_name(&self) -> &str {
                concat!("std.", stringify!($name))
            }

            fn ns(&self) -> &Namespace {
                &self.ns
            }
        }

        impl ObjectTrait for $type_name {
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

            fn as_type(&self) -> Option<&dyn TypeTrait> {
                Some(self)
            }
        }
    };
}

/// Generate standard obj impls.
macro_rules! standard_object_impls {
    (  $name:ident ) => {
        unsafe impl Send for $name {}
        unsafe impl Sync for $name {}
    };
}

/// Generate `ObjectTrait` header--i.e., the standard implementations of
/// the required `ObjectTrait` methods.
///
/// Args:
///
/// $class: ident
///     The singleton type instance. E.g. `NIL_TYPE`.
macro_rules! object_trait_header {
    ( $class:ident ) => {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn class(&self) -> TypeRef {
            $class.clone()
        }

        fn type_obj(&self) -> ObjectRef {
            $class.clone()
        }

        fn ns(&self) -> &Namespace {
            &self.ns
        }

        fn ns_mut(&mut self) -> &mut Namespace {
            &mut self.ns
        }

        fn as_type(&self) -> Option<&dyn TypeTrait> {
            None
        }
    };
}

pub(crate) use object_trait_header;
pub(crate) use standard_object_impls;
pub(crate) use type_and_impls;

// Methods -------------------------------------------------------------

/// Make a class or instance method for an intrinsic type.
///
/// Args:
///
/// $name: &str
///     The method name.
///
/// $this_type: type
///     Method receiver type (AKA `this`).
///
/// $params:
///     Params The method's parameters.
///
/// $func: fn
///     The function that implements the method. Accepts 3 args: `this`
///     (ObjectRef), `args` (Args), and `vm` (&mut VM).
///
/// This is used for adding methods to intrinsic types. It reduces a bit
/// of tedium in the process of adding methods.
///
/// Note that, in general, both class and instance methods are added to
/// the type, e.g. `IntType` and shared among instance of the type. It's
/// possible to create instance-specific methods, but I'm not sure if
/// that's useful.
///
/// Returns a 2-tuple containing the method name and the intrinsic
/// function object itself. This makes it easy to add the method to the
/// type's namespace by calling `ns.add_obj(meth!(...))`.
macro_rules! meth {
    ( $name:literal, $this_type:expr, $params:expr, $doc:literal, $func:expr ) => {
        (
            $name,
            new::intrinsic_func(
                "std",
                $name,
                Some($this_type.clone()),
                $params,
                $doc,
                $func,
            ),
        )
    };
}

/// This is similar to `meth!` but it creates a property instead of a
/// method and has no `$params` arg.
macro_rules! prop {
    ( $name:literal, $this_type:expr, $doc:literal, $func:expr ) => {
        (
            $name,
            new::prop(new::intrinsic_func(
                "std",
                $name,
                Some($this_type.clone()),
                &[],
                $doc,
                $func,
            )),
        )
    };
}

/// Get arg by index. If the index is out of bounds, return an error.
///
/// Args:
///
/// $args:  Args
/// $index: usize
macro_rules! use_arg {
    ( $args:ident, $index:literal ) => {{
        if $index < $args.len() {
            $args[$index].read().unwrap()
        } else {
            // NOTE: This should never happen from user code.
            return Err(RuntimeErr::index_out_of_bounds("Arg", $index));
        }
    }};
}

// The use_arg_<type> macros convert the supplied arg to a value of the
// given type if possible or return an Err if not.

macro_rules! use_arg_str {
    ( $func_name:ident, $arg_name:ident, $arg:ident ) => {{
        if let Some(val) = $arg.get_str_val() {
            val
        } else {
            let msg = format!(
                "{}() expected {} to be a Str",
                stringify!($func_name),
                stringify!($arg_name)
            );
            return Ok(new::arg_err(msg, new::nil()));
        }
    }};
}

macro_rules! use_arg_map {
    ( $func_name:ident, $arg_name:ident, $arg:ident ) => {{
        if let Some(val) = $arg.get_map_val() {
            val.clone()
        } else {
            let msg = format!(
                "{}() expected {} to be a Map",
                stringify!($func_name),
                stringify!($arg_name)
            );
            return Ok(new::arg_err(msg, new::nil()));
        }
    }};
}

macro_rules! use_arg_usize {
    ( $func_name:ident, $arg_name:ident, $args:ident, $index:literal ) => {{
        if $index < $args.len() {
            let arg = $args[$index].read().unwrap();
            if let Some(val) = arg.get_usize_val() {
                val
            } else {
                let msg = format!(
                    "{}() expected {} to be an index (usize)",
                    stringify!($func_name),
                    stringify!($arg_name)
                );
                return Ok(new::arg_err(msg, new::nil()));
            }
        } else {
            // NOTE: This should never happen from user code.
            let msg =
                format!("{}() didn't receive enough args", stringify!($func_name));
            return Err(RuntimeErr::index_out_of_bounds(msg, $index));
        }
    }};
}

pub(crate) use meth;
pub(crate) use prop;
pub(crate) use use_arg;
pub(crate) use use_arg_map;
pub(crate) use use_arg_str;
pub(crate) use use_arg_usize;
