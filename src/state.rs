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

    current: usize,
    // TODO: Implement garbage collection on `buf`
}

pub struct Hold(usize);

impl<'a> ParseState<Chars<'a>> {
    pub fn new(s: &'a str) -> ParseState<Chars<'a>> {
        ParseState {
            buf: vec![],
            next: Some(s.chars()),
            current: 0,
        }
    }
    pub fn from_reader<R: io::Read>(r: R) -> ParseState<impl Iterator<Item = char>> {
        ParseState {
            buf: vec![],
            next: Some(UTF8Reader(utf8reader::UTF8Reader::new(r))),
            current: 0,
        }
    }
}

impl<Iter: Iterator<Item = char>> ParseState<Iter> {
    const PREFILL_DEFAULT: usize = 1024;
    pub fn index(&mut self) -> usize {
        self.current
    }
    pub fn hold(&mut self) -> Hold {
        Hold(self.current)
    }
    pub fn release(&mut self, _h: Hold) {
        // TODO: Implement when hold tracking is needed (for garbage collection).
    }
    pub fn reset(&mut self, h: Hold) {
        self.current = h.0;
    }
    pub fn finished(&self) -> bool {
        self.next.is_none() && self.current == self.buf.len()
    }
    pub fn undo_next(&mut self) {
        assert!(self.current > 0);
        self.current -= 1;
    }
    pub fn current(&self) -> Option<Iter::Item> {
        if self.current < self.buf.len() {
            Some(self.buf[self.current])
        } else {
            None
        }
    }

    fn prefill(&mut self, n: usize) {
        if let Some(next) = self.next.as_mut() {
            let mut v: Vec<char> = next.take(n).collect();
            self.buf.append(&mut v)
        }
    }
    pub fn peek(&mut self) -> Option<Iter::Item> {
        if self.current + 1 < self.buf.len() {
            Some(self.buf[self.current + 1])
        } else {
            let c = self.next();
            if c == None {
                return None;
            }
            self.current -= 1;
            c
        }
    }
}

impl<Iter: Iterator<Item = char>> Iterator for ParseState<Iter> {
    type Item = char;

    fn next(&mut self) -> Option<Iter::Item> {
        if self.current < self.buf.len() {
            self.current += 1;
            Some(self.buf[self.current - 1])
        } else if let Some(cs) = self.next.as_mut() {
            if let Some(c) = cs.next() {
                self.buf.push(c);
                self.current += 1;
                Some(c)
            } else {
                self.next = None;
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn init() {
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
