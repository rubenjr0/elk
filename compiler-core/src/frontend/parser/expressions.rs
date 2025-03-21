use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{i32, i8, multispace1, u32, u8},
    combinator::{map, opt, value},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, separated_pair},
    IResult, Parser,
};

use crate::frontend::ast::expressions::{
    BinaryOp, Expression, Literal, MatchArm, MatchBody, UnaryOp,
};

use super::{
    common::{opt_parenthesis, parse_identifier_lower, parse_identifier_upper, ws},
    statements::parse_block,
};

/// TODO: Find best order
pub fn parse_expr(input: &str) -> IResult<&str, Expression> {
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
    .parse(input)
}

fn parse_literal(input: &str) -> IResult<&str, Expression> {
    let (remaining, lit) = alt((
        map(parse_bool, Literal::Bool),
        map(parse_string, Literal::String),
        map(u8, Literal::int),
        map(i8, Literal::int),
        map(u32, Literal::int),
        map(i32, Literal::int),
    ))
    .parse(input)?;
    Ok((remaining, Expression::literal(lit)))
}

fn parse_identifier_expr(input: &str) -> IResult<&str, Expression> {
    map(parse_identifier_lower, |id| {
        Expression::identifier(id.to_owned())
    })
    .parse(input)
}

fn parse_unit(input: &str) -> IResult<&str, Expression> {
    map(tag("Unit"), |_| Expression::unit()).parse(input)
}

fn parse_string(input: &str) -> IResult<&str, String> {
    delimited(tag("\""), in_quotes, tag("\"")).parse(input)
}

/// https://users.rust-lang.org/t/solved-nom5-parse-a-string-containing-escaped-quotes-and-delimited-by-quotes/32818/2
fn in_quotes(buf: &str) -> IResult<&str, String> {
    let mut ret = String::new();
    let mut skip_delimiter = false;
    for (i, ch) in buf.char_indices() {
        if ch == '\\' && !skip_delimiter {
            skip_delimiter = true;
        } else if ch == '"' && !skip_delimiter {
            return Ok((&buf[i..], ret));
        } else {
            ret.push(ch);
            skip_delimiter = false;
        }
    }
    Err(nom::Err::Incomplete(nom::Needed::Unknown))
}

fn parse_bool(input: &str) -> IResult<&str, bool> {
    alt((value(true, tag("True")), value(false, tag("False")))).parse(input)
}

/// Expression for creating a new instance of an enum
/// Example: `MyType.Variant`
/// Example: `MyType.Variant(1, 2)`
fn parse_enum_instance(input: &str) -> IResult<&str, Expression> {
    let (input, ty) = parse_identifier_upper(input)?;
    let (input, _) = tag(".").parse(input)?;
    let (input, variant) = parse_identifier_upper(input)?;
    let (input, args) = parse_variant_args(input)?;

    Ok((
        input,
        Expression::new_enum_instance(ty.to_owned(), variant.to_owned(), args),
    ))
}

fn parse_variant_args(input: &str) -> IResult<&str, Vec<Expression>> {
    opt(delimited(
        tag("("),
        separated_list1(ws(tag(",")), parse_expr),
        tag(")"),
    ))
    .map(Option::unwrap_or_default)
    .parse(input)
}

/// Example: `MyType { field1: 1, field2: 2 }`
fn parse_new_type_instance(input: &str) -> IResult<&str, Expression> {
    let (input, ty) = parse_identifier_upper(input)?;
    let (input, fields) = parse_fields(input)?;

    Ok((
        input,
        Expression::new_record_instance(ty.to_owned(), fields),
    ))
}

fn parse_fields(input: &str) -> IResult<&str, Vec<(String, Expression)>> {
    delimited(
        ws(tag("{")),
        separated_list0(ws(tag(",")), parse_field),
        ws(tag("}")),
    )
    .parse(input)
}

fn parse_field(input: &str) -> IResult<&str, (String, Expression)> {
    separated_pair(
        parse_identifier_lower.map(str::to_owned),
        ws(tag(":")),
        parse_expr,
    )
    .parse(input)
}

/// example: `my_val.some_field`
fn parse_field_access(input: &str) -> IResult<&str, Expression> {
    let (input, parsed) =
        separated_pair(parse_identifier_lower, tag("."), parse_identifier_lower).parse(input)?;
    Ok((
        input,
        Expression::record_access(parsed.0.to_owned(), parsed.1.to_owned()),
    ))
}

fn parse_function_call(input: &str) -> IResult<&str, Expression> {
    let (input, function_name) = parse_identifier_lower(input)?;
    let (input, args) = parse_function_args(input)?;
    Ok((
        input,
        Expression::function_call(function_name.to_owned(), args),
    ))
}

/// Expressions separated by spaces, optionally between parentheses
/// This is horrible, investigate a better way to do it
///
/// expr -> is function call?
/// - yes: nested function calls must go between parenthesis
/// - no: function call doesnt need to go between parenthesis
fn parse_function_args(input: &str) -> IResult<&str, Vec<Expression>> {
    ws(separated_list1(
        multispace1,
        alt((
            opt_parenthesis(parse_enum_instance),
            opt_parenthesis(parse_new_type_instance),
            parse_literal,
            parse_match,
            parse_unit,
            delimited(tag("("), parse_function_call, tag(")")),
            parse_identifier_expr,
        )),
    ))
    .parse(input)
}

fn parse_match(input: &str) -> IResult<&str, Expression> {
    let (input, _) = tag("match").parse(input)?;
    let (input, pat) = parse_expr(input)?;
    let (input, _) = ws(tag("{")).parse(input)?;
    let (input, cases) = separated_list0(ws(tag(",")), parse_match_arm).parse(input)?;
    let (input, _) = ws(tag("}")).parse(input)?;

    Ok((input, Expression::match_expr(pat, cases)))
}

fn parse_match_arm(input: &str) -> IResult<&str, MatchArm> {
    let (input, pattern) = parse_expr(input)?;
    let (input, _) = ws(tag("->")).parse(input)?;
    let (input, body) = parse_match_body(input)?;

    Ok((input, MatchArm::new(pattern, body)))
}

fn parse_match_body(input: &str) -> IResult<&str, MatchBody> {
    alt((
        map(parse_expr, MatchBody::Expr),
        map(parse_block, MatchBody::Block),
    ))
    .parse(input)
}

/// Kinda same problem as `parse_function_call`
fn parse_binary_op(input: &str) -> IResult<&str, Expression> {
    let (input, left) = alt((
        parse_literal,
        parse_identifier_expr,
        delimited(tag("("), parse_function_call, tag(")")),
        delimited(tag("("), parse_binary_op, tag(")")),
    ))
    .parse(input)?;
    let (input, op) = ws(parse_binary_operator).parse(input)?;
    let (input, right) = parse_expr(input)?;

    Ok((input, Expression::binary_op(left, op, right)))
}

fn parse_binary_operator(input: &str) -> IResult<&str, BinaryOp> {
    alt((
        value(BinaryOp::Add, tag("+")),
        value(BinaryOp::Sub, tag("-")),
        value(BinaryOp::Mul, tag("*")),
        value(BinaryOp::Div, tag("/")),
        value(BinaryOp::Mod, tag("%")),
        value(BinaryOp::And, tag("&&")),
        value(BinaryOp::Or, tag("||")),
        value(BinaryOp::Eq, tag("==")),
        value(BinaryOp::NotEq, tag("!=")),
        value(BinaryOp::Greater, tag(">")),
        value(BinaryOp::GreaterEq, tag(">=")),
        value(BinaryOp::Less, tag("<")),
        value(BinaryOp::LessEq, tag("<=")),
    ))
    .parse(input)
}

fn parse_unary_op(input: &str) -> IResult<&str, Expression> {
    let (input, op) = parse_unary_operator(input)?;
    let (input, operand) = alt((parse_literal, parse_identifier_expr)).parse(input)?;

    Ok((input, Expression::unary_op(op, operand)))
}

/// TODO: Add more operators (?)
fn parse_unary_operator(input: &str) -> IResult<&str, UnaryOp> {
    value(UnaryOp::Negate, tag("¬")).parse(input)
}

#[cfg(test)]
mod tests {
    use crate::frontend::ast::statements::Block;

    use super::*;

    #[test]
    fn test_parse_literal_bool() {
        let input = "True";
        let (_, expr) = parse_expr(input).unwrap();
        assert_eq!(expr, Expression::literal(Literal::Bool(true)));
    }

    #[test]
    fn test_parse_literal_u8() {
        let input = "37";
        let (_, expr) = parse_expr(input).unwrap();
        assert_eq!(expr, Expression::literal(Literal::int(37)));
    }

    #[test]
    fn test_parse_literal_binary() {
        let input = "0b110";
        let (_, expr) = parse_expr(input).unwrap();
        assert_eq!(expr, Expression::literal(Literal::int(6)));
    }

    #[test]
    fn test_parse_literal_octal() {
        let input = "0o20";
        let (_, expr) = parse_expr(input).unwrap();
        assert_eq!(expr, Expression::literal(Literal::int(16)));
    }

    #[test]
    fn test_parse_literal_hexadecimal() {
        let input = "0x20";
        let (_, expr) = parse_expr(input).unwrap();
        assert_eq!(expr, Expression::literal(Literal::int(32)));
    }

    #[test]
    fn test_parse_literal_f32() {
        let input = "0.12";
        let (_, expr) = parse_expr(input).unwrap();
        assert_eq!(expr, Expression::literal(Literal::float(0.12)));
    }

    #[test]
    fn test_parse_literal_i8() {
        let input = "-37";
        let (_, expr) = parse_expr(input).unwrap();
        assert_eq!(expr, Expression::literal(Literal::int(-37)));
    }

    #[test]
    fn test_parse_string() {
        let input = r#""hello, \"world\"""#;
        let (_, expr) = parse_expr(input).unwrap();

        assert_eq!(
            expr,
            Expression::literal(Literal::String("hello, \"world\"".to_owned()))
        );
    }

    #[test]
    fn test_parse_identifier() {
        let input = "my_var";
        let (_, expr) = parse_expr(input).unwrap();

        assert_eq!(expr, Expression::identifier("my_var".to_owned()));
    }

    #[test]
    fn test_parse_unit() {
        let input = "Unit";
        let (_, expr) = parse_expr(input).unwrap();

        assert_eq!(expr, Expression::unit());
    }

    #[test]
    fn test_parse_new_enum_instance() {
        let input = "Option.None";
        let (_, parsed) = parse_expr(input).unwrap();
        assert_eq!(
            parsed,
            Expression::new_enum_instance("Option".to_owned(), "None".to_owned(), vec![])
        );
    }

    #[test]
    fn test_parse_new_enum_instance_with_args() {
        let input = "Option.Some(1)";
        let (rem, parsed) = parse_expr(input).unwrap();
        assert!(rem.is_empty());
        assert_eq!(
            parsed,
            Expression::new_enum_instance(
                "Option".to_owned(),
                "Some".to_owned(),
                vec![Expression::literal(Literal::int(1)),]
            )
        );
    }

    #[test]
    fn test_parse_new_record_instance_with_fields() {
        let input = "Person { name: \"Bob\", is_builder: True }";
        let (_, parsed) = parse_expr(input).unwrap();
        assert_eq!(
            parsed,
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
        let input = "my_function arg1 arg2";
        let (_, parsed) = parse_expr(input).unwrap();
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
        let input = "my_function (other_fn 42) (Person { name: \"Bob\" })";
        let (_, parsed) = parse_expr(input).unwrap();
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
    fn test_parse_function_call_complex2() {
        let input = "my_function (other_fn 42)";
        let (_, parsed) = parse_expr(input).unwrap();
        assert_eq!(
            parsed,
            Expression::function_call(
                "my_function".to_owned(),
                vec![Expression::function_call(
                    "other_fn".to_owned(),
                    vec![Expression::literal(Literal::int(42))]
                ),]
            )
        );
    }

    #[test]
    fn test_parse_match() {
        let input = "match my_bool {
            True -> 1,
            False -> 0
        }";
        let (rem, parsed) = parse_expr(input).unwrap();
        assert!(rem.is_empty());
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
        let input = "match Option.Some(x) {
            1 -> True,
            _ -> False
        }";
        let (rem, parsed) = parse_expr(input).unwrap();
        assert!(rem.is_empty());
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

    /// Important decision:
    /// Should all custom types (included those in the stdlib) be fully qualified? ie: `Option.None`
    /// Should all custom types (except those in the stdlib) be fully qualified? ie: Some, `MyType.Var1`
    /// In match blocks, should the qualification be omitted for branches? ie: if the type of the expression being matched is MyType, skip `MyType.` in the branches.
    #[test]
    fn test_parse_match_patterns_2() {
        let input = "match my_option {
            Option.Some(x) -> {
                True
            },
            Option.None -> False
        }";
        let (rem, parsed) = parse_expr(input).unwrap();
        assert!(rem.is_empty());
        assert_eq!(
            parsed,
            Expression::match_expr(
                Expression::identifier("my_option".to_owned()),
                vec![
                    MatchArm {
                        pattern: Expression::new_enum_instance(
                            "Option".to_owned(),
                            "Some".to_owned(),
                            vec![Expression::identifier("x".to_owned())]
                        ),
                        body: MatchBody::Block(Block::new(
                            vec![],
                            Expression::literal(Literal::Bool(true))
                        )),
                    },
                    MatchArm {
                        pattern: Expression::new_enum_instance(
                            "Option".to_owned(),
                            "None".to_owned(),
                            vec![]
                        ),
                        body: MatchBody::Expr(Expression::literal(Literal::Bool(false)))
                    }
                ]
            )
        );
    }

    #[test]
    fn test_parse_binary_op() {
        let input = "1 + 2";
        let (_, parsed) = parse_expr(input).unwrap();
        assert_eq!(
            parsed,
            Expression::binary_op(
                Expression::literal(Literal::int(1)),
                BinaryOp::Add,
                Expression::literal(Literal::int(2))
            )
        );
    }

    #[test]
    fn test_parse_unary_op() {
        let input = "¬True";
        let (rem, parsed) = parse_expr(input).unwrap();
        assert!(rem.is_empty());
        assert_eq!(
            parsed,
            Expression::unary_op(UnaryOp::Negate, Expression::literal(Literal::Bool(true)))
        );
    }
}
