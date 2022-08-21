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
/// type's namespace by calling `ns.add_entry(make_meth!(...))`.
macro_rules! make_meth {
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

pub(crate) use make_meth;
pub(crate) use use_arg;
pub(crate) use use_arg_str;
pub(crate) use use_arg_usize;
