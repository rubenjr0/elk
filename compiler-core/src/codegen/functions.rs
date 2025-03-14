use cranelift::{
    codegen::{
        ir::{Function, UserFuncName},
        Context,
    },
    prelude::{FunctionBuilder, FunctionBuilderContext, InstBuilder},
};
use cranelift_module::Module;

use crate::frontend::ast::functions::{FunctionDeclaration, FunctionImplementation};

use super::Codegen;

impl Codegen {
    pub fn gen_function_declaration(&mut self, function_declaration: &FunctionDeclaration) {
        let sig = function_declaration.signature().into();

        let func_id = self
            .module
            .declare_function(
                function_declaration.name(),
                cranelift_module::Linkage::Local,
                &sig,
            )
            .unwrap();

        self.functions.insert(
            function_declaration.name().to_owned(),
            (
                func_id,
                sig,
                function_declaration
                    .signature()
                    .arguments()
                    .iter()
                    .map(|arg| arg.into())
                    .collect(),
            ),
        );
    }

    pub fn gen_function_implementation(
        &mut self,
        function_implementation: &FunctionImplementation,
    ) {
        let mut gen = Codegen::default();

        let (fid, sig, typ) = self.functions.get(function_implementation.name()).unwrap();

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
            .zip(typ)
            .enumerate()
            .for_each(|(i, (var_name, ty))| {
                let var = gen.new_variable(&var_name, ty.clone());
                builder.declare_var(var, ty.clone());
                let tmp = builder.block_params(entry_block)[i];
                builder.def_var(var, tmp);
            });

        match function_implementation.body() {
            crate::frontend::ast::functions::FunctionBody::SingleLine(expression) => {
                let ret = gen.gen_expression(expression, &mut builder);
                builder.ins().return_(&[ret]);
            }
            crate::frontend::ast::functions::FunctionBody::MultiLine(block) => {
                gen.gen_block(block, &mut builder);
            }
        };

        builder.finalize();

        let mut ctx = Context::for_function(func);

        self.module.define_function(*fid, &mut ctx).unwrap();
    }
}
