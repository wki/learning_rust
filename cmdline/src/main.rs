use std::path::PathBuf;
use clap::{Parser};

#[derive(Debug)]
#[derive(Parser)]
struct Cli {
    /// Specifies the input file
    #[arg(short, long, value_name = "FILE")]
    input_file: PathBuf,

    /// Specifies the output file
    #[arg(short, long, value_name = "FILE")]
    output_file: PathBuf,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

fn main() {
    let cli = Cli::parse();

    println!("Hello, {} -> {}", cli.input_file.to_str().unwrap(), cli.output_file.to_str().unwrap());
}
