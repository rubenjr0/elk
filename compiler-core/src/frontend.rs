pub mod analysis;
pub mod ast;
pub mod parser;

use ast::program::Program;

pub fn process(input: &str) -> Result<Program, String> {
    let (rem, program) = parser::program::parse_program(input).map_err(|e| format!("{:?}", e))?;
    assert!(rem.is_empty(), "Could not parse entire input");
    Ok(program)
}
