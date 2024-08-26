use super::{
    functions::{FunctionDefinition, FunctionImplementation},
    statements::Block,
    types::CustomType,
};

pub enum TopLevel {
    FunctionDefinition(FunctionDefinition),
    FunctionImplementation(FunctionImplementation),
    CustomType(CustomType),
    EntryPoint(Block),
}

impl TopLevel {
    pub fn into_function_definition(&self) -> Option<&FunctionDefinition> {
        match self {
            TopLevel::FunctionDefinition(fd) => Some(fd),
            _ => None,
        }
    }

    pub fn into_function_implementation(&self) -> Option<&FunctionImplementation> {
        match self {
            TopLevel::FunctionImplementation(fi) => Some(fi),
            _ => None,
        }
    }

    pub fn into_custom_type(&self) -> Option<&CustomType> {
        match self {
            TopLevel::CustomType(ct) => Some(ct),
            _ => None,
        }
    }

    pub fn into_entry_point(&self) -> Option<&Block> {
        match self {
            TopLevel::EntryPoint(ep) => Some(ep),
            _ => None,
        }
    }
}
