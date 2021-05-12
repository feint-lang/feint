type ExitData = (i32, String);

/// Result type used by top level runners.
///
/// On success, Ok(None) or OK(Some(message: String)) should be
/// returned. In both cases, the program will exit with error code 0. In
/// the latter case, the specified message will be printed to stdout
/// just before exiting.
///
/// On error, Err((code: i32, message: String)) should be returned. Note
/// that on error, a message is *always* required. The program will
/// print the specified message to stderr and then exit with the
/// specified error code.
pub(crate) type ExitResult = Result<Option<String>, ExitData>;
