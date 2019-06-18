# `rcombinators`

[![Build Status](https://travis-ci.org/dermesser/rcombinators.svg?branch=master)](https://travis-ci.org/dermesser/rcombinators)
[![crates.io](https://img.shields.io/crates/v/rcombinators.svg)](https://crates.io/crates/rcombinators)

`rcombinators` is a Rust version of the [`pcombinators`](https://github.com/dermesser/pcombinators)
library, providing parser combinators in Rust. As opposed to some other parser libraries it works
without much magic syntax for users; however this also means a bit more boilerplate and occasionally
less performance due to the "pedestrian" way of doing things.

Compared to `pcombinators` we still achieve up to 100x more throughput as well as type safety,
making writing parsers less error-prone.

An example of a working parser can be found in `examples/json/`, which is a working yet simplistic
JSON parser (it doesn't work with escaped characters in strings, for example), demonstrating how to
combine the parsers provided by rcombinators.
