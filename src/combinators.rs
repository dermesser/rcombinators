use crate::parser::{ParseResult, Parser};
use crate::state::ParseState;

pub struct Sequence<T> {
    t: T,
}

impl<T> Sequence<T> {
    pub fn new(tuple: T) -> Sequence<T> {
        Sequence { t: tuple }
    }
}

macro_rules! seq_impl {
    ( ( $($ptype:ident/$ix:tt),+ ) ) => {
        impl<$($ptype : Parser<Result=impl Default>, )*> Parser for Sequence<($($ptype,)*)> {
            type Result = ($($ptype::Result,)*);
            fn parse(&mut self, st: &mut ParseState<impl Iterator<Item = char>>) -> ParseResult<Self::Result> {
                let hold = st.hold();
                let mut result = Self::Result::default();
                $(
                    let r = (self.t.$ix).parse(st);
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
}
