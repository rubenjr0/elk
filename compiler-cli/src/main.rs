use clap::Parser;

use elk_core::parse_program;

#[derive(Parser)]
struct Args {
    path: String,
}

fn main() {
    let args = Args::parse();
    let path = args.path;
    println!("Compiling {path}...");
    let src = std::fs::read_to_string(&path).unwrap();
    match parse_program(&src) {
        Ok((_, program)) => println!("{:#?}", program),
        Err(e) => println!("{}", e),
    }
}
