pub mod compound;
pub mod custom;
pub mod function;
pub mod primitive;

pub use compound::CompoundType;
pub use custom::CustomType;
pub use function::FunctionType;
pub use primitive::PrimitiveType;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Primitive(PrimitiveType),
    Custom(String),
    Compound(CompoundType),
    Function(FunctionType),
}
