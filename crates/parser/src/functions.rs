use ast::{
    functions::{FunctionBody, FunctionDeclaration, FunctionImplementation},
    types::{FunctionSignature, Type},
};
use winnow::{
    Parser, Result,
    combinator::{alt, delimited, opt, separated, separated_pair, terminated},
    error::{StrContext, StrContextValue},
};

use crate::{
    expressions::parse_expr,
    identifiers::parse_identifier_lower,
    statements::parse_block,
    types::{parse_custom_type, parse_primitive_type, parse_type},
    ws,
};

/// Types separated by commas. The last type is the return type and it's separated by an arrow.
/// If parenthesis are encountered, its a function signature, parse recursively.
/// Example: `(U8, U8) -> U8`
/// Example: `(((U8) -> Bool), U8) -> Bool`
pub fn parse_function_signature(input: &mut &str) -> Result<FunctionSignature> {
    let (args, out) = separated_pair(
        delimited(
            ws('('),
            separated(
                1..,
                (
                    opt(terminated(parse_identifier_lower, ws(':')))
                        .context(StrContext::Label("ArgLabel")),
                    alt((
                        parse_primitive_type,
                        parse_custom_type,
                        delimited(ws('('), parse_function_signature, ws(')'))
                            .map(Type::Function)
                            .context(StrContext::Label("sub-function")),
                    )),
                )
                    .map(|(_label, t)| t),
                ws(','),
            ),
            ws(')'),
        )
        .context(StrContext::Label("SignatureArgs")),
        ws("->"),
        parse_type,
    )
    .parse_next(input)?;

    Ok(FunctionSignature::new(args, out))
}

pub fn parse_function_definition(input: &mut &str) -> Result<FunctionDeclaration> {
    let identifier = parse_identifier_lower(input)?;
    let signature = parse_function_signature
        .context(StrContext::Label("signature"))
        .parse_next(input)?;
    let _ = ';'
        .context(StrContext::Expected(StrContextValue::CharLiteral(';')))
        .parse_next(input)?;

    Ok(FunctionDeclaration::new(identifier, signature))
}

pub fn parse_function_impl(input: &mut &str) -> Result<FunctionImplementation> {
    let name = parse_identifier_lower(input)?;
    let args = delimited(
        ws('('),
        separated(0.., parse_identifier_lower.map(ToOwned::to_owned), ws(',')),
        ws(')'),
    )
    .context(winnow::error::StrContext::Label("arguments"))
    .parse_next(input)?;
    let body = alt((
        parse_function_body_single_line,
        parse_block.map(FunctionBody::MultiLine),
    ))
    .context(winnow::error::StrContext::Label("body"))
    .parse_next(input)?;

    Ok(FunctionImplementation::new(name, args, body))
}

fn parse_function_body_single_line(input: &mut &str) -> Result<FunctionBody> {
    let _ = ws('=').parse_next(input)?;
    let body = terminated(ws(parse_expr), ';').parse_next(input)?;
    Ok(FunctionBody::SingleLine(body))
}

#[cfg(test)]
mod tests {
    use ast::{
        expressions::{Expression, Literal},
        statements::{Block, Statement},
        types::{FunctionSignature, Type},
    };

    use super::*;

    #[test]
    fn test_parse_function_definition() {
        let mut input = "my_function(U8, U8) -> U8;";
        let function = parse_function_definition(&mut input).unwrap();
        assert!(input.is_empty());
        assert_eq!(function.name(), "my_function");
        assert_eq!(
            function.signature(),
            &FunctionSignature::new(vec![Type::U8, Type::U8], Type::U8)
        );
    }

    #[test]
    fn test_parse_basic_function_impl() {
        let mut input = "my_function(_x) = ();";
        let function_impl = parse_function_impl(&mut input).unwrap();

        assert_eq!(function_impl.name(), "my_function");
        assert_eq!(function_impl.arguments(), &["_x"]);
        assert_eq!(
            function_impl.body(),
            &FunctionBody::SingleLine(Expression::void())
        );
    }

    #[test]
    fn test_parse_function_impl() {
        let mut input = "my_function(_x, _y) = 1;";
        let function_impl = parse_function_impl(&mut input).unwrap();
        assert!(input.is_empty());
        assert_eq!(function_impl.name(), "my_function");
        assert_eq!(function_impl.arguments(), &["_x", "_y"]);
        assert_eq!(
            function_impl.body(),
            &FunctionBody::SingleLine(Expression::literal(Literal::int(1)))
        );
    }

    #[test]
    fn test_parse_multiline_function_impl() {
        let mut input = "my_function(_x, _y) {
              _z = 1;
            _z
        }";

        let expected_statements = vec![Statement::Assignment(
            "_z".to_owned(),
            Expression::literal(Literal::int(1)),
        )];

        let function_impl =
            parse_function_impl(&mut input).expect("Failed to parse multiline function");
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
    fn test_parse_function_signature() {
        let mut input = "(A, U8) -> Bool";
        let expected = FunctionSignature::new(
            vec![Type::Custom("A".to_owned(), vec![]), Type::U8],
            Type::Bool,
        );

        let parsed = parse_function_signature(&mut input).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_higher_order_function_signature() {
        let mut input = "(f: ((U8) -> U8), n: U8) -> U8";
        let expected = FunctionSignature::new(
            vec![
                Type::Function(FunctionSignature::new(vec![Type::U8], Type::U8)),
                Type::U8,
            ],
            Type::U8,
        );

        let parsed = parse_function_signature(&mut input).unwrap();
        assert_eq!(parsed, expected);
    }
}
