use nom::{multi::many0, IResult, Parser};

use crate::ast::{
    functions::{FunctionDefinition, FunctionImplementation},
    program::Program,
    statements::Block,
    top_level::TopLevel,
    types::CustomType,
};

use super::top_level::parse_top_level;

pub fn parse_program(input: &str) -> IResult<&str, Program> {
    // imports at the top of the file

    // the rest of the file
    let (input, top_levels) = many0(parse_top_level).parse(input)?;
    let mut function_definitions: Vec<FunctionDefinition> = vec![];
    let mut function_implementations: Vec<FunctionImplementation> = vec![];
    let mut custom_types: Vec<CustomType> = vec![];
    let mut entry_point: Option<Block> = None;

    for top_level in top_levels {
        match top_level {
            TopLevel::FunctionDefinition(fd) => function_definitions.push(fd),
            TopLevel::FunctionImplementation(fi) => function_implementations.push(fi),
            TopLevel::CustomType(ct) => custom_types.push(ct),
            TopLevel::EntryPoint(ep) => {
                if entry_point.is_some() {
                    panic!("Multiple entry points found");
                }
                entry_point = Some(ep);
            }
        }
    }
    if entry_point.is_none() {
        panic!("No entry point found");
    }

    Ok((
        input,
        Program::new(
            function_definitions,
            function_implementations,
            custom_types,
            entry_point.unwrap(),
        ),
    ))
}
