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
    Void,
}

impl FromStr for Type {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = match s {
            "U8" => Self::U8,
            "U16" => Self::U16,
            "U32" => Self::U32,
            "U64" => Self::U64,
            "I8" => Self::I8,
            "I16" => Self::I16,
            "I32" => Self::I32,
            "I64" => Self::I64,
            "F32" => Self::F32,
            "F64" => Self::F64,
            "Bool" => Self::Bool,
            "String" => Self::String,
            "Void" => Self::Void,
            _ => Err("Invalid type")?,
        };
        Ok(t)
    }
}
