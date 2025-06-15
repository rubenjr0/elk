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

    /// Type name, generic parameters
    Custom(String, Vec<String>),
    Function(FunctionSignature),

    // Special types
    Unit,
}
