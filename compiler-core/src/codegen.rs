use std::collections::BTreeMap;

use cranelift::{
    codegen::{
        ir::{Function, UserFuncName},
        Context,
    },
    prelude::{
        types, AbiParam, Configurable, FunctionBuilder, FunctionBuilderContext, Signature, Type,
        Variable,
    },
};
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

pub struct Codegen {
    variables: BTreeMap<String, (Type, Variable)>,
    functions: BTreeMap<String, (FuncId, Signature, Vec<Type>)>,
    pub module: ObjectModule,
}

impl Default for Codegen {
    fn default() -> Self {
        let mut flags_builder = cranelift::prelude::settings::builder();
        flags_builder.set("opt_level", "none").unwrap();
        flags_builder.set("is_pic", "false").unwrap();
        flags_builder.set("enable_probestack", "false").unwrap();
        let flags = cranelift::prelude::settings::Flags::new(flags_builder);
        let isa = cranelift_native::builder().unwrap().finish(flags).unwrap();
        let module_builder =
            ObjectBuilder::new(isa, "main", cranelift_module::default_libcall_names()).unwrap();
        let module = ObjectModule::new(module_builder);
        Self {
            variables: BTreeMap::new(),
            functions: BTreeMap::new(),
            module,
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
        let mut main_sig = Signature::new(cranelift::prelude::isa::CallConv::SystemV);
        main_sig.returns.push(AbiParam::new(types::I8));

        let main_id = self
            .module
            .declare_function("main", cranelift_module::Linkage::Export, &main_sig)
            .unwrap();

        let mut func = Function::with_name_signature(UserFuncName::user(0, 0), main_sig);
        let mut func_ctx = FunctionBuilderContext::new();
        let mut fn_builder = FunctionBuilder::new(&mut func, &mut func_ctx);

        self.gen_block(entry_point, &mut fn_builder);
        fn_builder.finalize();

        eprintln!("IR:\n{}", func);

        let mut context = Context::for_function(func);
        self.module.define_function(main_id, &mut context).unwrap();
    }

    pub fn compile_program_to_object(&mut self, program: &Program) {
        self.compile_function_declarations(&program.function_declarations);
        self.compile_function_implementations(&program.function_implementations);
        self.compile_entrypoint(&program.entry_point);
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
