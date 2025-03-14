use nom::{
    branch::alt, bytes::complete::tag, character::complete::multispace1, combinator::map,
    multi::separated_list1, sequence::terminated, IResult, Parser,
};

use crate::frontend::ast::functions::{FunctionBody, FunctionDeclaration, FunctionImplementation};

use super::{
    common::{parse_identifier_lower, ws},
    expressions::parse_expr,
    statements::parse_block,
    types::parse_function_signature,
};

pub fn parse_function_definition(input: &str) -> IResult<&str, FunctionDeclaration> {
    let (input, name) = parse_identifier_lower(input)?;
    let (input, _) = ws(tag(":")).parse(input)?;
    let (input, signature) = parse_function_signature(input)?;
    let (input, _) = ws(tag(";")).parse(input)?;

    Ok((input, FunctionDeclaration::new(name, signature)))
}

pub fn parse_function_impl(input: &str) -> IResult<&str, FunctionImplementation> {
    let (input, name) = parse_identifier_lower(input)?;
    let (input, args) = parse_function_args(input)?;
    let (input, body) = parse_function_body(input)?;

    Ok((input, FunctionImplementation::new(name, args, body)))
}

/// lowercase identifiers separated by spaces
/// Example: `arg1 arg2 arg3`
fn parse_function_args(input: &str) -> IResult<&str, Vec<String>> {
    ws(separated_list1(
        multispace1,
        parse_identifier_lower.map(str::to_owned),
    ))
    .parse(input)
}

fn parse_function_body(input: &str) -> IResult<&str, FunctionBody> {
    alt((
        parse_function_body_single_line,
        parse_function_body_multi_line,
    ))
    .parse(input)
}

fn parse_function_body_single_line(input: &str) -> IResult<&str, FunctionBody> {
    let (input, _) = ws(tag("=")).parse(input)?;
    let (input, body) = terminated(ws(parse_expr), tag(";")).parse(input)?;
    Ok((input, FunctionBody::SingleLine(body)))
}

fn parse_function_body_multi_line(input: &str) -> IResult<&str, FunctionBody> {
    map(parse_block, FunctionBody::MultiLine).parse(input)
}

#[cfg(test)]
mod tests {

    use crate::frontend::ast::{
        expressions::{Expression, Literal},
        statements::{Block, Statement},
        types::{FunctionSignature, Type},
    };

    use super::*;

    #[test]
    fn test_parse_function_definition() {
        let input = "my_function: U8 -> U8;";
        let (remaining, function) = parse_function_definition(input).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(function.name(), "my_function");
        assert_eq!(
            function.signature(),
            &FunctionSignature::new(vec![Type::U8], Type::U8)
        );
    }

    #[test]
    fn test_parse_basic_function_impl() {
        let input = "my_function _x = Unit;";
        let (_, function_impl) = parse_function_impl(input).unwrap();

        assert_eq!(function_impl.name(), "my_function");
        assert_eq!(function_impl.arguments(), &["_x"]);
        assert_eq!(
            function_impl.body(),
            &FunctionBody::SingleLine(Expression::unit())
        );
    }

    #[test]
    fn test_parse_function_impl() {
        let input = "my_function _x _y = 1;";
        let (remaining, function_impl) = parse_function_impl(input).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(function_impl.name(), "my_function");
        assert_eq!(function_impl.arguments(), &["_x", "_y"]);
        assert_eq!(
            function_impl.body(),
            &FunctionBody::SingleLine(Expression::literal(Literal::u8(1)))
        );
    }

    #[test]
    fn test_parse_multiline_function_impl() {
        let input = "my_function _x _y {
              _z = 1;
            _z
        }";
        let (_, function_impl) = parse_function_impl(input).unwrap();

        let expected_statements = vec![Statement::Assignment(
            "_z".to_owned(),
            Expression::literal(Literal::u8(1)),
        )];

        assert_eq!(function_impl.name(), "my_function");
        assert_eq!(function_impl.arguments(), &["_x", "_y"]);
        assert_eq!(
            function_impl.body(),
            &FunctionBody::MultiLine(Block::new(
                expected_statements,
                Expression::identifier("_z".to_owned())
            ))
        );
    }

    #[test]
    fn test_parse_function_args() {
        let input = "arg1 arg2 arg3";
        let (_, args) = parse_function_args(input).unwrap();

        assert_eq!(args, vec!["arg1", "arg2", "arg3"]);
    }

    #[test]
    fn test_parse_function_body_singleline() {
        let input = "= 1;";
        let (remaining, function_body) = parse_function_body(input).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(
            function_body,
            FunctionBody::SingleLine(Expression::literal(Literal::u8(1)))
        );
    }

    #[test]
    fn test_parse_function_body_multiline() {
        let input = "{ _z = 1; }";
        let (_, function_body) = parse_function_body(input).unwrap();

        assert_eq!(
            function_body,
            FunctionBody::MultiLine(Block::new_without_return(vec![Statement::Assignment(
                "_z".to_owned(),
                Expression::literal(Literal::u8(1))
            )]))
        );
    }
}
