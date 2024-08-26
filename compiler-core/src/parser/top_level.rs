use nom::{branch::alt, bytes::complete::tag, combinator::map, IResult, Parser};

use crate::ast::{statements::Block, top_level::TopLevel};

use super::{
    common::ws,
    custom_types::parse_custom_type,
    functions::{parse_function_definition, parse_function_impl},
    statements::parse_block,
};

pub fn parse_top_level(input: &str) -> IResult<&str, TopLevel> {
    alt((
        map(parse_function_definition, TopLevel::FunctionDefinition),
        map(parse_function_impl, TopLevel::FunctionImplementation),
        map(parse_custom_type, TopLevel::CustomType),
        map(parse_entrypoint, TopLevel::EntryPoint),
    ))
    .parse(input)
}

fn parse_entrypoint(input: &str) -> IResult<&str, Block> {
    let (input, _) = ws(tag("main")).parse(input)?;
    parse_block(input)
}
