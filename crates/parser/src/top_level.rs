use ast::top_level::TopLevel;
use winnow::{
    Parser, Result,
    combinator::{alt, preceded, repeat},
    error::StrContext,
};

use crate::{
    custom_types::parse_custom_type_definition,
    functions::{parse_function_definition, parse_function_impl},
    keyword,
    statements::parse_block,
    ws,
};

pub fn parse_top_levels(input: &mut &str) -> Result<Vec<TopLevel>> {
    repeat(
        1..,
        ws(parse_top_level).context(StrContext::Label("TopLevel")),
    )
    .context(StrContext::Label("TopLevels"))
    .parse_next(input)
}

fn parse_top_level(input: &mut &str) -> Result<TopLevel> {
    alt((
        parse_custom_type_definition
            .context(StrContext::Label("CustomType"))
            .map(TopLevel::CustomType),
        preceded(ws(keyword("main")), parse_block)
            .context(StrContext::Label("EntryPoint"))
            .map(TopLevel::EntryPoint),
        parse_function_definition
            .context(StrContext::Label("FunctionDef"))
            .map(TopLevel::FunctionDefinition),
        parse_function_impl
            .context(StrContext::Label("FunctionImpl"))
            .map(TopLevel::FunctionImplementation),
    ))
    .parse_next(input)
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
        let mut input = "func(MyType) -> U8;";
        let parsed = parse_top_level(&mut input).unwrap();
        assert!(matches!(parsed, TopLevel::FunctionDefinition(_)));
    }

    #[test]
    fn test_parse_function_impl() {
        let mut input = "func(x) = 2;";
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
        type MyType { Var1, Var2 }

        func(MyType) -> U8;

        func(x) = 2;

        main {}";
        let parsed = parse_top_levels(&mut input).unwrap();
        assert!(input.is_empty(), "Did not parse all input: {input:?}");
        assert_eq!(parsed.len(), 4);
        assert!(matches!(parsed[0], TopLevel::CustomType(_)));
        assert!(matches!(parsed[1], TopLevel::FunctionDefinition(_)));
        assert!(matches!(parsed[2], TopLevel::FunctionImplementation(_)));
        assert!(matches!(parsed[3], TopLevel::EntryPoint(_)));
    }
}
