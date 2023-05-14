use anyhow::{anyhow, Context, Result};
use glob::glob;
use std::result::Result as StdResult;

use std::fs;
use std::process::Command;

fn main() -> Result<()> {
    compile_levis().context("compiling interpreter")?;

    let scripts: Vec<_> = glob("tests/*.les")?.collect::<StdResult<_, _>>()?;
    let outputs = scripts
        .iter()
        .map(|p| format!("tests/{}.out", p.file_stem().unwrap().to_str().unwrap()));
    for (script, expected_output) in scripts.iter().zip(outputs) {
        let expected_output = fs::read_to_string(&expected_output)
            .context(format!("loading expected output: {}", &expected_output))?;
        let output_bytes = Command::new("../interpreter/target/release/levis")
            .arg(script)
            .output()
            .context(format!("running script {}", script.display()))?
            .stdout;
        let output = String::from_utf8(output_bytes)?;
        if output == expected_output {
            println!("{}: passed", script.display());
        } else {
            println!("{}: failed\nactual output:\n{}", script.display(), output);
        }
    }
    Ok(())
}

fn compile_levis() -> Result<()> {
    let st = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir("../interpreter")
        .status()?;
    if st.success() {
        Ok(())
    } else {
        Err(anyhow!("compiling the interpreter failed"))
    }
}
