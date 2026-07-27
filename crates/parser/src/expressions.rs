use ast::expressions::{BinaryOp, Expression, Literal, MatchArm, MatchBody, UnaryOp};
use winnow::{
    Parser, Result, dispatch,
    ascii::{dec_uint, hex_uint, multispace0, multispace1},
    combinator::{
        Infix, Prefix, alt, delimited, empty, expression, fail, not, opt, peek, preceded,
        repeat, separated, separated_pair, terminated,
    },
    error::ContextError,
    token::{any, none_of, take_while},
};

use crate::{
    identifiers::{parse_identifier_lower, parse_identifier_upper},
    keyword,
    statements::parse_block,
    ws,
};

// Binding powers (higher binds tighter), following Rust's operator
// precedence: `||` < `&&` < comparisons (non-associative) < `^`
// < `+`/`-` < `*`/`/`/`%` < prefix `-`/`!`.
const OR_POWER: i64 = 1;
const AND_POWER: i64 = 3;
const CMP_POWER: i64 = 5;
const XOR_POWER: i64 = 7;
const ADD_POWER: i64 = 9;
const MUL_POWER: i64 = 11;
const PREFIX_POWER: i64 = 13;

macro_rules! infix_fold {
    ($name:ident, $op:expr) => {
        fn $name(_: &mut &str, l: Expression, r: Expression) -> Result<Expression> {
            Ok(Expression::binary_op(l, $op, r))
        }
    };
}

infix_fold!(fold_or, BinaryOp::Or);
infix_fold!(fold_and, BinaryOp::And);
infix_fold!(fold_xor, BinaryOp::Xor);
infix_fold!(fold_eq, BinaryOp::Eq);
infix_fold!(fold_not_eq, BinaryOp::NotEq);
infix_fold!(fold_greater, BinaryOp::Greater);
infix_fold!(fold_greater_eq, BinaryOp::GreaterEq);
infix_fold!(fold_less, BinaryOp::Less);
infix_fold!(fold_less_eq, BinaryOp::LessEq);
infix_fold!(fold_add, BinaryOp::Add);
infix_fold!(fold_sub, BinaryOp::Sub);
infix_fold!(fold_mul, BinaryOp::Mul);
infix_fold!(fold_div, BinaryOp::Div);
infix_fold!(fold_mod, BinaryOp::Mod);

fn fold_negate(_: &mut &str, e: Expression) -> Result<Expression> {
    Ok(Expression::unary_op(UnaryOp::Negate, e))
}

fn fold_not(_: &mut &str, e: Expression) -> Result<Expression> {
    Ok(Expression::unary_op(UnaryOp::Not, e))
}

/// Expressions are parsed with a Pratt parser (`expression`).
///
/// Operator precedence and associativity are handled declaratively. Operands
/// are atoms; prefix and infix operators are dispatched on their first
/// character.
pub fn parse_expr(input: &mut &str) -> Result<Expression> {
    expression(preceded(multispace0, parse_atom))
        .prefix(dispatch! {any;
            '-' => empty.value(Prefix(PREFIX_POWER, fold_negate)),
            '!' => empty.value(Prefix(PREFIX_POWER, fold_not)),
            _ => fail,
        })
        .infix(dispatch! {ws(any);
            '|' => '|'.value(Infix::Left(OR_POWER, fold_or)),
            '&' => '&'.value(Infix::Left(AND_POWER, fold_and)),
            '^' => empty.value(Infix::Left(XOR_POWER, fold_xor)),
            '=' => '='.value(Infix::Neither(CMP_POWER, fold_eq)),
            '!' => '='.value(Infix::Neither(CMP_POWER, fold_not_eq)),
            '>' => opt('=').map(|e| Infix::Neither(
                CMP_POWER,
                if e.is_some() {
                    fold_greater_eq as fn(&mut &str, Expression, Expression) -> Result<Expression>
                } else {
                    fold_greater
                },
            )),
            '<' => opt('=').map(|e| Infix::Neither(
                CMP_POWER,
                if e.is_some() {
                    fold_less_eq as fn(&mut &str, Expression, Expression) -> Result<Expression>
                } else {
                    fold_less
                },
            )),
            '+' => empty.value(Infix::Left(ADD_POWER, fold_add)),
            // `not('>')`: don't mistake the `->` of lambdas for subtraction
            '-' => peek(not('>')).value(Infix::Left(ADD_POWER, fold_sub)),
            '*' => empty.value(Infix::Left(MUL_POWER, fold_mul)),
            '/' => empty.value(Infix::Left(MUL_POWER, fold_div)),
            '%' => empty.value(Infix::Left(MUL_POWER, fold_mod)),
            _ => fail,
        })
        .parse_next(input)
}

/// An operand of the expression grammar, dispatched on its first character.
fn parse_atom(input: &mut &str) -> Result<Expression> {
    dispatch! {peek(any);
        '"' => parse_string.map(|s| Expression::literal(Literal::String(s))),
        c if c.is_ascii_digit() => parse_number.map(Expression::literal),
        '(' => parse_paren,
        c if c.is_ascii_uppercase() => alt((
            parse_bool.map(|b| Expression::literal(Literal::Bool(b))),
            parse_enum_instance,
            parse_new_type_instance,
            parse_namespaced_function_call,
        )),
        _ => alt((
            parse_match,
            parse_function_call,
            parse_field_access,
            parse_identifier_expr,
        )),
    }
    .parse_next(input)
}

/// `()` (unit) or a parenthesized expression
fn parse_paren(input: &mut &str) -> Result<Expression> {
    preceded(
        '(',
        alt((
            ws(')').map(|_| Expression::unit()),
            terminated(parse_expr, ws(')')),
        )),
    )
    .parse_next(input)
}

pub(crate) fn parse_number(input: &mut &str) -> Result<Literal> {
    alt((
        // Float before integer: "1.5" must not be consumed as "1" then fail on ".5"
        parse_float.map(Literal::float),
        // Prefixed integers before plain integer: "0x1f" must not be consumed as "0"
        parse_hex_int.map(Literal::int),
        parse_bin_int.map(Literal::int),
        parse_oct_int.map(Literal::int),
        dec_uint::<&str, u128, ContextError>.map(Literal::int),
    ))
    .parse_next(input)
}

fn parse_float(input: &mut &str) -> Result<f64> {
    // Require a decimal point so plain integers don't match as floats.
    // Non-negative: unary minus handles negation in expressions.
    (
        take_while(1.., |c: char| c.is_ascii_digit()),
        '.',
        take_while(1.., |c: char| c.is_ascii_digit()),
    )
        .take()
        .map(|s: &str| s.parse::<f64>().unwrap())
        .parse_next(input)
}

fn parse_hex_int(input: &mut &str) -> Result<u128> {
    preceded(alt(("0x", "0X")), hex_uint::<_, u128, _>).parse_next(input)
}

fn parse_bin_int(input: &mut &str) -> Result<u128> {
    preceded(
        alt(("0b", "0B")),
        take_while(1.., |c: char| c == '0' || c == '1')
            .verify_map(|s: &str| u128::from_str_radix(s, 2).ok()),
    )
    .parse_next(input)
}

fn parse_oct_int(input: &mut &str) -> Result<u128> {
    preceded(
        alt(("0o", "0O")),
        take_while(1.., |c: char| ('0'..='7').contains(&c))
            .verify_map(|s: &str| u128::from_str_radix(s, 8).ok()),
    )
    .parse_next(input)
}

fn parse_identifier_expr(input: &mut &str) -> Result<Expression> {
    parse_identifier_lower
        .parse_next(input)
        .map(|id| Expression::identifier(id.to_owned()))
}

pub(crate) fn parse_string(input: &mut &str) -> Result<String> {
    delimited('"', repeat(0.., parse_string_char), '"').parse_next(input)
}

fn parse_string_char(input: &mut &str) -> Result<char> {
    alt((
        preceded('\\', any).map(|c: char| match c {
            '"' => '"',
            '\\' => '\\',
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            other => other,
        }),
        none_of(['"', '\\']),
    ))
    .parse_next(input)
}

pub(crate) fn parse_bool(input: &mut &str) -> Result<bool> {
    alt((keyword("True"), keyword("False")))
        .parse_next(input)
        .map(|p| match p {
            "True" => true,
            "False" => false,
            _ => unreachable!(),
        })
}

/// Expression for creating a new instance of an enum
/// Example: `MyType::Variant`
/// Example: `MyType::Variant(1, 2)`
fn parse_enum_instance(input: &mut &str) -> Result<Expression> {
    let ty = parse_identifier_upper(input)?;
    let _ = "::".parse_next(input)?;
    let variant = parse_identifier_upper(input)?;
    let args = parse_variant_args(input)?;

    Ok(Expression::new_enum_instance(
        ty.to_owned(),
        variant.to_owned(),
        args,
    ))
}

fn parse_variant_args(input: &mut &str) -> Result<Vec<Expression>> {
    opt(delimited('(', separated(0.., parse_expr, ws(',')), ')'))
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
            0..,
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

/// A call to a namespaced function: `Option::map(opt, f)`
fn parse_namespaced_function_call(input: &mut &str) -> Result<Expression> {
    let namespace = parse_identifier_upper(input)?;
    let _ = "::".parse_next(input)?;
    let function_name = parse_identifier_lower(input)?;
    let args = parse_function_call_args(input)?;
    Ok(Expression::namespaced_function_call(
        Some(namespace.to_owned()),
        function_name.to_owned(),
        args,
    ))
}

fn parse_function_call_args(input: &mut &str) -> Result<Vec<Expression>> {
    delimited('(', separated(0.., parse_expr, ws(',')), ws(')')).parse_next(input)
}

fn parse_match(input: &mut &str) -> Result<Expression> {
    let _ = terminated(keyword("match"), multispace1).parse_next(input)?;
    let pat = parse_expr(input)?;
    let cases =
        delimited(ws('{'), separated(0.., parse_match_arm, ws(',')), ws('}')).parse_next(input)?;
    Ok(Expression::match_expr(pat, cases))
}

fn parse_match_arm(input: &mut &str) -> Result<MatchArm> {
    let (pattern, body) =
        separated_pair(crate::patterns::parse_pattern, ws("=>"), parse_match_body)
            .parse_next(input)?;

    Ok(MatchArm::new(pattern, body))
}

fn parse_match_body(input: &mut &str) -> Result<MatchBody> {
    alt((
        parse_expr.map(MatchBody::Expr),
        parse_block.map(MatchBody::Block),
    ))
    .parse_next(input)
}

#[cfg(test)]
mod tests {

    use super::*;
    use ast::patterns::Pattern;

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
    fn test_parse_literal_negative() {
        let mut input = "-37";
        let expr = parse_expr(&mut input).unwrap();
        assert_eq!(
            expr,
            Expression::unary_op(UnaryOp::Negate, Expression::literal(Literal::int(37)))
        );
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
        let mut input = "()";
        let expr = parse_expr(&mut input).unwrap();

        assert_eq!(expr, Expression::unit());
    }

    #[test]
    fn test_parse_new_enum_instance() {
        let mut input = "Option::None";
        let parsed = parse_expr(&mut input).unwrap();
        assert_eq!(
            parsed,
            Expression::new_enum_instance("Option".to_owned(), "None".to_owned(), vec![])
        );
    }

    #[test]
    fn test_parse_new_enum_instance_with_args() {
        let mut input = "Option::Some(1)";
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
            True => 1,
            False => 0
        }";
        let parsed = parse_expr(&mut input).unwrap();
        assert!(input.is_empty());
        assert_eq!(
            parsed,
            Expression::match_expr(
                Expression::identifier("my_bool".to_owned()),
                vec![
                    MatchArm {
                        pattern: Pattern::Literal(Literal::Bool(true)),
                        body: MatchBody::Expr(Expression::literal(Literal::int(1))),
                    },
                    MatchArm {
                        pattern: Pattern::Literal(Literal::Bool(false)),
                        body: MatchBody::Expr(Expression::literal(Literal::int(0)))
                    }
                ]
            )
        );
    }

    #[test]
    fn test_parse_match_patterns_1() {
        let mut input = "match Option::Some(x) {
            1 => True,
            _ => False
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
                        pattern: Pattern::Literal(Literal::int(1)),
                        body: MatchBody::Expr(Expression::literal(Literal::Bool(true))),
                    },
                    MatchArm {
                        pattern: Pattern::Wildcard,
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

    #[test]
    fn test_parse_octal_with_seven() {
        // Regression: the octal range used to exclude the digit '7'.
        let mut input = "0o17";
        let expr = parse_expr(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(expr, Expression::literal(Literal::int(15)));
    }

    #[test]
    fn test_parse_bin_overflow_errors_instead_of_panicking() {
        // Regression: binary/octal literals used to `unwrap` a u64 parse.
        let mut input = "0b11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111";
        assert!(parse_expr(&mut input).is_err() || !input.is_empty());
    }

    #[test]
    fn test_parse_hex_u128() {
        let mut input = "0xffffffffffffffffffffffffffffffff"; // 32 f's: only fits u128
        let expr = parse_expr(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(expr, Expression::literal(Literal::int(u128::MAX)));
    }

    #[test]
    fn test_parse_bool_keyword_boundary() {
        // Regression: `Truex` used to parse as `True` leaving `x`.
        let mut input = "Truex";
        assert!(parse_expr(&mut input).is_err());
    }

    #[test]
    fn test_parse_unary_not_vs_negate() {
        // Regression: `!` used to map to `Negate`, same as `-`.
        let mut input = "!True";
        let expr = parse_expr(&mut input).unwrap();
        assert_eq!(
            expr,
            Expression::unary_op(UnaryOp::Not, Expression::literal(Literal::Bool(true)))
        );

        let mut input = "-True";
        let expr = parse_expr(&mut input).unwrap();
        assert_eq!(
            expr,
            Expression::unary_op(UnaryOp::Negate, Expression::literal(Literal::Bool(true)))
        );
    }

    #[test]
    fn test_parse_enum_instance_empty_parens() {
        // Regression: `Option::None()` used to fail, leaving `()` unconsumed.
        let mut input = "Option::None()";
        let parsed = parse_expr(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(
            parsed,
            Expression::new_enum_instance("Option".to_owned(), "None".to_owned(), vec![])
        );
    }

    #[test]
    fn test_parse_empty_record_instance() {
        // Regression: `MyType {}` used to fail (fields required 1+).
        let mut input = "MyType {}";
        let parsed = parse_expr(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(
            parsed,
            Expression::new_record_instance("MyType".to_owned(), vec![])
        );
    }

    #[test]
    fn test_parse_xor_operator() {
        // `Xor` existed in the AST but no token produced it.
        let mut input = "1 ^ 2";
        let parsed = parse_expr(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(
            parsed,
            Expression::binary_op(
                Expression::literal(Literal::int(1)),
                BinaryOp::Xor,
                Expression::literal(Literal::int(2))
            )
        );
    }

    #[test]
    fn test_precedence_mul_before_add() {
        let mut input = "1 * 2 + 3";
        let expr = parse_expr(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(
            expr,
            Expression::binary_op(
                Expression::binary_op(
                    Expression::literal(Literal::int(1)),
                    BinaryOp::Mul,
                    Expression::literal(Literal::int(2))
                ),
                BinaryOp::Add,
                Expression::literal(Literal::int(3))
            )
        );
    }

    #[test]
    fn test_precedence_add_after_mul() {
        let mut input = "2 + 3 * 4";
        let expr = parse_expr(&mut input).unwrap();
        assert_eq!(
            expr,
            Expression::binary_op(
                Expression::literal(Literal::int(2)),
                BinaryOp::Add,
                Expression::binary_op(
                    Expression::literal(Literal::int(3)),
                    BinaryOp::Mul,
                    Expression::literal(Literal::int(4))
                )
            )
        );
    }

    #[test]
    fn test_sub_is_left_associative() {
        let mut input = "10 - 4 - 3";
        let expr = parse_expr(&mut input).unwrap();
        assert_eq!(
            expr,
            Expression::binary_op(
                Expression::binary_op(
                    Expression::literal(Literal::int(10)),
                    BinaryOp::Sub,
                    Expression::literal(Literal::int(4))
                ),
                BinaryOp::Sub,
                Expression::literal(Literal::int(3))
            )
        );
    }

    #[test]
    fn test_parenthesized_expr() {
        let mut input = "(1 + 2) * 3";
        let expr = parse_expr(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(
            expr,
            Expression::binary_op(
                Expression::binary_op(
                    Expression::literal(Literal::int(1)),
                    BinaryOp::Add,
                    Expression::literal(Literal::int(2))
                ),
                BinaryOp::Mul,
                Expression::literal(Literal::int(3))
            )
        );
    }

    #[test]
    fn test_comparison_is_non_associative() {
        // `1 < 2 < 3` parses `1 < 2` and stops, leaving `< 3` unconsumed
        // (the enclosing context will error on the leftover).
        let mut input = "1 < 2 < 3";
        let expr = parse_expr(&mut input).unwrap();
        assert_eq!(input, " < 3");
        assert_eq!(
            expr,
            Expression::binary_op(
                Expression::literal(Literal::int(1)),
                BinaryOp::Less,
                Expression::literal(Literal::int(2))
            )
        );
    }

    #[test]
    fn test_xor_binds_tighter_than_comparison() {
        // Rust-style: `x ^ y == z` is `(x ^ y) == z`, not C's `x ^ (y == z)`.
        let mut input = "x ^ y == z";
        let expr = parse_expr(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(
            expr,
            Expression::binary_op(
                Expression::binary_op(
                    Expression::identifier("x".to_owned()),
                    BinaryOp::Xor,
                    Expression::identifier("y".to_owned())
                ),
                BinaryOp::Eq,
                Expression::identifier("z".to_owned())
            )
        );
    }

    #[test]
    fn test_unary_binds_tighter_than_mul() {
        let mut input = "-x * 2";
        let expr = parse_expr(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(
            expr,
            Expression::binary_op(
                Expression::unary_op(UnaryOp::Negate, Expression::identifier("x".to_owned())),
                BinaryOp::Mul,
                Expression::literal(Literal::int(2))
            )
        );
    }

    #[test]
    fn test_unary_binds_tighter_than_comparison() {
        let mut input = "!True == False";
        let expr = parse_expr(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(
            expr,
            Expression::binary_op(
                Expression::unary_op(UnaryOp::Not, Expression::literal(Literal::Bool(true))),
                BinaryOp::Eq,
                Expression::literal(Literal::Bool(false))
            )
        );
    }

    #[test]
    fn test_function_call_with_expr_args() {
        // Was impossible before the Pratt rewrite (args were a restricted alt).
        let mut input = "f(1 + 2, 3)";
        let expr = parse_expr(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(
            expr,
            Expression::function_call(
                "f".to_owned(),
                vec![
                    Expression::binary_op(
                        Expression::literal(Literal::int(1)),
                        BinaryOp::Add,
                        Expression::literal(Literal::int(2))
                    ),
                    Expression::literal(Literal::int(3)),
                ]
            )
        );
    }

    #[test]
    fn test_field_access_in_binary_op() {
        let mut input = "a.b + c.d";
        let expr = parse_expr(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(
            expr,
            Expression::binary_op(
                Expression::record_access("a".to_owned(), "b".to_owned()),
                BinaryOp::Add,
                Expression::record_access("c".to_owned(), "d".to_owned())
            )
        );
    }

    #[test]
    fn test_parse_namespaced_function_call() {
        let mut input = "Option::map(opt, f)";
        let expr = parse_expr(&mut input).unwrap();
        assert!(input.is_empty(), "Remaining input: {input}");
        assert_eq!(
            expr,
            Expression::namespaced_function_call(
                Some("Option".to_owned()),
                "map".to_owned(),
                vec![
                    Expression::identifier("opt".to_owned()),
                    Expression::identifier("f".to_owned()),
                ]
            )
        );
    }
}

