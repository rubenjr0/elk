use super::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum CompoundType {
    Tuple(Vec<Type>),
    List(Box<Type>),
}
