use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Take};
use std::{fmt, io};

/// This is used to set the initial capacity for the source's char
/// queue up front to avoid allocations. It assumes reasonable line
/// lengths are in use plus some additional space for end-of-line
/// comments, etc. 2^8 - 1 is used because using 2^8 will cause the
/// queue's initial capacity to be doubled.
const INITIAL_CAPACITY: usize = 255; // 2^8 - 1

/// Maximum length of a line in bytes. This keeps malicious input from
/// causing issues (e.g., extremely long lines or no EOF).
///
/// TODO: Determine an optimal max line length. It could probably be
///       quite a bit larger before causing any issues. The only place
///       this is likely to be relevant is when a user has a *really*
///       long literal string with no newlines that they for some reason
///       want to store in their source code.
const MAX_LINE_LENGTH: u64 = 4096; // 2^12
const MAX_LINE_LENGTH_USIZE: usize = MAX_LINE_LENGTH as usize;

/// Create source from the specified file.
pub fn source_from_file(file_path: &str) -> Result<Source<BufReader<File>>, io::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let source = Source::new(reader);
    Ok(source)
}

/// Create source from the specified text.
pub fn source_from_text(text: &str) -> Source<Cursor<&str>> {
    let cursor = Cursor::new(text);
    let source = Source::new(cursor);
    source
}

/// Create source from stdin.
pub fn source_from_stdin() -> Source<BufReader<io::Stdin>> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let source = Source::new(reader);
    source
}

/// A wrapper around some source, typically either some text or a file.
/// The source is read line by line and the characters from each line
/// are yielded (so to speak) in turn. Other features:
///
/// - Emits an initial newline to prime the queue and allow for
///   consistent start-of-line handling/logic.
/// - Normalizes \r\n line endings to \n. NOTE: \r as a line ending
///   is *not* handled. TODO: Detect use of \r as line ending?
/// - Tracks current line and column.
/// - Tracks previous and current characters.
/// - Panics when lines are too long.
pub struct Source<T: BufRead> {
    stream: Take<T>,
    /// String buffer the source reader reads into.
    buffer: String,
    /// The queue of characters for the current line.
    queue: VecDeque<char>,
    pub line_no: usize,
    pub col: usize,
    pub previous_char: Option<char>,
    pub current_char: Option<char>,
    pub current_line: Option<String>,
}

impl<T: BufRead> Source<T> {
    pub fn new(source: T) -> Self {
        let mut source = Source {
            stream: source.take(MAX_LINE_LENGTH + 1),
            buffer: String::with_capacity(INITIAL_CAPACITY),
            queue: VecDeque::with_capacity(INITIAL_CAPACITY),
            line_no: 0,
            col: 0,
            previous_char: None,
            current_char: None,
            current_line: None,
        };
        source.queue.push_back('\n');
        source
    }

    pub fn get_current_line(&self) -> Option<&str> {
        if let Some(line) = &self.current_line {
            Some(line.as_str())
        } else {
            None
        }
    }

    fn check_queue(&mut self) -> bool {
        if self.queue.is_empty() {
            // See if character queue can be refilled from the next line.
            self.buffer.clear();
            match self.stream.read_line(&mut self.buffer) {
                Ok(0) => {
                    // All lines read; done.
                    return false;
                }
                Ok(n) => {
                    if n > MAX_LINE_LENGTH_USIZE {
                        panic!("Line is too long (> {})", MAX_LINE_LENGTH);
                    }
                    self.line_no += 1;
                    self.col = 1;
                    self.current_line = Some(self.buffer.clone());
                    self.queue.extend(self.buffer.chars());
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
                if c == '\r' {
                    if let Some('\n') = self.queue.front() {
                        self.queue.pop_front();
                        self.current_char = Some('\n');
                    }
                } else {
                    self.current_char = Some(c);
                }
                self.col += 1;
                self.previous_char = self.current_char;
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

    pub fn loc(&self) -> Location {
        Location::new(self.line_no, self.col)
    }
}

impl<T: BufRead> Iterator for Source<T> {
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
