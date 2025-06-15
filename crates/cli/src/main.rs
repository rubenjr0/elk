use anyhow::Result;
use clap::Parser;
use core::compile_to_object;

#[derive(Parser)]
struct Args {
    /// Input file path
    input_path: String,

    /// Output file path
    #[arg(short, long, default_value = "temp.o")]
    output_path: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = args.input_path;
    println!("Compiling {path}...");
    let src = std::fs::read_to_string(&path)?;

    let compiled = compile_to_object(&src);

    std::fs::write(&args.output_path, compiled)?;
    // std::process::Command::new("gcc")
    //     .arg(&args.output_path)
    //     .status()
    //     .unwrap();

    // std::fs::remove_file(args.output_path).unwrap();

    Ok(())
}
