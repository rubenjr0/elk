#![warn(clippy::all, clippy::perf, clippy::style)]

use codegen::Codegen;
use inference::TypeInference;

/// Exposes the pipeline for compiling source code.
pub fn compile_to_object(source: &mut &str) -> Vec<u8> {
    let mut program = parser::program::parse_program(source).unwrap();

    let mut inference = TypeInference::default();
    inference.infer_program(&mut program);

    let codegen = Codegen::default();
    codegen.compile_program_to_object(&program)
}
