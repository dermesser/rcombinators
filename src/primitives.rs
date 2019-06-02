use crate::combinators::{Repeat, RepeatSpec};
use crate::parser::{ParseError, ParseResult, Parser};
use crate::state::ParseState;

use std::collections::HashSet;
use std::iter::FromIterator;

pub struct StringParser(String);

impl StringParser {
    pub fn new<S: AsRef<str>>(s: S) -> StringParser {
        StringParser(s.as_ref().to_owned())
    }
}

impl Parser for StringParser {
    type Result = String;
    fn parse(
        &mut self,
        st: &mut ParseState<impl Iterator<Item = char>>,
    ) -> ParseResult<Self::Result> {
        let mut cs = self.0.chars();
        let expect = self.0.len();
        let mut have = 0;
        let hold = st.hold();
        loop {
            let (next, pk) = (cs.next(), st.peek());
            if next.is_none() || pk.is_none() {
                break;
            }
            if next != pk {
                break;
            }
            let c = st.next().unwrap();
            have += c.len_utf8();
        }
        if expect == have {
            st.release(hold);
            return Ok(self.0.clone());
        }
        let ix = st.index();
        st.reset(hold);
        return Err(ParseError::Fail("string not matched", ix));
    }
}

pub struct Int;

impl Parser for Int {
    type Result = i64;
    fn parse(
        &mut self,
        st: &mut ParseState<impl Iterator<Item = char>>,
    ) -> ParseResult<Self::Result> {
        let mut negative: i64 = 1;
        let mut result: i64 = 0;

        match st.peek() {
            None => return Err(ParseError::EOF),
            Some('-') => negative = -1,
            Some(c) if c.is_digit(10) => result = result * 10 + ((c as i64) - ('0' as i64)),
            Some(_) => return Err(ParseError::Fail("not an int", st.index())),
        }
        let hold = st.hold();
        st.next();

        loop {
            match st.next() {
                Some(c) if c.is_digit(10) => result = result * 10 + ((c as i64) - ('0' as i64)),
                Some(_) => {
                    st.undo_next();
                    break;
                }
                None => break,
            }
        }
        st.release(hold);
        return Ok(result * negative);
    }
}

pub struct OneOf(HashSet<char>);

impl OneOf {
    pub fn new<S: AsRef<str>>(chars: S) -> OneOf {
        OneOf(chars.as_ref().chars().collect())
    }
}

impl Parser for OneOf {
    type Result = char;
    fn parse(
        &mut self,
        st: &mut ParseState<impl Iterator<Item = char>>,
    ) -> ParseResult<Self::Result> {
        match st.peek() {
            Some(c) if self.0.contains(&c) => {
                st.next();
                Ok(c)
            }
            _ => Err(ParseError::Fail("char not matched", st.index())),
        }
    }
}

/// A parser that parses a string consisting of characters `chars`.
fn string_of<S: AsRef<str>>(chars: S, rp: RepeatSpec) -> impl Parser<Result = String> {
    let oo = OneOf::new(chars);
    let rp = Repeat::new(oo, rp);
    let make_string = |charvec: Vec<char>| Ok(String::from_iter(charvec.into_iter()));
    rp.apply(make_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let mut s = ParseState::new("abc def");
        let mut p = StringParser::new("abc ".to_owned());
        assert_eq!(Ok("abc ".to_owned()), p.parse(&mut s));
        assert_eq!(4, s.index());
    }

    #[test]
    fn test_parse_int() {
        let mut s = ParseState::new("-1252 353");
        let mut ip = Int;
        let mut sp = StringParser::new(" ".to_string());
        assert_eq!(Ok(-1252), ip.parse(&mut s));
        assert_eq!(Ok(" ".to_string()), sp.parse(&mut s));
        assert_eq!(Ok(353), ip.parse(&mut s));
    }

    #[test]
    fn test_string_of() {
        let mut st = ParseState::new("aaabcxxzy");
        let mut p = string_of("abcd", RepeatSpec::Min(1));
        assert_eq!(Ok("aaabc".to_string()), p.parse(&mut st));
    }
}
