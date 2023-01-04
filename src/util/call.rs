use crate::types::{new, Args, ObjectRef};
use crate::vm::RuntimeErr;

/// Check args and return info.
///
/// # Args
///
/// - function name
/// - minimum number of args
/// - maximum number of args (`None` indicates no max)
/// - whether the function has var args
/// - the args that were passed
///
/// # Returns
///
/// A tuple with the following items:
///
/// - this
/// - total number of args
/// - number of var args
/// - vargs
pub(crate) fn check_args(
    name: &str,
    args: &Args,
    has_var_args: bool,
    min: usize,
    max: Option<usize>,
) -> Result<(usize, usize, ObjectRef), RuntimeErr> {
    let mut n_args = args.len();

    let (n_var_args, var_args) = if has_var_args {
        let var_args_ref = args.last().unwrap();
        let var_args_guard = var_args_ref.read().unwrap();
        let var_args = var_args_guard.down_to_tuple().unwrap();
        let n_var_args = var_args.len();
        n_args = n_args - 1 + n_var_args;
        (n_var_args, var_args_ref.clone())
    } else {
        (0, new::empty_tuple())
    };

    // NOTE: Slightly hacky, but it's extremely unlikely anyone would
    //       ever create a function with billions of arguments, and this
    //       makes the flow below simpler.
    let max = max.unwrap_or(usize::MAX);

    if n_args < min {
        let msg = if min == max {
            let ess = if min == 1 { "" } else { "s" };
            format!("{name} expected {min} arg{ess}; got {n_args}")
        } else {
            format!("{name} expected {min} to {max} args; got {n_args}")
        };
        return Err(RuntimeErr::arg_err(msg));
    }

    Ok((n_args, n_var_args, var_args))
}
