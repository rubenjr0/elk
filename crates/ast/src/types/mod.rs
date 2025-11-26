pub mod compound;
pub mod custom;
pub mod function;

pub use custom::CustomType;
pub use function::FunctionSignature;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Primitive types
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    String,

    /// Type name, generic parameters
    Custom(String, Vec<String>),
    Function(FunctionSignature),

    // Special types
    Unit,
}

impl FromStr for Type {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = match s {
            "U8" => Type::U8,
            "U16" => Type::U16,
            "U32" => Type::U32,
            "U64" => Type::U64,
            "I8" => Type::I8,
            "I16" => Type::I16,
            "I32" => Type::I32,
            "I64" => Type::I64,
            "F32" => Type::F32,
            "F64" => Type::F64,
            "Bool" => Type::Bool,
            "String" => Type::String,
            _ => Err("Invalid type")?,
        };
        Ok(t)
    }
}
