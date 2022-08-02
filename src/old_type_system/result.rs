use crate::vm::RuntimeErr;

use super::object::ObjectRef;

pub type GetAttrResult = Result<ObjectRef, RuntimeErr>;
pub type SetAttrResult = Result<(), RuntimeErr>;

// TODO: Move call-related types elsewhere
pub type Params = Option<Vec<String>>;
pub type Args = Vec<ObjectRef>;
pub type CallResult = Result<Option<ObjectRef>, RuntimeErr>;
