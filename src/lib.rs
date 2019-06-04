#![allow(dead_code)]

//! rcombinators is a parser combinator library without special magic. It aims to be both easy to
//! use and reasonably fast, without using too much special syntax or macros.

#[allow(unused_imports)]
#[macro_use]
extern crate time_test;

mod combinators;
mod parser;
mod primitives;
mod state;

pub use combinators::*;
pub use parser::*;
pub use primitives::*;
pub use state::*;
