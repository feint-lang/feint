/// Make a class or instance method for a builtin type.
///
/// Args:
///
/// $ty:     type   The type the method belongs. To create a class
///                 method, use a type like `IntType`. To create and
///                 instance method, use a type like `Int`.
/// $name:   ident  The method name as an identifier (no quotes).
/// $params: Params The method's parameters.
/// $func:   fn     The function that implements the method. Accepts
///                 3 args: `this` (ObjectRef), `args` (Args), and
///                 `vm` (&mut VM).
///
/// This is used for adding methods to builtin types. It checks to
/// ensure the method is called with a receiver (AKA `this`), that the
/// receiver is of the correct type, and that the correct number of args
/// were passed.
///
/// Note that, in general, both class and instance methods are added to
/// the type impl, e.g. `IntType` and shared among instance of the type.
/// It's possible to create instance-specific methods, but I'm not sure
/// if that's useful.
///
/// Returns a 2-tuple containing the method name and the function object
/// itself. This makes it easy to add the method to the type's namespace
/// by calling `ns.add_entry(make_meth!(...))`.
macro_rules! make_meth {
    ( $ty:ty, $name:ident, $params:expr, $func:expr ) => {(
        stringify!($name),

        create::new_builtin_func(
            stringify!($name), $params,|this_opt: This, args: Args, vm: &mut VM| {
                if this_opt.is_none() {
                    let msg = format!(
                        "Method {}.{}() expected receiver",
                        stringify!($ty),
                        stringify!($name),
                    );
                    return Err(RuntimeErr::new_type_err(msg));
                }

                let this_ref = this_opt.unwrap();
                let this = this_ref.read().unwrap();

                // XXX: This isn't great
                let this_is_correct_type = match stringify!($ty) {
                    // For class methods
                    "TypeType" => this.is_type_type(),
                    "BoolType" => this.is_bool_type(),
                    "BoundFuncType" => this.is_bound_func_type(),
                    "BuiltinFuncType" => this.is_builtin_func_type(),
                    "ClosureType" => this.is_closure_type(),
                    "FloatType" => this.is_float_type(),
                    "FuncType" => this.is_func_type(),
                    "IntType" => this.is_int_type(),
                    "ListType" => this.is_list_type(),
                    "ModuleType" => this.is_mod_type(),
                    "NilType" => this.is_nil_type(),
                    "StrType" => this.is_str_type(),
                    "TupleType" => this.is_tuple_type(),

                    // For instance methods
                    "Type" => this.is_type(),
                    "Bool" => this.is_bool(),
                    "BoundFunc" => this.is_bound_func(),
                    "BuiltinFunc" => this.is_builtin_func(),
                    "Closure" => this.is_closure(),
                    "Float" => this.is_float(),
                    "Func" => this.is_func(),
                    "Int" => this.is_int(),
                    "List" => this.is_list(),
                    "Module" => this.is_mod(),
                    "Nil" => this.is_nil(),
                    "Str" => this.is_str(),
                    "Tuple" => this.is_tuple(),

                    _ => panic!("Unknown builtin type: {}", stringify!($ty)),
                };

                if !this_is_correct_type {
                    let msg = format!(
                        "Method {}.{}() expected receiver to be type {}; got {:?}",
                        stringify!($ty),
                        stringify!($name),
                        stringify!($ty),
                        &*this.class().read().unwrap(),
                    );
                    return Err(RuntimeErr::new_type_err(msg));
                }

                $func(this_ref.clone(), args, vm)
            }
        )
    )};
}

/// Get `this` value (by locking the object ref).
///
/// Args:
///
/// $this: ObjectRef
macro_rules! use_this {
    ( $this:ident ) => {{
        $this.read().unwrap()
    }};
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
            return Err(RuntimeErr::new_index_out_of_bounds("Arg", $index));
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
            return Err(RuntimeErr::new_type_err(msg));
        }
    }};
}

macro_rules! use_arg_usize {
    ( $arg:ident ) => {{
        if let Some(val) = $arg.get_usize_val() {
            val
        } else {
            let msg = format!("Expected index; got {}", $arg.class().read().unwrap());
            return Err(RuntimeErr::new_type_err(msg));
        }
    }};
}

pub(crate) use make_meth;
pub(crate) use use_arg;
pub(crate) use use_arg_str;
pub(crate) use use_arg_usize;
pub(crate) use use_this;
