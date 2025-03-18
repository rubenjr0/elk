use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use cranelift::prelude::{settings::Flags, Configurable, Signature, Type, Variable};
use cranelift_module::FuncId;
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

#[derive(Clone)]
pub struct Codegen {
    variables: BTreeMap<String, (Type, Variable)>,
    functions: BTreeMap<String, (u32, FuncId, Signature, Vec<Type>)>,
    pub module: Rc<RefCell<ObjectModule>>,
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
        let module = Rc::new(RefCell::new(module));
        Self {
            variables: BTreeMap::new(),
            functions: BTreeMap::new(),
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
        Rc::into_inner(self.module)
            .unwrap()
            .into_inner()
            .finish()
            .emit()
            .unwrap()
    }
}

impl Codegen {
    pub fn new_variable(&mut self, var_name: &str, ty: Type) -> Variable {
        let idx = self.variables.len();
        let var = Variable::from_u32(idx as u32);
        self.variables.insert(var_name.to_owned(), (ty, var));
        var
    }
}
