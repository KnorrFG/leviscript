use crate::{
    bytecode::{Data, DataAs},
    opcode::{self, get_body, DataRef, OpCode},
};
use std::result::Result as StdResult;
use std::{any::type_name, process};
use thiserror::Error;

pub type Stack = Vec<StackEntry>;

pub struct Memory<'a> {
    pub stack: Stack,
    pub data: &'a Vec<Data>,
}

#[derive(Debug)]
pub enum StackEntry {
    FrameBorder,
    Entry(Data),
}

#[derive(Error, Debug)]
pub enum Error {
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

pub type Result<T> = StdResult<T, Error>;
pub type ExecResult = Result<ExecOutcome>;

macro_rules! rt_err{
    ($msg:literal $(, $args:expr)*) => { Error::Runtime(format!($msg $(, $args)*)) };
}

macro_rules! rt_assert{
    ($cond:expr, $msg:literal $(, $args:expr)*) => {
        if ! $cond { return Err(rt_err!($msg $(, $args)*)); }
    };
}

macro_rules! bail{
    ($($err:tt)*) => {
        return Err(Error::$($err)*);
    };
}

macro_rules! size_of {
    ($i:ident) => {
        OpCode::serialized_size_of(opcode::$i)
    };
}

macro_rules! isize_of {
    ($u:ident) => {
        size_of!($u) as isize
    };
}

macro_rules! ok_pc {
    ($pc:expr) => {
        Ok(ExecOutcome::Pc($pc))
    };
}

pub unsafe fn exec_exec(pc: *const u8, mem: &mut Memory) -> ExecResult {
    let (bin, args) = get_body!(Exec, pc.offset(2));
    let bin_name = mem.get_as::<&str>(&bin)?;
    let args = mem.get_as::<Vec<&str>>(&args)?;
    let stat = process::Command::new(bin_name)
        .args(args)
        .status()
        .map_err(|e| rt_err!("Executing {}: {}", bin_name, e))?;
    rt_assert!(stat.success(), "{} did not execute successfully", bin_name);
    ok_pc!(pc.offset(isize_of!(EXEC)))
}

pub unsafe fn exec_strcat(pc: *const u8, mem: &mut Memory) -> ExecResult {
    let n = get_body!(StrCat, pc.offset(2));
    let n_is = *n as isize;
    let elems: Vec<&str> = (0isize..n_is)
        .map(|i| {
            let addr = pc.offset(isize_of!(STRCAT) + i * isize_of!(DATAREF) + 2);
            mem.get_as::<&str>(get_body!(DataRef, addr))
        })
        .collect::<StdResult<_, _>>()?;
    mem.stack
        .push(StackEntry::Entry(Data::String(elems.join(""))));
    ok_pc!(pc.offset(isize_of!(STRCAT) + n_is * isize_of!(DATAREF)))
}

pub unsafe fn exec_dataref(_: *const u8, _: &mut Memory) -> ExecResult {
    Err(Error::NonExecutableOpCode)
}

pub unsafe fn exec_exit(pc: *const u8, _: &mut Memory) -> ExecResult {
    let res = get_body!(Exit, pc.offset(2));
    Ok(ExecOutcome::ExitCode(*res))
}

pub trait FromMemory<'a>: Sized {
    fn get_as(mem: &'a Memory, dref: &'a Data) -> Result<Self>;
}

/// get's any Datatype which has a data as, which means all primitive ones
impl<'a, T: DataAs<'a>> FromMemory<'a> for T {
    fn get_as(mem: &'a Memory, dref: &'a Data) -> Result<T> {
        if let Data::Ref(dref) = dref {
            mem.get_as(dref)
        } else {
            dref.get_as().ok_or_else(|| Error::TypeError {
                accessed_data: dref.clone(),
                expected_type: type_name::<T>(),
            })
        }
    }
}

impl<'a, T: DataAs<'a>> FromMemory<'a> for Vec<T> {
    fn get_as(mem: &'a Memory, dref: &'a Data) -> Result<Vec<T>> {
        match dref {
            Data::Vec(elems) => elems
                .iter()
                .map(|e| <T as FromMemory>::get_as(mem, e))
                .collect(),
            _ => Err(Error::TypeError {
                accessed_data: dref.clone(),
                expected_type: type_name::<T>(),
            }),
        }
    }
}

impl<'a> Memory<'a> {
    pub fn get_as<T: FromMemory<'a>>(&'a self, reference: &DataRef) -> Result<T> {
        let data = self.resolve_ref(reference)?;
        <T as FromMemory>::get_as(self, data)
    }

    pub fn resolve_ref(&self, dref: &DataRef) -> Result<&Data> {
        use DataRef::*;
        match dref {
            StackIdx(i) => {
                if let StackEntry::Entry(e) = &self.stack[*i] {
                    Ok(e)
                } else {
                    Err(Error::UnexpectedStackEntry {
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

impl std::fmt::Display for StackEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            StackEntry::FrameBorder => "FrameBorder".into(),
            StackEntry::Entry(e) => format!("{:?}", e),
        };
        write!(f, "{}", s)
    }
}
