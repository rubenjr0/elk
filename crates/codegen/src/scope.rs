use std::collections::BTreeMap;

use ast::types::{CustomType, FunctionSignature, Type};
use cranelift::prelude::Variable;
use cranelift_module::FuncId;

/// Variable identifier, type and pointer
pub type Var = (Variable, Type);

pub struct Scope {
    variables: BTreeMap<String, Var>,
    functions: BTreeMap<String, (FuncId, FunctionSignature)>,
    types: Vec<CustomType>,
    var_counter: u32,
}

impl Scope {
    pub fn new(counter_start: u32) -> Self {
        Self {
            variables: BTreeMap::new(),
            functions: BTreeMap::new(),
            types: Vec::new(),
            var_counter: counter_start,
        }
    }

    pub fn counter(&self) -> u32 {
        self.var_counter
    }

    pub fn declare_variable(&mut self, var_name: &str, ty: Type) -> Variable {
        let var = Variable::from_u32(self.var_counter);
        self.variables.insert(var_name.to_owned(), (var, ty));
        self.var_counter += 1;
        var
    }

    pub fn declare_function(
        &mut self,
        func_name: &str,
        func_id: FuncId,
        signature: FunctionSignature,
    ) {
        self.functions
            .insert(func_name.to_owned(), (func_id, signature));
    }

    pub fn define_type(&mut self, ty: &CustomType) {
        self.types.push(ty.clone());
    }

    pub fn get_variable(&self, var_name: &str) -> Option<&Var> {
        self.variables.get(var_name)
    }

    pub fn get_function(&self, func_name: &str) -> Option<&(FuncId, FunctionSignature)> {
        self.functions.get(func_name)
    }

    pub fn get_type(&self, ty_name: &str) -> Option<&CustomType> {
        self.types.iter().find(|ty| ty.name() == ty_name)
    }
}
