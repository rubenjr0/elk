use super::{expressions::Expression, statements::Block, types::FunctionSignature};

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    name: String,
    signature: FunctionSignature,
}

#[derive(Debug, Clone)]
pub struct FunctionImplementation {
    name: String,
    arguments: Vec<String>,
    body: FunctionBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    SingleLine(Expression),
    MultiLine(Block),
}

impl FunctionDefinition {
    pub fn new(name: &str, signature: FunctionSignature) -> Self {
        Self {
            name: name.to_string(),
            signature,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn signature(&self) -> &FunctionSignature {
        &self.signature
    }
}

impl FunctionImplementation {
    pub fn new(name: &str, arguments: Vec<String>, body: FunctionBody) -> Self {
        Self {
            name: name.to_string(),
            arguments,
            body,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    pub const fn body(&self) -> &FunctionBody {
        &self.body
    }
}
