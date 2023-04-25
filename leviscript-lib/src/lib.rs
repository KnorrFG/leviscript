//! Currently, what you need to do to execute a script is the following:
//! 1. load a source file into a string.
//! 1. convert that into a parse tree using `parser::LsParser::parse`
//! 1. convert the parse tree to an ast using [`parser::to_ast`]
//! 1. compile the ast to intermediate bytecode using the .compile() method on the ast-node
//!    that is implemented in the [`compiler::Compilable`] trait
//! 1. generate the final bytecode from that by using [`compiler::intermediate_to_final`]
//! 1. create an instance of [`core::Memory`], and create a program pointer from the final
//!    bytecodes text field:
//!   
//!    ```
//!    let mut mem = vm::Memory {
//!        stack: vec![],
//!        data: &final_bytecode.data,
//!    };
//!    let mut pc = final_bytecode.text.as_ptr();
//!    ```
//!
//! 1. run a loop like this and add error handling:
//!
//!    ```
//!    use vm::ExecOutcome::*;
//!    loop {
//!        let disc_ptr = pc as *const u16;
//!        match unsafe { OpCode::dispatch_discriminant(*disc_ptr, pc, &mut mem) }? {
//!            Pc(new_pc) => { pc = new_pc; }
//!            ExitCode(res) => return Ok(res),
//!        }
//!    }
//!    ```
//!
//!
//!
pub mod compiler;
pub mod core;
pub mod parser;
pub mod utils;
pub mod vm;
