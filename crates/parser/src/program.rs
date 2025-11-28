use ast::program::Program;
use winnow::{Parser, Result};

use crate::top_level::parse_top_levels;

/// # Errors
/// todo
pub fn parse_program(input: &mut &str) -> Result<Program> {
    parse_top_levels
        .parse_next(input)
        .map(Program::from_top_levels)
}
