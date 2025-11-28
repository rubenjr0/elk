use ast::{statements::Block, top_level::TopLevel};
use winnow::{
    Parser, Result,
    combinator::{alt, repeat},
};

use crate::{
    custom_types::parse_custom_type_definition,
    functions::{parse_function_definition, parse_function_impl},
    statements::parse_block,
    ws,
};

pub fn parse_top_levels(input: &mut &str) -> Result<Vec<TopLevel>> {
    repeat(0.., parse_top_level).parse_next(input)
}

fn parse_top_level(input: &mut &str) -> Result<TopLevel> {
    alt((
        parse_custom_type_definition.map(TopLevel::CustomType),
        parse_entrypoint.map(TopLevel::EntryPoint),
        parse_function_definition.map(TopLevel::FunctionDefinition),
        parse_function_impl.map(TopLevel::FunctionImplementation),
    ))
    .parse_next(input)
}

fn parse_entrypoint(input: &mut &str) -> Result<Block> {
    let _ = ws("main").parse_next(input)?;
    parse_block(input)
}

#[cfg(test)]
mod tests {
    use ast::top_level::TopLevel;

    use crate::top_level::{parse_top_level, parse_top_levels};

    #[test]
    fn test_parse_entrypoint() {
        let mut input = "main { }";
        let parsed = parse_top_level(&mut input).unwrap();
        assert!(matches!(parsed, TopLevel::EntryPoint(_)));
    }

    #[test]
    fn test_parse_function_definition() {
        let mut input = "my_func(U8) -> U8;";
        let parsed = parse_top_level(&mut input).unwrap();
        assert!(matches!(parsed, TopLevel::FunctionDefinition(_)));
    }

    #[test]
    fn test_parse_function_impl() {
        let mut input = "my_func(x) = x;";
        let parsed = parse_top_level(&mut input).unwrap();
        assert!(matches!(parsed, TopLevel::FunctionImplementation(_)));
    }

    #[test]
    fn test_parse_custom_type() {
        let mut input = "type MyType { Var1, Var2 }";
        let parsed = parse_top_level(&mut input).unwrap();
        assert!(matches!(parsed, TopLevel::CustomType(_)));
    }

    #[test]
    fn test_parse_top_levels() {
        let mut input = "
        type MyType {Var1,Var2}

        main {}";
        let parsed = parse_top_levels(&mut input).unwrap();
        assert_eq!(parsed.len(), 2);
        assert!(matches!(parsed[0], TopLevel::CustomType(_)));
        assert!(matches!(parsed[1], TopLevel::EntryPoint(_)));
    }
}
