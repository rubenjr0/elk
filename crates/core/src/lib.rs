#![warn(clippy::all, clippy::perf, clippy::style)]

use ast::program::Program;
use codegen::Codegen;
use inference::TypeInference;

/// Exposes the pipeline for compiling source code.
pub fn compile_to_object(source: &str) -> Vec<u8> {
    let mut program = process(source).unwrap();

    let mut inference = TypeInference::default();
    inference.infer_program(&mut program);

    let codegen = Codegen::default();
    codegen.compile_program_to_object(&program)
}

fn process(input: &str) -> Result<Program, String> {
    let (rem, program) = parser::program::parse_program(input).map_err(|e| format!("{:?}", e))?;
    assert!(rem.is_empty(), "Could not parse entire input");
    Ok(program)
}
