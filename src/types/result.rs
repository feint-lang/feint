use crate::vm::RuntimeErr;

use super::base::ObjectRef;

pub type GetAttrResult = Result<ObjectRef, RuntimeErr>;
pub type SetAttrResult = Result<(), RuntimeErr>;

// TODO: Move call-related types elsewhere
pub type ThisOpt = Option<ObjectRef>;
pub type Params = Vec<String>;
pub type Args = Vec<ObjectRef>;
pub type CallResult = Result<ObjectRef, RuntimeErr>;
