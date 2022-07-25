use super::object::ObjectRef;
use crate::vm::RuntimeErr;

pub type CallResult = Result<Option<ObjectRef>, RuntimeErr>;
