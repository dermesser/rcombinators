use std::ops::Shr;

use crate::parser::{ParseError, ParseResult, Parser};
use crate::state::ParseState;

/// Transform applies a function (which may fail) to the result of a parser. Transform only
/// succeeds if the applied function succeeds, too.
pub struct Transform<R, R2, P: Parser<Result = R>, F: Fn(R) -> ParseResult<R2>> {
    f: F,
    p: P,
}

impl<R, R2, P: Parser<Result = R>, F: Fn(R) -> ParseResult<R2>> Transform<R, R2, P, F> {
    /// Create a new Transform parser using f.
    pub fn new(p: P, f: F) -> Transform<R, R2, P, F> {
        Transform { f: f, p: p }
    }
}

impl<R, R2, P: Parser<Result = R>, F: Fn(R) -> ParseResult<R2>> Parser for Transform<R, R2, P, F> {
    type Result = R2;
    fn parse(
        &mut self,
        st: &mut ParseState<impl Iterator<Item = char>>,
    ) -> ParseResult<Self::Result> {
        match self.p.parse(st) {
            Ok(o) => (self.f)(o),
            Err(e) => Err(e),
        }
    }
}

pub struct Alternative<T>(T);

impl<T> Alternative<T> {
    pub fn new(tuple: T) -> Alternative<T> {
        Alternative(tuple)
    }
}

macro_rules! alt_impl {
    ( ( $($ptype:ident/$ix:tt),* ) ) => {
        impl<R, $($ptype : Parser<Result=R>, )*> Parser for Alternative<($($ptype,)*)> {
            type Result = R;
            fn parse(&mut self, st: &mut ParseState<impl Iterator<Item = char>>) -> ParseResult<Self::Result> {
                $(
                    let hold = st.hold();
                    match (self.0).$ix.parse(st) {
                        Err(_) => (),
                        Ok(o) => { st.release(hold); return Ok(o) }
                    }
                    st.reset(hold);
                )*
                return Err(ParseError::Fail("no alternative matched", st.index()))
            }
        }
    }
}

alt_impl!((P0 / 0, P1 / 1));
alt_impl!((P0 / 0, P1 / 1, P2 / 2));
alt_impl!((P0 / 0, P1 / 1, P2 / 2, P3 / 3));
alt_impl!((P0 / 0, P1 / 1, P2 / 2, P3 / 3, P4 / 4));
alt_impl!((P0 / 0, P1 / 1, P2 / 2, P3 / 3, P4 / 4, P5 / 5));
alt_impl!((P0 / 0, P1 / 1, P2 / 2, P3 / 3, P4 / 4, P5 / 5, P6 / 6));
alt_impl!((
    P0 / 0,
    P1 / 1,
    P2 / 2,
    P3 / 3,
    P4 / 4,
    P5 / 5,
    P6 / 6,
    P7 / 7
));
alt_impl!((
    P0 / 0,
    P1 / 1,
    P2 / 2,
    P3 / 3,
    P4 / 4,
    P5 / 5,
    P6 / 6,
    P7 / 7,
    P8 / 8
));
alt_impl!((
    P0 / 0,
    P1 / 1,
    P2 / 2,
    P3 / 3,
    P4 / 4,
    P5 / 5,
    P6 / 6,
    P7 / 7,
    P8 / 8,
    P9 / 9
));

/// Sequence concatenates parsers and only succeeds if all of them do. T is always a tuple in order
/// for Sequence to implement the Parser trait. The result is a tuple of all the parser results.
///
/// Individual parsers need to have result types implementing Default.
pub struct Sequence<T>(T);

impl<T> Sequence<T> {
    pub fn new(tuple: T) -> Sequence<T> {
        Sequence(tuple)
    }
}

/// Macro for implementing sequence parsers for arbitrary tuples. Not for public use.
macro_rules! seq_impl {
    ( ( $($ptype:ident/$ix:tt),+ ) ) => {
        impl<$($ptype : Parser<Result=impl Default>, )*> Parser for Sequence<($($ptype,)*)> {
            type Result = ($($ptype::Result,)*);
            fn parse(&mut self, st: &mut ParseState<impl Iterator<Item = char>>) -> ParseResult<Self::Result> {
                let hold = st.hold();
                let mut result = Self::Result::default();
                $(
                    let r = (self.0).$ix.parse(st);
                    if r.is_err() {
                        st.reset(hold);
                        return Err(r.err().unwrap());
                    }
                    result.$ix = r.unwrap();
                )*
                st.release(hold);
                return Ok(result);
            }
        }
    }
}

seq_impl!((P0 / 0, P1 / 1));
seq_impl!((P0 / 0, P1 / 1, P2 / 2));
seq_impl!((P0 / 0, P1 / 1, P2 / 2, P3 / 3));
seq_impl!((P0 / 0, P1 / 1, P2 / 2, P3 / 3, P4 / 4));
seq_impl!((P0 / 0, P1 / 1, P2 / 2, P3 / 3, P4 / 4, P5 / 5));
seq_impl!((P0 / 0, P1 / 1, P2 / 2, P3 / 3, P4 / 4, P5 / 5, P6 / 6));
seq_impl!((
    P0 / 0,
    P1 / 1,
    P2 / 2,
    P3 / 3,
    P4 / 4,
    P5 / 5,
    P6 / 6,
    P7 / 7
));
seq_impl!((
    P0 / 0,
    P1 / 1,
    P2 / 2,
    P3 / 3,
    P4 / 4,
    P5 / 5,
    P6 / 6,
    P7 / 7,
    P8 / 8
));
seq_impl!((
    P0 / 0,
    P1 / 1,
    P2 / 2,
    P3 / 3,
    P4 / 4,
    P5 / 5,
    P6 / 6,
    P7 / 7,
    P8 / 8,
    P9 / 9
));

/// PartialSequence concatenates parsers and tries to parse as far as possible.
///
/// Individual parsers need to have result types implementing Default.
pub struct PartialSequence<T>(T);

impl<T> PartialSequence<T> {
    pub fn new(tuple: T) -> PartialSequence<T> {
        PartialSequence(tuple)
    }
}

/// Macro for implementing sequence parsers for arbitrary tuples. Not for public use.
macro_rules! pseq_impl {
    ( ( $($ptype:ident/$ix:tt),+ ) ) => {
        impl<$($ptype : Parser<Result=impl Default>, )*> Parser for PartialSequence<($($ptype,)*)> {
            type Result = ($(Option<$ptype::Result>,)*);
            fn parse(&mut self, st: &mut ParseState<impl Iterator<Item = char>>) -> ParseResult<Self::Result> {
                let hold = st.hold();
                let mut result = Self::Result::default();
                $(
                    let r = (self.0).$ix.parse(st);
                    if r.is_err() {
                        st.release(hold);
                        return Ok(result);
                    }
                    result.$ix = Some(r.unwrap());
                )*
                st.release(hold);
                return Ok(result);
            }
        }
    }
}

pseq_impl!((P0 / 0, P1 / 1));
pseq_impl!((P0 / 0, P1 / 1, P2 / 2));
pseq_impl!((P0 / 0, P1 / 1, P2 / 2, P3 / 3));
pseq_impl!((P0 / 0, P1 / 1, P2 / 2, P3 / 3, P4 / 4));
pseq_impl!((P0 / 0, P1 / 1, P2 / 2, P3 / 3, P4 / 4, P5 / 5));
pseq_impl!((P0 / 0, P1 / 1, P2 / 2, P3 / 3, P4 / 4, P5 / 5, P6 / 6));
pseq_impl!((
    P0 / 0,
    P1 / 1,
    P2 / 2,
    P3 / 3,
    P4 / 4,
    P5 / 5,
    P6 / 6,
    P7 / 7
));
pseq_impl!((
    P0 / 0,
    P1 / 1,
    P2 / 2,
    P3 / 3,
    P4 / 4,
    P5 / 5,
    P6 / 6,
    P7 / 7,
    P8 / 8
));
pseq_impl!((
    P0 / 0,
    P1 / 1,
    P2 / 2,
    P3 / 3,
    P4 / 4,
    P5 / 5,
    P6 / 6,
    P7 / 7,
    P8 / 8,
    P9 / 9
));

pub enum RepeatSpec {
    /// Any is equivalent to Min(0).
    Any,
    Min(usize),
    Max(usize),
    Between(usize, usize),
}

pub struct Repeat<P: Parser> {
    inner: P,
    repeat: RepeatSpec,
}

impl<P: Parser> Repeat<P> {
    pub fn new(p: P, r: RepeatSpec) -> Repeat<P> {
        Repeat {
            inner: p,
            repeat: r,
        }
    }
}

impl<R, P: Parser<Result = R>> Parser for Repeat<P> {
    type Result = Vec<R>;
    fn parse(
        &mut self,
        st: &mut ParseState<impl Iterator<Item = char>>,
    ) -> ParseResult<Self::Result> {
        let (min, max) = match self.repeat {
            RepeatSpec::Any => (0, std::usize::MAX),
            RepeatSpec::Min(min) => (min as usize, std::usize::MAX),
            RepeatSpec::Max(max) => (0, max as usize),
            RepeatSpec::Between(min, max) => (min as usize, max as usize),
        };
        let mut v: Self::Result = Vec::new();
        let hold = st.hold();
        for i in 0.. {
            if i > max {
                st.release(hold);
                return Ok(v);
            }
            match self.inner.parse(st) {
                Ok(r) => v.push(r),
                Err(e) => {
                    if i >= min {
                        st.release(hold);
                        return Ok(v);
                    } else {
                        st.reset(hold);
                        return Err(e);
                    }
                }
            }
        }
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::primitives::*;

    #[test]
    fn test_pair() {
        let mut p = Sequence::new((Int, StringParser::new(" aba".to_string())));
        let mut ps = ParseState::new("123 aba");
        assert_eq!(Ok((123, " aba".to_string())), p.parse(&mut ps));
    }

    #[test]
    fn test_long_seq() {
        let s = || StringParser::new("a");
        let mut p = Sequence::new((s(), s(), s(), s(), s(), s(), s(), s(), s(), s()));
        let mut ps = ParseState::new("aaaaaaaaaa");
        assert_eq!(
            Ok((
                "a".to_string(),
                "a".to_string(),
                "a".to_string(),
                "a".to_string(),
                "a".to_string(),
                "a".to_string(),
                "a".to_string(),
                "a".to_string(),
                "a".to_string(),
                "a".to_string()
            )),
            p.parse(&mut ps)
        );
    }

    #[test]
    fn test_alternative() {
        let mut p = Alternative::new((
            StringParser::new("ab"),
            StringParser::new("de"),
            StringParser::new(" "),
            Transform::new(Int, |i| Ok(i.to_string())),
        ));
        let mut ps = ParseState::new("de 34");
        assert_eq!(Ok("de".to_string()), p.parse(&mut ps));
        assert_eq!(Ok(" ".to_string()), p.parse(&mut ps));
        assert_eq!(Ok("34".to_string()), p.parse(&mut ps));
    }

    #[test]
    fn test_partial_sequence() {
        let mut p = PartialSequence::new((StringParser::new("a"), StringParser::new("c"), Int));
        let mut ps = ParseState::new("acde");
        assert_eq!(
            Ok((Some("a".to_string()), Some("c".to_string()), None)),
            p.parse(&mut ps)
        );

        let mut p = PartialSequence::new((
            Sequence::new((Int, StringParser::new(" "), Int)),
            StringParser::new("x"),
        ));
        let mut ps = ParseState::new("12 -12 nothing else");
        assert_eq!(
            Ok((Some((12, " ".to_string(), -12)), None)),
            p.parse(&mut ps)
        );
    }
}
