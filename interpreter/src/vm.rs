use crate::{
    bytecode::{self, Data, DataAs},
    opcode::{DataRef, OpCode},
    parser,
};
use pest::Span;
use std::process;
use thiserror::Error;

pub struct VmState {
    pub pc: *const u8,
    pub memory: Memory,
}

pub struct Memory {
    pub stack: Vec<StackEntry>,
    pub data: Vec<Data>,
}

#[derive(Debug)]
pub enum StackEntry {
    FrameBorder,
    Entry(Data),
}

#[derive(Error, Debug)]
pub enum VmError {
    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error(
        "TypeError, tried to access:\n{accessed_data:#?} \n\
        as type {expected_type} via {data_ref:?}"
    )]
    TypeError {
        accessed_data: Data,
        expected_type: &'static str,
        data_ref: DataRef,
    },

    #[error("Unexpected stack entry at {index} via {data_ref:?}: {msg}")]
    UnexpectedStackEntry {
        index: usize,
        msg: String,
        data_ref: DataRef,
    },
}

pub enum ExecOutcome {
    Pc(*const u8),
    ExitCode(i32),
}

macro_rules! rt_err{
    ($msg:literal $(, $args:expr)*) => { VmError::Runtime(format!($msg $(, $args)*)) };
}

macro_rules! rt_assert{
    ($cond:expr, $msg:literal $(, $args:expr)*) => {
        if ! $cond { return Err(rt_err!($msg $(, $args)*)); }
    };
}

pub unsafe fn exec(pc: *const u8, mem: &mut Memory) -> Result<ExecOutcome, VmError> {
    let opcode = OpCode::from_ptr(pc);
    use ExecOutcome::*;
    use OpCode::*;
    Ok(match opcode {
        Exec((bin, args)) => {
            let bin_name = mem.get_as::<&str>(bin)?;
            let args = mem.get_as::<Vec<&str>>(args)?;
            let stat = process::Command::new(bin_name)
                .args(args)
                .status()
                .map_err(|e| rt_err!("Executing {}: {}", bin_name, e))?;
            rt_assert!(stat.success(), "{} did not execute successfully", bin_name);
            Pc(pc.offset(opcode.serialized_size() as isize))
        }
        Exit(res) => ExitCode(res),
    })
}

pub fn run(bc: bytecode::Final, spans: &[Span]) -> Result<i32, String> {
    let mut mem = Memory {
        stack: vec![],
        data: bc.data,
    };
    let mut pc = bc.text.as_ptr();

    use ExecOutcome::*;
    loop {
        match unsafe { exec(pc, &mut mem) } {
            Ok(Pc(new_pc)) => {
                pc = new_pc;
            }
            Ok(ExitCode(res)) => return Ok(res),
            Err(VmError::Runtime(msg)) => {
                let byte_offset = pc as usize - bc.text.as_ptr() as usize;
                let opcode_index = bc.header.index[&byte_offset];
                let ast_id = bc.header.ast_ids[opcode_index];

                return Err(format!(
                    "Runtime error: {}",
                    pest::error::Error::new_from_span(
                        pest::error::ErrorVariant::<parser::Rule>::CustomError { message: msg },
                        spans[ast_id]
                    )
                ));
            }
            Err(e) => return Err(format!("VM-Error: {}", e)),
        }
    }
}

impl Memory {
    pub fn get_as<'a, T: DataAs<'a>>(&'a self, reference: DataRef) -> Result<T, VmError> {
        use DataRef::*;
        let mk_type_error = |e| VmError::TypeError {
            accessed_data: e,
            expected_type: std::any::type_name::<T>(),
            data_ref: reference,
        };
        match reference {
            StackIdx(i) => {
                if let StackEntry::Entry(e) = &self.stack[i] {
                    e.get_as().ok_or_else(|| mk_type_error(e.clone()))
                } else {
                    Err(VmError::UnexpectedStackEntry {
                        index: i,
                        msg: format!("Expected Entry, found {:#?}", self.stack[i]),
                        data_ref: reference,
                    })
                }
            }
            DataSectionIdx(i) => {
                let e = &self.data[i];
                e.get_as().ok_or_else(|| mk_type_error(e.clone()))
            }
        }
    }
}
