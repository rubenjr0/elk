/// We need Expr, Statements, and Blocks.
/// Blocks contain statements and return an expression.

/// Expr is the base of the AST.
/// Example: 1 + 2 * 3
/// Expr: my_value
#[derive(Debug, PartialEq)]
pub enum Expr {
    Identifier(String),
    Literal(Literal),
    Unit,
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    I8(i8),
    U8(u8),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),
}

/// Statements are the building blocks of a block.
/// Example: my_value = 1 + 2 * 3;
/// Example: return 1 + 2 * 3;
#[derive(Debug, PartialEq)]
pub enum Statement {
    Assignment(String, Expr),
    Return(Expr),
}

/// Blocks contain statements and return an expression.
#[derive(Debug, PartialEq)]
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
