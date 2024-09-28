use std::sync::Arc;

use codegen::Context;
use cranelift::prelude::*;
use isa::TargetIsa;

use crate::frontend::ast::{
    functions::{FunctionDefinition, FunctionImplementation},
    program::Program,
    statements::Block,
    types::CustomType,
};

pub struct Codegen {
    ctx: Context,
    builder_context: FunctionBuilderContext,
    isa: Arc<dyn TargetIsa>,
}

impl Codegen {
    pub fn new() -> Self {
        let ctx = cranelift::codegen::Context::new();
        let builder_context = FunctionBuilderContext::new();
        let flag_builder = settings::builder();
        let flags = settings::Flags::new(flag_builder);
        let isa = cranelift_native::builder().unwrap().finish(flags).unwrap();

        Self {
            ctx,
            builder_context,
            isa,
        }
    }

    pub fn compile_program(&mut self, program: &Program) -> Vec<u8> {
        for custom_type in program.custom_types() {
            self.compile_custom_type(custom_type);
        }

        for function_definition in program.function_definitions() {
            self.compile_function_definition(function_definition);
        }

        for function_implementation in program.function_implementations() {
            self.compile_function_implementation(function_implementation);
        }

        self.compile_block(program.entry_point())
    }

    fn compile_custom_type(&mut self, custom_type: &CustomType) {
        eprintln!("Compiling custom type: {:?}", custom_type);
    }

    fn compile_function_definition(&mut self, function_definition: &FunctionDefinition) {
        eprintln!("Compiling function definition: {:?}", function_definition);
    }

    fn compile_function_implementation(
        &mut self,
        function_implementation: &FunctionImplementation,
    ) {
        eprintln!(
            "Compiling function implementation: {:?}",
            function_implementation
        );
    }

    // WIP, DRAFT, NOT IMPLEMENTED
    fn compile_block(&mut self, block: &Block) -> Vec<u8> {
        eprintln!("Compiling block: {:?}", block);
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        let blk = builder.create_block();
        builder.switch_to_block(blk);
        builder.ins().return_(&[]);
        builder.seal_block(blk);
        builder.finalize();
        let mut buffer = Vec::new();
        self.ctx
            .compile_and_emit(
                self.isa.as_ref(),
                &mut buffer,
                &mut codegen::control::ControlPlane::default(),
            )
            .unwrap();
        buffer
    }
}
