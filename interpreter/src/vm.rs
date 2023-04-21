use crate::{
    bytecode::{self, Data, DataAs},
    opcode::{self, DataRef, OpCode},
    parser,
};
use pest::Span;
use std::{any::type_name, process};
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
        as type {expected_type}"
    )]
    TypeError {
        accessed_data: Data,
        expected_type: &'static str,
    },

    #[error("Unexpected stack entry at {index} via {data_ref:?}: {msg}")]
    UnexpectedStackEntry {
        index: usize,
        msg: String,
        data_ref: DataRef,
    },

    #[error("Unexpected Opcode at {pc:?}, expected {expected}, found:\n{found}")]
    UnexpectedOpcode {
        pc: *const u8,
        expected: String,
        found: String,
    },

    #[error("Opcode at pc is Non-executable")]
    NonExecutableOpCode,
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

macro_rules! bail{
    ($($err:tt)*) => {
        return Err(VmError::$($err)*);
    };
}

pub unsafe fn exec(pc: *const u8, mem: &mut Memory) -> Result<ExecOutcome, VmError> {
    let opcode = OpCode::from_ptr(pc);
    use ExecOutcome::*;
    use OpCode::*;
    Ok(match opcode {
        Exec((bin, args)) => {
            let bin_name = mem.get_as::<&str>(&bin)?;
            let args = mem.get_as::<Vec<&str>>(&args)?;
            let stat = process::Command::new(bin_name)
                .args(args)
                .status()
                .map_err(|e| rt_err!("Executing {}: {}", bin_name, e))?;
            rt_assert!(stat.success(), "{} did not execute successfully", bin_name);
            Pc(pc.offset(opcode.serialized_size() as isize))
        }
        Exit(res) => ExitCode(res),
        DataRef(_) => return Err(VmError::NonExecutableOpCode),
        StrCat(n) => {
            let elems: Vec<&str> = (0..n)
                .map(|i| {
                    let addr = pc.offset(
                        (opcode.serialized_size() + i * OpCode::serialized_size_of(opcode::DATAREF))
                            as isize,
                    );
                    let oc = OpCode::from_ptr(addr);
                    if let OpCode::DataRef(dref) = oc {
                        mem.get_as::<&str>(&dref)
                    } else {
                        bail!(UnexpectedOpcode {
                            pc: addr,
                            expected: "DataRef".into(),
                            found: format!("{:#?}", oc)
                        });
                    }
                })
                .collect::<Result<_, _>>()?;
            mem.stack
                .push(StackEntry::Entry(Data::String(elems.join(""))));
            Pc(pc.offset(
                (opcode.serialized_size() + n * OpCode::serialized_size_of(opcode::DATAREF))
                    as isize,
            ))
        }
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

pub trait FromMemory<'a>: Sized {
    fn get_as(mem: &'a Memory, dref: &'a Data) -> Result<Self, VmError>;
}

/// get's any Datatype which has a data as, which means all primitive ones
impl<'a, T: DataAs<'a>> FromMemory<'a> for T {
    fn get_as(mem: &'a Memory, dref: &'a Data) -> Result<T, VmError> {
        if let Data::Ref(dref) = dref {
            mem.get_as(dref)
        } else {
            dref.get_as().ok_or_else(|| VmError::TypeError {
                accessed_data: dref.clone(),
                expected_type: type_name::<T>(),
            })
        }
    }
}

impl<'a, T: DataAs<'a>> FromMemory<'a> for Vec<T> {
    fn get_as(mem: &'a Memory, dref: &'a Data) -> Result<Vec<T>, VmError> {
        match dref {
            Data::Vec(elems) => elems
                .iter()
                .map(|e| <T as FromMemory>::get_as(mem, e))
                .collect(),
            _ => Err(VmError::TypeError {
                accessed_data: dref.clone(),
                expected_type: type_name::<T>(),
            }),
        }
    }
}

impl Memory {
    pub fn get_as<'a, T: FromMemory<'a>>(&'a self, reference: &DataRef) -> Result<T, VmError> {
        let data = self.resolve_ref(reference)?;
        <T as FromMemory>::get_as(self, data)
    }

    pub fn resolve_ref(&self, dref: &DataRef) -> Result<&Data, VmError> {
        use DataRef::*;
        match dref {
            StackIdx(i) => {
                if let StackEntry::Entry(e) = &self.stack[*i] {
                    Ok(e)
                } else {
                    Err(VmError::UnexpectedStackEntry {
                        index: *i,
                        msg: format!("Expected Entry, found {:#?}", self.stack[*i]),
                        data_ref: *dref,
                    })
                }
            }
            DataSectionIdx(i) => Ok(&self.data[*i]),
        }
    }
}
