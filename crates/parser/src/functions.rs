use ast::{
    functions::{FunctionBody, FunctionDeclaration, FunctionImplementation, QualifiedName},
    types::{FunctionSignature, Type},
};
use winnow::{
    Parser, Result,
    combinator::{alt, delimited, opt, separated, separated_pair, terminated},
    error::{StrContext, StrContextValue},
};

use crate::{
    custom_types::parse_custom_type_generics,
    expressions::parse_expr,
    identifiers::{parse_identifier_lower, parse_identifier_upper},
    patterns::parse_pattern,
    statements::parse_block,
    types::{parse_custom_type, parse_primitive_type, parse_type},
    ws,
};

/// A qualified function name: `map` or `Option::map`.
///
/// Grammar reference: `QualifiedName = (TypeIdentifier "::")? Identifier`
fn parse_qualified_name(input: &mut &str) -> Result<QualifiedName> {
    let namespace = opt(terminated(parse_identifier_upper, "::")).parse_next(input)?;
    let name = parse_identifier_lower(input)?;
    Ok(QualifiedName::new(namespace.map(str::to_owned), name))
}

/// Types separated by commas. The last type is the return type and it's separated by an arrow.
/// If parenthesis are encountered, its a function signature, parse recursively.
/// Example: `(U8, U8) -> U8`
/// Example: `(((U8) -> Bool), U8) -> Bool`
pub fn parse_function_signature(input: &mut &str) -> Result<FunctionSignature> {
    let (args, out) = separated_pair(
        delimited(
            ws('('),
            separated(
                0..,
                (
                    opt(terminated(parse_identifier_lower, ws(':')))
                        .context(StrContext::Label("ArgLabel")),
                    alt((
                        parse_primitive_type,
                        parse_custom_type,
                        // Bare function type: `(A) -> B`
                        parse_function_signature
                            .map(Type::Function)
                            .context(StrContext::Label("sub-function")),
                        // Parenthesized function type: `((A) -> B)` — the outer
                        // parens scope it when a bare parse would run into the
                        // next parameter's `,`.
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
    let name = parse_qualified_name(input)?;
    let type_params = opt(parse_custom_type_generics)
        .parse_next(input)?
        .unwrap_or_default();
    let signature = parse_function_signature
        .context(StrContext::Label("signature"))
        .parse_next(input)?;
    let _ = ';'
        .context(StrContext::Expected(StrContextValue::CharLiteral(';')))
        .parse_next(input)?;

    Ok(FunctionDeclaration::new(name, type_params, signature))
}

pub fn parse_function_impl(input: &mut &str) -> Result<FunctionImplementation> {
    let name = parse_qualified_name(input)?;
    let args = delimited(
        ws('('),
        separated(0.., ws(parse_pattern), ws(',')),
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
        patterns::Pattern,
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
        assert_eq!(function_impl.arguments(), &[Pattern::Identifier("_x".to_owned())]);
        assert_eq!(
            function_impl.body(),
            &FunctionBody::SingleLine(Expression::unit())
        );
    }

    #[test]
    fn test_parse_function_impl() {
        let mut input = "my_function(_x, _y) = 1;";
        let function_impl = parse_function_impl(&mut input).unwrap();
        assert!(input.is_empty());
        assert_eq!(function_impl.name(), "my_function");
        assert_eq!(
            function_impl.arguments(),
            &[
                Pattern::Identifier("_x".to_owned()),
                Pattern::Identifier("_y".to_owned())
            ]
        );
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
        assert_eq!(
            function_impl.arguments(),
            &[
                Pattern::Identifier("_x".to_owned()),
                Pattern::Identifier("_y".to_owned())
            ]
        );
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

#[cfg(test)]
mod regression_tests {
    use super::*;
    use ast::patterns::Pattern;

    #[test]
    fn test_parse_nullary_function_signature() {
        // Regression: `() -> U8` used to fail (args required 1+).
        let mut input = "() -> U8";
        let parsed = parse_function_signature(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(parsed, FunctionSignature::new(vec![], Type::U8));
    }

    #[test]
    fn test_parse_namespaced_function_definition() {
        let mut input = "Option::is_some(Self<A>) -> Bool;";
        let def = parse_function_definition(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(def.name(), "is_some");
        assert_eq!(def.qualified_name().namespace.as_deref(), Some("Option"));
        assert_eq!(def.qualified_name().qualified(), "Option::is_some");
    }

    #[test]
    fn test_parse_function_definition_with_type_params() {
        let mut input = "Option::map<A, B>(Self<A>, (A) -> B) -> Self<B>;";
        let def = parse_function_definition(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(def.type_params(), &["A".to_owned(), "B".to_owned()]);
        assert_eq!(
            def.signature(),
            &FunctionSignature::new(
                vec![
                    Type::Custom("Self".to_owned(), vec!["A".to_owned()]),
                    Type::Function(FunctionSignature::new(
                        vec![Type::Custom("A".to_owned(), vec![])],
                        Type::Custom("B".to_owned(), vec![])
                    )),
                ],
                Type::Custom("Self".to_owned(), vec!["B".to_owned()])
            )
        );
    }

    #[test]
    fn test_parse_pattern_matched_impl() {
        let mut input = "Option::is_some(Option::None) = False;";
        let imp = parse_function_impl(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(imp.qualified_name().qualified(), "Option::is_some");
        assert_eq!(
            imp.arguments(),
            &[Pattern::EnumInstance {
                enum_name: "Option".to_owned(),
                variant_name: "None".to_owned(),
                args: vec![],
            }]
        );
    }

    #[test]
    fn test_parse_pattern_matched_impl_with_binding() {
        let mut input = "Option::unwrap(Option::Some(x)) = x;";
        let imp = parse_function_impl(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(
            imp.arguments(),
            &[Pattern::EnumInstance {
                enum_name: "Option".to_owned(),
                variant_name: "Some".to_owned(),
                args: vec![Pattern::Identifier("x".to_owned())],
            }]
        );
    }
}

