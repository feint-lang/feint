use std::io::{BufRead, Cursor, Lines};
use std::iter::once;
use std::str::Chars;

fn main() {
    let cursor = Cursor::new("line 1\nline 2\r\nline 3");
    let mut lines = cursor.lines();
    loop {
        match lines.next() {
            Some(Ok(line)) => {
                let mut chars = line.chars().chain(once('\n'));
                loop {
                    match chars.next() {
                        Some(c) => print!("{}", c),
                        None => break,
                    }
                }
            }
            Some(Err(_)) => panic!(),
            None => break,
        };
    }
}
