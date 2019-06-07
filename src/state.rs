use std::io;
use std::str::Chars;

use utf8reader;

struct UTF8Reader<R: io::Read>(utf8reader::UTF8Reader<R>);

impl<R: io::Read> Iterator for UTF8Reader<R> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                None => return None,
                Some(Err(_)) => continue,
                Some(Ok(c)) => return Some(c),
            }
        }
    }
}

/// ParseState encapsulates a stream of chars.
#[derive(Debug)]
pub struct ParseState<Iter: Iterator<Item = char>> {
    buf: Vec<char>,
    next: Option<Iter>,

    // Position in total stream, monotonically increasing except for hold resets.
    global: usize,
    // Position in buffer.
    current: usize,
    // Smallest held index, count of how many holds refer to it.
    oldest_hold_count: Option<(usize, usize)>,
}

/// A Hold represents the parsing state at a certain point. It can be used to "un-consume" input.
/// Currently, a panic occurs if a `Hold` object is dropped without first releasing or resetting it
/// using `ParseState::release()` or `ParseState::drop()`.
pub struct Hold {
    ix: usize,
    released: bool,
}

impl Hold {
    fn new(ix: usize) -> Hold {
        Hold {
            ix: ix,
            released: false,
        }
    }
    fn defuse(&mut self) {
        self.released = true;
    }
}

impl Drop for Hold {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(self.released, "Dropped unreleased hold! This is a bug");
        }
    }
}

impl<'a> ParseState<Chars<'a>> {
    /// Initialize ParseState from a string.
    pub fn new(s: &'a str) -> ParseState<Chars<'a>> {
        ParseState {
            buf: vec![],
            next: Some(s.chars()),
            current: 0,
            global: 0,
            oldest_hold_count: None,
        }
    }
    /// Initialize ParseState from a UTF-8 encoded source.
    pub fn from_reader<R: io::Read>(r: R) -> ParseState<impl Iterator<Item = char>> {
        ParseState {
            buf: vec![],
            next: Some(UTF8Reader(utf8reader::UTF8Reader::new(r))),
            current: 0,
            global: 0,
            oldest_hold_count: None,
        }
    }
}

impl<Iter: Iterator<Item = char>> ParseState<Iter> {
    const PREFILL_DEFAULT: usize = 1024;

    /// Return current index in input.
    pub fn index(&mut self) -> usize {
        self.current
    }

    /// Remember the current position in the input and protect it from buffer garbage collection.
    pub fn hold(&mut self) -> Hold {
        if self.oldest_hold_count.is_none() {
            self.oldest_hold_count = Some((self.global, 1))
        } else if let Some((ix, count)) = self.oldest_hold_count {
            if self.global == ix {
                self.oldest_hold_count = Some((self.global, count + 1));
            }
        }
        Hold::new(self.global)
    }

    /// Notifiy the ParseState that a `Hold` is no longer needed (and the referenced piece of input
    /// could be cleaned up, for example).
    pub fn release(&mut self, mut h: Hold) {
        match self.oldest_hold_count {
            Some((ix, count)) if ix == h.ix && count > 1 => {
                self.oldest_hold_count = Some((ix, count - 1));
            }
            Some((ix, count)) if ix == h.ix && count == 1 => {
                self.oldest_hold_count = None;
                // TODO: trigger garbage collection
            }
            _ => {}
        }
        h.defuse();
    }

    /// Reset state to what it was when `h` was created.
    pub fn reset(&mut self, mut h: Hold) {
        match self.oldest_hold_count {
            Some((ix, count)) if ix == h.ix && count > 1 => {
                self.oldest_hold_count = Some((ix, count - 1));
            }
            Some((ix, count)) if ix == h.ix && count == 1 => {
                self.oldest_hold_count = None;
                // No garbage collection needed as current index references this hold.
            }
            _ => {}
        }
        self.current -= self.global - h.ix;
        self.global = h.ix;
        h.defuse();
    }

    /// Returns true if no input is left.
    pub fn finished(&self) -> bool {
        self.next.is_none() && self.current == self.buf.len()
    }

    /// Shorthand for using a hold to undo a single call to `next()`.
    pub fn undo_next(&mut self) {
        assert!(self.current > 0);
        self.current -= 1;
        self.global -= 1;
    }

    /// Fill buffer from source with at most `n` characters.
    fn prefill(&mut self, n: usize) -> bool {
        if let Some(next) = self.next.as_mut() {
            let oldlen = self.buf.len();
            self.buf.extend(next.take(n));
            return (self.buf.len() - oldlen) > 0;
        }
        false
    }

    /// Return next character in input without advancing.
    pub fn peek(&mut self) -> Option<Iter::Item> {
        if self.current < self.buf.len() {
            return Some(self.buf[self.current]);
        } else if self.current == self.buf.len() && self.next.is_some() {
            match self.next() {
                Some(c) => {
                    self.current -= 1;
                    self.global -= 1;
                    return Some(c);
                }
                None => return None,
            }
        } else if self.next.is_none() {
            return None;
        }
        unreachable!()
    }
}

impl<Iter: Iterator<Item = char>> Iterator for ParseState<Iter> {
    type Item = char;

    fn next(&mut self) -> Option<Iter::Item> {
        if self.current < self.buf.len() {
            self.current += 1;
            self.global += 1;
            Some(self.buf[self.current - 1])
        } else {
            if self.prefill(Self::PREFILL_DEFAULT) {
                self.next()
            } else {
                // Mark reader as finished.
                self.next = None;
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_basic() {
        let mut s = ParseState::new("Hello");
        assert_eq!(Some('H'), s.next());
        let rest: String = s.collect();
        assert_eq!("ello", rest);

        let mut s = ParseState::new("Hello");
        let hold = s.hold();
        s.next();
        s.next();
        s.next();
        assert_eq!(Some('l'), s.peek());
        assert_eq!(Some('l'), s.next());
        s.reset(hold);
        let rest: String = s.collect();
        assert_eq!("Hello", rest);
    }

    #[test]
    #[should_panic]
    fn test_hold_unreleased() {
        let mut s = ParseState::new("abcde");
        let _hold = s.hold();
    }

    use crate::primitives;

    #[test]
    fn test_utf8_stream() {
        let s = "Hüðslþ".to_owned();
        let mut ps = ParseState::from_reader(s.as_bytes());
        assert_eq!(Some('H'), ps.next());
        assert_eq!(
            Ok("üð".to_string()),
            primitives::StringParser::new("üð".to_string()).parse(&mut ps)
        );
    }
}
