use super::expressions::Expr;

/// Statements are the building blocks of a block.
/// Example: my_value = 1 + 2 * 3;
/// Example: return 1 + 2 * 3;
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment(String, Expr),
    Return(Expr),
}

/// Blocks contain statements and return an expression.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    statements: Vec<Statement>,
    return_expr: Expr,
}

impl Block {
    pub fn new(statements: Vec<Statement>, return_expr: Expr) -> Self {
        Self {
            statements,
            return_expr,
        }
    }

    pub fn new_without_return(statements: Vec<Statement>) -> Self {
        Self {
            statements,
            return_expr: Expr::Unit,
        }
    }

    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }

    pub fn return_expr(&self) -> &Expr {
        &self.return_expr
    }
}
