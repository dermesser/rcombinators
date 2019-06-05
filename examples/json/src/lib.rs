//! A simplistic JSON parser library based on the `rcombinators` crate.
//!

#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::iter::FromIterator;

use rcombinators::combinators;
use rcombinators::primitives;
use rcombinators::{ParseResult, ParseState, Parser};

#[derive(Debug, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Dict(HashMap<String, Value>),
    List(Vec<Value>),
    None,
}

impl Default for Value {
    fn default() -> Value {
        Value::None
    }
}

struct PWrapper<P>(P);

impl<P: Parser<Result=Value>> Parser for PWrapper<P> {
    type Result = Value;
    fn parse(&mut self, st: &mut ParseState<impl Iterator<Item=char>>) -> ParseResult<Self::Result> {
        self.0.parse(st)
    }
}

#[derive(Default)]
struct ValueParser<P>(Option<P>);

impl<P: Parser<Result=Value>> Parser for ValueParser<P> {
    type Result = Value;
    fn parse(&mut self, st: &mut ParseState<impl Iterator<Item=char>>) -> ParseResult<Self::Result> {
        combinators::Alternative::new((dict(), list(), string(), number())).parse(st)
    }
}

fn number() -> impl Parser<Result = Value> {
    primitives::float().apply(|n| Ok(Value::Number(n)))
}

fn string() -> impl Parser<Result = Value> {
    let quote = primitives::StringParser::new("\"");
    let middle = primitives::string_none_of("\"", combinators::RepeatSpec::Any);
    let string_with_quotes = combinators::Sequence::new((quote.clone(), middle, quote));
    let string = string_with_quotes.apply(|(_, s, _)| Ok(Value::String(s)));
    string
}

fn list<P: Parser<Result=Value>>(val: Option<PWrapper<P>>) -> impl Parser<Result = Value> {
    let (open, close) = (
        primitives::StringParser::new("["),
        primitives::StringParser::new("]"),
    );
    let val = val;
    let comma = primitives::StringParser::new(",");
    let separated_element = combinators::Sequence::new((
        primitives::whitespace(),
        val,
        primitives::whitespace(),
        combinators::Maybe::new(comma),
    ));
    let separated_element = separated_element.apply(|(_, v, _, _)| Ok(v));
    let separated_elements =
        combinators::Repeat::new(separated_element, combinators::RepeatSpec::Any);
    let list = combinators::Sequence::new((open, separated_elements, close))
        .apply(|(_, es, _)| Ok(Value::List(es)));
    list
}

fn dict() -> impl Parser<Result = Value> {
    let (open, close) = (
        primitives::StringParser::new("{"),
        primitives::StringParser::new("}"),
    );
    let comma = primitives::StringParser::new(",");
    let sep = primitives::StringParser::new(":");
    let key = string().apply(|v| match v {
        Value::String(s) => Ok(s),
        _ => panic!("unexpected value type in string position"),
    });
    let value = ValueParser(None);
    let separated_element = combinators::Sequence::new((
        primitives::whitespace(),
        key,
        primitives::whitespace(),
        sep,
        primitives::whitespace(),
        value,
        primitives::whitespace(),
        combinators::Maybe::new(comma),
    ));
    let separated_element =
        separated_element.apply(|(_ws1, k, _ws2, _sep, _ws3, v, _ws4, _comma)| Ok((k, v)));
    let separated_elements =
        combinators::Repeat::new(separated_element, combinators::RepeatSpec::Any);
    let dict = combinators::Sequence::new((open, separated_elements, close))
        .apply(|(_, es, _)| Ok(Value::Dict(HashMap::from_iter(es.into_iter()))));
    dict
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_number() {
        let mut ps = ParseState::new("-1.2e0");
        assert_eq!(Ok(Value::Number(-1.2)), number().parse(&mut ps));
    }

    #[test]
    fn test_string() {
        let mut ps = ParseState::new("\"Hello, World\n\"");
        assert_eq!(
            Ok(Value::String("Hello, World\n".to_string())),
            string().parse(&mut ps)
        );
    }

    #[test]
    fn test_list() {
        let mut ps = ParseState::new(r#"[1, 2, "Hello",]"#);
        let want = Value::List(vec![
            Value::Number(1.),
            Value::Number(2.),
            Value::String("Hello".to_string()),
        ]);
        assert_eq!(Ok(want), list().parse(&mut ps));
    }

    #[test]
    fn test_dict() {
        let mut ps = ParseState::new(r#"{"hello": ["world", []], "x": 4}"#);
        let want = Value::Dict(HashMap::from_iter(vec![
            (
                "hello".to_string(),
                Value::List(vec![
                    Value::String("world".to_string()),
                    Value::List(vec![]),
                ]),
            ),
            ("x".to_string(), Value::Number(4.)),
        ]));
        assert_eq!(Ok(want), dict().parse(&mut ps));
    }

    #[test]
    fn test_value() {
        let mut ps = ParseState::new(r#"{"hello": ["world", []], "x": 4}"#);
        let want = Value::Dict(HashMap::from_iter(vec![
            (
                "hello".to_string(),
                Value::List(vec![
                    Value::String("world".to_string()),
                    Value::List(vec![]),
                ]),
            ),
            ("x".to_string(), Value::Number(4.)),
        ]));
        assert_eq!(Ok(want), ValueParser.parse(&mut ps));
    }
}
