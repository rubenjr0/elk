pub mod analysis;
mod ast;
pub mod parser;

pub fn process(input: &str) -> Result<(), String> {
    let (rem, program) = parser::program::parse_program(input).map_err(|e| format!("{:?}", e))?;
    assert!(rem.is_empty(), "Could not parse entire input");
    eprintln!("{:#?}", program);
    analysis::analyze(&program)
}
