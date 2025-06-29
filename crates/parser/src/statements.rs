use ast::statements::{Block, Statement};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    multi::many0,
    sequence::{delimited, terminated},
};

use crate::{
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
    terminated(
        alt((parse_assign_statement, parse_return_statement)),
        ws(tag(";")),
    )
    .parse(input)
}

fn parse_assign_statement(input: &str) -> IResult<&str, Statement> {
    let (input, identifier) = parse_identifier_lower(input)?;
    let (input, _) = ws(tag("=")).parse(input)?;
    let (input, expr) = parse_expr(input)?;
    Ok((input, Statement::Assignment(identifier.to_owned(), expr)))
}

fn parse_return_statement(input: &str) -> IResult<&str, Statement> {
    let (input, _) = ws(tag("return")).parse(input)?;
    let (input, expr) = parse_expr(input)?;

    Ok((input, Statement::Return(expr)))
}

#[cfg(test)]
mod tests {
    use ast::{
        expressions::{Expression, Literal},
        statements::{Block, Statement},
    };

    use crate::statements::{parse_block, parse_statement};

    #[test]
    fn test_parse_assignment_statement() {
        let input = "_z = 1;";
        let (_, statement) = parse_statement(input).unwrap();

        assert_eq!(
            statement,
            Statement::Assignment("_z".to_owned(), Expression::literal(Literal::int(1)))
        );
    }

    #[test]
    fn test_parse_return_statement() {
        let input = "return 1;";
        let (_, statement) = parse_statement(input).unwrap();

        assert_eq!(
            statement,
            Statement::Return(Expression::literal(Literal::int(1)))
        );
    }

    #[test]
    fn test_parse_block() {
        let input = "{ _z = 1; _z }";
        let (_, block) = parse_block(input).unwrap();

        assert_eq!(
            block,
            Block::new(
                vec![Statement::Assignment(
                    "_z".to_owned(),
                    Expression::literal(Literal::int(1))
                ),],
                Expression::identifier("_z".to_owned())
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
                    "_z".to_owned(),
                    Expression::literal(Literal::int(1))
                ),],
                Expression::unit()
            )
        );
    }

    #[test]
    fn test_parse_block_without_statements() {
        let input = "{ Unit }";
        let (_, block) = parse_block(input).unwrap();

        assert_eq!(block, Block::new(vec![], Expression::unit()));
    }

    #[test]
    fn test_parse_empty_block() {
        let input = "{}";
        let (_, block) = parse_block(input).unwrap();

        assert_eq!(block, Block::new_without_return(vec![]));
    }
}
