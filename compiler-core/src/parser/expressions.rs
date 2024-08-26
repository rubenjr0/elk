use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{i8, u8},
    combinator::{map, value},
    IResult, Parser,
};

use crate::ast::expressions::{Expr, Literal};

use super::common::parse_identifier_lower;

pub fn parse_expr(input: &str) -> IResult<&str, Expr> {
    alt((parse_literal, parse_identifier, parse_unit)).parse(input)
}

fn parse_literal(input: &str) -> IResult<&str, Expr> {
    let (remaining, lit) = alt((
        map(parse_bool, Literal::Bool),
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

pub fn parse_bool(input: &str) -> IResult<&str, bool> {
    alt((value(true, tag("True")), value(false, tag("False")))).parse(input)
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
}
