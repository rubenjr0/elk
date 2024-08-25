pub mod custom_type;
pub mod expr;
pub mod function;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Unit,
    I8,
    U8,
    F32,
    F64,
    P8,
    P16,
    P32,
    Bool,
    String,
    FunctionSignature(Vec<Type>, Box<Type>),
    Tuple(Vec<Type>),
    List(Box<Type>),
    Custom(String),
}
