use std::fmt;

use crate::state::ParseState;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    EOF,
    Fail(&'static str, usize),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::EOF => f.write_str("EOF"),
            ParseError::Fail(s, pos) => write!(f, "Parse fail: {} at {}", s, pos),
        }
    }
}

pub type ParseResult<R> = Result<R, ParseError>;

pub trait Parser {
    type Result;
    fn parse(
        &mut self,
        st: &mut ParseState<impl Iterator<Item = char>>,
    ) -> ParseResult<Self::Result>;
}
