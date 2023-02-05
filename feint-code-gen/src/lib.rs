#[macro_export]
macro_rules! obj_ref_t {
    ( $ty:ty ) => {
        Arc<RwLock<$ty>>
    };
}

#[macro_export]
macro_rules! obj_ref {
    ( $obj:expr ) => {
        Arc::new(RwLock::new($obj))
    };
}

#[macro_export]
macro_rules! std_type {
    ( $name:ident, $class_name:ident ) => {
        pub static $name: Lazy<TypeRef> =
            Lazy::new(|| obj_ref!(Type::new("std", stringify!($class_name))));
    };
}

/// Generate standard obj impls.
#[macro_export]
macro_rules! standard_object_impls {
    ( $name:ident ) => {
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
#[macro_export]
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

        fn ns(&self) -> &Namespace {
            &self.ns
        }

        fn ns_mut(&mut self) -> &mut Namespace {
            &mut self.ns
        }
    };
}

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
#[macro_export]
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
#[macro_export]
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
#[macro_export]
macro_rules! use_arg {
    ( $args:ident, $index:literal ) => {{
        if $index < $args.len() {
            $args[$index].read().unwrap()
        } else {
            let msg =
                format!("{}() didn't receive enough args", stringify!($func_name));
            return new::arg_err(msg, new::nil());
        }
    }};
}

// The use_arg_<type> macros convert the supplied arg to a value of the
// given type if possible or return an Err if not.

#[macro_export]
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
            return new::arg_err(msg, new::nil());
        }
    }};
}

#[macro_export]
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
            return new::arg_err(msg, new::nil());
        }
    }};
}

#[macro_export]
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
                return new::arg_err(msg, new::nil());
            }
        } else {
            let msg =
                format!("{}() didn't receive enough args", stringify!($func_name));
            return new::arg_err(msg, new::nil());
        }
    }};
}
