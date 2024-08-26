use std::path::PathBuf;

use super::{
    functions::{FunctionDefinition, FunctionImplementation},
    statements::Block,
    types::CustomType,
};

#[derive(Debug)]
pub struct Program {
    // imports: Vec<Import>,
    function_definitions: Vec<FunctionDefinition>,
    function_implementations: Vec<FunctionImplementation>,
    type_definitions: Vec<CustomType>,
    entry_point: Block,
}

impl Program {
    pub fn new(
        function_definitions: Vec<FunctionDefinition>,
        function_implementations: Vec<FunctionImplementation>,
        type_definitions: Vec<CustomType>,
        entry_point: Block,
    ) -> Self {
        Self {
            function_definitions,
            function_implementations,
            type_definitions,
            entry_point,
        }
    }
}
