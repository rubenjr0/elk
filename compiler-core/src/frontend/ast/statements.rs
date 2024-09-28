use super::expressions::Expression;

/// Statements are the building blocks of a block.
/// Example: `my_value = 1 + 2 * 3;`
/// Example: `return 1 + 2 * 3;`
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment(String, Expression),
    Return(Expression),
}

/// Blocks contain statements and return an expression.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    statements: Vec<Statement>,
    pub return_expr: Expression,
}

impl Block {
    pub const fn new(statements: Vec<Statement>, return_expr: Expression) -> Self {
        Self {
            statements,
            return_expr,
        }
    }

    pub const fn new_without_return(statements: Vec<Statement>) -> Self {
        Self {
            statements,
            return_expr: Expression::unit(),
        }
    }

    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }
}
