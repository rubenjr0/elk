use ast::{
    functions::FunctionDeclaration,
    types::{FunctionSignature, Type},
};
use winnow::{
    ascii::{alpha1, multispace1}, combinator::{separated, todo},
    Parser,
    Result,
};

use crate::{
    // expressions::parse_expr,
    identifiers::parse_identifier_lower,
    // statements::parse_block,
    // types::parse_function_signature,
};

/// Types separated by commas. The last type is the return type and it's separated by an arrow.
/// If parenthesis are encountered, its a function signature, parse recursively.
/// Example: `(U8, U8) -> U8`
/// Example: `((U8 -> Bool), U8) -> Bool`
pub fn parse_function_signature<'s>(input: &mut &'s str) -> Result<FunctionSignature> {
    let args = parse_function_args(input)?;
    let (return_type, args) = args.split_last().unwrap();
    if args.is_empty() {
        panic!("Function signature must have at least one argument")
        // return Err(nom::Err::Error(nom::error::Error::new(
        //     remaining,
        //     nom::error::ErrorKind::SeparatedList,
        // )));
    }
    Ok(FunctionSignature::new(args.to_vec(), return_type.clone()))
}

fn parse_function_args<'s>(input: &mut &'s str) -> Result<Vec<Type>> {
    let a = separated(1.., alpha1, multispace1).parse_next(input);
    todo(input)
    // let (remaining, args) = separated_list1(
    //     ws(tag("->")),
    //     alt((
    //         parse_primitive_type,
    //         map(parse_custom_type, |(name, generics)| {
    //             Type::Custom(name, generics)
    //         }),
    //         map(
    //             delimited(tag("("), parse_function_signature, tag(")")),
    //             Type::Function,
    //         ),
    //     )),
    // )
    // .parse(input)?;
    // Ok((remaining, args))
}

pub fn parse_function_definition<'s>(input: &mut &'s str) -> Result<FunctionDeclaration> {
    let identifier = parse_identifier_lower(input)?;
    let _ = ':'.parse_next(input)?;
    let signature = parse_function_signature(input)?;
    let _ = ';'.parse_next(input)?;

    Ok(FunctionDeclaration::new(identifier, signature))
}

pub fn parse_function_impl(input: &str) -> IResult<&str, FunctionImplementation> {
    let (input, name) = parse_identifier_lower(input)?;
    let (input, args) = parse_function_args(input)?;
    let (input, body) = parse_function_body(input)?;

    Ok((input, FunctionImplementation::new(name, args, body)))
}

// lowercase identifiers separated by spaces
// Example: `arg1 arg2 arg3`
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
    use ast::types::{FunctionSignature, Type};

    use super::*;

    #[test]
    fn test_parse_function_definition() {
        let mut input = "my_function: U8, U8 -> U8;";
        let function = parse_function_definition(&mut input).unwrap();
        assert!(input.is_empty());
        assert_eq!(function.name(), "my_function");
        assert_eq!(
            function.signature(),
            &FunctionSignature::new(vec![Type::U8], Type::U8)
        );
    }

    #[test]
    fn test_parse_basic_function_impl() {
        let mut input = "my_function _x = Unit;";
        let function_impl = parse_function_impl(&mut input).unwrap();

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
            &FunctionBody::SingleLine(Expression::literal(Literal::int(1)))
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
            Expression::literal(Literal::int(1)),
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
        let mut input = "arg1 arg2 arg3";
        let args = parse_function_args(&mut input).unwrap();

        assert_eq!(args, vec!["arg1", "arg2", "arg3"]);
    }

    #[test]
    fn test_parse_function_body_singleline() {
        let input = "= 1;";
        let (remaining, function_body) = parse_function_body(input).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(
            function_body,
            FunctionBody::SingleLine(Expression::literal(Literal::int(1)))
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
                Expression::literal(Literal::int(1))
            )]))
        );
    }
}
