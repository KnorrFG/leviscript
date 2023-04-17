use anyhow::{Context, Result};
use interpreter_rs::compiler::Compilable;
use interpreter_rs::parser::parse;
use interpreter_rs::{compiler, vm};

fn main() -> Result<()> {
    // let src = std::fs::read_to_string("../test-script/xexp.les")?;
    let src = "x{ echo ababua }\nx{echo-no uahahahaha}";
    let (ast, _) = parse(&src)?;
    let intermediate = ast.compile()?;
    let final_bc = compiler::intermediate_to_final(intermediate, ast);
    std::process::exit(vm::run(final_bc).context("Runtime error")?);
}
