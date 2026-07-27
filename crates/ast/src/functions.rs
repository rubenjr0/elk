use super::{
    expressions::Expression,
    patterns::Pattern,
    statements::Block,
    types::{FunctionSignature, Type},
};

/// A qualified function name: `map` or `Option::map`.
///
/// Grammar reference: `QualifiedName = (TypeIdentifier "::")? Identifier`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualifiedName {
    pub namespace: Option<String>,
    pub name: String,
}

impl QualifiedName {
    pub fn new(namespace: Option<String>, name: &str) -> Self {
        Self {
            namespace,
            name: name.to_owned(),
        }
    }

    pub fn unqualified(name: &str) -> Self {
        Self::new(None, name)
    }

    /// `Option::map`, or just `map` when unnamespaced.
    pub fn qualified(&self) -> String {
        self.namespace
            .as_ref()
            .map_or_else(|| self.name.clone(), |ns| format!("{ns}::{}", self.name))
    }
}

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    name: QualifiedName,
    /// Declared type variables, e.g. `<A, B>` in `map<A, B>([A], f: (A) -> B) -> [B];`
    type_params: Vec<String>,
    signature: FunctionSignature,
}

#[derive(Debug, Clone)]
pub struct FunctionImplementation {
    name: QualifiedName,
    arguments: Vec<Pattern>,
    body: FunctionBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    SingleLine(Expression),
    MultiLine(Block),
}

impl FunctionDeclaration {
    pub const fn new(
        name: QualifiedName,
        type_params: Vec<String>,
        signature: FunctionSignature,
    ) -> Self {
        Self {
            name,
            type_params,
            signature,
        }
    }

    pub fn main(ty: &Type) -> Self {
        Self {
            name: QualifiedName::unqualified("main"),
            type_params: vec![],
            signature: FunctionSignature::new(vec![], ty.to_owned()),
        }
    }

    pub const fn qualified_name(&self) -> &QualifiedName {
        &self.name
    }

    pub fn name(&self) -> &str {
        &self.name.name
    }

    pub fn type_params(&self) -> &[String] {
        &self.type_params
    }

    pub const fn signature(&self) -> &FunctionSignature {
        &self.signature
    }
}

impl FunctionImplementation {
    pub const fn new(name: QualifiedName, arguments: Vec<Pattern>, body: FunctionBody) -> Self {
        Self {
            name,
            arguments,
            body,
        }
    }

    pub fn main(block: &Block) -> Self {
        let body = FunctionBody::MultiLine(block.clone());
        Self {
            name: QualifiedName::unqualified("main"),
            arguments: vec![],
            body,
        }
    }

    pub const fn qualified_name(&self) -> &QualifiedName {
        &self.name
    }

    pub fn name(&self) -> &str {
        &self.name.name
    }

    pub fn arguments(&self) -> &[Pattern] {
        &self.arguments
    }

    pub const fn body(&self) -> &FunctionBody {
        &self.body
    }
}