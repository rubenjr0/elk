use super::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    arguments: Vec<Type>,
    return_type: Box<Type>,
}

impl FunctionType {
    pub fn new(arguments: Vec<Type>, return_type: Type) -> Self {
        Self {
            arguments,
            return_type: Box::new(return_type),
        }
    }

    pub fn arguments(&self) -> &[Type] {
        &self.arguments
    }

    pub fn return_type(&self) -> &Type {
        &self.return_type
    }
}
