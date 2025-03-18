use cranelift::{
    codegen::{
        ir::{Function, UserFuncName},
        verify_function, Context,
    },
    prelude::{settings::FlagsOrIsa, FunctionBuilder, FunctionBuilderContext, InstBuilder},
};
use cranelift_module::Module;

use crate::frontend::ast::functions::{FunctionDeclaration, FunctionImplementation};

use super::Codegen;

impl Codegen {
    pub fn gen_function_declaration(&mut self, function_declaration: &FunctionDeclaration) {
        let sig = function_declaration.signature().to_cranelift();

        let func_id = self
            .module
            .borrow_mut()
            .declare_function(
                function_declaration.name(),
                cranelift_module::Linkage::Export,
                &sig,
            )
            .unwrap();

        self.functions.insert(
            function_declaration.name().to_owned(),
            (
                self.functions.len() as u32,
                func_id,
                sig,
                function_declaration
                    .signature()
                    .arguments()
                    .iter()
                    .map(|arg| arg.to_cranelift())
                    .collect(),
            ),
        );
    }

    pub fn gen_function_implementation(
        &mut self,
        function_implementation: &FunctionImplementation,
    ) {
        let mut gen = self.clone();

        let (idx, fid, sig, typ) = self.functions.get(function_implementation.name()).unwrap();

        let mut func = Function::with_name_signature(UserFuncName::user(0, *idx), sig.clone());

        let mut func_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        function_implementation
            .arguments()
            .iter()
            .zip(typ)
            .enumerate()
            .for_each(|(i, (var_name, ty))| {
                let var = gen.new_variable(var_name, *ty);
                builder.declare_var(var, *ty);
                let tmp = builder.block_params(entry_block)[i];
                builder.def_var(var, tmp);
            });

        let val = match function_implementation.body() {
            crate::frontend::ast::functions::FunctionBody::SingleLine(expression) => {
                gen.gen_expression(expression, &mut builder)
            }
            crate::frontend::ast::functions::FunctionBody::MultiLine(block) => {
                gen.gen_block(block, &mut builder)
            }
        };

        builder.ins().return_(&[val]);

        builder.finalize();
        verify_function(
            &func,
            FlagsOrIsa {
                flags: &self.flags,
                isa: None,
            },
        )
        .unwrap();

        eprintln!("{func}");

        let mut ctx = Context::for_function(func);

        self.module
            .borrow_mut()
            .define_function(*fid, &mut ctx)
            .unwrap();
    }
}
