use super::Type;

#[derive(Debug)]
pub struct CustomType {
    name: String,
    content: CustomTypeContent,
}

#[derive(Debug, PartialEq)]
pub enum CustomTypeContent {
    Variants(Vec<Variant>),
    Fields(Vec<(String, Type)>),
    Empty,
}

/// Named: `VariantA, VariantB`
/// Tuple: `VariantA(U8, Bool), VariantB(I8)`
#[derive(Debug, PartialEq)]
pub struct Variant {
    name: String,
    types: Vec<Type>,
}

impl CustomType {
    pub fn new(name: &str, content: CustomTypeContent) -> Self {
        Self {
            name: name.to_string(),
            content,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn content(&self) -> &CustomTypeContent {
        &self.content
    }
}

impl Variant {
    pub fn new(name: &str, types: Vec<Type>) -> Self {
        Self {
            name: name.to_string(),
            types,
        }
    }

    pub fn named(name: &str) -> Self {
        Self::new(name, vec![])
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn types(&self) -> &Vec<Type> {
        &self.types
    }
}
