use anyhow::Result;
use clap::Parser;
use elk_core::process;

#[derive(Parser)]
struct Args {
    path: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = args.path;
    println!("Compiling {path}...");
    let src = std::fs::read_to_string(&path)?;
    process(&src).unwrap();

    Ok(())
}
