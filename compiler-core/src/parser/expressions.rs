use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{i8, multispace1, u8},
    combinator::{map, opt, value},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, separated_pair},
    IResult, Parser,
};

use crate::ast::expressions::{Expr, Literal};

use super::common::{opt_parenthesis, parse_identifier_lower, parse_identifier_upper, ws};

pub fn parse_expr(input: &str) -> IResult<&str, Expr> {
    alt((
        parse_new_variant,
        parse_new_type_instance,
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
/// Example: MyType.Variant
/// Example: MyType.Variant(1, 2)
fn parse_new_variant(input: &str) -> IResult<&str, Expr> {
    let (input, ty) = parse_identifier_upper(input)?;
    let (input, _) = tag(".").parse(input)?;
    let (input, variant) = parse_identifier_upper(input)?;
    let (input, args) = parse_variant_args(input)?;

    Ok((
        input,
        Expr::NewVariant(ty.to_string(), variant.to_string(), args),
    ))
}

/// example: "(1,2,my_id)" -> "vec![1,2,my_id]"
fn parse_variant_args(input: &str) -> IResult<&str, Vec<Expr>> {
    opt(delimited(
        tag("("),
        separated_list1(ws(tag(",")), parse_expr),
        tag(")"),
    ))
    .map(|o| o.unwrap_or_default())
    .parse(input)
}

/// Example: MyType { field1 = 1, field2 = 2 }
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
fn parse_function_args(input: &str) -> IResult<&str, Vec<Expr>> {
    separated_list1(
        multispace1,
        opt_parenthesis(alt((
            parse_new_variant,
            parse_new_type_instance,
            parse_literal,
            parse_unit,
            parse_identifier,
            parse_function_call,
        ))),
    )
    .parse(input)
}

fn parse_match(input: &str) -> IResult<&str, Expr> {
    todo!()
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
                        vec![Expr::Literal(Literal::U8(32))]
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
}
