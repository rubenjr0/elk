use super::statements::Block;

/// We need Expr, Statements, and Blocks.
/// Blocks contain statements and return an expression.

/// Expr is the base of the AST.
/// Example: 1 + 2 * 3
/// Expr: my_value
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Identifier(String),
    Literal(Literal),
    NewVariant(String, String, Vec<Expr>),
    /// Expression for creating a new instance of a type with fields.
    /// Example: MyType { field1 =  1, field2 = 2 };
    NewTypeInstance(String, Vec<(String, Expr)>),
    FunctionCall(String, Vec<Expr>),
    Match(Pattern, Vec<MatchArm>),
    BinaryOp(Box<Expr>, BinaryOp, Box<Expr>),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: MatchBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchBody {
    Block(Block),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(Literal),
    Identifier(String),
    Variant(String, String, Vec<Pattern>),
    Tuple(Vec<Pattern>),
    Wildcard,
}

impl TryFrom<Expr> for Pattern {
    type Error = &'static str;

    fn try_from(value: Expr) -> Result<Self, Self::Error> {
        match value {
            Expr::Identifier(s) if s == "_" => Ok(Self::Wildcard),
            Expr::Identifier(s) => Ok(Self::Identifier(s)),
            Expr::Literal(literal) => Ok(Self::Literal(literal)),
            Expr::NewVariant(type_name, variant_name, patterns) => {
                let patterns = patterns
                    .into_iter()
                    .map(Self::try_from)
                    .collect::<Result<Vec<Self>, _>>()
                    .expect("Failed to convert field to pattern");
                Ok(Self::Variant(type_name, variant_name, patterns))
            }
            Expr::NewTypeInstance(_, _) => Err("Cannot convert NewTypeInstance to Pattern"),
            Expr::FunctionCall(_, _) => Err("Cannot convert FunctionCall to Pattern"),
            Expr::Match(_, _) => Err("Cannot convert Match to Pattern"),
            _ => Err("Cannot convert Expr to Pattern"),
        }
    }
}
