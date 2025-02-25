use anyhow::Result;
use clap::Parser;
use elk_core::{process, Codegen};

#[derive(Parser)]
struct Args {
    input_path: String,
    output_path: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = args.input_path;
    println!("Compiling {path}...");
    let src = std::fs::read_to_string(&path)?;
    let program = process(&src).unwrap();

    let mut codegen = Codegen::default();
    let compiled = codegen.compile_program(&program);

    let output_path = args.output_path.unwrap_or_else(|| "a.out".to_string());
    std::fs::write(output_path, compiled)?;

    Ok(())
}
