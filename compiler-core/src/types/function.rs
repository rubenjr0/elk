use super::{
    expr::{Block, Expr},
    Type,
};

pub struct Function {
    name: String,
    signature: Type,
    implementations: Vec<FunctionImplementation>,
}

pub struct FunctionImplementation {
    name: String,
    arguments: Vec<String>,
    body: FunctionBody,
}

#[derive(Debug, PartialEq)]
pub enum FunctionBody {
    SingleLine(Expr),
    MultiLine(Block),
}

impl Function {
    pub fn new(name: &str, signature: Type) -> Self {
        Self {
            name: name.to_string(),
            signature,
            implementations: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn signature(&self) -> &Type {
        &self.signature
    }
}

impl FunctionImplementation {
    pub fn new(name: &str, arguments: Vec<&str>, body: FunctionBody) -> Self {
        Self {
            name: name.to_string(),
            arguments: arguments.iter().map(|a| a.to_string()).collect(),
            body,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    pub fn body(&self) -> &FunctionBody {
        &self.body
    }
}
