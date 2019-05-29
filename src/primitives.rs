
use crate::state::ParseState;
use crate::parser::{Parser, ParseResult, ParseError};

pub struct StringParser(pub String);

impl Parser for StringParser {
    type Result = String;
    fn parse(&mut self, st: &mut ParseState<impl Iterator<Item=char>>) -> ParseResult<Self::Result> {
        let mut cs = self.0.chars();
        let expect = self.0.len();
        let mut have = 0;
        let hold = st.hold();
        loop {
            let (next, pk) = (cs.next(), st.peek());
            if next.is_none() || pk.is_none() {
                break
            }
            if next != pk {
                break
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
        return Err(ParseError::Fail("string not matched", ix))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let mut s = super::ParseState::new("abc def");
        let mut p = StringParser("abc ".to_owned());
        assert_eq!(Ok("abc ".to_owned()), p.parse(&mut s));
        assert_eq!(4, s.index());
    }
}
