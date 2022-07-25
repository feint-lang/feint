use crate::types::ObjectRef;
use crate::vm::RuntimeErr;

pub(crate) type CallResult = Result<Option<ObjectRef>, RuntimeErr>;
