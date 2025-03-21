use cranelift::prelude::{settings::Flags, Variable};
use cranelift_module::{FuncId, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use scope::{Scope, Var};

use crate::frontend::ast::{
    functions::{FunctionDeclaration, FunctionImplementation},
    program::Program,
    statements::Block,
    types::{CustomType, FunctionSignature, Type},
};

mod custom_types;
mod expressions;
mod functions;
pub mod inference;
mod scope;
mod statements;

pub trait Generable {
    /// Returns the size in bytes of the construct
    fn size(&self) -> u32;

    fn to_cranelift(&self) -> cranelift::prelude::Type;
}

pub struct Codegen {
    scopes: Vec<Scope>,
    pub module: ObjectModule,
    flags: Flags,
}

impl Default for Codegen {
    fn default() -> Self {
        let flags_builder = cranelift::prelude::settings::builder();
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
    fn compile_type_definitions(&mut self, type_definitions: &[CustomType]) {
        for td in type_definitions {
            self.define_type(td);
        }
    }

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
        let ty = entry_point.return_expr.associated_type().unwrap();
        self.gen_function_declaration(&FunctionDeclaration::main(ty));

        self.gen_function_implementation(&FunctionImplementation::main(entry_point));
    }

    pub fn compile_program_to_object(mut self, program: &Program) -> Vec<u8> {
        self.compile_type_definitions(&program.type_definitions);
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

    fn declare_function(&mut self, func_name: &str, signature: FunctionSignature) {
        let func_id = self
            .module
            .declare_function(
                func_name,
                cranelift_module::Linkage::Export,
                &signature.to_cranelift(),
            )
            .unwrap();
        self.scopes
            .last_mut()
            .unwrap()
            .declare_function(func_name, func_id, signature);
    }

    fn define_type(&mut self, custom_type: &CustomType) {
        self.scopes.last_mut().unwrap().define_type(custom_type);
    }

    fn get_variable(&self, var_name: &str) -> Option<&Var> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.get_variable(var_name))
    }

    fn get_function(&self, func_name: &str) -> Option<&(FuncId, FunctionSignature)> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.get_function(func_name))
    }

    fn get_type(&self, type_name: &str) -> Option<&CustomType> {
        self.scopes.iter().rev().find_map(|s| s.get_type(type_name))
    }
}
