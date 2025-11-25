use std::array::repeat;

use winnow::{
    Parser,
    ascii::multispace0,
    combinator::delimited,
    error::{FromExternalError, ParserError},
};

// mod custom_types;
// mod functions;
// pub mod program;
// mod top_level;
pub mod expressions;
mod identifiers;
mod statements;
mod types;

pub fn ws<'a, F, O, E: ParserError<&'a str>>(inner: F) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}
