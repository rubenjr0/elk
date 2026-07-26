use ast::statements::{Block, Statement};
use winnow::{
    Parser, Result,
    combinator::{alt, delimited, opt, repeat, terminated},
};

use crate::{expressions::parse_expr, identifiers::parse_identifier_lower, keyword, ws};

pub fn parse_block(input: &mut &str) -> Result<Block> {
    delimited(ws('{'), parse_block_content, ws('}')).parse_next(input)
}

fn parse_block_content(input: &mut &str) -> Result<Block> {
    let statements = parse_statements(input)?;
    // NOTE: `opt` (not `if let Ok(...)`) — it resets the input when the
    // expression parser fails, so the closing `}` is left unconsumed.
    match opt(parse_expr).parse_next(input)? {
        Some(return_expr) => Ok(Block::new(statements, return_expr)),
        None => Ok(Block::new_without_return(statements)),
    }
}

fn parse_statements(input: &mut &str) -> Result<Vec<Statement>> {
    repeat(0.., parse_statement).parse_next(input)
}

fn parse_statement(input: &mut &str) -> Result<Statement> {
    terminated(
        alt((parse_assign_statement, parse_return_statement)),
        ws(';'),
    )
    .parse_next(input)
}

fn parse_assign_statement(input: &mut &str) -> Result<Statement> {
    let identifier = parse_identifier_lower(input)?;
    let _ = ws('=').parse_next(input)?;
    let expr = parse_expr(input)?;
    Ok(Statement::Assignment(identifier.to_owned(), expr))
}

fn parse_return_statement(input: &mut &str) -> Result<Statement> {
    let _ = ws(keyword("return")).parse_next(input)?;
    let expr = parse_expr(input)?;
    Ok(Statement::Return(expr))
}

// #[cfg(test)]
// mod tests {
//     use ast::{
//         expressions::{Expression, Literal},
//         statements::{Block, Statement},
//     };

//     use crate::statements::{parse_block, parse_statement};

//     #[test]
//     fn test_parse_assignment_statement() {
//         let input = "_z = 1;";
//         let (_, statement) = parse_statement(input).unwrap();

//         assert_eq!(
//             statement,
//             Statement::Assignment("_z".to_owned(), Expression::literal(Literal::int(1)))
//         );
//     }

//     #[test]
//     fn test_parse_return_statement() {
//         let input = "return 1;";
//         let (_, statement) = parse_statement(input).unwrap();

//         assert_eq!(
//             statement,
//             Statement::Return(Expression::literal(Literal::int(1)))
//         );
//     }

//     #[test]
//     fn test_parse_block() {
//         let input = "{ _z = 1; _z }";
//         let (_, block) = parse_block(input).unwrap();

//         assert_eq!(
//             block,
//             Block::new(
//                 vec![Statement::Assignment(
//                     "_z".to_owned(),
//                     Expression::literal(Literal::int(1))
//                 ),],
//                 Expression::identifier("_z".to_owned())
//             )
//         );
//     }

//     #[test]
//     fn test_parse_block_without_return() {
//         let input = "{ _z = 1; }";
//         let (_, block) = parse_block(input).unwrap();

//         assert_eq!(
//             block,
//             Block::new(
//                 vec![Statement::Assignment(
//                     "_z".to_owned(),
//                     Expression::literal(Literal::int(1))
//                 ),],
//                 Expression::unit()
//             )
//         );
//     }

//     #[test]
//     fn test_parse_block_without_statements() {
//         let input = "{ Unit }";
//         let (_, block) = parse_block(input).unwrap();

//         assert_eq!(block, Block::new(vec![], Expression::unit()));
//     }

//     #[test]
//     fn test_parse_empty_block() {
//         let input = "{}";
//         let (_, block) = parse_block(input).unwrap();

//         assert_eq!(block, Block::new_without_return(vec![]));
//     }
// }

#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn test_return_keyword_boundary() {
        // Regression: `returnfoo;` used to parse as `return foo`.
        let mut input = "returnfoo;";
        assert!(parse_statement(&mut input).is_err());
    }
}
