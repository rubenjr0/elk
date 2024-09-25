use anyhow::Result;
use clap::Parser;

use elk_core::{analyze, parse_program};

#[derive(Parser)]
struct Args {
    path: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = args.path;
    println!("Compiling {path}...");
    let src = std::fs::read_to_string(&path)?;
    match parse_program(&src) {
        Ok((_, program)) => {
            println!("{:#?}", program);
            analyze(&program).unwrap();
        }
        Err(e) => println!("{}", e),
    }

    Ok(())
}
