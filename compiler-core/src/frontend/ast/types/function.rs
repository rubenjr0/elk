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

impl FunctionSignature {
    pub fn to_cranelift(&self) -> cranelift::prelude::Signature {
        let params: Vec<_> = self
            .arguments()
            .iter()
            .map(|arg| AbiParam::new(arg.to_cranelift()))
            .collect();
        let returns = AbiParam::new(self.return_type().to_cranelift());
        Signature {
            params,
            returns: vec![returns],
            call_conv: CallConv::SystemV,
        }
    }
}
