/// Format docstring. The assumption is that docstrings are written in
/// the following format:
///
/// ```ignore
/// f = (x: Int) =>
///     "This is a function that does stuff.
///
///     Args:
///         x: A number for controlling how the stuff is done
///
///     "
/// ```
///
/// Leading and trailing blank lines will be removed and the content
/// will be dedented such that the output will be formatted like this:
///
/// ```ignore
/// This is a function that does stuff.
///
/// Args:
///     x: A number for controlling how the stuff is done
/// ```
pub fn format_doc(string: &str) -> String {
    let string = string.trim();
    let mut new_string = String::with_capacity(string.len());
    let mut shortest_prefix = usize::MAX;

    // Find the shortest prefix
    for line in string.lines().skip(1) {
        let trimmed_line = line.trim_start();
        if !trimmed_line.is_empty() {
            let prefix_len = line.len() - trimmed_line.len();
            if prefix_len < shortest_prefix {
                shortest_prefix = prefix_len;
            }
        }
    }

    if shortest_prefix == usize::MAX {
        shortest_prefix = 0;
    }

    let prefix = " ".repeat(shortest_prefix);

    // Trim the shortest prefix from all non-empty lines.
    for line in string.lines() {
        if line.trim().is_empty() {
            new_string.push('\n');
        } else {
            if let Some(line) = line.strip_prefix(&prefix) {
                new_string.push_str(line);
            } else {
                new_string.push_str(line);
            }
            new_string.push('\n');
        }
    }

    new_string
}
