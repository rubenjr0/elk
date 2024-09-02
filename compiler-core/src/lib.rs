#![warn(clippy::all, clippy::nursery)]

mod ast;
mod parser;

pub use parser::program::parse_program;
