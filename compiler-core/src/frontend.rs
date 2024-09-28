pub mod analysis;
pub mod ast;
mod inference;
pub mod parser;
mod translation;

use ast::program::Program;
use parser::parse_expr;

pub fn process(input: &str) -> Result<Program, String> {
    let (rem, program) = parser::program::parse_program(input).map_err(|e| format!("{:?}", e))?;
    assert!(rem.is_empty(), "Could not parse entire input");
    Ok(program)
}
