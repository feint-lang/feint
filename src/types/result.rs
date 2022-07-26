use super::object::ObjectRef;
use crate::vm::RuntimeErr;

pub type GetAttributeResult = Result<ObjectRef, RuntimeErr>;
pub type SetAttributeResult = Result<(), RuntimeErr>;

pub type Args = Vec<ObjectRef>;
pub type CallResult = Result<Option<ObjectRef>, RuntimeErr>;
