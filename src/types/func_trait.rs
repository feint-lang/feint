use std::fmt;

use super::result::Params;

// Function Trait ------------------------------------------------------

pub(crate) trait FuncTrait {
    fn name(&self) -> &str;
    fn params(&self) -> &Params;

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

    fn var_args_index(&self) -> Option<usize> {
        let params = self.params();
        if let Some(name) = params.last() {
            if name.is_empty() {
                return Some(params.len() - 1);
            }
        }
        None
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
