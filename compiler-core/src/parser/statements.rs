use nom::{branch::alt, bytes::complete::tag, multi::many0, sequence::delimited, IResult, Parser};

use crate::ast::statements::{Block, Statement};

use super::{
    common::{parse_identifier_lower, ws},
    expressions::parse_expr,
};

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
    use crate::{
        ast::{
            expressions::{Expr, Literal},
            statements::{Block, Statement},
        },
        parser::statements::{parse_block, parse_statement},
    };

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
