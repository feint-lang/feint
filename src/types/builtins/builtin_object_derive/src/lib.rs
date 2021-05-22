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
        use std::sync::Arc;

        use super::super::object::Object;
        use super::super::class::Type;

        use super::BUILTIN_TYPES;

        impl Object for #name {
            fn class(&self) -> &Arc<Type> {
                BUILTIN_TYPES.get(stringify!(#name))
            }

            fn as_any(&self) -> &dyn Any {
                self
            }
        }
    };
    gen.into()
}
