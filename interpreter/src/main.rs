use anyhow::Result;
use levis::compiler::Compilable;
use levis::parser::parse;
use levis::{compiler, vm};

use clap::Parser;

use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    script: PathBuf,
}

fn main() -> Result<()> {
    // let src = std::fs::read_to_string("../test-script/xexp.les")?;
    let cli = Cli::parse();
    let src = std::fs::read_to_string(cli.script)?;

    let (ast, spans) = parse(&src)?;
    let intermediate = ast.compile()?;
    let final_bc = compiler::intermediate_to_final(intermediate, ast);
    let res = match vm::run(final_bc, &spans) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    };
    std::process::exit(res);
}
