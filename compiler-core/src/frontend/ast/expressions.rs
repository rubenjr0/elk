use super::{statements::Block, types::Type};

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub associated_type: Type,
}

/// Expr is the base of the AST.
/// Example: `1 + 2 * 3`
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    Identifier(String),
    Literal(Literal),
    NewEnumInstance(String, String, Vec<Expression>),
    /// Expression for creating a new instance of a type with fields.
    /// Example: `MyType { field1 =  1, field2 = 2 };`
    NewRecordInstance(String, Vec<(String, Expression)>),
    /// Identifier, Field
    /// Example: `my_val.my_field`
    RecordAccess(String, String),
    FunctionCall(String, Vec<Expression>),
    Match(Box<Expression>, Vec<MatchArm>),
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

    pub fn literal(literal: Literal) -> Self {
        let associated_type = literal.get_type();
        Self {
            kind: ExpressionKind::Literal(literal),
            associated_type,
        }
    }

    pub fn new_enum_instance(enum_name: String, variant_name: String, fields: Vec<Self>) -> Self {
        Self {
            kind: ExpressionKind::NewEnumInstance(enum_name.to_owned(), variant_name, fields),
            associated_type: Type::Custom(enum_name, vec![]),
        }
    }

    pub fn new_record_instance(record_name: String, fields: Vec<(String, Self)>) -> Self {
        let associated_type = Type::Custom(record_name.clone(), vec![]);
        Self {
            kind: ExpressionKind::NewRecordInstance(record_name, fields),
            associated_type,
        }
    }

    pub const fn record_access(record_name: String, field_name: String) -> Self {
        Self {
            kind: ExpressionKind::RecordAccess(record_name, field_name),
            associated_type: Type::Pending,
        }
    }

    pub const fn function_call(name: String, args: Vec<Self>) -> Self {
        Self {
            kind: ExpressionKind::FunctionCall(name, args),
            associated_type: Type::Pending,
        }
    }

    pub fn match_expr(expr: Expression, arms: Vec<MatchArm>) -> Self {
        Self {
            kind: ExpressionKind::Match(Box::new(expr), arms),
            associated_type: Type::Pending,
        }
    }

    pub fn binary_op(lhs: Self, op: BinaryOp, rhs: Self) -> Self {
        Self {
            kind: ExpressionKind::BinaryOp(Box::new(lhs), op, Box::new(rhs)),
            associated_type: Type::Pending,
        }
    }

    pub fn unary_op(op: UnaryOp, expr: Self) -> Self {
        Self {
            kind: ExpressionKind::UnaryOp(op, Box::new(expr)),
            associated_type: Type::Pending,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(u64, Type),
    Float(f64, Type),
    Bool(bool),
    String(String),
}

impl Literal {
    pub fn get_type(&self) -> Type {
        match self {
            Literal::Integer(_, ty) => ty.clone(),
            Literal::Float(_, ty) => ty.clone(),
            Literal::Bool(_) => Type::Bool,
            Literal::String(_) => Type::String,
        }
    }

    pub fn i8(val: i8) -> Self {
        Self::Integer(val as u64, Type::I8)
    }

    pub fn u8(val: u8) -> Self {
        Self::Integer(val as u64, Type::U8)
    }

    pub fn i32(val: i32) -> Self {
        Self::Integer(val as u64, Type::I32)
    }

    pub fn u32(val: u32) -> Self {
        Self::Integer(val as u64, Type::U32)
    }

    pub fn f32(val: f32) -> Self {
        Self::Float(val as f64, Type::F32)
    }
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
    Xor,
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
    pub pattern: Expression,
    pub body: MatchBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchBody {
    Block(Block),
    Expr(Expression),
}

impl MatchArm {
    pub fn new(pattern: Expression, body: MatchBody) -> Self {
        match pattern.kind {
            ExpressionKind::Identifier(_)
            | ExpressionKind::Literal(_)
            | ExpressionKind::NewEnumInstance(_, _, _) => (),
            _ => panic!(),
        };
        Self { pattern, body }
    }
}
