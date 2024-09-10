use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{i8, multispace1, u8},
    combinator::{map, opt, value},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, separated_pair},
    IResult, Parser,
};

use crate::ast::expressions::{BinaryOp, Expr, Literal, MatchArm, MatchBody, Pattern};

use super::common::{opt_parenthesis, parse_identifier_lower, parse_identifier_upper, ws};

pub fn parse_expr(input: &str) -> IResult<&str, Expr> {
    alt((
        parse_variant,
        parse_new_type_instance,
        parse_match,
        parse_binary_op,
        parse_literal,
        parse_unit,
        parse_function_call,
        parse_identifier,
    ))
    .parse(input)
}

fn parse_literal(input: &str) -> IResult<&str, Expr> {
    let (remaining, lit) = alt((
        map(parse_bool, Literal::Bool),
        map(parse_string, Literal::String),
        map(u8, Literal::U8),
        map(i8, Literal::I8),
    ))
    .parse(input)?;
    Ok((remaining, Expr::Literal(lit)))
}

fn parse_identifier(input: &str) -> IResult<&str, Expr> {
    map(parse_identifier_lower, |id| {
        Expr::Identifier(id.to_string())
    })
    .parse(input)
}

fn parse_unit(input: &str) -> IResult<&str, Expr> {
    map(tag("Unit"), |_| Expr::Unit).parse(input)
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

/// Expression for creating a new instance of a type with variants
/// Example: `MyType.Variant`
/// Example: `MyType.Variant(1, 2)`
fn parse_variant(input: &str) -> IResult<&str, Expr> {
    let (input, ty) = parse_identifier_upper(input)?;
    let (input, _) = tag(".").parse(input)?;
    let (input, variant) = parse_identifier_upper(input)?;
    let (input, args) = parse_variant_args(input)?;

    Ok((
        input,
        Expr::NewVariant(ty.to_string(), variant.to_string(), args),
    ))
}

fn parse_variant_args(input: &str) -> IResult<&str, Vec<Expr>> {
    opt(delimited(
        tag("("),
        separated_list1(ws(tag(",")), parse_expr),
        tag(")"),
    ))
    .map(Option::unwrap_or_default)
    .parse(input)
}

/// Example: `MyType { field1 = 1, field2 = 2 }`
fn parse_new_type_instance(input: &str) -> IResult<&str, Expr> {
    let (input, ty) = parse_identifier_upper(input)?;
    let (input, fields) = parse_fields(input)?;

    Ok((input, Expr::NewTypeInstance(ty.to_string(), fields)))
}

fn parse_fields(input: &str) -> IResult<&str, Vec<(String, Expr)>> {
    delimited(
        ws(tag("{")),
        separated_list0(ws(tag(",")), parse_field),
        ws(tag("}")),
    )
    .parse(input)
}

fn parse_field(input: &str) -> IResult<&str, (String, Expr)> {
    separated_pair(
        parse_identifier_lower.map(str::to_string),
        ws(tag("=")),
        parse_expr,
    )
    .parse(input)
}

fn parse_function_call(input: &str) -> IResult<&str, Expr> {
    let (input, function_name) = parse_identifier_lower(input)?;
    let (input, args) = parse_function_args(input)?;
    Ok((input, Expr::FunctionCall(function_name.to_string(), args)))
}

/// Expressions separated by spaces, optionally between parentheses
/// This is horrible, investigate a better way to do it
///
/// expr -> is function call?
/// - yes: nested function calls must go between parenthesis
/// - no: function call doesnt need to go between parenthesis
fn parse_function_args(input: &str) -> IResult<&str, Vec<Expr>> {
    ws(separated_list1(
        multispace1,
        alt((
            opt_parenthesis(parse_variant),
            opt_parenthesis(parse_new_type_instance),
            parse_literal,
            parse_match,
            parse_unit,
            delimited(tag("("), parse_function_call, tag(")")),
            parse_identifier,
        )),
    ))
    .parse(input)
}

fn parse_match(input: &str) -> IResult<&str, Expr> {
    let (input, _) = tag("match").parse(input)?;
    let (input, pat) = parse_pattern(input)?;
    let (input, _) = ws(tag("{")).parse(input)?;
    let (input, cases) = separated_list0(ws(tag(",")), parse_match_arm).parse(input)?;
    let (input, _) = ws(tag("}")).parse(input)?;

    Ok((input, Expr::Match(pat, cases)))
}

fn parse_pattern(input: &str) -> IResult<&str, Pattern> {
    let (input, expr) = parse_expr(input)?;
    let pat = Pattern::try_from(expr).expect("Invalid pattern");
    Ok((input, pat))
}

fn parse_match_arm(input: &str) -> IResult<&str, MatchArm> {
    let (input, pattern) = parse_pattern(input)?;
    let (input, _) = ws(tag("->")).parse(input)?;
    let (input, body) = parse_match_body(input)?;

    Ok((input, MatchArm { pattern, body }))
}

fn parse_match_body(input: &str) -> IResult<&str, MatchBody> {
    alt((
        map(parse_expr, MatchBody::Expr),
        // map(parse_block, MatchBody::Block),
    ))
    .parse(input)
}

/// Kinda same problem as `parse_function_call`
fn parse_binary_op(input: &str) -> IResult<&str, Expr> {
    let (input, left) = alt((
        parse_literal,
        parse_identifier,
        delimited(tag("("), parse_function_call, tag(")")),
        delimited(tag("("), parse_binary_op, tag(")")),
    ))
    .parse(input)?;
    let (input, op) = ws(parse_binary_operator).parse(input)?;
    let (input, right) = parse_expr(input)?;

    Ok((input, Expr::BinaryOp(Box::new(left), op, Box::new(right))))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_literal_bool() {
        let input = "True";
        let (_, expr) = parse_expr(input).unwrap();
        assert_eq!(expr, Expr::Literal(Literal::Bool(true)));
    }

    #[test]
    fn test_parse_literal_u8() {
        let input = "37";
        let (_, expr) = parse_expr(input).unwrap();
        assert_eq!(expr, Expr::Literal(Literal::U8(37)));
    }

    #[test]
    fn test_parse_literal_i8() {
        let input = "-37";
        let (_, expr) = parse_expr(input).unwrap();
        assert_eq!(expr, Expr::Literal(Literal::I8(-37)));
    }

    #[test]
    fn test_parse_string() {
        let input = r#""hello, \"world\"""#;
        let (_, expr) = parse_expr(input).unwrap();

        assert_eq!(
            expr,
            Expr::Literal(Literal::String("hello, \"world\"".to_string()))
        );
    }

    #[test]
    fn test_parse_identifier() {
        let input = "my_var";
        let (_, expr) = parse_expr(input).unwrap();

        assert_eq!(expr, Expr::Identifier("my_var".to_string()));
    }

    #[test]
    fn test_parse_unit() {
        let input = "Unit";
        let (_, expr) = parse_expr(input).unwrap();

        assert_eq!(expr, Expr::Unit);
    }

    #[test]
    fn test_parse_new_variant() {
        let input = "Option.None";
        let (_, parsed) = parse_expr(input).unwrap();
        assert_eq!(
            parsed,
            Expr::NewVariant("Option".to_string(), "None".to_string(), vec![])
        );
    }

    #[test]
    fn test_parse_new_variant_with_args() {
        let input = "Option.Some(1, some_id)";
        let (rem, parsed) = parse_expr(input).unwrap();
        assert!(rem.is_empty());
        assert_eq!(
            parsed,
            Expr::NewVariant(
                "Option".to_string(),
                "Some".to_string(),
                vec![
                    Expr::Literal(Literal::U8(1)),
                    Expr::Identifier("some_id".to_string()),
                ]
            )
        );
    }

    #[test]
    fn test_parse_new_type_instance_with_fields() {
        let input = "Person { name = \"Bob\", is_builder = True }";
        let (_, parsed) = parse_expr(input).unwrap();
        assert_eq!(
            parsed,
            Expr::NewTypeInstance(
                "Person".to_string(),
                vec![
                    (
                        "name".to_string(),
                        Expr::Literal(Literal::String("Bob".to_string()))
                    ),
                    ("is_builder".to_string(), Expr::Literal(Literal::Bool(true))),
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
            Expr::FunctionCall(
                "my_function".to_string(),
                vec![
                    Expr::Identifier("arg1".to_string()),
                    Expr::Identifier("arg2".to_string()),
                ]
            )
        );
    }

    #[test]
    fn test_parse_function_call_complex() {
        let input = "my_function (other_fn 42) (Person { name = \"Bob\" })";
        let (_, parsed) = parse_expr(input).unwrap();
        assert_eq!(
            parsed,
            Expr::FunctionCall(
                "my_function".to_string(),
                vec![
                    Expr::FunctionCall(
                        "other_fn".to_string(),
                        vec![Expr::Literal(Literal::U8(42))]
                    ),
                    Expr::NewTypeInstance(
                        "Person".to_string(),
                        vec![(
                            "name".to_string(),
                            Expr::Literal(Literal::String("Bob".to_string()))
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
            Expr::FunctionCall(
                "my_function".to_string(),
                vec![Expr::FunctionCall(
                    "other_fn".to_string(),
                    vec![Expr::Literal(Literal::U8(42))]
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
            Expr::Match(
                Pattern::Identifier("my_bool".to_string()),
                vec![
                    MatchArm {
                        pattern: Pattern::Literal(Literal::Bool(true)),
                        body: MatchBody::Expr(Expr::Literal(Literal::U8(1))),
                    },
                    MatchArm {
                        pattern: Pattern::Literal(Literal::Bool(false)),
                        body: MatchBody::Expr(Expr::Literal(Literal::U8(0)))
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
            Expr::Match(
                Pattern::Variant(
                    "Option".to_string(),
                    "Some".to_string(),
                    vec![Pattern::Identifier("x".to_string())]
                ),
                vec![
                    MatchArm {
                        pattern: Pattern::Literal(Literal::U8(1)),
                        body: MatchBody::Expr(Expr::Literal(Literal::Bool(true))),
                    },
                    MatchArm {
                        pattern: Pattern::Wildcard,
                        body: MatchBody::Expr(Expr::Literal(Literal::Bool(false)))
                    }
                ]
            )
        );
    }

    /// Important decision:
    /// Should all custom types (included those in the stdlib) be fully qualified? ie: `Option.None`
    /// Should all custom types (except those in the stdlib) be fully qualified? ie: Some, `MyType.Var1`
    /// In match blocks, should the qualification be omitted for branches? ie: if the type of the expression being matched is MyType, skip `MyType.` in the branches.
    // #[test]
    // fn test_parse_match_patterns_2() {
    //     let input = "match my_option {
    //         Some(x) -> True,
    //         None -> False
    //     }";
    //     let (rem, parsed) = parse_expr(input).unwrap();
    //     assert!(rem.is_empty());
    //     assert_eq!(
    //         parsed,
    //         Expr::Match(
    //             Pattern::Identifier("my_option".to_string()),
    //             vec![
    //                 MatchArm {
    //                     pattern: Pattern::Variant(
    //                         "Option".to_string(),
    //                         "Some".to_string(),
    //                         vec![Pattern::Identifier("x".to_string())]
    //                     ),
    //                     body: MatchBody::Expr(Expr::Literal(Literal::Bool(true))),
    //                 },
    //                 MatchArm {
    //                     pattern: Pattern::Variant("Option".to_string(), "None".to_string(), vec![]),
    //                     body: MatchBody::Expr(Expr::Literal(Literal::Bool(false)))
    //                 }
    //             ]
    //         )
    //     );
    // }

    #[test]
    fn test_parse_binary_op() {
        let input = "1 + 2";
        let (_, parsed) = parse_expr(input).unwrap();
        assert_eq!(
            parsed,
            Expr::BinaryOp(
                Box::new(Expr::Literal(Literal::U8(1))),
                BinaryOp::Add,
                Box::new(Expr::Literal(Literal::U8(2)))
            )
        );
    }
}
