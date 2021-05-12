use std::collections::VecDeque;
use std::io::BufRead;
use std::iter::Peekable;
use std::str::Chars;

/// A wrapper around some source, typically either some text or a file.
/// The source is read line by line and the characters from each line
/// are yielded (so to speak) in turn. Other features:
///
/// - The current line and column in the source are tracked
/// - The previous and current characters are tracked
/// - Newlines are normalized (\r\n will be converted to \n)
///
/// IMPLEMENTATION NOTE: The chars for each line are collected into a
/// temporary vector in order to get around borrowing/lifetime issues
/// with trying to store the chars iter for the current line on self.
/// This seems slightly inefficient.
pub struct Source<T> {
    source: T,
    /// The queue of characters for the current line.
    char_queue: VecDeque<char>,
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
        Self {
            source,
            char_queue: VecDeque::new(),
            line: 0,
            col: 0,
            previous_char: None,
            current_char: None,
        }
    }

    fn check_queue(&mut self) -> bool {
        if self.char_queue.is_empty() {
            // See if character queue can be refilled from the next line.
            let mut line = String::new();
            let mut queue: VecDeque<char> = match self.source.read_line(&mut line) {
                // No more lines; done.
                Ok(0) => return false,
                Ok(_) => {
                    self.line += 1;
                    self.col = 1;
                    line.chars().collect()
                }
                Err(err) => panic!("Could not read line from source: {}", err),
            };
            if queue.len() > 1 {
                // Normalize \r\n to \n
                let i = queue.len() - 2;
                if let (Some('\r'), Some('\n')) = (queue.get(i), queue.back()) {
                    queue.remove(i);
                }
            }
            self.char_queue = queue;
        }
        // Queue wasn't empty or was refilled.
        true
    }

    fn next_from_queue(&mut self) -> Option<char> {
        if self.check_queue() {
            let result = self.char_queue.pop_front();
            match result {
                Some(c) => {
                    self.col += 1;
                    self.previous_char = self.current_char;
                    self.current_char = Some(c);
                }
                None => panic!("This shouldn't happen"),
            }
            result
        } else {
            None
        }
    }

    pub fn peek(&mut self) -> Option<&char> {
        if self.check_queue() {
            self.char_queue.front()
        } else {
            None
        }
    }

    pub fn peek_n(&mut self, n: usize) -> Option<Vec<&char>> {
        if self.check_queue() {
            Some(vec![self.char_queue.front().unwrap()])
        } else {
            None
        }
    }

    pub fn next_if(&mut self, func: impl FnOnce(&char) -> bool) -> Option<char> {
        match self.peek() {
            Some(c) => match func(c) {
                true => self.next(),
                false => None,
            },
            None => None,
        }
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
