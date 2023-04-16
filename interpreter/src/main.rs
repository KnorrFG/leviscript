use anyhow::Result;
use interpreter_rs::opcode;
use interpreter_rs::parser::parse;

fn main() -> Result<()> {
    let src = std::fs::read_to_string("../test-script/xexp.les")?;
    let ast = parse(&src)?;
    dbg!(ast);
    dbg!(opcode::EXEC);
    Ok(())
}
