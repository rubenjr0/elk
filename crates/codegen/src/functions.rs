use ast::{
    expressions::Expression,
    functions::{FunctionBody, FunctionDeclaration, FunctionImplementation},
    types::FunctionSignature,
};
use cranelift::{
    codegen::{
        Context,
        ir::{Function, UserFuncName},
        verify_function,
    },
    prelude::{
        AbiParam, FunctionBuilder, FunctionBuilderContext, InstBuilder, Signature, Value,
        isa::CallConv, settings::FlagsOrIsa,
    },
};
use cranelift_module::Module;

use crate::{Codegen, Generable};

impl Generable for FunctionSignature {
    type Output = Signature;

    fn size(&self) -> u32 {
        todo!()
    }

    fn to_cranelift(&self) -> Self::Output {
        let params: Vec<_> = self
            .arguments()
            .iter()
            .map(|arg| AbiParam::new(arg.to_cranelift()))
            .collect();
        let returns = AbiParam::new(self.return_type().to_cranelift());
        Signature {
            params,
            returns: vec![returns],
            call_conv: CallConv::SystemV,
        }
    }
}

impl Codegen {
    pub fn gen_function_declaration(&mut self, function_declaration: &FunctionDeclaration) {
        let sig = function_declaration.signature();

        self.declare_function(function_declaration.name(), sig.to_owned());
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
            let mut func =
                Function::with_name_signature(UserFuncName::user(0, 0), sig.to_cranelift());

            let mut func_ctx = FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            function_implementation
                .arguments()
                .iter()
                .zip(sig.arguments())
                .enumerate()
                .for_each(|(i, (var_name, ty))| {
                    let var = codegen.declare_variable(var_name, ty.to_owned());
                    builder.declare_var(var, ty.to_cranelift());
                    let tmp = builder.block_params(entry_block)[i];
                    builder.def_var(var, tmp);
                });

            let val = match function_implementation.body() {
                FunctionBody::SingleLine(expression) => {
                    codegen.gen_expression(expression, &mut builder)
                }
                FunctionBody::MultiLine(block) => codegen.gen_block(block, &mut builder),
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

            eprintln!("== IR ==\n{:?}", func);

            let mut ctx = Context::for_function(func);
            codegen.module.define_function(fid, &mut ctx).unwrap();
        });
    }

    pub fn gen_function_call(
        &mut self,
        function_name: &str,
        args: &[Expression],
        builder: &mut FunctionBuilder,
    ) -> Value {
        let args: Vec<_> = args
            .iter()
            .map(|e| self.gen_expression(e, builder))
            .collect();
        let (func_id, _) = self.get_function(function_name).unwrap();
        let fref = self.module.declare_func_in_func(*func_id, builder.func);

        let i = builder.ins().call(fref, &args);
        builder.inst_results(i)[0]
    }
}
