//! A simple derive macro that implements the Object trait on builtin
//! object types. Used like:
//!
//!     use builtin_object_derive::BuiltinObject;
//!
//!     #[derive(Debug, PartialEq, BuiltinObject)]
//!     pub struct Bool {
//!         value: bool,
//!     }

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(BuiltinObject)]
pub fn builtin_object_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_builtin_object_derive(&ast)
}

fn impl_builtin_object_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        use std::any::Any;

        impl Object for #name {
            fn class(&self) -> &TypeRef {
                &self.class
            }

            fn as_any(&self) -> &dyn Any {
                self
            }

            fn is_equal(&self, rhs: ObjectRef, _ctx: &RuntimeContext) -> Result<bool, RuntimeError> {
                if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
                    Ok(self.is(rhs) || self == rhs)
                } else {
                    Err(RuntimeError::new_type_error(format!(
                        "Could not compare {} to {}",
                        self.class().name(), rhs.class().name()
                    )))
                }
            }
        }
    };
    gen.into()
}
