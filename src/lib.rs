#![allow(dead_code)]

//! rcombinators is a parser combinator library without special magic. It aims to be both easy to
//! use and reasonably fast, without using too much special syntax or macros.
//!
//! You will notice two kinds of parsers that however differ only in minor aspects:
//!
//!   * Ones starting with a capital letter are `struct`s (such as `Int`, `Sequence`). You can
//!     create them using `ParserName::new()`, or a specialized constructor method.
//!   * Ones starting with a lower case letter (and in snake case, such as `string_of`). Those are
//!     functions returning `Parser` objects combined from one or more elementary parsers.
//!
//! The resulting objects implementing the `Parser` trait are identical to use.
//!
//! Note that not all primitives and combinators are exported at the crate level! Only "important"
//! ones are.
//!
//! Here's a short example of how to use it:
//!
//! ```
//! use rcombinators::combinators;
//! use rcombinators::primitives;
//! use rcombinators::ParseState;
//! use rcombinators::Parser;
//!
//! // Goal: Parse the string between the parentheses, and then the float.
//! let mut ps = ParseState::new("(a1b3c4) -1.25e-1");
//!
//! let mut some_string = combinators::Alternative::new(
//!     (primitives::StringParser::new("xyz"),
//!      primitives::string_of("abcde12345",
//!      combinators::RepeatSpec::Min(1))));
//! let mut in_parens = combinators::Sequence::new(
//!     (primitives::StringParser::new("("),
//!      some_string,
//!      primitives::StringParser::new(")")));
//! assert_eq!(Ok(
//!     ("(".to_string(),
//!      "a1b3c4".to_string(),
//!      ")".to_string())), in_parens.parse(&mut ps));
//!
//! // You can continue using a ParseState, for example when implementing your own parsers.
//! let _ = primitives::whitespace().parse(&mut ps);
//! // Parsers returned by functions such as float() should be cached when used more frequently.
//! // This saves time due to not needing to construct the parsers repeatedly.
//! assert_eq!(Ok(-0.125), primitives::float().parse(&mut ps));
//! ```

#[allow(unused_imports)]
#[macro_use]
extern crate time_test;

pub mod combinators;
pub mod parser;
pub mod primitives;
mod state;

pub use combinators::{Alternative, Maybe, PartialSequence, Repeat, Sequence, Then, Transform};
pub use parser::{execerr, Parser};
pub use primitives::{float, string_none_of, string_of, whitespace, Int, StringParser};
pub use state::ParseState;
