use std::fmt;

use crate::combinators;
use crate::state::ParseState;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// Input is over.
    EOF,
    /// Input didn't match expectations, try next option if possible.
    Fail(&'static str, usize),
    /// Error during application of Transform.
    TransformFail(&'static str, usize, Box<ParseError>),
    /// ExecFail is an error that occurred while executing "user code", e.g. during a Transform
    /// parser.
    ExecFail(String),
}

/// This function returns an error for returning from a function called by a `Transform` parser.
pub fn execerr<S: AsRef<str>>(s: S) -> ParseError {
    ParseError::ExecFail(s.as_ref().to_string())
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::EOF => f.write_str("EOF"),
            ParseError::Fail(s, pos) => write!(f, "Parse fail: {} at {}", s, pos),
            ParseError::TransformFail(s, pos, inner) => {
                write!(f, "Transform fail: {} at {} due to ", s, pos).and_then(|()| inner.fmt(f))
            }
            ParseError::ExecFail(s) => write!(f, "Logic error: {}", s),
        }
    }
}

pub type ParseResult<R> = Result<R, ParseError>;

pub trait Parser {
    type Result;

    /// parse consumes input from `st` and returns a result or an error.
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
