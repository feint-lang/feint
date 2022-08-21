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
///
/// $name: ident
///     The object name. E.g., `Nil`
macro_rules! object_trait_header {
    ( $class:ident, $name:ident ) => {
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

pub(crate) use object_trait_header;
pub(crate) use standard_object_impls;
pub(crate) use type_and_impls;
