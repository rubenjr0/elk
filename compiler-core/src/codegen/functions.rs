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

        self.declare_function(function_declaration.name(), sig);
    }

    pub fn gen_function_implementation(
        &mut self,
        function_implementation: &FunctionImplementation,
    ) {
        let (fid, sig) = self
            .get_function(function_implementation.name())
            .unwrap()
            .clone();
        self.with_scope(|codegen| {
            let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig.clone());

            let mut func_ctx = FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            function_implementation
                .arguments()
                .iter()
                .zip(sig.params.iter())
                .enumerate()
                .for_each(|(i, (var_name, ty))| {
                    let var = codegen.declare_variable(var_name, ty.value_type);
                    builder.declare_var(var, ty.value_type);
                    let tmp = builder.block_params(entry_block)[i];
                    builder.def_var(var, tmp);
                });

            let val = match function_implementation.body() {
                crate::frontend::ast::functions::FunctionBody::SingleLine(expression) => {
                    codegen.gen_expression(expression, &mut builder)
                }
                crate::frontend::ast::functions::FunctionBody::MultiLine(block) => {
                    codegen.gen_block(block, &mut builder)
                }
            };

            builder.ins().return_(&[val]);
            builder.finalize();

            verify_function(
                &func,
                FlagsOrIsa {
                    flags: &codegen.flags,
                    isa: None,
                },
            )
            .unwrap();

            eprintln!("IR: {func}");

            let mut ctx = Context::for_function(func);
            codegen.module.define_function(fid, &mut ctx).unwrap();
        });
    }
}
