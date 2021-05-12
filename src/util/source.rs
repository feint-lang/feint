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
/// - The current line and column in the source are tracked
/// - Start-of-line state is tracked (true initially and when the end of
///   a line is reached; false otherwise)
/// - The previous and current characters are tracked
/// - Newlines are normalized (\r\n will be converted to \n)
pub struct Source<T> {
    source: T,
    /// The queue of characters for the current line.
    queue: VecDeque<char>,
    pub line: usize,
    pub col: usize,
    pub at_start_of_line: bool,
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
            queue: VecDeque::with_capacity(CAPACITY),
            line: 0,
            col: 0,
            at_start_of_line: true,
            previous_char: None,
            current_char: None,
        };
        source.check_queue();
        source
    }

    fn check_queue(&mut self) -> bool {
        if self.queue.is_empty() {
            // See if character queue can be refilled from the next line.
            let mut line = String::new();
            match self.source.read_line(&mut line) {
                // No more lines; done.
                Ok(0) => {
                    self.at_start_of_line = false;
                    return false;
                }
                Ok(_) => {
                    self.line += 1;
                    self.col = 1;
                    self.at_start_of_line = true;
                    self.queue.extend(line.chars());
                }
                Err(err) => {
                    // Panicking seems wonky, but if the source can't be
                    // read, I'm pretty sure that's unrecoverable.
                    panic!("Could not read line from source: {}", err);
                }
            };
            if self.queue.len() > 1 {
                // Normalize \r\n to \n
                let i = self.queue.len() - 2;
                if self.queue[i] == '\r' && self.queue[i + 1] == '\n' {
                    self.queue.remove(i);
                }
            }
        }
        // Queue wasn't empty or was refilled.
        true
    }

    fn next_from_queue(&mut self) -> Option<char> {
        if self.check_queue() {
            if let Some(c) = self.queue.pop_front() {
                self.col += 1;
                self.at_start_of_line = false;
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
