use super::{
    functions::{FunctionDefinition, FunctionImplementation},
    statements::Block,
    top_level::TopLevel,
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

    pub fn from_top_levels(top_levels: Vec<TopLevel>) -> Self {
        let mut function_definitions: Vec<FunctionDefinition> = vec![];
        let mut function_implementations: Vec<FunctionImplementation> = vec![];
        let mut type_definitions: Vec<CustomType> = vec![];
        let mut entry_point: Option<Block> = None;

        eprintln!("Parsing from toplevels: {top_levels:#?}");

        for top_level in top_levels {
            match top_level {
                TopLevel::FunctionDefinition(fd) => function_definitions.push(fd),
                TopLevel::FunctionImplementation(fi) => function_implementations.push(fi),
                TopLevel::CustomType(ct) => type_definitions.push(ct),
                TopLevel::EntryPoint(ep) => {
                    if entry_point.is_some() {
                        panic!("Multiple entry points found");
                    }
                    entry_point = Some(ep);
                }
            }
        }
        let entry_point = entry_point.expect("No entry point found");

        Self {
            function_definitions,
            function_implementations,
            type_definitions,
            entry_point,
        }
    }
}
