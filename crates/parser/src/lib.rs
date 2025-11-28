use winnow::{Parser, ascii::multispace0, combinator::delimited, error::ParserError};

mod custom_types;
pub mod expressions;
mod functions;
mod identifiers;
pub mod program;
mod statements;
mod top_level;
mod types;

pub fn ws<'a, F, O, E: ParserError<&'a str>>(inner: F) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}
