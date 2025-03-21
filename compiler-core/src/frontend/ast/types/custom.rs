use crate::codegen::Generable;

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
    Record(Vec<Field>),
    Empty,
}

/// Variants of an enum, can contain types
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    name: String,
    types: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    name: String,
    ty: Type,
}

impl CustomType {
    pub fn new(name: &str, content: CustomTypeContent, generics: Vec<String>) -> Self {
        Self {
            name: name.to_owned(),
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

    pub fn size(&self) -> u32 {
        match &self.content {
            CustomTypeContent::Enum(variants) => variants
                .iter()
                .map(|v| v.types.iter().map(|t| t.size()).sum::<u32>())
                .sum(),
            CustomTypeContent::Record(items) => items.iter().map(|f| f.ty.size()).sum(),
            CustomTypeContent::Empty => 0,
        }
    }

    pub fn get_record_fields(&self) -> Option<&Vec<Field>> {
        if let CustomTypeContent::Record(fields) = &self.content {
            Some(fields)
        } else {
            None
        }
    }

    pub fn get_enum_variants(&self) -> Option<&Vec<Variant>> {
        if let CustomTypeContent::Enum(variants) = &self.content {
            Some(variants)
        } else {
            None
        }
    }
}

impl Variant {
    pub fn new(name: &str, types: Vec<Type>) -> Self {
        Self {
            name: name.to_owned(),
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

impl Field {
    pub fn new(name: &str, ty: Type) -> Self {
        Self {
            name: name.to_owned(),
            ty,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ty(&self) -> &Type {
        &self.ty
    }
}
