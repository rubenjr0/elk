use cranelift::prelude::{isa::CallConv, AbiParam, Signature};

use super::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    arguments: Vec<Type>,
    return_type: Box<Type>,
}

impl FunctionSignature {
    pub fn new(arguments: Vec<Type>, return_type: Type) -> Self {
        Self {
            arguments,
            return_type: Box::new(return_type),
        }
    }

    pub fn arguments(&self) -> &[Type] {
        &self.arguments
    }

    pub const fn return_type(&self) -> &Type {
        &self.return_type
    }
}

impl From<&FunctionSignature> for cranelift::prelude::Signature {
    fn from(value: &FunctionSignature) -> Self {
        let params: Vec<_> = value
            .arguments()
            .iter()
            .map(|arg| AbiParam::new(arg.into()))
            .collect();
        let returns = AbiParam::new(value.return_type().into());
        Signature {
            params,
            returns: vec![returns],
            call_conv: CallConv::SystemV,
        }
    }
}
