#![warn(clippy::all, clippy::nursery)]

mod frontend;

pub use frontend::analysis::analyze;
pub use frontend::parser::program::parse_program;
