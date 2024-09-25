use nom::{branch::alt, bytes::complete::tag, combinator::map, multi::many0, IResult, Parser};

use crate::frontend::ast::{statements::Block, top_level::TopLevel};

use super::{
    common::ws,
    custom_types::parse_custom_type,
    functions::{parse_function_definition, parse_function_impl},
    statements::parse_block,
};

pub fn parse_top_levels(input: &str) -> IResult<&str, Vec<TopLevel>> {
    many0(parse_top_level).parse(input)
}

fn parse_top_level(input: &str) -> IResult<&str, TopLevel> {
    alt((
        map(parse_custom_type, TopLevel::CustomType),
        map(parse_entrypoint, TopLevel::EntryPoint),
        map(parse_function_definition, TopLevel::FunctionDefinition),
        map(parse_function_impl, TopLevel::FunctionImplementation),
    ))
    .parse(input)
}

fn parse_entrypoint(input: &str) -> IResult<&str, Block> {
    let (input, _) = ws(tag("main")).parse(input)?;
    parse_block(input)
}

#[cfg(test)]
mod tests {
    use crate::frontend::ast::top_level::TopLevel;

    use super::{parse_top_level, parse_top_levels};

    #[test]
    fn test_parse_entrypoint() {
        let input = "main { }";
        let (_, parsed) = parse_top_level(input).unwrap();
        assert!(matches!(parsed, TopLevel::EntryPoint(_)));
    }

    #[test]
    fn test_parse_function_definition() {
        let input = "my_func : U8 -> U8;";
        let (_, parsed) = parse_top_level(input).unwrap();
        assert!(matches!(parsed, TopLevel::FunctionDefinition(_)));
    }

    #[test]
    fn test_parse_function_impl() {
        let input = "my_func x = x;";
        let (_, parsed) = parse_top_level(input).unwrap();
        assert!(matches!(parsed, TopLevel::FunctionImplementation(_)));
    }

    #[test]
    fn test_parse_custom_type() {
        let input = "type MyType { Var1, Var2 }";
        let (_, parsed) = parse_top_level(input).unwrap();
        assert!(matches!(parsed, TopLevel::CustomType(_)));
    }

    #[test]
    fn test_parse_top_levels() {
        let input = "
        type MyType {Var1,Var2}

        main {}";
        let (_, parsed) = parse_top_levels(input).unwrap();
        assert_eq!(parsed.len(), 2);
        assert!(matches!(parsed[0], TopLevel::CustomType(_)));
        assert!(matches!(parsed[1], TopLevel::EntryPoint(_)));
    }
}
