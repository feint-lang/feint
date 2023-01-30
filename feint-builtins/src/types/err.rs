//! # Error Type
//!
//! The error type represents _recoverable_ runtime errors that can be
//! checked in user code using this pattern:
//!
//! result = assert(false)
//! if result.err ->
//!     # Handle `result` as an error
//!     print(result)
//!
//! _All_ objects respond to `err`, which returns either an `Err` object
//! or `nil`. `Err` objects evaluate as `false` in a boolean context:
//!
//! if !assert(false) ->
//!     print("false is not true")
use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use feint_code_gen::*;

use crate::util::check_args;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::err_type::ErrKind;
use super::new;
use super::ns::Namespace;

// Err Type ------------------------------------------------------------

type_and_impls!(ErrType, Err);

pub static ERR_TYPE: Lazy<obj_ref_t!(ErrType)> = Lazy::new(|| {
    let type_ref = obj_ref!(ErrType::new());
    let mut type_obj = type_ref.write().unwrap();

    type_obj.add_attrs(&[
        // Class Methods -----------------------------------------------
        meth!("new", type_ref, &["type", "msg"], "", |_, args| {
            let name = "Err.new()";

            let result = check_args(name, &args, false, 2, Some(2));
            if let Err(err) = result {
                return err;
            }

            let type_arg = use_arg!(args, 0);
            let msg_arg = use_arg!(args, 1);

            let err_type = if let Some(err_type) = type_arg.down_to_err_type_obj() {
                err_type
            } else {
                let arg_err_msg = format!("{name} expected type to be an ErrType");
                // NOTE: This is problematic because user code won't be
                //       able to tell if the arg error was the result of
                //       creating an arg err explicitly or the result of
                //       an internal error. Note that this applies to
                //       *any* user-constructible error.
                //
                // TODO: Figure out a solution for this, perhaps an err
                //       type that is *not* user-constructible or a
                //       nested err type?
                return new::arg_err(arg_err_msg, new::nil());
            };

            let kind = err_type.kind().clone();

            let msg = if let Some(msg) = msg_arg.get_str_val() {
                msg
            } else {
                let arg_err_msg = format!("{name} expected message to be a Str");
                return new::arg_err(arg_err_msg, new::nil());
            };

            new::err(kind, msg, new::nil())
        }),
        // Instance Attributes -----------------------------------------
        prop!("type", type_ref, "", |this, _| {
            let this = this.read().unwrap();
            let this = this.down_to_err().unwrap();
            this.kind.get_obj().unwrap()
        }),
        prop!("message", type_ref, "", |this, _| {
            let this = this.read().unwrap();
            let this = this.down_to_err().unwrap();
            new::str(&this.message)
        }),
    ]);

    type_ref.clone()
});

// Error Object --------------------------------------------------------

// NOTE: This is named `ErrObj` instead of `Err` to avoid conflict with
//       Rust's `Err`.
pub struct ErrObj {
    ns: Namespace,
    pub kind: ErrKind,
    pub message: String,
    pub obj: ObjectRef,
    bool_val: bool,
    responds_to_bool: bool,
}

standard_object_impls!(ErrObj);

impl ErrObj {
    pub fn new(kind: ErrKind, message: String, obj: ObjectRef) -> Self {
        let bool_val = kind != ErrKind::Ok;
        Self {
            ns: Namespace::default(),
            kind,
            message,
            obj,
            bool_val,
            responds_to_bool: false,
        }
    }

    pub fn with_responds_to_bool(
        kind: ErrKind,
        message: String,
        obj: ObjectRef,
    ) -> Self {
        let mut instance = Self::new(kind, message, obj);
        instance.responds_to_bool = true;
        instance
    }

    pub fn retrieve_bool_val(&self) -> bool {
        self.bool_val
    }
}

impl ObjectTrait for ErrObj {
    object_trait_header!(ERR_TYPE);

    fn bool_val(&self) -> Option<bool> {
        if self.responds_to_bool {
            Some(self.bool_val)
        } else {
            // Err(RuntimeErr::type_err(concat!(
            //     "An Err object cannot be evaluated directly as a ",
            //     "Bool. You must access it via the `.err` attribute of ",
            //     "the result object.",
            // )))
            None
        }
    }

    fn and(&self, rhs: &dyn ObjectTrait) -> Option<bool> {
        let lhs = self.bool_val()?;
        let rhs = rhs.bool_val()?;
        Some(lhs && rhs)
    }

    fn or(&self, rhs: &dyn ObjectTrait) -> Option<bool> {
        let lhs = self.bool_val()?;
        let rhs = rhs.bool_val()?;
        Some(lhs && rhs)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for ErrObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = &self.kind;
        let msg = &self.message;
        if self.message.is_empty() {
            write!(f, "{} [{}]", kind, kind.name())
        } else {
            write!(f, "[{}] {}: {}", kind.name(), kind, msg)
        }
    }
}

impl fmt::Debug for ErrObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
