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

#[cfg(test)]
mod tests {
    use crate::program::parse_program;

    #[test]
    fn parse_simple_program() {
        let mut input = "sum(U8, U8) -> U8;
            sum(a, b) = a + b;

            main {
                x = 1;
                y = 2;
                z = sum(x, y);
                z
            }
            ";
        let _parsed = parse_program(&mut input).unwrap();
    }
}
