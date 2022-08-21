/// Generate a builtin type definition. This includes the type's
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
            pub fn new() -> Self {
                Self {
                    ns: Namespace::with_entries(&[
                        // Class Attributes
                        ("$name", new::str(stringify!($name))),
                        (
                            "$full_name",
                            new::str(concat!("builtins.", stringify!($name))),
                        ),
                    ]),
                }
            }
        }

        impl TypeTrait for $type_name {
            fn name(&self) -> &str {
                stringify!($name)
            }

            fn full_name(&self) -> &str {
                concat!("builtins.", stringify!($name))
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
    };
}

/// Make a class or instance method for a builtin type.
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
/// This is used for adding methods to builtin types. It reduces a bit
/// of tedium in the process of adding methods.
///
/// Note that, in general, both class and instance methods are added to
/// the type, e.g. `IntType` and shared among instance of the type. It's
/// possible to create instance-specific methods, but I'm not sure if
/// that's useful.
///
/// Returns a 2-tuple containing the method name and the builtin
/// function object itself. This makes it easy to add the method to the
/// type's namespace by calling `ns.add_entry(meth!(...))`.
macro_rules! meth {
    ( $name:literal, $this_type:expr, $params:expr, $func:expr ) => {
        ($name, new::builtin_func($name, Some($this_type.clone()), $params, $func))
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

/// Get the value of the arg if it's a Str or return a type error.
///
/// Args:
///
/// $arg: ObjectRef
macro_rules! use_arg_str {
    ( $arg:ident ) => {{
        if let Some(val) = $arg.get_str_val() {
            val
        } else {
            let msg = format!("Expected string; got {}", $arg.class().read().unwrap());
            return Err(RuntimeErr::type_err(msg));
        }
    }};
}

macro_rules! use_arg_usize {
    ( $arg:ident ) => {{
        if let Some(val) = $arg.get_usize_val() {
            val
        } else {
            let msg = format!("Expected index; got {}", $arg.class().read().unwrap());
            return Err(RuntimeErr::type_err(msg));
        }
    }};
}

pub(crate) use object_trait_header;
pub(crate) use standard_object_impls;
pub(crate) use type_and_impls;

pub(crate) use meth;
pub(crate) use use_arg;
pub(crate) use use_arg_str;
pub(crate) use use_arg_usize;
