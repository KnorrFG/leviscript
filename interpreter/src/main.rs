use anyhow::Result;
use interpreter_rs::compiler::Compilable;
use interpreter_rs::parser::parse;
use interpreter_rs::{compiler, vm};

fn main() -> Result<()> {
    // let src = std::fs::read_to_string("../test-script/xexp.les")?;
    let src = "x{ echo ababua }\nx{echo-no uahahahaha}";
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
