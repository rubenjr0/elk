use super::{
    statements::Block,
    types::{PrimitiveType, Type},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    kind: ExpressionKind,
    associated_type: Type,
}

/// Expr is the base of the AST.
/// Example: `1 + 2 * 3`
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    Identifier(String),
    Literal(Literal),
    NewVariant(String, String, Vec<Expression>),
    /// Expression for creating a new instance of a type with fields.
    /// Example: `MyType { field1 =  1, field2 = 2 };`
    NewTypeInstance(String, Vec<(String, Expression)>),
    /// Identifier, Field
    /// Example: `my_val.my_field`
    FieldAccess(String, String),
    FunctionCall(String, Vec<Expression>),
    Match(Pattern, Vec<MatchArm>),
    BinaryOp(Box<Expression>, BinaryOp, Box<Expression>),
    UnaryOp(UnaryOp, Box<Expression>),
    Unit,
}

impl Expression {
    pub const fn unit() -> Self {
        Self {
            kind: ExpressionKind::Unit,
            associated_type: Type::Unit,
        }
    }

    pub const fn identifier(identifier: String) -> Self {
        Self {
            kind: ExpressionKind::Identifier(identifier),
            associated_type: Type::Pending,
        }
    }

    pub const fn literal(literal: Literal) -> Self {
        let associated_type = match literal {
            Literal::I8(_) => Type::Primitive(PrimitiveType::I8),
            Literal::U8(_) => Type::Primitive(PrimitiveType::U8),
            Literal::F32(_) => Type::Primitive(PrimitiveType::F32),
            Literal::F64(_) => Type::Primitive(PrimitiveType::F64),
            Literal::Bool(_) => Type::Primitive(PrimitiveType::Bool),
            Literal::String(_) => Type::Primitive(PrimitiveType::String),
        };
        Self {
            kind: ExpressionKind::Literal(literal),
            associated_type,
        }
    }

    pub fn new_variant(type_name: String, variant_name: String, fields: Vec<Expression>) -> Self {
        Self {
            kind: ExpressionKind::NewVariant(type_name.clone(), variant_name, fields),
            associated_type: Type::Custom(type_name, vec![]),
        }
    }

    pub fn new_type_instance(type_name: String, fields: Vec<(String, Expression)>) -> Self {
        let associated_type = Type::Custom(type_name.clone(), vec![]);
        Self {
            kind: ExpressionKind::NewTypeInstance(type_name, fields),
            associated_type,
        }
    }

    pub fn field_access(type_name: String, field_name: String) -> Self {
        Self {
            kind: ExpressionKind::FieldAccess(type_name, field_name),
            associated_type: Type::Pending,
        }
    }

    pub fn function_call(name: String, args: Vec<Expression>) -> Self {
        Self {
            kind: ExpressionKind::FunctionCall(name, args),
            associated_type: Type::Pending,
        }
    }

    pub fn match_expr(pattern: Pattern, arms: Vec<MatchArm>) -> Self {
        Self {
            kind: ExpressionKind::Match(pattern, arms),
            associated_type: Type::Pending,
        }
    }

    pub fn binary_op(lhs: Expression, op: BinaryOp, rhs: Expression) -> Self {
        Self {
            kind: ExpressionKind::BinaryOp(Box::new(lhs), op, Box::new(rhs)),
            associated_type: Type::Pending,
        }
    }

    pub fn unary_op(op: UnaryOp, expr: Expression) -> Self {
        Self {
            kind: ExpressionKind::UnaryOp(op, Box::new(expr)),
            associated_type: Type::Pending,
        }
    }
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

/// TODO: Add more operators
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: MatchBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchBody {
    Block(Block),
    Expr(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(Literal),
    Identifier(String),
    /// Type, Variant, Fields
    Variant(String, String, Vec<Pattern>),
    /// Identifier, Field
    /// Example: `my_val.my_field`
    Field(String, String),
    Tuple(Vec<Pattern>),
    Wildcard,
}

impl TryFrom<Expression> for Pattern {
    type Error = &'static str;

    fn try_from(value: Expression) -> Result<Self, Self::Error> {
        match value.kind {
            ExpressionKind::Identifier(s) if s == "_" => Ok(Self::Wildcard),
            ExpressionKind::Identifier(s) => Ok(Self::Identifier(s)),
            ExpressionKind::Literal(literal) => Ok(Self::Literal(literal)),
            ExpressionKind::NewVariant(type_name, variant_name, patterns) => {
                let patterns = patterns
                    .into_iter()
                    .map(Self::try_from)
                    .collect::<Result<Vec<Self>, _>>()
                    .expect("Failed to convert field to pattern");
                Ok(Self::Variant(type_name, variant_name, patterns))
            }
            ExpressionKind::FieldAccess(ty, fd) => Ok(Self::Field(ty, fd)),
            ExpressionKind::NewTypeInstance(_, _) => {
                Err("Cannot convert NewTypeInstance to Pattern")
            }
            ExpressionKind::FunctionCall(_, _) => Err("Cannot convert FunctionCall to Pattern"),
            ExpressionKind::Match(_, _) => Err("Cannot convert Match to Pattern"),
            _ => Err("Cannot convert Expr to Pattern"),
        }
    }
}
