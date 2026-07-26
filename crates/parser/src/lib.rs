use winnow::{
    Parser as Pw, ascii::multispace0,
    combinator::{delimited, not, peek, terminated},
    error::ParserError,
    token::one_of,
};

mod custom_types;
pub mod expressions;
mod functions;
mod identifiers;
mod patterns;
pub mod program;
mod statements;
mod top_level;
mod types;

pub fn ws<'a, F, O, E: ParserError<&'a str>>(inner: F) -> impl Pw<&'a str, O, E>
where
    F: Pw<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

/// A literal keyword that must not be immediately followed by an identifier
/// character, so e.g. `returnfoo` doesn't parse as `return foo`.
pub fn keyword<'a, E: ParserError<&'a str>>(
    lit: &'static str,
) -> impl Pw<&'a str, &'a str, E> {
    terminated(
        lit,
        peek(not(one_of(|c: char| c.is_alphanumeric() || c == '_'))),
    )
}
