use super::{
    functions::{FunctionDefinition, FunctionImplementation},
    statements::Block,
    types::CustomType,
};

#[derive(Debug)]
pub enum TopLevel {
    FunctionDefinition(FunctionDefinition),
    FunctionImplementation(FunctionImplementation),
    CustomType(CustomType),
    EntryPoint(Block),
}
