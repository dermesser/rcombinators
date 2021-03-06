use crate::combinators::{Ignore, Maybe, Repeat, RepeatSpec, Sequence};
use crate::parser::{execerr, ParseError, ParseResult, Parser};
use crate::state::ParseState;

use std::collections::HashSet;
use std::error::Error;
use std::iter::FromIterator;
use std::str::{self, FromStr};

/// StringParser consumes a fixed string.
#[derive(Clone, Debug, PartialEq)]
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

/// Int parses an integer resulting in `IType`. It is recommended to use the specializations such
/// as `Int64`, `Uint32`, etc.
///
/// This is an optimized parser, not using combinators.
pub struct Int<IType: Default + str::FromStr>(IType);

/// Parse a 128 bit signed integer.
pub type Int128 = Int<i128>;
/// Parse a 64 bit signed integer.
pub type Int64 = Int<i64>;
/// Parse a 32 bit signed integer.
pub type Int32 = Int<i32>;
/// Parse a 16 bit signed integer.
pub type Int16 = Int<i16>;
/// Parse a 8 bit signed integer.
pub type Int8 = Int<i8>;
/// Parse a 128 bit unsigned integer.
pub type Uint128 = Int<u128>;
/// Parse a 64 bit unsigned integer.
pub type Uint64 = Int<u64>;
/// Parse a 32 bit unsigned integer.
pub type Uint32 = Int<u32>;
/// Parse a 16 bit unsigned integer.
pub type Uint16 = Int<u16>;
/// Parse a 8 bit unsigned integer.
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
        const BUFSIZE: usize = 16;
        let mut buf: [char; BUFSIZE] = [' '; BUFSIZE];
        let mut widebuf: Option<Vec<char>> = None;
        let mut i = 0;

        // Only check for negative sign if type is a signed integer.
        if IType::try_from(-1 as i8).is_ok() {
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

fn assemble_float(
    s: Option<()>,
    big: String,
    dot: Option<String>,
    mut little: Option<String>,
    exp: Option<((), i32)>,
) -> ParseResult<f64> {
    if dot.is_some() && little.is_none() {
        little = Some("0".to_string());
    }
    assert!((dot.is_some() && little.is_some()) || (dot.is_none() && little.is_none()));
    let bigf = match f64::from_str(&big) {
        Ok(f) => f,
        Err(e) => return Err(execerr(e.description())),
    };
    let mut littlef = 0.;
    if let Some(mut d) = dot {
        d.push_str(little.as_ref().unwrap());
        littlef = match f64::from_str(&d) {
            Ok(f) => f,
            Err(e) => return Err(execerr(e.description())),
        }
    }
    let mut multiplier: f64 = if s.is_some() { -1. } else { 1. };
    if let Some((_, e)) = exp {
        multiplier *= (10. as f64).powi(e);
    }
    return Ok(multiplier * (bigf + littlef));
}

/// float parses floats in the format of `[-]dd[.[dd]][e[-]ddd]`.
///
/// TODO: Compare speed with "native" parser, i.e. without combinators, and keep this as example.
pub fn float() -> impl Parser<Result = f64> {
    let digits_set = "0123456789";
    let minus = Maybe::new(Ignore::new(StringParser::new("-")));
    let digits = string_of(digits_set, RepeatSpec::Min(1));
    let point = Maybe::new(StringParser::new("."));
    let smalldigits = Maybe::new(string_of(digits_set, RepeatSpec::Min(1)));
    let exp = Maybe::new(Sequence::new((
        Ignore::new(StringParser::new("e")),
        Int32::new(),
    )));
    let parser = Sequence::new((minus, digits, point, smalldigits, exp))
        .apply(|(m, d, p, sd, exp)| assemble_float(m, d, p, sd, exp));
    parser
}

/// Nothing is a parser that always succeeds.
pub struct Nothing;

impl Parser for Nothing {
    type Result = ();
    fn parse(
        &mut self,
        _: &mut ParseState<impl Iterator<Item = char>>,
    ) -> ParseResult<Self::Result> {
        Ok(())
    }
}

/// Efficient matching depending on number of characters (a HashSet is much slower than a Vec for
/// less than ~20 chars).
enum MatchSpec {
    One(char),
    Some(Vec<char>),
    Many(HashSet<char>),
}

/// Threshold above which a HashSet is used instead of a Vec. Empirically determined.
const MATCHSPEC_MANY_THRESHOLD: usize = 20;

impl MatchSpec {
    fn new<S: AsRef<str>>(chars: S) -> MatchSpec {
        if chars.as_ref().len() == 1 {
            MatchSpec::One(chars.as_ref().chars().next().unwrap())
        } else if chars.as_ref().len() <= MATCHSPEC_MANY_THRESHOLD {
            MatchSpec::Some(chars.as_ref().chars().collect())
        } else {
            MatchSpec::Many(chars.as_ref().chars().collect())
        }
    }
    fn matches(&self, c: char) -> bool {
        match self {
            MatchSpec::One(cc) => c == *cc,
            MatchSpec::Some(cs) => cs.contains(&c),
            MatchSpec::Many(cs) => cs.contains(&c),
        }
    }
}

/// OneOf matches any character that is (or is not) in its set.
pub struct OneOf(MatchSpec, bool);

impl OneOf {
    pub fn new<S: AsRef<str>>(chars: S) -> OneOf {
        OneOf(MatchSpec::new(chars), false)
    }
    /// Create a OneOf parser that parses all characters *not* in the given set.
    pub fn new_none_of<S: AsRef<str>>(chars: S) -> OneOf {
        OneOf(MatchSpec::new(chars), true)
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
                if self.0.matches(c) ^ self.1 {
                    st.next();
                    Ok(c)
                } else {
                    Err(ParseError::Fail("char not matched", st.index()))
                }
            }
            _ => Err(ParseError::EOF),
        }
    }
}

/// A parser that parses a string consisting of characters `chars`.
pub fn string_of<S: AsRef<str>>(chars: S, rp: RepeatSpec) -> impl Parser<Result = String> {
    let oo = OneOf::new(chars);
    let rp = Repeat::new(oo, rp);
    let make_string = |charvec: Vec<char>| Ok(String::from_iter(charvec.into_iter()));
    rp.apply(make_string)
}

/// A parser that parses a string consisting of any characters not in the set.
pub fn string_none_of<S: AsRef<str>>(chars: S, rp: RepeatSpec) -> impl Parser<Result = String> {
    let oo = OneOf::new_none_of(chars);
    let rp = Repeat::new(oo, rp);
    let make_string = |charvec: Vec<char>| Ok(String::from_iter(charvec.into_iter()));
    rp.apply(make_string)
}

/// whitespace consumes any number of tabs, spaces, newlines.
pub fn whitespace() -> impl Parser<Result = ()> {
    Ignore::new(Repeat::new(OneOf::new(" \n\r\t"), RepeatSpec::Any))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combinators::Sequence;

    #[test]
    fn test_parse_string() {
        let mut s = ParseState::new("abc def");
        let mut p = StringParser::new("abc ".to_owned());
        assert_eq!(Ok("abc ".to_owned()), p.parse(&mut s));
        assert_eq!(4, s.index());
    }

    #[test]
    fn test_parse_int() {
        let mut s = ParseState::new("-1252 353 354 -1253 422345");
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
        assert_eq!(Ok(-1253), ip.parse(&mut s));
        assert_eq!(Ok(" ".to_string()), sp.parse(&mut s));
        assert_eq!(Ok(422345), up.parse(&mut s));
    }

    #[test]
    fn test_parse_long_int() {
        let mut s = ParseState::new("123456789");
        let mut up = Uint128::new();
        assert_eq!(Ok(123456789 as u128), up.parse(&mut s));
    }

    #[test]
    fn test_parse_floats() {
        let mut ps = ParseState::new("1 1. 1.5 -1.5 -1.75 2.5e-4 -2e-2");
        let mut p = float();
        let want = vec![1., 1., 1.5, -1.5, -1.75, 2.5e-4, -0.02];
        for &f in want.iter() {
            assert_eq!(Ok(f), p.parse(&mut ps));
            let _ = StringParser::new(" ").parse(&mut ps);
        }
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

    use std::iter;

    #[test]
    fn bench_integer_medium() {
        let piece = "-422345812310928 ";
        let repeats = 1000000;
        let mut input = String::with_capacity(piece.len() * repeats);
        input.extend(iter::repeat(piece).take(repeats));
        let mut ps = ParseState::new(&input);
        let mut p = Sequence::new((Int64::new(), StringParser::new(" ")));
        {
            time_test!("parse-int with static buffer");
            for _ in 0..repeats {
                let _ = p.parse(&mut ps);
            }
        }

        let piece = "-4223458123109289 ";
        let mut input = String::with_capacity(piece.len() * repeats);
        input.extend(iter::repeat(piece).take(repeats));
        let mut ps = ParseState::new(&input);
        {
            time_test!("parse-int with dynamic buffer");
            for _ in 0..repeats {
                let _ = p.parse(&mut ps);
            }
        }
    }

    #[test]
    fn bench_float() {
        let piece = "-32.334 ";
        let repeats = 1000000;
        let mut input = String::with_capacity(piece.len() * repeats);
        input.extend(iter::repeat(piece).take(repeats));
        let mut ps = ParseState::new(&input);
        let mut p = Sequence::new((float(), StringParser::new(" ")));
        {
            time_test!("parse-float with combinators");
            for _ in 0..repeats {
                let _ = p.parse(&mut ps);
            }
        }
    }
}
