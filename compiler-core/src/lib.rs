#![warn(clippy::all, clippy::nursery)]

mod codegen;
mod frontend;

pub use codegen::Codegen;
pub use frontend::process;
