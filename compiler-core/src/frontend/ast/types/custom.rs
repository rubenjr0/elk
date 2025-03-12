use super::Type;

#[derive(Debug, Clone, PartialEq)]
/// Data type defined by the user
pub struct CustomType {
    name: String,
    content: CustomTypeContent,
    generics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
/// Data types defined by the user can be Enums, Records, or Markers (empty)
pub enum CustomTypeContent {
    Enum(Vec<Variant>),
    Record(Vec<(String, Type)>),
    Empty,
}

/// Variants of an enum, can contain types
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    name: String,
    types: Vec<Type>,
}

impl CustomType {
    pub fn new(name: &str, content: CustomTypeContent, generics: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            content,
            generics,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn content(&self) -> &CustomTypeContent {
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

    pub const fn types(&self) -> &Vec<Type> {
        &self.types
    }
}
