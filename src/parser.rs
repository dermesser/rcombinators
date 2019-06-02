use std::fmt;

use crate::combinators;
use crate::state::ParseState;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    EOF,
    Fail(&'static str, usize),
    /// Error during application of Transform.
    TransformFail(&'static str, usize),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::EOF => f.write_str("EOF"),
            ParseError::Fail(s, pos) => write!(f, "Parse fail: {} at {}", s, pos),
            ParseError::TransformFail(s, pos) => write!(f, "Transform fail: {} at {}", s, pos),
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

    /// apply transforms the result of this parser using a Transform combinator.
    fn apply<R2, F: Fn(Self::Result) -> ParseResult<R2>>(
        self,
        f: F,
    ) -> combinators::Transform<Self::Result, R2, Self, F>
    where
        Self: std::marker::Sized,
    {
        combinators::Transform::new(self, f)
    }
}
