use crate::parser::{ParseError, ParseResult, Parser};
use crate::state::ParseState;

pub struct Sequence<T> {
    t: T,
}

impl<P1: Parser, P2: Parser> Parser for Sequence<(P1, P2)> {
    type Result = (P1::Result, P2::Result);
    fn parse(
        &mut self,
        st: &mut ParseState<impl Iterator<Item = char>>,
    ) -> ParseResult<Self::Result> {
        let hold = st.hold();
        let r1 = self.t.0.parse(st);
        if r1.is_err() {
            st.reset(hold);
            return Err(r1.err().unwrap());
        }
        let r2 = self.t.1.parse(st);
        if r2.is_err() {
            st.reset(hold);
            return Err(r2.err().unwrap());
        }
        st.release(hold);
        return Ok((r1.unwrap(), r2.unwrap()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::*;

    #[test]
    fn test_pair() {
        let mut p = (Int, StringParser(" aaa".to_string()));
        let mut ps = ParseState::new("123 aba");
        assert_eq!(Ok((123, " aaa".to_string())), p.parse(&mut ps));
    }
}
