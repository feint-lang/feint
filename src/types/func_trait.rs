use std::fmt;

use crate::types::{Namespace, ObjectRef};

use super::result::Params;

// Function Trait ------------------------------------------------------

pub trait FuncTrait {
    fn ns(&self) -> &Namespace;
    fn module_name(&self) -> &String;
    fn module(&self) -> ObjectRef;
    fn name(&self) -> &String;
    fn params(&self) -> &Params;

    fn get_doc(&self) -> ObjectRef {
        self.ns().get_obj("$doc").unwrap().clone()
    }

    /// Returns the required number of args.
    fn arity(&self) -> usize {
        let params = self.params();
        if let Some(name) = params.last() {
            if name.is_empty() {
                // Has var args; return number of required args
                params.len() - 1
            } else {
                // Does not have var args; all args required
                params.len()
            }
        } else {
            0
        }
    }

    /// If the function has var args, this returns the index of the var
    /// args in the args list (which is also equal to the required
    /// number of args).
    fn var_args_index(&self) -> Option<usize> {
        let params = self.params();
        if let Some(name) = params.last() {
            if name.is_empty() {
                return Some(params.len() - 1);
            }
        }
        None
    }

    fn has_var_args(&self) -> bool {
        self.var_args_index().is_some()
    }

    fn format_string(&self, id: Option<usize>) -> String {
        let name = &self.name();
        let arity = self.arity();
        let suffix = if self.var_args_index().is_some() { "+" } else { "" };
        let id = id.map_or_else(|| "".to_string(), |id| format!(" @ {id}"));
        format!("function {name}/{arity}{suffix}{id}")
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for dyn FuncTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_string(None))
    }
}

impl fmt::Debug for dyn FuncTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
