use super::{
    expressions::Expression,
    patterns::Pattern,
    statements::Block,
    types::{FunctionSignature, Type},
};

/// A possibly namespaced function name: `map` or `Option::map`.
///
/// Grammar reference: `FunctionNamespace = (TypeIdentifier "::")? Identifier`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionPath {
    pub namespace: Option<String>,
    pub name: String,
}

impl FunctionPath {
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
    path: FunctionPath,
    /// Declared type variables, e.g. `<A, B>` in `map<A, B>([A], f: (A) -> B) -> [B];`
    type_params: Vec<String>,
    signature: FunctionSignature,
}

#[derive(Debug, Clone)]
pub struct FunctionImplementation {
    path: FunctionPath,
    arguments: Vec<Pattern>,
    body: FunctionBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    SingleLine(Expression),
    MultiLine(Block),
}

impl FunctionDeclaration {
    pub const fn new(path: FunctionPath, type_params: Vec<String>, signature: FunctionSignature) -> Self {
        Self {
            path,
            type_params,
            signature,
        }
    }

    pub fn main(ty: &Type) -> Self {
        Self {
            path: FunctionPath::unqualified("main"),
            type_params: vec![],
            signature: FunctionSignature::new(vec![], ty.to_owned()),
        }
    }

    pub const fn path(&self) -> &FunctionPath {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.path.name
    }

    pub fn type_params(&self) -> &[String] {
        &self.type_params
    }

    pub const fn signature(&self) -> &FunctionSignature {
        &self.signature
    }
}

impl FunctionImplementation {
    pub const fn new(path: FunctionPath, arguments: Vec<Pattern>, body: FunctionBody) -> Self {
        Self {
            path,
            arguments,
            body,
        }
    }

    pub fn main(block: &Block) -> Self {
        let body = FunctionBody::MultiLine(block.clone());
        Self {
            path: FunctionPath::unqualified("main"),
            arguments: vec![],
            body,
        }
    }

    pub const fn path(&self) -> &FunctionPath {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.path.name
    }

    pub fn arguments(&self) -> &[Pattern] {
        &self.arguments
    }

    pub const fn body(&self) -> &FunctionBody {
        &self.body
    }
}
