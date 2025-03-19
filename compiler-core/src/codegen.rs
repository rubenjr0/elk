use std::collections::BTreeMap;

use cranelift::prelude::{settings::Flags, Configurable, Signature, Type, Variable};
use cranelift_module::{FuncId, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use crate::frontend::ast::{
    functions::{FunctionDeclaration, FunctionImplementation},
    program::Program,
    statements::Block,
};

mod expressions;
mod functions;
pub mod inference;
mod statements;

struct Scope {
    variables: BTreeMap<String, (Type, Variable)>,
    functions: BTreeMap<String, (FuncId, Signature)>,
    var_counter: u32,
}

impl Scope {
    fn new(counter_start: u32) -> Self {
        Self {
            variables: BTreeMap::new(),
            functions: BTreeMap::new(),
            var_counter: counter_start,
        }
    }

    fn counter(&self) -> u32 {
        self.var_counter
    }

    fn declare_variable(&mut self, var_name: &str, ty: Type) -> Variable {
        let var = Variable::from_u32(self.var_counter);
        self.variables.insert(var_name.to_owned(), (ty, var));
        self.var_counter += 1;
        var
    }

    fn declare_function(&mut self, func_name: &str, func_id: FuncId, signature: Signature) {
        self.functions
            .insert(func_name.to_owned(), (func_id, signature));
    }

    fn get_variable(&self, var_name: &str) -> Option<&(Type, Variable)> {
        self.variables.get(var_name)
    }

    fn get_function(&self, func_name: &str) -> Option<&(FuncId, Signature)> {
        self.functions.get(func_name)
    }
}

pub struct Codegen {
    scopes: Vec<Scope>,
    pub module: ObjectModule,
    flags: Flags,
}

impl Default for Codegen {
    fn default() -> Self {
        let mut flags_builder = cranelift::prelude::settings::builder();
        flags_builder.set("opt_level", "none").unwrap();
        flags_builder.set("is_pic", "false").unwrap();
        flags_builder.set("enable_probestack", "false").unwrap();
        let flags = cranelift::prelude::settings::Flags::new(flags_builder);
        let isa = cranelift_native::builder()
            .unwrap()
            .finish(flags.clone())
            .unwrap();
        let module_builder =
            ObjectBuilder::new(isa, "main", cranelift_module::default_libcall_names()).unwrap();
        let module = ObjectModule::new(module_builder);
        Self {
            scopes: vec![Scope::new(0)],
            module,
            flags,
        }
    }
}

/// Pipeline implementations
impl Codegen {
    fn compile_function_declarations(&mut self, function_declarations: &[FunctionDeclaration]) {
        for fd in function_declarations {
            self.gen_function_declaration(fd);
        }
    }

    fn compile_function_implementations(
        &mut self,
        function_implementations: &[FunctionImplementation],
    ) {
        for fi in function_implementations {
            self.gen_function_implementation(fi);
        }
    }

    fn compile_entrypoint(&mut self, entry_point: &Block) {
        self.gen_function_declaration(&FunctionDeclaration::main());

        self.gen_function_implementation(&FunctionImplementation::main(entry_point));
    }

    pub fn compile_program_to_object(mut self, program: &Program) -> Vec<u8> {
        self.compile_function_declarations(&program.function_declarations);
        self.compile_function_implementations(&program.function_implementations);
        self.compile_entrypoint(&program.entry_point);
        self.module.finish().emit().unwrap()
    }
}

/// Scoping
impl Codegen {
    fn enter_scope(&mut self) {
        let current_counter = self.scopes.last().map_or(0, |scope| scope.counter());
        self.scopes.push(Scope::new(current_counter + 1));
    }

    fn exit_scope(&mut self) {
        self.scopes.pop().expect("Cannot exit global scope");
    }

    fn with_scope<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.enter_scope();
        let result = f(self);
        self.exit_scope();
        result
    }

    pub fn declare_variable(&mut self, var_name: &str, ty: Type) -> Variable {
        self.scopes
            .last_mut()
            .unwrap()
            .declare_variable(var_name, ty)
    }

    fn declare_function(&mut self, func_name: &str, signature: Signature) {
        let func_id = self
            .module
            .declare_function(func_name, cranelift_module::Linkage::Export, &signature)
            .unwrap();
        self.scopes
            .last_mut()
            .unwrap()
            .declare_function(func_name, func_id, signature);
    }

    fn get_variable(&self, var_name: &str) -> Option<&(Type, Variable)> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.get_variable(var_name))
    }

    fn get_function(&self, func_name: &str) -> Option<&(FuncId, Signature)> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.get_function(func_name))
    }
}
