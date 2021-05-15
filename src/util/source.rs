use std::collections::VecDeque;
use std::fmt;
use std::io::BufRead;

/// Maximum line length in chars. This is used to set the capacity for
/// the source's char queue up front to avoid allocations.
const CAPACITY: usize = 255; // 2^8 - 1

/// A wrapper around some source, typically either some text or a file.
/// The source is read line by line and the characters from each line
/// are yielded (so to speak) in turn. Other features:
///
/// - Emits an initial newline to prime the queue and allow for
///   consistent start-of-line handling/logic
/// - Newlines are normalized (\r\n will be converted to \n)
/// - The current line and column in the source are tracked
/// - The previous and current characters are tracked
pub struct Source<T>
where
    T: BufRead,
{
    source: T,
    /// String buffer the source reader reads into.
    buffer: String,
    /// The queue of characters for the current line.
    queue: VecDeque<char>,
    pub line: usize,
    pub col: usize,
    pub previous_char: Option<char>,
    pub current_char: Option<char>,
}

impl<T> Source<T>
where
    T: BufRead,
{
    pub fn new(source: T) -> Self {
        let mut source = Source {
            source,
            buffer: String::with_capacity(CAPACITY),
            queue: VecDeque::with_capacity(CAPACITY),
            line: 0,
            col: 0,
            previous_char: None,
            current_char: None,
        };
        source.queue.push_back('\n');
        source
    }

    fn check_queue(&mut self) -> bool {
        let queue = &mut self.queue;
        if queue.is_empty() {
            // See if character queue can be refilled from the next line.
            let buffer = &mut self.buffer;
            buffer.clear();
            match self.source.read_line(buffer) {
                Ok(0) => {
                    return false;
                }
                Ok(_) => {
                    queue.extend(buffer.chars());
                    self.line += 1;
                    self.col = 1;
                    let len = queue.len();
                    if len > 1 {
                        let (i, j) = (len - 2, len - 1);
                        if queue[i] == '\r' && queue[j] == '\n' {
                            queue.remove(i);
                        }
                    }
                }
                Err(err) => {
                    panic!("Could not read line from source: {}", err);
                }
            };
        }
        // Queue wasn't empty or was refilled.
        true
    }

    fn next_from_queue(&mut self) -> Option<char> {
        if self.check_queue() {
            if let Some(c) = self.queue.pop_front() {
                self.col += 1;
                self.previous_char = self.current_char;
                self.current_char = Some(c);
                return self.current_char;
            }
        }
        None
    }

    /// Peek at the next char.
    pub fn peek(&mut self) -> Option<&char> {
        if self.check_queue() {
            return self.queue.front();
        }
        None
    }

    /// Peek at the next two chars.
    pub fn peek_2(&mut self) -> (Option<&char>, Option<&char>) {
        if self.check_queue() {
            let queue = &self.queue;
            return (queue.get(0), queue.get(1));
        }
        (None, None)
    }

    /// Peek at the next three chars.
    pub fn peek_3(&mut self) -> (Option<&char>, Option<&char>, Option<&char>) {
        if self.check_queue() {
            let queue = &self.queue;
            return (queue.get(0), queue.get(1), queue.get(2));
        }
        (None, None, None)
    }

    /// Get the next char if it matches the specified condition.
    pub fn next_if(&mut self, func: impl FnOnce(&char) -> bool) -> Option<char> {
        if let Some(c) = self.peek() {
            if func(c) {
                return self.next();
            }
        }
        None
    }

    pub fn location(&self) -> Location {
        Location::new(self.line, self.col)
    }
}

impl<T> Iterator for Source<T>
where
    T: BufRead,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_from_queue()
    }
}

/// Represents a line and column in the source.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Location {
    pub line: usize,
    pub col: usize,
}

impl Location {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

mod tests {
    #[test]
    fn source_from_text() {}
}
