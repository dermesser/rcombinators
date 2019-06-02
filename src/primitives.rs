use crate::combinators::{Repeat, RepeatSpec};
use crate::parser::{ParseError, ParseResult, Parser};
use crate::state::ParseState;

use std::collections::HashSet;
use std::error::Error;
use std::iter::FromIterator;
use std::str;

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

pub struct Int<IType: Default + str::FromStr>(IType);

pub type Int128 = Int<i128>;
pub type Int64 = Int<i64>;
pub type Int32 = Int<i32>;
pub type Int16 = Int<i16>;
pub type Int8 = Int<i8>;
pub type Uint128 = Int<u128>;
pub type Uint64 = Int<u64>;
pub type Uint32 = Int<u32>;
pub type Uint16 = Int<u16>;
pub type Uint8 = Int<u8>;

impl<IType: Default + str::FromStr> Int<IType> {
    pub fn new() -> Int<IType> {
        Int(IType::default())
    }
}

impl<IType: Default + str::FromStr<Err = std::num::ParseIntError> + std::convert::TryFrom<i8>>
    Parser for Int<IType>
{
    type Result = IType;
    fn parse(
        &mut self,
        st: &mut ParseState<impl Iterator<Item = char>>,
    ) -> ParseResult<Self::Result> {
        // Optimization for most ints.
        const BUFSIZE: usize = 8;
        let mut buf: [char; BUFSIZE] = [' '; BUFSIZE];
        let mut widebuf: Option<Vec<char>> = None;
        let mut i = 0;

        if IType::try_from(-1 as i8).is_ok() {
            // Check for negative sign, only if integer can be signed.
            match st.peek() {
                None => return Err(ParseError::EOF),
                Some('-') => {
                    buf[i] = '-';
                    i += 1;
                }
                Some(c) if c.is_digit(10) => {
                    buf[i] = c;
                    i += 1;
                }
                Some(_) => return Err(ParseError::Fail("not start of integer", st.index())),
            }
        }

        let hold = st.hold();
        if i > 0 {
            st.next();
        }

        // Consume digits
        loop {
            match st.next() {
                Some(c) if c.is_digit(10) => {
                    if widebuf.is_none() {
                        buf[i] = c;
                        i += 1;
                        if i >= BUFSIZE {
                            widebuf = Some(buf.to_vec());
                        }
                    } else {
                        widebuf.as_mut().unwrap().push(c);
                        i += 1;
                    }
                }
                Some(_) => {
                    st.undo_next();
                    break;
                }
                None => break,
            }
        }
        if i == 0 {
            st.reset(hold);
            return Err(ParseError::Fail("no appropriate integer found", st.index()));
        }
        let intstr: String;
        if widebuf.is_none() {
            intstr = buf[..i].iter().collect();
        } else {
            intstr = widebuf.unwrap().iter().collect();
        }
        match IType::from_str(&intstr) {
            Ok(i) => {
                st.release(hold);
                Ok(i)
            }
            Err(e) => {
                st.reset(hold);
                Err(ParseError::ExecFail(e.description().to_string()))
            }
        }
    }
}

/// OneOf matches any character that is in its specification.
pub struct OneOf(HashSet<char>, bool);

impl OneOf {
    pub fn new<S: AsRef<str>>(chars: S) -> OneOf {
        OneOf(chars.as_ref().chars().collect(), false)
    }
    /// Create a OneOf parser that parses all characters *not* in the given set.
    pub fn new_inverse<S: AsRef<str>>(chars: S) -> OneOf {
        OneOf(chars.as_ref().chars().collect(), true)
    }
}

impl Parser for OneOf {
    type Result = char;
    fn parse(
        &mut self,
        st: &mut ParseState<impl Iterator<Item = char>>,
    ) -> ParseResult<Self::Result> {
        match st.peek() {
            Some(c) => {
                let present = self.0.contains(&c);
                if self.1 {
                    // Inverse mode
                    if present {
                        return Err(ParseError::Fail("char (inverse) not matched", st.index()));
                    }
                    st.next();
                    return Ok(c);
                } else {
                    if present {
                        st.next();
                        return Ok(c);
                    }
                    return Err(ParseError::Fail("char not matched", st.index()));
                }
            }
            _ => Err(ParseError::EOF),
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

/// A parser that parses a string consisting of any characters not in the set.
fn string_none_of<S: AsRef<str>>(chars: S, rp: RepeatSpec) -> impl Parser<Result = String> {
    let oo = OneOf::new_inverse(chars);
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
        let mut s = ParseState::new("-1252 353 354 -1253");
        let mut ip = Int64::new();
        let mut up = Uint64::new();
        let mut sp = StringParser::new(" ".to_string());
        assert_eq!(Ok(-1252), ip.parse(&mut s));
        assert_eq!(Ok(" ".to_string()), sp.parse(&mut s));
        assert_eq!(Ok(353), ip.parse(&mut s));
        assert_eq!(Ok(" ".to_string()), sp.parse(&mut s));
        assert_eq!(Ok(354), up.parse(&mut s));
        assert_eq!(Ok(" ".to_string()), sp.parse(&mut s));
        assert!(up.parse(&mut s).is_err());
    }

    #[test]
    fn test_parse_long_int() {
        let mut s = ParseState::new("123456789");
        let mut up = Uint128::new();
        assert_eq!(Ok(123456789 as u128), up.parse(&mut s));
    }

    #[test]
    fn test_string_of() {
        let mut st = ParseState::new("aaabcxxzy");
        let mut p = string_of("abcd", RepeatSpec::Min(1));
        assert_eq!(Ok("aaabc".to_string()), p.parse(&mut st));
    }

    #[test]
    fn test_string_none_of() {
        let mut st = ParseState::new("aaabcxxzy");
        let mut p = string_none_of("xyz", RepeatSpec::Min(1));
        assert_eq!(Ok("aaabc".to_string()), p.parse(&mut st));
    }
}
