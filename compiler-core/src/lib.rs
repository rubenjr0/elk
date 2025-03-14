#![warn(clippy::all)]

mod codegen;
mod frontend;

use codegen::{inference::TypeInference, Codegen};
use frontend::process;

/// Exposes the pipeline for compiling source code.
pub fn compile_to_object(source: &str) -> Vec<u8> {
    let mut program = process(source).unwrap();

    let mut inference = TypeInference::default();
    inference.infer_program(&mut program);

    let mut codegen = Codegen::default();
    codegen.compile_program_to_object(&program);
    codegen.module.finish().emit().unwrap()
}
