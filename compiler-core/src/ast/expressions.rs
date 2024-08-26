/// We need Expr, Statements, and Blocks.
/// Blocks contain statements and return an expression.

/// Expr is the base of the AST.
/// Example: 1 + 2 * 3
/// Expr: my_value
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Identifier(String),
    Literal(Literal),
    /// Fix, Model branches
    Match(Box<Expr>, Vec<(String, Expr)>),
    Unit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    I8(i8),
    U8(u8),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),
}
