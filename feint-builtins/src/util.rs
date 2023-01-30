use crate::types::{new, Args, ObjectRef};

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
/// If the check is successful, a tuple with the following items:
///
/// - total number of args
/// - number of var args
/// - vargs
///
/// If the check is *not* successful, an error object.
///
/// NOTE: This returns a `Result` so that it's easy to tell when the
///       check succeeds or fails. This result should always be checked,
///       and if it's an `Err`, it should be unwrapped and returned:
///
/// ```rust, ignore
/// let result = check_args("name", &[], false, 1, None);
///
/// if let Err(err) = result {
///     return Ok(err);
/// }
///
/// let (n_args, n_var_args, var_args) = result.unwrap();
/// ```
pub(crate) fn check_args(
    name: &str,
    args: &Args,
    has_var_args: bool,
    min: usize,
    max: Option<usize>,
) -> Result<(usize, usize, ObjectRef), ObjectRef> {
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
        return Err(new::arg_err(msg, new::nil()));
    }

    Ok((n_args, n_var_args, var_args))
}
