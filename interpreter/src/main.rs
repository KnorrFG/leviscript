use anyhow::Result;
use clap::Parser;
use crossterm::{self as ct, terminal};

use leviscript_lib::bytecode::{self, Scopes, StackInfo};
use leviscript_lib::compiler::{self, Compilable};
use leviscript_lib::parser::{PestErrVariant, PestError, PestParser, Span};
use leviscript_lib::{opcode, parser, vm};

use std::io::stdout;
use std::path::PathBuf;

mod debugger;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    script: PathBuf,

    #[cfg(feature = "dev")]
    #[arg(short = 'p', long)]
    show_parse_tree: bool,

    #[cfg(feature = "dev")]
    #[arg(short = 'i', long)]
    show_byte_code: bool,

    #[cfg(feature = "dev")]
    #[arg(short = 'a', long)]
    show_ast: bool,

    #[cfg(feature = "dev")]
    #[arg(short = 'b', long)]
    debug_bytecode: bool,
}

fn main() -> Result<()> {
    // let src = std::fs::read_to_string("../test-script/xexp.les")?;
    let cli = Cli::parse();
    let src = std::fs::read_to_string(cli.script)?;
    let parse_tree = parser::LsParser::parse(parser::Rule::file, &src)?;

    if cfg!(feature = "dev") && cli.show_parse_tree {
        println!("{:#?}", parse_tree);
        return Ok(());
    }

    let (ast, spans) = parser::to_ast(parse_tree)?;
    if cfg!(feature = "dev") && cli.show_ast {
        println!("{:#?}", ast);
        return Ok(());
    }

    let intermediate = ast.compile(&Scopes::default(), &StackInfo::default())?;
    if cfg!(feature = "dev") && cli.show_byte_code {
        dbg!(&intermediate);
        return Ok(());
    }

    let final_bc = compiler::intermediate_to_final(intermediate.clone(), ast);
    if cfg!(feature = "dev") && cli.debug_bytecode {
        let mut stdout = stdout();
        ct::execute!(stdout, terminal::EnterAlternateScreen)?;
        let res = debugger::run(&final_bc, &intermediate, &spans, &src, &mut stdout);
        ct::execute!(stdout, terminal::LeaveAlternateScreen)?;
        res
    } else {
        let res = match run(final_bc, &spans) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("{}", e);
                1
            }
        };
        std::process::exit(res);
    }
}

pub fn run(bc: bytecode::Final, spans: &[Span]) -> Result<i32, String> {
    let mut mem = vm::Memory {
        stack: vec![],
        data: &bc.data,
    };
    let mut pc = bc.text.as_ptr();

    use vm::ExecOutcome::*;
    loop {
        let disc_ptr = pc as *const u16;
        match unsafe { opcode::dispatch_discriminant(*disc_ptr, pc, &mut mem) } {
            Ok(Pc(new_pc)) => {
                pc = new_pc;
            }
            Ok(ExitCode(res)) => return Ok(res),
            Err(vm::Error::Runtime(msg)) => {
                let byte_offset = pc as usize - bc.text.as_ptr() as usize;
                let opcode_index = bc.header.index[&byte_offset];
                let ast_id = bc.header.ast_ids[opcode_index];

                return Err(format!(
                    "Runtime error: {}",
                    PestError::new_from_span(
                        PestErrVariant::<parser::Rule>::CustomError { message: msg },
                        spans[ast_id]
                    )
                ));
            }
            Err(e) => return Err(format!("VM-Error: {}", e)),
        }
    }
}
