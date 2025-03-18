pub mod compound;
pub mod custom;
pub mod function;

pub use custom::CustomType;
pub use function::FunctionSignature;

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

    Custom(String, Vec<String>),
    Function(FunctionSignature),

    // Special types
    Unit,
    /// For type inference, illegal
    Pending,
}

impl Type {
    pub fn to_cranelift(&self) -> cranelift::prelude::Type {
        use cranelift::prelude::types as T;
        match self {
            Type::I8 | Type::U8 => T::I8,
            Type::I16 | Type::U16 => T::I16,
            Type::I32 | Type::U32 => T::I32,
            Type::I64 | Type::U64 | Type::Function(_) => T::I64,
            Type::F32 => T::F32,
            Type::F64 => T::F64,
            Type::Bool => T::I8,
            Type::Pending => panic!("Pending type cannot be converted to Cranelift type"),
            _ => todo!(),
        }
    }
}
