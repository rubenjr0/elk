pub mod compound;
pub mod custom;
pub mod function;

pub use custom::CustomType;
pub use function::FunctionSignature;

use crate::codegen::Generable;

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

impl Generable for Type {
    fn size(&self) -> u32 {
        match self {
            Self::I8 | Self::U8 | Self::Bool => 1,
            Self::I16 | Self::U16 => 2,
            Self::I32 | Self::U32 | Self::F32 => 4,
            Self::I64 | Self::U64 | Self::F64 | Self::Function(_) | Self::Custom(_, _) => 8,
            _ => todo!(),
        }
    }

    fn to_cranelift(&self) -> cranelift::prelude::Type {
        use cranelift::prelude::types as T;
        match self {
            Self::I8 | Self::U8 => T::I8,
            Self::I16 | Self::U16 => T::I16,
            Self::I32 | Self::U32 => T::I32,
            Self::I64 | Self::U64 | Self::Function(_) | Self::Custom(_, _) => T::I64,
            Self::F32 => T::F32,
            Self::F64 => T::F64,
            Self::Bool => T::I8,
            _ => todo!(),
        }
    }
}
