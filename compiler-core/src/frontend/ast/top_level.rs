use super::{
    functions::{FunctionDeclaration, FunctionImplementation},
    statements::Block,
    types::CustomType,
};

#[derive(Debug)]
pub enum TopLevel {
    FunctionDefinition(FunctionDeclaration),
    FunctionImplementation(FunctionImplementation),
    CustomType(CustomType),
    EntryPoint(Block),
}
