use anyhow::{Context, Result};
use clap::Parser;

use leviscript_lib::compiler::Compilable;
use leviscript_lib::parser::{PestErrVariant, PestError, PestParser, Span};
use leviscript_lib::type_inference::{inference_start, TypeInferable};
use leviscript_lib::{core::*, parser, vm};

use std::path::PathBuf;

#[cfg(feature = "dev")]
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
    let file_name = cli.script.display().to_string();
    let src = std::fs::read_to_string(&cli.script).context(format!(
        "loading script: {}\ncwd:{}",
        cli.script.display(),
        std::env::current_dir()?.display()
    ))?;
    let parse_tree = parser::LsParser::parse(parser::Rule::file, &src)?;

    #[cfg(feature = "dev")]
    if cli.show_parse_tree {
        println!("{:#?}", parse_tree);
        return Ok(());
    }

    let (ast, spans) = parser::to_ast(parse_tree)?;
    #[cfg(feature = "dev")]
    if cli.show_ast {
        println!("{:#?}", ast);
        return Ok(());
    }

    let (def_env, def_t_idx) = inference_start();
    let (_, type_index) = match ast.infer_types(def_env, def_t_idx) {
        Ok(x) => x,
        Err(e) => exit_with_error(&e.to_string(), e.get_ast_id(), &spans, &file_name),
    };
    let compilation_result = ast.compile(ByteCodeBuilder::default(), &type_index);
    match compilation_result {
        Ok(builder) => {
            #[cfg(feature = "dev")]
            if cli.show_byte_code {
                for c in builder.text {
                    println!("{:?}", c);
                }
                return Ok(());
            }

            #[cfg(feature = "dev")]
            let opcodes: Vec<OpCode> = builder.text.iter().cloned().collect();
            let (final_bc, debug_info) = builder.build();
            #[cfg(feature = "dev")]
            if cli.debug_bytecode {
                use crossterm::{self as ct, terminal};
                use std::io::stdout;

                ct::execute!(stdout(), terminal::EnterAlternateScreen)?;
                let res = debugger::run(final_bc, debug_info, &opcodes, &spans, &src);
                ct::execute!(stdout(), terminal::LeaveAlternateScreen)?;
                return res;
            }

            let res = match run(final_bc, &spans, &debug_info) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{}", e);
                    1
                }
            };
            std::process::exit(res);
        }
        Err(e) => {
            exit_with_error(&e.to_string(), e.get_ast_id(), &spans, &file_name);
        }
    }
}

pub fn run(bc: ByteCode, spans: &[Span], dinfo: &DebugInformation) -> Result<i32, String> {
    let mut runner = Runner::new(bc);
    runner.reset_pc();

    loop {
        match unsafe { runner.step() } {
            StepResult::Ok => {}
            StepResult::Done(res) => return Ok(res),
            StepResult::Err(vm::Error::Runtime(msg)) => {
                let ast_id = runner.pc_to_ast_id(dinfo);

                return Err(format!(
                    "Runtime error: {}",
                    PestError::new_from_span(
                        PestErrVariant::<parser::Rule>::CustomError { message: msg },
                        spans[ast_id]
                    )
                ));
            }
            StepResult::Err(e) => return Err(format!("VM-Error: {}", e)),
        }
    }
}

pub fn format_error(msg: &str, ast_id: usize, spans: &[Span], file_name: &str) -> String {
    format!(
        "Compilation error: {}",
        PestError::new_from_span(
            PestErrVariant::<parser::Rule>::CustomError {
                message: msg.into()
            },
            spans[ast_id]
        )
        .with_path(file_name)
    )
}

pub fn exit_with_error(msg: &str, ast_id: usize, spans: &[Span], file_name: &str) -> ! {
    eprintln!("{}", format_error(msg, ast_id, spans, file_name));
    std::process::exit(1);
}
