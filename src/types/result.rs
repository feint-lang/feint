use crate::vm::RuntimeErr;

use super::object::ObjectRef;

pub type GetAttributeResult = Result<ObjectRef, RuntimeErr>;
pub type SetAttributeResult = Result<(), RuntimeErr>;

pub type Args = Vec<ObjectRef>;
pub type CallResult = Result<Option<ObjectRef>, RuntimeErr>;
