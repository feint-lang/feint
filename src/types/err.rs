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

use crate::util::check_args;
use crate::vm::{RuntimeBoolResult, RuntimeErr};

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::err_type::ErrKind;
use super::ns::Namespace;

// Err Type ------------------------------------------------------------

gen::type_and_impls!(ErrType, Err);

pub static ERR_TYPE: Lazy<new::obj_ref_t!(ErrType)> = Lazy::new(|| {
    let type_ref = new::obj_ref!(ErrType::new());
    let mut class = type_ref.write().unwrap();

    class.ns_mut().add_entries(&[
        // Class Methods -----------------------------------------------
        gen::meth!("new", type_ref, &["type", "msg"], |_, args, _| {
            let name = "Err.new()";
            check_args(name, &args, false, 2, Some(2))?;
            let type_arg = gen::use_arg!(args, 0);
            let msg_arg = gen::use_arg!(args, 1);

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
                return Ok(new::arg_err(arg_err_msg));
            };

            let kind = if let Some(kind) = err_type.kind() {
                kind
            } else {
                let arg_err_msg = format!("{name} got unexpected err type: {type_arg}");
                return Ok(new::arg_err(arg_err_msg));
            };

            let msg = if let Some(msg) = msg_arg.get_str_val() {
                msg
            } else {
                let arg_err_msg = format!("{name} expected message to be a Str");
                return Ok(new::arg_err(arg_err_msg));
            };

            Ok(new::err(kind, msg))
        }),
        // Instance Attributes -----------------------------------------
        gen::prop!("type", type_ref, |this, _, _| {
            let this = this.read().unwrap();
            let this = this.down_to_err().unwrap();
            Ok(this.kind.get_obj().unwrap())
        }),
    ]);

    type_ref.clone()
});

// Error Object --------------------------------------------------------

// NOTE: This is named `ErrObj` instead of `Err` to avoid conflict with
//       Rust's `Err`.
pub struct ErrObj {
    ns: Namespace,
    inverted_bool_val: bool,
    pub kind: ErrKind,
    pub message: String,
}

gen::standard_object_impls!(ErrObj);

impl ErrObj {
    pub fn new(kind: ErrKind, message: String) -> Self {
        Self { ns: Namespace::new(), kind, message, inverted_bool_val: false }
    }

    pub fn with_inverted_bool_val(kind: ErrKind, message: String) -> Self {
        Self { ns: Namespace::new(), kind, message, inverted_bool_val: true }
    }
}

impl ObjectTrait for ErrObj {
    gen::object_trait_header!(ERR_TYPE);

    /// `Err` object's evaluate to `false` in boolean context *except*
    /// for the special OK value, which evaluates to `true`.
    ///
    /// When `self.invert_err_arg` is set, the boolean semantics are
    /// inverted.
    fn bool_val(&self) -> RuntimeBoolResult {
        let mut val = self.kind == ErrKind::Ok;
        if self.inverted_bool_val {
            val = !val
        }
        Ok(val)
    }

    fn and(&self, rhs: &dyn ObjectTrait) -> RuntimeBoolResult {
        let lhs = self.bool_val()?;
        let rhs = rhs.bool_val()?;
        Ok(lhs && rhs)
    }

    fn or(&self, rhs: &dyn ObjectTrait) -> RuntimeBoolResult {
        let lhs = self.bool_val()?;
        let rhs = rhs.bool_val()?;
        Ok(lhs || rhs)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for ErrObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.kind == ErrKind::Ok {
            write!(f, "{}", self.kind)
        } else if self.message.is_empty() {
            write!(f, "ERROR: {}", self.kind)
        } else {
            write!(f, "ERROR: {}: {}", self.kind, self.message)
        }
    }
}

impl fmt::Debug for ErrObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
