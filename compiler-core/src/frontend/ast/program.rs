use super::{
    functions::{FunctionDeclaration, FunctionImplementation},
    statements::Block,
    top_level::TopLevel,
    types::CustomType,
};

#[derive(Debug)]
pub struct Program {
    // imports: Vec<Import>,
    pub function_declarations: Vec<FunctionDeclaration>,
    pub function_implementations: Vec<FunctionImplementation>,
    pub type_definitions: Vec<CustomType>,
    pub entry_point: Block,
}

impl Program {
    pub fn from_top_levels(top_levels: Vec<TopLevel>) -> Self {
        let mut function_declarations: Vec<FunctionDeclaration> = vec![];
        let mut function_implementations: Vec<FunctionImplementation> = vec![];
        let mut type_definitions: Vec<CustomType> = vec![];
        let mut entry_point: Option<Block> = None;

        for top_level in top_levels {
            match top_level {
                TopLevel::FunctionDefinition(fd) => function_declarations.push(fd),
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
            function_declarations,
            function_implementations,
            type_definitions,
            entry_point,
        }
    }
}
