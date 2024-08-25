use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{i8, u8},
    combinator::{map, value},
    multi::many0,
    sequence::delimited,
    IResult, Parser,
};

use crate::{
    parser::{parse_identifier_lower, ws},
    types::expr::{Block, Expr, Literal, Statement},
};

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
    map(tag("Void"), |_| Expr::Unit).parse(input)
}

pub fn parse_bool(input: &str) -> IResult<&str, bool> {
    alt((value(true, tag("True")), value(false, tag("False")))).parse(input)
}

pub fn parse_block(input: &str) -> IResult<&str, Block> {
    delimited(ws(tag("{")), parse_block_content, ws(tag("}"))).parse(input)
}

fn parse_block_content(input: &str) -> IResult<&str, Block> {
    let (input, statements) = parse_statements(input)?;
    if let Ok((input, return_expr)) = parse_expr(input) {
        Ok((input, Block::new(statements, return_expr)))
    } else {
        Ok((input, Block::new_without_return(statements)))
    }
}

fn parse_statements(input: &str) -> IResult<&str, Vec<Statement>> {
    let (input, statements) = many0(parse_statement).parse(input)?;
    Ok((input, statements))
}

fn parse_statement(input: &str) -> IResult<&str, Statement> {
    alt((parse_assign_statement, parse_return_statement)).parse(input)
}

fn parse_assign_statement(input: &str) -> IResult<&str, Statement> {
    let (input, identifier) = parse_identifier_lower(input)?;
    let (input, _) = ws(tag("=")).parse(input)?;
    let (input, expr) = parse_expr(input)?;
    let (input, _) = ws(tag(";")).parse(input)?;
    Ok((input, Statement::Assignment(identifier.to_string(), expr)))
}

fn parse_return_statement(input: &str) -> IResult<&str, Statement> {
    let (input, _) = ws(tag("return")).parse(input)?;
    let (input, expr) = parse_expr(input)?;
    let (input, _) = ws(tag(";")).parse(input)?;

    Ok((input, Statement::Return(expr)))
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
        let input = "Void";
        let (_, expr) = parse_expr(input).unwrap();

        assert_eq!(expr, Expr::Unit);
    }

    #[test]
    fn test_parse_assignment_statement() {
        let input = "_z = 1;";
        let (_, statement) = parse_statement(input).unwrap();

        assert_eq!(
            statement,
            Statement::Assignment("_z".to_string(), Expr::Literal(Literal::U8(1)))
        );
    }

    #[test]
    fn test_parse_return_statement() {
        let input = "return 1;";
        let (_, statement) = parse_statement(input).unwrap();

        assert_eq!(statement, Statement::Return(Expr::Literal(Literal::U8(1))));
    }

    #[test]
    fn test_parse_block() {
        let input = "{ _z = 1; _z }";
        let (_, block) = parse_block(input).unwrap();

        assert_eq!(
            block,
            Block::new(
                vec![Statement::Assignment(
                    "_z".to_string(),
                    Expr::Literal(Literal::U8(1))
                ),],
                Expr::Identifier("_z".to_string())
            )
        );
    }

    #[test]
    fn test_parse_block_without_return() {
        let input = "{ _z = 1; }";
        let (_, block) = parse_block(input).unwrap();

        assert_eq!(
            block,
            Block::new(
                vec![Statement::Assignment(
                    "_z".to_string(),
                    Expr::Literal(Literal::U8(1))
                ),],
                Expr::Unit
            )
        );
    }

    #[test]
    fn test_parse_block_without_statements() {
        let input = "{ Void }";
        let (_, block) = parse_block(input).unwrap();

        assert_eq!(block, Block::new(vec![], Expr::Unit));
    }
}
