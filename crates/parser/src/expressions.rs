use ast::expressions::{BinaryOp, Expression, Literal, MatchArm, MatchBody, UnaryOp};
use winnow::{
    Parser, Result,
    ascii::{alphanumeric0, dec_int, dec_uint, multispace1},
    combinator::{alt, delimited, opt, separated, separated_pair, terminated},
    error::ContextError,
};

use crate::{
    identifiers::{parse_identifier_lower, parse_identifier_upper},
    statements::parse_block,
    ws,
};

/// TODO: Find best order
pub fn parse_expr(input: &mut &str) -> Result<Expression> {
    alt((
        parse_enum_instance,
        parse_new_type_instance,
        parse_match,
        parse_binary_op,
        parse_unary_op,
        parse_literal,
        parse_field_access,
        parse_unit,
        parse_function_call,
        parse_identifier_expr,
    ))
    .parse_next(input)
}

fn parse_literal(input: &mut &str) -> Result<Expression> {
    alt((
        parse_bool.map(Literal::Bool),
        parse_string.map(Literal::String),
        dec_uint::<&str, u32, ContextError>.map(Literal::int),
        dec_int::<&str, i32, ContextError>.map(Literal::int),
    ))
    .map(Expression::literal)
    .parse_next(input)
}

fn parse_identifier_expr(input: &mut &str) -> Result<Expression> {
    parse_identifier_lower
        .parse_next(input)
        .map(|id| Expression::identifier(id.to_owned()))
}

fn parse_unit(input: &mut &str) -> Result<Expression> {
    "Unit".parse_next(input).map(|_| Expression::unit())
}

// TODO: Escaped strings and so on
fn parse_string(input: &mut &str) -> Result<String> {
    delimited('"', alphanumeric0, '"')
        .parse_next(input)
        .map(|s| s.to_owned())
}

fn parse_bool(input: &mut &str) -> Result<bool> {
    alt(("True", "False")).parse_next(input).map(|p| match p {
        "True" => true,
        "False" => false,
        _ => unreachable!(),
    })
}

/// Expression for creating a new instance of an enum
/// Example: `MyType.Variant`
/// Example: `MyType.Variant(1, 2)`
fn parse_enum_instance(input: &mut &str) -> Result<Expression> {
    let ty = parse_identifier_upper(input)?;
    let _ = '.'.parse_next(input)?;
    let variant = parse_identifier_upper(input)?;
    let args = parse_variant_args(input)?;

    Ok(Expression::new_enum_instance(
        ty.to_owned(),
        variant.to_owned(),
        args,
    ))
}

fn parse_variant_args(input: &mut &str) -> Result<Vec<Expression>> {
    opt(delimited('(', separated(1.., parse_expr, ws(',')), ')'))
        .map(|r| r.unwrap_or_default())
        .parse_next(input)
}

/// Example: `MyType { field1: 1, field2: 2 }`
fn parse_new_type_instance(input: &mut &str) -> Result<Expression> {
    let ty = parse_identifier_upper(input)?;
    let fields = parse_fields(input)?;
    Ok(Expression::new_record_instance(ty.to_owned(), fields))
}

fn parse_fields(input: &mut &str) -> Result<Vec<(String, Expression)>> {
    delimited(
        ws('{'),
        separated(
            1..,
            separated_pair(
                parse_identifier_lower.map(str::to_owned),
                ws(':'),
                parse_expr,
            ),
            ws(','),
        ),
        ws('}'),
    )
    .parse_next(input)
}

/// example: `my_val.some_field`
fn parse_field_access(input: &mut &str) -> Result<Expression> {
    let (name, field) =
        separated_pair(parse_identifier_lower, '.', parse_identifier_lower).parse_next(input)?;
    Ok(Expression::record_access(name.to_owned(), field.to_owned()))
}

fn parse_function_call(input: &mut &str) -> Result<Expression> {
    let function_name = parse_identifier_lower(input)?;
    let args = parse_function_call_args(input)?;
    Ok(Expression::function_call(function_name.to_owned(), args))
}

/// Expressions separated by spaces, optionally between parentheses
/// This is horrible, investigate a better way to do it
///
/// expr -> is function call?
/// - yes: nested function calls must go between parenthesis
/// - no: function call doesnt need to go between parenthesis
fn parse_function_call_args(input: &mut &str) -> Result<Vec<Expression>> {
    delimited(
        '(',
        separated(
            0..,
            alt((
                parse_enum_instance,
                parse_new_type_instance,
                parse_literal,
                // parse_match,
                parse_unit,
                parse_function_call,
                parse_identifier_expr,
            )),
            ws(','),
        ),
        ')',
    )
    .parse_next(input)
}

fn parse_match(input: &mut &str) -> Result<Expression> {
    let _ = terminated("match", multispace1).parse_next(input)?;
    let pat = parse_expr(input)?;
    let cases =
        delimited(ws('{'), separated(0.., parse_match_arm, ws(',')), ws('}')).parse_next(input)?;
    Ok(Expression::match_expr(pat, cases))
}

fn parse_match_arm(input: &mut &str) -> Result<MatchArm> {
    let (pattern, body) =
        separated_pair(parse_expr, ws("->"), parse_match_body).parse_next(input)?;

    Ok(MatchArm::new(pattern, body))
}

fn parse_match_body(input: &mut &str) -> Result<MatchBody> {
    alt((
        parse_expr.map(MatchBody::Expr),
        parse_block.map(MatchBody::Block),
    ))
    .parse_next(input)
}

// /// Kinda same problem as `parse_function_call`
fn parse_binary_op(input: &mut &str) -> Result<Expression> {
    let left = alt((
        parse_literal,
        parse_identifier_expr,
        delimited('(', parse_function_call, ')'),
        delimited('(', parse_binary_op, ')'),
    ))
    .parse_next(input)?;
    let op = ws(parse_binary_operator).parse_next(input)?;
    let right = parse_expr(input)?;

    Ok(Expression::binary_op(left, op, right))
}

// TODO: Can probably use sth other than alt, dispatch?
fn parse_binary_operator(input: &mut &str) -> Result<BinaryOp> {
    alt((
        '+'.map(|_| BinaryOp::Add),
        '-'.map(|_| BinaryOp::Sub),
        '*'.map(|_| BinaryOp::Mul),
        '/'.map(|_| BinaryOp::Div),
        '%'.map(|_| BinaryOp::Mod),
        "&&".map(|_| BinaryOp::And),
        "||".map(|_| BinaryOp::Or),
        "==".map(|_| BinaryOp::Eq),
        "!=".map(|_| BinaryOp::NotEq),
        '>'.map(|_| BinaryOp::Greater),
        ">=".map(|_| BinaryOp::GreaterEq),
        '<'.map(|_| BinaryOp::Less),
        "<=".map(|_| BinaryOp::LessEq),
    ))
    .parse_next(input)
}

fn parse_unary_op(input: &mut &str) -> Result<Expression> {
    (
        parse_unary_operator,
        alt((parse_literal, parse_identifier_expr)),
    )
        .map(|(op, expr)| Expression::unary_op(op, expr))
        .parse_next(input)
}

// /// TODO: Add more operators (?)
fn parse_unary_operator(input: &mut &str) -> Result<UnaryOp> {
    '!'.map(|_| UnaryOp::Negate).parse_next(input)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_literal_bool() {
        let mut input = "True";
        let expr = parse_expr(&mut input).unwrap();
        assert_eq!(expr, Expression::literal(Literal::Bool(true)));
    }

    #[test]
    fn test_parse_literal_u8() {
        let mut input = "37";
        let expr = parse_expr(&mut input).unwrap();
        assert_eq!(expr, Expression::literal(Literal::int(37)));
    }

    #[test]
    fn test_parse_literal_binary() {
        let mut input = "0b110";
        let expr = parse_expr(&mut input).unwrap();
        assert_eq!(expr, Expression::literal(Literal::int(6)));
    }

    #[test]
    fn test_parse_literal_octal() {
        let mut input = "0o20";
        let expr = parse_expr(&mut input).unwrap();
        assert_eq!(expr, Expression::literal(Literal::int(16)));
    }

    #[test]
    fn test_parse_literal_hexadecimal() {
        let mut input = "0x20";
        let expr = parse_expr(&mut input).unwrap();
        assert_eq!(expr, Expression::literal(Literal::int(32)));
    }

    #[test]
    fn test_parse_literal_f32() {
        let mut input = "0.12";
        let expr = parse_expr(&mut input).unwrap();
        assert_eq!(expr, Expression::literal(Literal::float(0.12)));
    }

    #[test]
    fn test_parse_literal_i8() {
        let mut input = "-37";
        let expr = parse_expr(&mut input).unwrap();
        assert_eq!(expr, Expression::literal(Literal::int(-37)));
    }

    #[test]
    fn test_parse_string() {
        let mut input = r#""hello, \"world\"""#;
        let expr = parse_expr(&mut input);
        eprintln!("{expr:?}");
        assert_eq!(
            expr.unwrap(),
            Expression::literal(Literal::String("hello, \"world\"".to_owned()))
        );
    }

    #[test]
    fn test_parse_identifier() {
        let mut input = "my_var";
        let expr = parse_expr(&mut input).unwrap();

        assert_eq!(expr, Expression::identifier("my_var".to_owned()));
    }

    #[test]
    fn test_parse_unit() {
        let mut input = "Unit";
        let expr = parse_expr(&mut input).unwrap();

        assert_eq!(expr, Expression::unit());
    }

    #[test]
    fn test_parse_new_enum_instance() {
        let mut input = "Option.None";
        let parsed = parse_expr(&mut input).unwrap();
        assert_eq!(
            parsed,
            Expression::new_enum_instance("Option".to_owned(), "None".to_owned(), vec![])
        );
    }

    #[test]
    fn test_parse_new_enum_instance_with_args() {
        let mut input = "Option.Some(1)";
        let parsed = parse_expr(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {}", input);
        assert_eq!(
            parsed,
            Expression::new_enum_instance(
                "Option".to_owned(),
                "Some".to_owned(),
                vec![Expression::literal(Literal::int(1)),]
            ),
        );
    }

    #[test]
    fn test_parse_new_record_instance_with_fields() {
        let mut input = "Person { name: \"Bob\", is_builder: True }";
        let parsed = parse_expr(&mut input);
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(
            parsed.unwrap(),
            Expression::new_record_instance(
                "Person".to_owned(),
                vec![
                    (
                        "name".to_owned(),
                        Expression::literal(Literal::String("Bob".to_owned()))
                    ),
                    (
                        "is_builder".to_owned(),
                        Expression::literal(Literal::Bool(true))
                    ),
                ]
            )
        );
    }

    #[test]
    fn test_parse_function_call() {
        let mut input = "my_function(arg1, arg2)";
        let parsed = parse_expr(&mut input).unwrap();
        assert!(input.is_empty());
        assert_eq!(
            parsed,
            Expression::function_call(
                "my_function".to_owned(),
                vec![
                    Expression::identifier("arg1".to_owned()),
                    Expression::identifier("arg2".to_owned()),
                ]
            )
        );
    }

    #[test]
    fn test_parse_function_call_complex() {
        let mut input = "my_function(other_fn(42), Person { name: \"Bob\" })";
        let parsed = parse_expr(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(
            parsed,
            Expression::function_call(
                "my_function".to_owned(),
                vec![
                    Expression::function_call(
                        "other_fn".to_owned(),
                        vec![Expression::literal(Literal::int(42))]
                    ),
                    Expression::new_record_instance(
                        "Person".to_owned(),
                        vec![(
                            "name".to_owned(),
                            Expression::literal(Literal::String("Bob".to_owned()))
                        )]
                    )
                ]
            )
        );
    }

    #[test]
    fn test_parse_match() {
        let mut input = "match my_bool {
            True -> 1,
            False -> 0
        }";
        let parsed = parse_expr(&mut input).unwrap();
        assert!(input.is_empty());
        assert_eq!(
            parsed,
            Expression::match_expr(
                Expression::identifier("my_bool".to_owned()),
                vec![
                    MatchArm {
                        pattern: Expression::literal(Literal::Bool(true)),
                        body: MatchBody::Expr(Expression::literal(Literal::int(1))),
                    },
                    MatchArm {
                        pattern: Expression::literal(Literal::Bool(false)),
                        body: MatchBody::Expr(Expression::literal(Literal::int(0)))
                    }
                ]
            )
        );
    }

    #[test]
    fn test_parse_match_patterns_1() {
        let mut input = "match Option.Some(x) {
            1 -> True,
            _ -> False
        }";
        let parsed = parse_expr(&mut input).unwrap();
        assert!(input.is_empty());
        assert_eq!(
            parsed,
            Expression::match_expr(
                Expression::new_enum_instance(
                    "Option".to_owned(),
                    "Some".to_owned(),
                    vec![Expression::identifier("x".to_owned())]
                ),
                vec![
                    MatchArm {
                        pattern: Expression::literal(Literal::int(1)),
                        body: MatchBody::Expr(Expression::literal(Literal::Bool(true))),
                    },
                    MatchArm {
                        pattern: Expression::identifier("_".to_owned()),
                        body: MatchBody::Expr(Expression::literal(Literal::Bool(false)))
                    }
                ]
            )
        );
    }

    // /// Important decision:
    // /// Should all custom types (included those in the stdlib) be fully qualified? ie: `Option.None`
    // /// Should all custom types (except those in the stdlib) be fully qualified? ie: Some, `MyType.Var1`
    // /// In match blocks, should the qualification be omitted for branches? ie: if the type of the expression being matched is MyType, skip `MyType.` in the branches.
    // #[test]
    // fn test_parse_match_patterns_2() {
    //     let input = "match my_option {
    //         Option.Some(x) -> {
    //             True
    //         },
    //         Option.None -> False
    //     }";
    //     let (rem, parsed) = parse_expr(input).unwrap();
    //     assert!(rem.is_empty());
    //     assert_eq!(
    //         parsed,
    //         Expression::match_expr(
    //             Expression::identifier("my_option".to_owned()),
    //             vec![
    //                 MatchArm {
    //                     pattern: Expression::new_enum_instance(
    //                         "Option".to_owned(),
    //                         "Some".to_owned(),
    //                         vec![Expression::identifier("x".to_owned())]
    //                     ),
    //                     body: MatchBody::Block(Block::new(
    //                         vec![],
    //                         Expression::literal(Literal::Bool(true))
    //                     )),
    //                 },
    //                 MatchArm {
    //                     pattern: Expression::new_enum_instance(
    //                         "Option".to_owned(),
    //                         "None".to_owned(),
    //                         vec![]
    //                     ),
    //                     body: MatchBody::Expr(Expression::literal(Literal::Bool(false)))
    //                 }
    //             ]
    //         )
    //     );
    // }

    // #[test]
    // fn test_parse_binary_op() {
    //     let input = "1 + 2";
    //     let (_, parsed) = parse_expr(input).unwrap();
    //     assert_eq!(
    //         parsed,
    //         Expression::binary_op(
    //             Expression::literal(Literal::int(1)),
    //             BinaryOp::Add,
    //             Expression::literal(Literal::int(2))
    //         )
    //     );
    // }

    // #[test]
    // fn test_parse_unary_op() {
    //     let input = "¬True";
    //     let (rem, parsed) = parse_expr(input).unwrap();
    //     assert!(rem.is_empty());
    //     assert_eq!(
    //         parsed,
    //         Expression::unary_op(UnaryOp::Negate, Expression::literal(Literal::Bool(true)))
    //     );
    // }
}
