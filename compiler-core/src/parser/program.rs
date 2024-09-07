use nom::IResult;

use crate::ast::program::Program;

use super::top_level::parse_top_levels;

pub fn parse_program(input: &str) -> IResult<&str, Program> {
    let (remaining, top_levels) = parse_top_levels(input)?;
    eprintln!("Remaining: {remaining:?}" );
    let program = Program::from_top_levels(top_levels);
    Ok((remaining, program))
}
