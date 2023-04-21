use anyhow::Result;
use clap::Parser;
use levis::bytecode::{Scopes, StackInfo};
use pest::Parser as _;

use levis::compiler::Compilable;
use levis::parser;
use levis::{compiler, vm};

use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    script: PathBuf,

    #[cfg(feature = "dev")]
    #[arg(short, long)]
    show_parse_tree: bool,
}

fn main() -> Result<()> {
    // let src = std::fs::read_to_string("../test-script/xexp.les")?;
    let cli = Cli::parse();
    let src = std::fs::read_to_string(cli.script)?;
    let parse_tree = parser::LsParser::parse(parser::Rule::file, &src)?;

    if cfg!(feature = "dev") {
        if cli.show_parse_tree {
            println!("{:#?}", parse_tree);
        }
    }

    let (ast, spans) = parser::to_ast(parse_tree)?;
    let intermediate = ast.compile(&Scopes::default(), &StackInfo::default())?;
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
