#![warn(clippy::all)]

mod codegen;
mod frontend;

pub use codegen::Codegen;
pub use frontend::process;
