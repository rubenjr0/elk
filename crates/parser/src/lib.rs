use pest_derive::Parser;
use winnow::{Parser as Pw, ascii::multispace0, combinator::delimited, error::ParserError};

mod custom_types;
#[cfg(test)]
mod pest_tests;
pub mod expressions;
mod functions;
mod identifiers;
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

#[derive(Parser)]
#[grammar = "../../../grammar_optimized.pest"]
struct Grammar;

#[cfg(test)]
mod tests {
    use pest::Parser;

    use super::*;

    #[test]
    fn sample_test() {
        let input = include_str!("../../../samples/sample.elk");
        let result = Grammar::parse(Rule::Program, input);
        result.expect("Failed to parse input");
    }
}
